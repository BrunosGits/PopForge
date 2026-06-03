<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { type UnlistenFn } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invokeCommand, onProgress, isTauriRuntime } from '$lib/tauri';
  import type { Mode, Job, ToolStatus, ConversionProgress, GameMetadata, AppSettings } from '$lib/types';
  import TopBar from '$lib/components/TopBar.svelte';
  import InputPanel from '$lib/components/InputPanel.svelte';
  import ConvertOptions from '$lib/components/ConvertOptions.svelte';
  import CustomizeAssets from '$lib/components/CustomizeAssets.svelte';
  import QueuePanel from '$lib/components/QueuePanel.svelte';
  import LogPanel from '$lib/components/LogPanel.svelte';

  let mode: Mode = $state('convert');
  let outputFolder = $state('');
  let backendFile = $state('');
  let backendMessage = $state('');
  let gameName = $state('');
  let gameId = $state('');
  let compression = $state(0);
  let outputTemplate = $state('{SERIAL}_{TITLE}');
  let popstationPath = $state('');
  let icon0Path = $state('');
  let pic0Path = $state('');
  let pic1Path = $state('');
  let logLines: string[] = $state([
    '[info] Select files, then run the queue.'
  ]);

  let jobs: Job[] = $state([]);

  let progress: ConversionProgress = $state({
    current: 0,
    total: 0,
    fileName: '',
    stage: 'idle',
    filePercent: null
  });

  let toolchain: ToolStatus[] = $state([]);
  let unlistenProgress: UnlistenFn | null = null;

  function appendLog(line: string) {
    logLines = [...logLines, line];
  }

  function updateJob(id: number, patch: Partial<Job>) {
    jobs = jobs.map((job) => (job.id === id ? { ...job, ...patch } : job));
  }

  function clearQueue() {
    jobs = [];
    progress = { current: 0, total: 0, fileName: '', stage: 'idle', filePercent: null };
  }

  async function loadSettings() {
    const settings = await invokeCommand<AppSettings>('get_settings');
    if (settings.lastOutputFolder) {
      outputFolder = settings.lastOutputFolder;
    }
  }

  async function refreshToolchainStatus() {
    const [popstation] = await invokeCommand<ToolStatus[]>('get_toolchain_status', {
      popstationPath: popstationPath || null
    });
    toolchain = [popstation];
    popstationPath = popstation.path || popstationPath;
    if (popstation.available && !popstationPath) popstationPath = popstation.path || '';
  }

  async function addJobs() {
    if (!isTauriRuntime()) {
      appendLog('[info] File picker is available when the app runs inside Tauri.');
      return;
    }

    const filterName = mode === 'extract' ? 'PBP' : 'ISO/BIN/CUE';

    const selected = await open({
      multiple: true,
      filters: [{ name: filterName, extensions: mode === 'extract' ? ['pbp'] : ['iso', 'bin', 'cue'] }]
    });

    if (!selected) return;

    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths.length === 0) return;

    let newJobs: Job[] = [];
    for (const filePath of paths) {
      const fileName = filePath.split(/[\\/]/).pop() || filePath;
      newJobs.push({
        id: Date.now() + Math.random(),
        filePath,
        fileName,
        mode,
        status: 'pending',
        message: null,
        outputPath: null,
        commandPreview: null,
        metadata: null
      });
    }

    jobs = [...jobs, ...newJobs];
    appendLog(`[info] Added ${newJobs.length} file(s) to the queue.`);

    for (const job of newJobs) {
      try {
        const metadata = await invokeCommand<GameMetadata>('scrape_metadata', {
          filePath: job.filePath,
          fileName: job.fileName
        });
        updateJob(job.id, { metadata });
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        appendLog(`[warn] Metadata lookup failed for ${job.fileName}: ${msg}`);
      }
    }
  }

  async function chooseOutputFolder() {
    const selected = await open({ directory: true });
    if (selected) {
      outputFolder = selected as string;
      await invokeCommand('save_settings', {
        settings: { lastOutputFolder: outputFolder } as AppSettings
      });
    }
  }

  async function chooseAsset(name: string) {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'PNG', extensions: ['png'] }]
    });
    if (!selected) return;
    const path = selected as string;
    if (name === 'icon0') icon0Path = path;
    else if (name === 'pic0') pic0Path = path;
    else pic1Path = path;
    appendLog(`[info] ${name.toUpperCase()} set to ${path}`);
  }

  function resetAsset(name: string) {
    if (name === 'icon0') icon0Path = '';
    else if (name === 'pic0') pic0Path = '';
    else pic1Path = '';
    appendLog(`[info] ${name.toUpperCase()} reset to bundled default.`);
  }

  function previewAssetImage(name: string) {
    appendLog(`[info] ${name.toUpperCase()} preview is shown inline below the asset row.`);
  }

  async function chooseToolPath(toolName: string) {
    const selected = await open({ filters: [{ name: toolName, extensions: ['*'] }] });
    if (selected) {
      const path = selected as string;
      await invokeCommand('print_file_path', { path, toolName });
      if (toolName === 'psxpackager') popstationPath = path;
      appendLog(`[info] ${toolName} path set to ${path}`);
      await refreshToolchainStatus();
    }
  }

  async function autoGameId() {
    const fileName = jobs.length > 0 ? jobs[jobs.length - 1].fileName : backendFile;
    if (!fileName) {
      appendLog('[info] Add a file to the queue first, or select a file to extract the Game ID.');
      return;
    }
    const serial = await invokeCommand<string | null>('extract_serial', { filename: fileName });
    if (serial) {
      gameId = serial;
      appendLog(`[info] Game ID set to ${serial} from ${fileName}`);
    } else {
      appendLog(`[info] No serial pattern found in ${fileName}`);
    }
  }

  async function runAll() {
    if (jobs.length === 0) return;

    for (const job of jobs) {
      if (job.status !== 'pending') continue;

      updateJob(job.id, { status: 'running', message: null, outputPath: null, commandPreview: null });

      try {
        const options = {
          mode: job.mode,
          gameName: job.metadata?.title || gameName,
          gameId: job.metadata?.serial || gameId,
          compression,
          outputTemplate,
          outputFolder,
          popstationPath,
          icon0Path,
          pic0Path,
          pic1Path
        };

        const result = await invokeCommand<{
          success: boolean;
          message: string;
          output_path: string | null;
          command_preview: string | null;
        }>('run_conversion', {
          filePath: job.filePath,
          fileName: job.fileName,
          options
        });

        updateJob(job.id, {
          status: result.success ? 'done' : 'error',
          message: result.message,
          outputPath: result.output_path,
          commandPreview: result.command_preview
        });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        updateJob(job.id, { status: 'error', message });
      }
    }
  }

  async function testBackend() {
    backendMessage = await invokeCommand<string>('test_backend');
    appendLog(`[backend] ${backendMessage}`);
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
    if (!isTauriRuntime()) return;
    try {
      unlistenProgress = await onProgress('conversion-progress', (data) => {
        const payload = data as ConversionProgress;
        progress = {
          current: payload.current,
          total: payload.total,
          fileName: payload.fileName,
          stage: payload.stage,
          filePercent: payload.filePercent
        };
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      appendLog(`[warn] Could not subscribe to conversion-progress: ${message}`);
    }
  }
</script>

<svelte:head>
  <title>PopForge</title>
</svelte:head>

<main class="app">
  <TopBar bind:mode isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'} />

  <section class="layout">
    <InputPanel
      {mode}
      isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
      {toolchain}
      onAddJobs={addJobs}
      onRefreshTools={refreshToolchainStatus}
      onChooseToolPath={chooseToolPath}
    />

    <section class="main-panel">
      {#if mode === 'convert'}
        <ConvertOptions
          bind:gameName
          bind:gameId
          bind:compression
          bind:outputTemplate
          bind:outputFolder
          isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
          onChooseOutputFolder={chooseOutputFolder}
          onAutoGameId={autoGameId}
        />

        <CustomizeAssets
          bind:icon0Path
          bind:pic0Path
          bind:pic1Path
          isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
          onChooseAsset={chooseAsset}
          onResetAsset={resetAsset}
          onPreviewAsset={previewAssetImage}
        />
      {/if}

      <QueuePanel
        {jobs}
        {progress}
        {mode}
        isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
        onRunAll={runAll}
        onClearQueue={clearQueue}
      />

      <LogPanel
        {logLines}
        {outputFolder}
        {backendFile}
        {backendMessage}
        isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
        onTestBackend={testBackend}
      />
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

  .layout {
    display: grid;
    grid-template-columns: 360px 1fr;
    gap: 16px;
  }

  .main-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  @media (max-width: 880px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
</style>
