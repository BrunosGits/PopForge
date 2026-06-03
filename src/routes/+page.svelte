<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';

  type Mode = 'convert' | 'extract';
  type JobStatus = 'pending' | 'running' | 'done' | 'error';

  type ConversionResult = {
    success: boolean;
    message: string;
    output_path: string | null;
    command_preview: string | null;
  };

  type ConversionProgress = {
    current: number;
    total: number;
    fileName: string;
    stage: string;
    filePercent: number | null;
  };

  type GameMetadata = {
    serial: string;
    title: string;
    region: string;
    coverPath: string | null;
    source: string;
    cached: boolean;
  };

  type ConversionOptions = {
    mode: Mode;
    gameName: string;
    gameId: string;
    compression: number;
    outputTemplate: string;
    outputFolder: string;
    popstationPath: string;
    chdmanPath: string;
    icon0Path: string;
    pic0Path: string;
    pic1Path: string;
  };

  type ToolStatus = {
    name: string;
    available: boolean;
    detail: string;
    path: string | null;
  };

  type AppSettings = {
    lastOutputFolder: string;
  };

  type Job = {
    id: number;
    fileName: string;
    filePath: string;
    mode: Mode;
    status: JobStatus;
    message?: string;
    outputPath?: string;
    commandPreview?: string;
    metadata?: GameMetadata;
  };

  let mode: Mode = 'convert';
  let gameName = '';
  let gameId = 'AUTO';
  let compression = 9;
  let outputTemplate = '[%GAMEID%] %TITLE% (%REGION%)';
  let outputFolder = '';
  let backendMessage = '';
  let backendFile = '';
  let isRunning = false;
  let popstationPath = '';
  let chdmanPath = '';
  let icon0Path = '';
  let pic0Path = '';
  let pic1Path = '';
  let previewAsset: 'icon0' | 'pic0' | 'pic1' | null = null;
  let toolchain: ToolStatus[] = [];
  let logLines: string[] = [
    '[ready] PopForge initialized.',
    '[info] Select files, then run the queue.'
  ];

  let jobs: Job[] = [];

  type ProgressState = {
    current: number;
    total: number;
    fileName: string;
    stage: string;
    filePercent: number | null;
  };

  let progress: ProgressState = {
    current: 0,
    total: 0,
    fileName: '',
    stage: 'idle',
    filePercent: null
  };

  let unlistenProgress: UnlistenFn | null = null;

  function isTauriRuntime() {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  }

  async function invokeCommand<T>(command: string, args?: Record<string, unknown>) {
    if (!isTauriRuntime()) {
      if (command === 'get_last_file') {
        return '' as T;
      }

      if (command === 'test_backend') {
        return 'Browser preview is running without the Tauri backend.' as T;
      }

      if (command === 'get_toolchain_status') {
        return [
          {
            name: 'psxpackager',
            available: false,
            detail: 'Tool probing runs inside Tauri.',
            path: popstationPath || null
          },
          {
            name: 'chdman',
            available: false,
            detail: 'Tool probing runs inside Tauri.',
            path: chdmanPath || null
          }
        ] as T;
      }

      if (command === 'run_conversion') {
        return {
          success: false,
          message: 'Run this inside Tauri to process queue jobs.',
          output_path: null,
          command_preview: null
        } as T;
      }

      if (command === 'get_settings') {
        return { lastOutputFolder: '' } as T;
      }

      if (command === 'save_settings') {
        return undefined as T;
      }

      if (command === 'scrape_metadata') {
        return {
          serial: '',
          title: 'Browser preview: scraping runs inside Tauri.',
          region: 'Unknown',
          coverPath: null,
          source: 'browser-preview',
          cached: false
        } as T;
      }
    }

    return invoke<T>(command, args);
  }

  function appendLog(line: string) {
    logLines = [...logLines, line];
  }

  function updateJob(id: number, patch: Partial<Job>) {
    jobs = jobs.map((job) => (job.id === id ? { ...job, ...patch } : job));
  }

  function queuePercent(): number {
    if (progress.total === 0) {
      return 0;
    }
    if (progress.stage === 'completed') {
      return 100;
    }
    const base = (Math.max(progress.current, 1) - 1) / progress.total;
    const step =
      progress.stage === 'starting' ? 0.05 : progress.stage === 'chdman' ? 0.15 : 0.3;
    return Math.min(100, Math.round((base + step / progress.total) * 100));
  }

  function getConversionOptions(job: Job): ConversionOptions {
    return {
      mode: job.mode,
      gameName,
      gameId,
      compression,
      outputTemplate,
      outputFolder,
      popstationPath,
      chdmanPath,
      icon0Path,
      pic0Path,
      pic1Path
    };
  }

  async function refreshBackendFile() {
    backendFile = await invokeCommand<string>('get_last_file');
  }

  async function refreshToolchainStatus() {
    toolchain = await invokeCommand<ToolStatus[]>('get_toolchain_status', {
      paths: {
        popstationPath,
        chdmanPath
      }
    });
    for (const tool of toolchain) {
      appendLog(
        `[tool] ${tool.name}: ${tool.available ? 'found' : 'missing'} - ${tool.detail}`
      );
    }
  }

  async function loadSettings() {
    const settings = await invokeCommand<AppSettings>('get_settings');
    if (settings.lastOutputFolder) {
      outputFolder = settings.lastOutputFolder;
      appendLog(`[info] Restored output folder: ${settings.lastOutputFolder}`);
    }
  }

  async function persistOutputFolder() {
    if (!outputFolder) {
      return;
    }
    try {
      await invokeCommand('save_settings', {
        settings: { lastOutputFolder: outputFolder }
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      appendLog(`[warn] Could not save settings: ${message}`);
    }
  }

  onMount(() => {
    loadSettings();
    refreshToolchainStatus();
    bindProgressListener();
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  });

  async function bindProgressListener() {
    if (!isTauriRuntime()) {
      return;
    }
    try {
      unlistenProgress = await listen<ConversionProgress>(
        'conversion-progress',
        (event) => {
          progress = {
            current: event.payload.current,
            total: event.payload.total,
            fileName: event.payload.fileName,
            stage: event.payload.stage,
            filePercent: event.payload.filePercent
          };
        }
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      appendLog(`[warn] Could not subscribe to conversion-progress: ${message}`);
    }
  }

  async function addJobs() {
    if (!isTauriRuntime()) {
      appendLog('[info] File picker is available when the app runs inside Tauri.');
      return;
    }

    const selected = await open({
      multiple: true,
          filters: [
            {
              name: mode === 'convert' ? 'PS1 Images' : 'PSP EBOOT',
              extensions: mode === 'convert' ? ['iso', 'bin', 'cue', 'img'] : ['pbp']
            }
          ]
    });

    if (!selected) {
      return;
    }

    const selectedPaths = Array.isArray(selected) ? selected : [selected];
    const nextJobs: Job[] = selectedPaths.map((filePath, index) => ({
      id: Date.now() + index,
      fileName: filePath.split('/').pop() ?? filePath,
      filePath,
      mode,
      status: 'pending' as JobStatus
    }));

    await invokeCommand('print_file_path', {
      path: selectedPaths[selectedPaths.length - 1]
    });
    await refreshBackendFile();

    jobs = [...jobs, ...nextJobs];
    appendLog(`[queue] Added ${nextJobs.length} file${nextJobs.length === 1 ? '' : 's'}.`);

    for (const job of nextJobs) {
      try {
        const metadata = await invokeCommand<GameMetadata>('scrape_metadata', {
          fileName: job.fileName
        });
        if (metadata.serial) {
          updateJob(job.id, { metadata });
          appendLog(
            `[meta] ${job.fileName}: ${metadata.title} (${metadata.region}) [${metadata.cached ? 'cache' : metadata.source}]`
          );
        } else {
          appendLog(`[meta] ${job.fileName}: ${metadata.source}`);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        appendLog(`[warn] Metadata scrape failed for ${job.fileName}: ${message}`);
      }
    }
  }

  async function chooseOutputFolder() {
    if (!isTauriRuntime()) {
      appendLog('[info] Folder picker is available when the app runs inside Tauri.');
      return;
    }

    const selected = await open({
      directory: true,
      multiple: false
    });

    if (typeof selected === 'string') {
      outputFolder = selected;
      appendLog(`[info] Output folder set: ${selected}`);
      await persistOutputFolder();
    }
  }

  async function chooseToolPath(tool: 'psxpackager' | 'chdman') {
    if (!isTauriRuntime()) {
      appendLog('[info] Tool picker is available when the app runs inside Tauri.');
      return;
    }

    const selected = await open({
      multiple: false
    });

    if (typeof selected !== 'string') {
      return;
    }

    if (tool === 'psxpackager') {
      popstationPath = selected;
    } else {
      chdmanPath = selected;
    }

    appendLog(`[tool] ${tool} path set: ${selected}`);
    await refreshToolchainStatus();
  }

  async function chooseAsset(asset: 'icon0' | 'pic0' | 'pic1') {
    if (!isTauriRuntime()) {
      appendLog('[info] Asset picker is available when the app runs inside Tauri.');
      return;
    }

    const selected = await open({
      multiple: false,
      filters: [{ name: 'PSP Assets', extensions: ['png', 'PNG'] }]
    });

    if (typeof selected !== 'string') {
      return;
    }

    if (asset === 'icon0') icon0Path = selected;
    if (asset === 'pic0') pic0Path = selected;
    if (asset === 'pic1') pic1Path = selected;

    appendLog(`[asset] ${asset.toUpperCase()}: ${selected}`);
  }

  function resetAsset(asset: 'icon0' | 'pic0' | 'pic1') {
    if (asset === 'icon0') icon0Path = '';
    if (asset === 'pic0') pic0Path = '';
    if (asset === 'pic1') pic1Path = '';
    appendLog(`[asset] ${asset.toUpperCase()}: reset to bundled default`);
  }

  function previewAssetImage(asset: 'icon0' | 'pic0' | 'pic1') {
    const path = asset === 'icon0' ? icon0Path : asset === 'pic0' ? pic0Path : pic1Path;
    if (!path) {
      appendLog(`[asset] ${asset.toUpperCase()}: using bundled default (no custom path)`);
      return;
    }
    previewAsset = asset;
  }

  function clearQueue() {
    jobs = [];
    backendMessage = '';
    appendLog('[queue] Cleared.');
  }

  async function runAll() {
    if (isRunning || jobs.length === 0) {
      return;
    }

    isRunning = true;
    progress = {
      current: 0,
      total: jobs.length,
      fileName: '',
      stage: 'starting',
      filePercent: null
    };
    appendLog(`[queue] Running ${jobs.length} job${jobs.length === 1 ? '' : 's'}...`);

    for (let index = 0; index < jobs.length; index += 1) {
      const job = jobs[index];
      updateJob(job.id, { status: 'running', message: 'Running...' });
      appendLog(`[run] ${job.fileName}`);

      try {
        const result = await invokeCommand<ConversionResult>('run_conversion', {
          filePath: job.filePath,
          options: getConversionOptions(job),
          queueIndex: index,
          queueTotal: jobs.length
        });

        updateJob(job.id, {
          status: result.success ? 'done' : 'error',
          message: result.message,
          outputPath: result.output_path ?? undefined,
          commandPreview: result.command_preview ?? undefined
        });
        backendMessage = result.message;
        appendLog(`${result.success ? '[done]' : '[error]'} ${result.message}`);
        if (result.command_preview) {
          appendLog(`[cmd] ${result.command_preview}`);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        updateJob(job.id, { status: 'error', message });
        backendMessage = message;
        appendLog(`[error] ${message}`);
      }
    }

    isRunning = false;
    progress = {
      current: jobs.length,
      total: jobs.length,
      fileName: '',
      stage: 'completed',
      filePercent: 1
    };
    appendLog('[queue] Finished.');
  }

  async function testBackend() {
    backendMessage = await invokeCommand<string>('test_backend');
    appendLog(`[backend] ${backendMessage}`);
  }
</script>

<svelte:head>
  <title>PopForge</title>
</svelte:head>

<main class="app">
  <header class="topbar">
    <div>
      <p class="eyebrow">PSX · PSP · Vita</p>
      <h1>PopForge</h1>
    </div>

    <div class="mode-toggle" aria-label="Mode">
      <button
        class:active={mode === 'convert'}
        on:click={() => (mode = 'convert')}
      >
        Convert
      </button>

      <button
        class:active={mode === 'extract'}
        on:click={() => (mode = 'extract')}
      >
        Extract
      </button>
    </div>
  </header>

  <section class="layout">
    <aside class="sidebar">
      <section class="panel">
        <h2>Input</h2>

        <button class="drop-zone" on:click={addJobs} disabled={isRunning}>
          <span class="drop-title">
            {mode === 'convert' ? 'Drop ISO / BIN+CUE / CHD here' : 'Drop EBOOT.PBP here'}
          </span>
          <span class="drop-subtitle">Click to add files to the queue</span>
        </button>
      </section>

      <section class="panel">
        <div class="panel-header">
          <h2>Toolchain</h2>
          <button type="button" on:click={refreshToolchainStatus} disabled={isRunning}>
            Check Tools
          </button>
        </div>

        <div class="tool-list">
          {#each toolchain as tool}
            <div class="tool-row">
              <span>
                <strong>{tool.name}</strong>
                <span>{tool.detail}</span>
                {#if tool.path}
                  <span>{tool.path}</span>
                {/if}
              </span>
              <span class:done={tool.available} class:error={!tool.available} class="status">
                {tool.available ? 'found' : 'missing'}
              </span>
            </div>
          {/each}
        </div>

        <div class="tool-actions">
          <button type="button" on:click={() => chooseToolPath('psxpackager')} disabled={isRunning}>
            Set PSXPackager
          </button>
          <button type="button" on:click={() => chooseToolPath('chdman')} disabled={isRunning}>
            Set chdman
          </button>
        </div>
      </section>

      {#if mode === 'convert'}
        <section class="panel">
          <h2>Convert Options</h2>

          <label>
            Game Name
            <input bind:value={gameName} placeholder="Tony Hawk's Pro Skater" />
          </label>

          <label>
            Game ID
            <div class="inline">
              <input bind:value={gameId} />
              <button type="button">Auto</button>
            </div>
          </label>

          <label>
            Compression
            <select bind:value={compression}>
              {#each Array.from({ length: 10 }, (_, i) => i) as level}
                <option value={level}>{level}</option>
              {/each}
            </select>
          </label>

          <label>
            Output Filename Template
            <input bind:value={outputTemplate} />
          </label>

          <label>
            Output Folder
            <div class="inline">
              <input bind:value={outputFolder} readonly />
              <button type="button" on:click={chooseOutputFolder}>
                Browse
              </button>
            </div>
          </label>

          <p class="muted">
            CHD inputs are normalized with chdman before PopForge runs PSXPackager.
          </p>
        </section>

        <section class="panel">
          <h2>Customize PSP Assets</h2>

          <div class="asset-row">
            <div class="asset-label">ICON0.PNG</div>
            <div class="inline">
              <input
                value={icon0Path}
                readonly
                placeholder="Bundled default"
              />
              <button type="button" on:click={() => chooseAsset('icon0')} disabled={isRunning}>
                Choose
              </button>
            </div>
            <div class="asset-actions">
              <button type="button" on:click={() => resetAsset('icon0')} disabled={isRunning}>
                Reset
              </button>
              <button type="button" on:click={() => previewAssetImage('icon0')} disabled={isRunning}>
                Preview
              </button>
            </div>
          </div>

          <div class="asset-row">
            <div class="asset-label">PIC0.PNG</div>
            <div class="inline">
              <input
                value={pic0Path}
                readonly
                placeholder="Bundled default"
              />
              <button type="button" on:click={() => chooseAsset('pic0')} disabled={isRunning}>
                Choose
              </button>
            </div>
            <div class="asset-actions">
              <button type="button" on:click={() => resetAsset('pic0')} disabled={isRunning}>
                Reset
              </button>
              <button type="button" on:click={() => previewAssetImage('pic0')} disabled={isRunning}>
                Preview
              </button>
            </div>
          </div>

          <div class="asset-row">
            <div class="asset-label">PIC1.PNG</div>
            <div class="inline">
              <input
                value={pic1Path}
                readonly
                placeholder="Bundled default"
              />
              <button type="button" on:click={() => chooseAsset('pic1')} disabled={isRunning}>
                Choose
              </button>
            </div>
            <div class="asset-actions">
              <button type="button" on:click={() => resetAsset('pic1')} disabled={isRunning}>
                Reset
              </button>
              <button type="button" on:click={() => previewAssetImage('pic1')} disabled={isRunning}>
                Preview
              </button>
            </div>
          </div>

          {#if previewAsset}
            {@const previewPath =
              previewAsset === 'icon0'
                ? icon0Path
                : previewAsset === 'pic0'
                  ? pic0Path
                  : pic1Path}
            <div class="asset-preview">
              {#if previewPath}
                <img
                  src={convertFileSrc(previewPath)}
                  alt="{previewAsset.toUpperCase()} preview"
                />
              {:else}
                <p class="muted">No custom image selected. Preview shows bundled default only when set in the temp staging folder.</p>
              {/if}
              <button type="button" on:click={() => (previewAsset = null)}>Close Preview</button>
            </div>
          {/if}
        </section>
      {/if}
    </aside>

    <section class="main-panel">
      <section class="panel queue-panel">
        <div class="panel-header">
          <h2>Queue</h2>
          <div class="actions">
            <button on:click={clearQueue} disabled={isRunning || jobs.length === 0}>Clear</button>
            <button class="primary" on:click={runAll} disabled={isRunning || jobs.length === 0}>
              {isRunning ? 'Running...' : 'Run All'}
            </button>
          </div>
        </div>

        {#if progress.total > 0}
          <div class="progress" aria-live="polite">
            <div class="progress-meta">
              <span>
                {#if progress.stage === 'starting'}
                  Starting…
                {:else if progress.stage === 'completed'}
                  Finished
                {:else}
                  Converting {progress.current} of {progress.total}
                {/if}
              </span>
              <span class="progress-percent">{Math.round(queuePercent())}%</span>
            </div>
            <div class="progress-track">
              <div class="progress-fill" style:width="{queuePercent()}%"></div>
            </div>
            {#if progress.fileName}
              <p class="progress-file">
                {progress.fileName}
                {#if progress.stage === 'chdman' || progress.stage === 'psxpackager'}
                  <span class="muted">· {progress.stage}</span>
                {/if}
              </p>
            {/if}
          </div>
        {/if}

        {#if jobs.length === 0}
          <div class="empty">
            No jobs yet. Add files from the input panel.
          </div>
        {:else}
          <div class="jobs">
            {#each jobs as job}
              <article class="job">
                <div class="job-info">
                  {#if job.metadata?.coverPath}
                    <img
                      class="job-cover"
                      src={convertFileSrc(job.metadata.coverPath)}
                      alt="{job.metadata.title} cover"
                    />
                  {/if}
                  <div>
                    <strong>
                      {job.metadata?.title ?? job.fileName}
                    </strong>
                    <span>{job.filePath}</span>
                    {#if job.metadata?.serial}
                      <span>
                        <span class="meta-tag">{job.metadata.serial}</span>
                        <span class="meta-tag">{job.metadata.region}</span>
                        {#if job.metadata.cached}
                          <span class="meta-tag muted">cached</span>
                        {:else if job.metadata.source && job.metadata.source !== 'stub' && job.metadata.source !== 'no-serial'}
                          <span class="meta-tag muted">{job.metadata.source}</span>
                        {/if}
                      </span>
                    {/if}
                    {#if job.message}
                      <span>{job.message}</span>
                    {/if}
                    {#if job.outputPath}
                      <span>{job.outputPath}</span>
                    {/if}
                    {#if job.commandPreview}
                      <span>{job.commandPreview}</span>
                    {/if}
                    <span>{job.mode}</span>
                  </div>
                </div>

                <span
                  class:done={job.status === 'done'}
                  class:error={job.status === 'error'}
                  class:running={job.status === 'running'}
                  class="status"
                >
                  {job.status}
                </span>
              </article>
            {/each}
          </div>
        {/if}
      </section>

      <section class="panel log-panel">
        <h2>Log</h2>

        <button on:click={testBackend} disabled={isRunning}>
          Test Backend
        </button>

        <pre>
[info] Output folder: {outputFolder}
[info] Last selected file: {backendFile}
{backendMessage}
{logLines.join('\n')}
        </pre>
      </section>
    </section>
  </section>
</main>

<style>
  :global(body) {
    margin: 0;
    background:
      radial-gradient(circle at top left, rgba(91, 156, 246, 0.14), transparent 26rem),
      #1a1a1a;
    color: #f2f2f2;
    font-family:
      ui-monospace,
      SFMono-Regular,
      Menlo,
      Monaco,
      Consolas,
      'Liberation Mono',
      'Courier New',
      monospace;
  }

  :global(button),
  :global(input),
  :global(select) {
    font: inherit;
  }

  .app {
    min-height: 100vh;
    padding: 20px;
  }

  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
  }

  .eyebrow {
    margin: 0 0 4px;
    color: #5b9cf6;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    font-size: 12px;
  }

  h1 {
    margin: 0;
    font-size: 32px;
    line-height: 1;
  }

  h2 {
    margin: 0 0 14px;
    font-size: 15px;
  }

  .mode-toggle {
    display: flex;
    padding: 4px;
    border: 1px solid #333;
    border-radius: 999px;
    background: #242424;
  }

  button {
    border: 1px solid #333;
    border-radius: 8px;
    background: #2c2c2c;
    color: #f2f2f2;
    padding: 8px 12px;
    cursor: pointer;
  }

  button:hover {
    border-color: #5b9cf6;
    color: #8bbcff;
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  button:disabled:hover {
    border-color: #333;
    color: #f2f2f2;
  }

  button.active,
  button.primary {
    border-color: #5b9cf6;
    background: #5b9cf6;
    color: #08111f;
  }

  .layout {
    display: grid;
    grid-template-columns: 360px 1fr;
    gap: 16px;
  }

  .sidebar,
  .main-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .panel {
    border: 1px solid #333;
    border-radius: 14px;
    background: rgba(36, 36, 36, 0.86);
    padding: 16px;
  }

  .drop-zone {
    width: 100%;
    min-height: 150px;
    border: 1px dashed #5b9cf6;
    background: rgba(91, 156, 246, 0.07);
    display: grid;
    place-items: center;
    gap: 8px;
    text-align: center;
  }

  .drop-title {
    display: block;
    font-weight: 700;
  }

  .drop-subtitle,
  .muted {
    color: #a0a0a0;
    font-size: 13px;
  }

  label {
    display: grid;
    gap: 6px;
    margin-bottom: 12px;
    color: #a0a0a0;
    font-size: 13px;
  }

  input,
  select {
    width: 100%;
    border: 1px solid #333;
    border-radius: 8px;
    background: #1a1a1a;
    color: #f2f2f2;
    padding: 9px 10px;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: #5b9cf6;
  }

  .inline {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  .asset-row {
    display: grid;
    gap: 6px;
    margin-bottom: 14px;
    padding: 10px;
    border: 1px solid #2c2c2c;
    border-radius: 10px;
    background: rgba(26, 26, 26, 0.45);
  }

  .asset-label {
    color: #cfe1ff;
    font-size: 13px;
    font-weight: 600;
  }

  .asset-actions {
    display: flex;
    gap: 8px;
  }

  .asset-preview {
    display: grid;
    gap: 8px;
    margin-top: 6px;
    padding: 10px;
    border: 1px dashed #5b9cf6;
    border-radius: 10px;
    background: rgba(91, 156, 246, 0.05);
  }

  .asset-preview img {
    max-width: 100%;
    max-height: 180px;
    object-fit: contain;
    border-radius: 6px;
    background: #1a1a1a;
  }

  .panel-header,
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .queue-panel {
    min-height: 330px;
  }

  .empty {
    display: grid;
    min-height: 230px;
    place-items: center;
    color: #777;
    border: 1px dashed #333;
    border-radius: 10px;
  }

  .jobs {
    display: grid;
    gap: 8px;
  }

  .tool-list {
    display: grid;
    gap: 8px;
  }

  .tool-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 10px;
  }

  .tool-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 10px;
    align-items: center;
    padding: 10px;
    border: 1px solid #333;
    border-radius: 8px;
    background: rgba(26, 26, 26, 0.65);
  }

  .tool-row span {
    display: block;
  }

  .tool-row span span {
    color: #a0a0a0;
    font-size: 12px;
    margin-top: 3px;
  }

  .job {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px;
    border: 1px solid #333;
    border-radius: 10px;
    background: rgba(26, 26, 26, 0.65);
  }

  .job-info {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    flex: 1;
    min-width: 0;
  }

  .job-cover {
    width: 72px;
    height: 72px;
    object-fit: cover;
    border-radius: 6px;
    background: #1a1a1a;
    flex-shrink: 0;
  }

  .meta-tag {
    display: inline-block;
    margin-right: 6px;
    padding: 1px 6px;
    border: 1px solid #2c3a55;
    border-radius: 4px;
    background: rgba(91, 156, 246, 0.08);
    color: #cfe1ff;
    font-size: 11px;
  }

  .meta-tag.muted {
    border-color: #333;
    background: rgba(26, 26, 26, 0.5);
    color: #a0a0a0;
  }

  .job strong,
  .job span {
    display: block;
  }

  .job span {
    color: #a0a0a0;
    font-size: 12px;
  }

  .status {
    color: #a0a0a0;
    text-transform: uppercase;
    font-size: 12px;
  }

  .status.done {
    color: #6ee785;
  }

  .status.error {
    color: #ff7b7b;
  }

  .status.running {
    color: #8bbcff;
  }

  .progress {
    display: grid;
    gap: 6px;
    padding: 12px;
    border: 1px solid #2c3a55;
    border-radius: 10px;
    background: rgba(91, 156, 246, 0.08);
    margin-bottom: 12px;
  }

  .progress-meta {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    font-size: 13px;
    color: #cfe1ff;
  }

  .progress-percent {
    color: #8bbcff;
    font-weight: 700;
  }

  .progress-track {
    height: 10px;
    border-radius: 999px;
    background: rgba(26, 26, 26, 0.85);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #5b9cf6, #8bbcff);
    transition: width 0.2s ease;
  }

  .progress-file {
    margin: 0;
    font-size: 12px;
    color: #a0a0a0;
  }

  .log-panel {
    flex: 1;
  }

  pre {
    min-height: 150px;
    margin: 0;
    overflow: auto;
    color: #a0a0a0;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  @media (max-width: 880px) {
    .layout {
      grid-template-columns: 1fr;
    }

    .topbar {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
