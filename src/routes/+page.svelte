<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invokeCommand, onProgress, isTauriRuntime } from '$lib/tauri';
  import type { Mode, Job, ToolStatus, ConversionProgress, GameMetadata, AppSettings, ToastNotification, ToastType } from '$lib/types';
  import TopBar from '$lib/components/TopBar.svelte';
  import InputPanel from '$lib/components/InputPanel.svelte';
  import ConvertOptions from '$lib/components/ConvertOptions.svelte';
  import QueuePanel from '$lib/components/QueuePanel.svelte';
  import LogPanel from '$lib/components/LogPanel.svelte';
  import AboutDialog from '$lib/components/AboutDialog.svelte';
  import Toast from '$lib/components/Toast.svelte';

  let mode: Mode = $state('convert');
  let outputFolder = $state('');
  let backendFile = $state('');
  let backendMessage = $state('');
  let gameName = $state('');
  let gameId = $state('');
  let compression = $state(0);
  let outputTemplate = $state('{SERIAL}_{TITLE}');
  let popstationPath = $state('');
  let subfolderPerGame = $state(false);
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

  let unlistenProgress: UnlistenFn | null = null;
  let unlistenDragDrop: UnlistenFn | null = null;
  let isDragOver = $state(false);
  let showAbout = $state(false);
  let showLog = $state(false);

  let collapsedQueue = $state(true);
  let collapsedOptions = $state(true);

  let toasts: ToastNotification[] = $state([]);
  let selectedJobIds: Set<number> = $state(new Set());

  function showToast(type: ToastType, message: string) {
    const id = Date.now() + Math.random();
    toasts = [...toasts, { id, type, message }];
    setTimeout(() => {
      toasts = toasts.filter((t) => t.id !== id);
    }, 3000);
  }

  function dismissToast(id: number) {
    toasts = toasts.filter((t) => t.id !== id);
  }

  const DISC_RE = /\s*[\(\[]\s*(disc|cd|disk)\s*(\d+)\s*[\)\]]|\s*[-–—]\s*(disc|cd|disk)\s*(\d+)\s*$/i;

  function stripDiscIndicator(name: string): string {
    return name.replace(DISC_RE, '').replace(/\.(iso|bin|cue|pbp)$/i, '').trim();
  }

  function extractDiscNumber(name: string): number | null {
    const match = name.match(DISC_RE);
    if (!match) return null;
    const num = match[2] || match[4];
    return num ? parseInt(num, 10) : null;
  }

  function autoDetectGroups() {
    const ungrouped = jobs.filter((j) => j.groupId === null && j.status === 'pending');
    if (ungrouped.length < 2) return;

    const serialGroups = new Map<string, typeof ungrouped>();
    for (const job of ungrouped) {
      if (job.metadata?.serial) {
        const g = serialGroups.get(job.metadata.serial) || [];
        g.push(job);
        serialGroups.set(job.metadata.serial, g);
      }
    }

    const nameGroups = new Map<string, typeof ungrouped>();
    for (const job of ungrouped) {
      if (!job.metadata?.serial) {
        const key = stripDiscIndicator(job.fileName);
        const g = nameGroups.get(key) || [];
        g.push(job);
        nameGroups.set(key, g);
      }
    }

    let nextGroupId = Date.now();

    for (const [, group] of serialGroups) {
      if (group.length < 2) continue;
      const gid = nextGroupId++;
      group.sort((a, b) => (extractDiscNumber(a.fileName) ?? 99) - (extractDiscNumber(b.fileName) ?? 99));
      group.forEach((job, idx) => updateJob(job.id, { groupId: gid, discIndex: idx }));
      appendLog(`[info] Auto-grouped ${group.length} discs by serial (${group[0].metadata?.serial})`);
    }

    for (const [, group] of nameGroups) {
      if (group.length < 2) continue;
      const gid = nextGroupId++;
      group.sort((a, b) => (extractDiscNumber(a.fileName) ?? 99) - (extractDiscNumber(b.fileName) ?? 99));
      group.forEach((job, idx) => updateJob(job.id, { groupId: gid, discIndex: idx }));
      appendLog(`[info] Auto-grouped ${group.length} discs by name`);
    }
  }

  function mergeSelectedJobs() {
    const selected = jobs.filter((j) => selectedJobIds.has(j.id));
    if (selected.length < 2) return;
    const gid = Date.now() + Math.random();
    selected.forEach((job, idx) => updateJob(job.id, { groupId: gid, discIndex: idx }));
    selectedJobIds = new Set();
    appendLog(`[info] Merged ${selected.length} jobs into a disc group`);
  }

  function ungroupJob(jobId: number) {
    const job = jobs.find((j) => j.id === jobId);
    if (!job || job.groupId === null) return;
    const gid = job.groupId;
    for (const j of jobs) {
      if (j.groupId === gid) {
        updateJob(j.id, { groupId: null, discIndex: null });
      }
    }
    appendLog(`[info] Ungrouped disc group`);
  }

  function toggleJobSelection(id: number) {
    const next = new Set(selectedJobIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedJobIds = next;
  }

  $effect(() => {
    const _mode = mode;
    const _compression = compression;
    const _gameName = gameName;
    const _gameId = gameId;
    if (isTauriRuntime()) {
      saveSettings();
    }
  });

  function appendLog(line: string) {
    logLines = [...logLines, line];
  }

  function updateJob(id: number, patch: Partial<Job>) {
    jobs = jobs.map((job) => (job.id === id ? { ...job, ...patch } : job));
  }

  function clearQueue() {
    if (jobs.length > 0 && !window.confirm('Clear all jobs?')) return;
    jobs = [];
    selectedJobIds = new Set();
    progress = { current: 0, total: 0, fileName: '', stage: 'idle', filePercent: null };
  }

  function removeJob(id: number) {
    const job = jobs.find((j) => j.id === id);
    if (job?.groupId !== null) {
      ungroupJob(id);
    }
    selectedJobIds = new Set(Array.from(selectedJobIds).filter((sid) => sid !== id));
    jobs = jobs.filter((j) => j.id !== id);
  }

  function retryJob(id: number) {
    updateJob(id, { status: 'pending', message: null, outputPath: null, commandPreview: null });
  }

  async function loadSettings() {
    const settings = await invokeCommand<AppSettings>('get_settings');
    if (settings.lastOutputFolder) outputFolder = settings.lastOutputFolder;
    if (settings.lastMode === 'convert' || settings.lastMode === 'extract') mode = settings.lastMode;
    if (settings.compression !== undefined) compression = settings.compression;
    if (settings.outputTemplate) outputTemplate = settings.outputTemplate;
    if (settings.gameName) gameName = settings.gameName;
    if (settings.gameId) gameId = settings.gameId;
    if (settings.subfolderPerGame !== undefined) subfolderPerGame = settings.subfolderPerGame;
  }

  async function saveSettings() {
    await invokeCommand('save_settings', {
      settings: {
        lastOutputFolder: outputFolder,
        lastMode: mode,
        compression,
        outputTemplate,
        gameName,
        gameId,
        windowWidth: 800,
        windowHeight: 600,
        subfolderPerGame
      } as AppSettings
    });
  }

  async function refreshToolchainStatus() {
    const [popstation] = await invokeCommand<ToolStatus[]>('get_toolchain_status', {
      popstationPath: popstationPath || null
    });
    popstationPath = popstation.path || popstationPath;
    if (popstation.available && !popstationPath) popstationPath = popstation.path || '';
  }

  async function enqueuePaths(paths: string[], enqueueMode: Mode = mode) {
    if (paths.length === 0) return;

    let newJobs: Job[] = [];
    for (const filePath of paths) {
      const fileName = filePath.split(/[\\/]/).pop() || filePath;
      newJobs.push({
        id: Date.now() + Math.random(),
        filePath,
        fileName,
        mode: enqueueMode,
        status: 'pending',
        message: null,
        outputPath: null,
        commandPreview: null,
        metadata: null,
        groupId: null,
        discIndex: null
      });
    }

    if (!outputFolder && paths[0]) {
      const parent = paths[0].split(/[\\/]/).slice(0, -1).join('/');
      if (parent) outputFolder = parent;
    }

    jobs = [...jobs, ...newJobs];
    collapsedQueue = false;
    appendLog(`[info] Added ${newJobs.length} file(s) to the queue.`);
    showToast('info', `Added ${newJobs.length} file(s)`);

    autoDetectGroups();
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
    await enqueuePaths(paths);
  }

  function handleDragEnter(e: DragEvent) {
    e.preventDefault();
    isDragOver = true;
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragOver = false;
    if (!e.dataTransfer?.files.length) return;
    const paths: string[] = [];
    for (const file of Array.from(e.dataTransfer.files)) {
      if ('path' in file) {
        paths.push((file as File & { path: string }).path);
      }
    }
    if (paths.length > 0) {
      await enqueuePaths(paths);
    }
  }

  async function bindDragDropListener() {
    if (!isTauriRuntime()) return;
    try {
      unlistenDragDrop = await listen<{ paths: string[] }>('tauri://drag-drop', async (event) => {
        if (event.payload.paths.length > 0) {
          await enqueuePaths(event.payload.paths);
        }
      });
    } catch {
      // Silently ignore — HTML5 drag-drop fallback handles it
    }
  }

  async function chooseOutputFolder() {
    const selected = await open({ directory: true });
    if (selected) {
      outputFolder = selected as string;
      await saveSettings();
    }
  }

  async function downloadGameInfo() {
    const lastJob = jobs.length > 0 ? jobs[jobs.length - 1] : null;
    const fileName = lastJob?.fileName || backendFile;
    const filePath = lastJob?.filePath || '';
    if (!fileName) {
      appendLog('[info] Add a file to the queue first to fetch metadata.');
      return;
    }
    try {
      const metadata = await invokeCommand<GameMetadata>('scrape_metadata', {
        filePath: filePath || null,
        fileName
      });
      if (metadata.title && metadata.title !== 'Unknown title') {
        gameName = metadata.title;
      }
      if (metadata.serial) {
        gameId = metadata.serial;
      }
      appendLog(`[info] Metadata fetched: ${metadata.title} (${metadata.serial})`);
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      appendLog(`[warn] Metadata fetch failed: ${msg}`);
    }
  }

  async function autoGameId() {
    const lastJob = jobs.length > 0 ? jobs[jobs.length - 1] : null;
    const fileName = lastJob?.fileName || backendFile;
    const filePath = lastJob?.filePath || '';
    if (!fileName) {
      appendLog('[info] Add a file to the queue first, or select a file to extract the Game ID.');
      return;
    }
    const serial = await invokeCommand<string | null>('extract_serial', { filename: fileName, filePath: filePath || null });
    if (serial) {
      gameId = serial;
      appendLog(`[info] Game ID set to ${serial} from ${fileName}`);
    } else {
      appendLog(`[info] No serial pattern found in ${fileName}`);
    }
  }

  async function runAll() {
    if (jobs.length === 0) return;

    let succeeded = 0;
    let failed = 0;

    const pending = jobs.filter((j) => j.status === 'pending' && j.mode === mode);
    const groups = new Map<number, typeof pending>();
    const singles: typeof pending = [];

    for (const job of pending) {
      if (job.groupId !== null) {
        const g = groups.get(job.groupId) || [];
        g.push(job);
        groups.set(job.groupId, g);
      } else {
        singles.push(job);
      }
    }

    async function runGroup(group: typeof pending) {
      const primary = group[0];
      for (const job of group) {
        updateJob(job.id, { status: 'running', message: null, outputPath: null, commandPreview: null });
      }

       try {
         const options = {
           mode: primary.mode,
           gameName: primary.metadata?.title || gameName,
           gameId: primary.metadata?.serial || gameId,
           compression,
           outputTemplate,
           outputFolder,
            popstationPath,
            icon0Path: '',
            pic0Path: '',
            pic1Path: '',
            discPaths: group.map((j) => j.filePath),
           subfolderPerGame
         };
         const result = await invokeCommand<{
           success: boolean;
           message: string;
           output_path: string | null;
           command_preview: string | null;
         }>('run_conversion', {
           filePath: primary.filePath,
           fileName: primary.fileName,
           options
         });

        for (const job of group) {
          updateJob(job.id, {
            status: result.success ? 'done' : 'error',
            message: result.message,
            outputPath: result.output_path,
            commandPreview: result.command_preview
          });
        }

        if (result.success) {
          succeeded++;
          showToast('success', `${primary.fileName} (${group.length} discs) done`);
        } else {
          failed++;
          showToast('error', `${primary.fileName} failed`);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        for (const job of group) {
          updateJob(job.id, { status: 'error', message });
        }
        failed++;
        showToast('error', `${primary.fileName}: ${message}`);
      }
    }

    async function runSingle(job: (typeof pending)[0]) {
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
            icon0Path: '',
            pic0Path: '',
            pic1Path: '',
            discPaths: [] as string[],
           subfolderPerGame
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

        if (result.success) {
          succeeded++;
          showToast('success', `${job.fileName} done`);
        } else {
          failed++;
          showToast('error', `${job.fileName} failed`);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        updateJob(job.id, { status: 'error', message });
        failed++;
        showToast('error', `${job.fileName}: ${message}`);
      }
    }

    for (const [, group] of groups) {
      if (group.length > 0) await runGroup(group);
    }

    for (const job of singles) {
      await runSingle(job);
    }

    if (succeeded > 0 && failed === 0) {
      showToast('success', 'All jobs finished');
    } else if (failed > 0) {
      showToast('error', `${failed} job(s) failed`);
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
    bindDragDropListener();
  });

  onDestroy(() => {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    if (unlistenDragDrop) {
      unlistenDragDrop();
      unlistenDragDrop = null;
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

<main
  class="app"
  class:drag-over={isDragOver}
  ondragenter={handleDragEnter}
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
>
  <TopBar bind:mode isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'} onAbout={() => (showAbout = true)} />

  {#if showAbout}
    <AboutDialog onClose={() => (showAbout = false)} />
  {/if}

  <div class="content">
    <InputPanel
      {mode}
      isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
      onAddJobs={addJobs}
    />

    <QueuePanel
      jobs={jobs.filter(j => j.mode === mode)}
      {progress}
      {mode}
      {selectedJobIds}
      isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
      collapsed={collapsedQueue}
      onToggle={() => (collapsedQueue = !collapsedQueue)}
      onRunAll={runAll}
      onClearQueue={clearQueue}
      onRemoveJob={removeJob}
      onRetryJob={retryJob}
      onToggleSelection={toggleJobSelection}
      onMergeSelected={mergeSelectedJobs}
      onUngroupJob={ungroupJob}
    />

      <ConvertOptions
        bind:mode
        bind:gameName
        bind:gameId
        bind:compression
        bind:outputTemplate
        bind:outputFolder
        bind:subfolderPerGame
        isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
        collapsed={collapsedOptions}
        onToggle={() => (collapsedOptions = !collapsedOptions)}
        onChooseOutputFolder={chooseOutputFolder}
        onGrabFromFile={autoGameId}
        onFetchMetadata={downloadGameInfo}
      />

    {#if showLog}
      <LogPanel
        {logLines}
        {outputFolder}
        {backendFile}
        {backendMessage}
        isRunning={progress.stage !== 'idle' && progress.stage !== 'completed'}
        onTestBackend={testBackend}
      />
    {/if}
  </div>

  <Toast notifications={toasts} onDismiss={dismissToast} />

  <button class="log-toggle" onclick={() => (showLog = !showLog)}>
    {showLog ? 'Hide Logs' : 'Show Logs'}
  </button>

</main>

<style>
  :root {
    --bg: #FFFFFF;
    --bg-secondary: #F3F6FB;
    --bg-tertiary: #F8FAFC;
    --bg-hover: #F3F6FB;
    --text: #1E2329;
    --text-secondary: #667085;
    --text-tertiary: #98A2B3;
    --border: #D7DCE3;
    --border-subtle: #E5E8ED;
    --accent: #2F7DF6;
    --accent-hover: #1F6FE5;
    --accent-bg: #EAF2FF;
    --accent-bg-hover: rgba(47, 125, 246, 0.10);
    --overlay: rgba(16, 24, 40, 0.5);
    --btn-text: #FFFFFF;
    --body-bg: #F5F6F8;
    --meta-tag-bg: rgba(47, 125, 246, 0.06);
    --meta-tag-border: #BFDBFE;
    --danger: #E5484D;
    --danger-bg: rgba(229, 72, 77, 0.08);
    --danger-border: rgba(229, 72, 77, 0.35);
  }

  :global(body) {
    margin: 0;
    background: var(--body-bg);
    color: var(--text);
    font-family:
      Inter,
      -apple-system,
      BlinkMacSystemFont,
      "SF Pro Display",
      "SF Pro Text",
      "Segoe UI",
      Roboto,
      Arial,
      sans-serif;
  }

  :global(button),
  :global(input),
  :global(select) {
    font: inherit;
  }

  .app {
    padding: 28px 24px 72px;
    position: relative;
  }

  .app.drag-over::after {
    content: 'Drop files here';
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 100;
    background: var(--accent-bg);
    border: 2px dashed var(--accent);
    border-radius: 14px;
    color: var(--accent);
    font-size: 24px;
    font-weight: 700;
    pointer-events: none;
    margin: 8px;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: min(100% - 48px, 960px);
    margin: 0 auto;
  }

  .log-toggle {
    position: fixed;
    right: 20px;
    bottom: 18px;
    z-index: 150;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 32px;
    padding: 0 14px;
    border-radius: 999px;
    border: 1px solid #D0D5DD;
    background: #FFFFFF;
    color: #667085;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.01em;
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(16, 24, 40, 0.10);
    transition: border-color 0.15s ease, color 0.15s ease;
  }

  .log-toggle:hover {
    border-color: #2F7DF6;
    color: #2F7DF6;
  }


</style>
