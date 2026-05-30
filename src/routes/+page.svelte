<script lang="ts">
  type Mode = 'convert' | 'extract';
  type JobStatus = 'pending' | 'running' | 'done' | 'error';

  type Job = {
    id: number;
    fileName: string;
    mode: Mode;
    status: JobStatus;
  };

  let mode: Mode = 'convert';
  let gameName = '';
  let gameId = 'AUTO';
  let compression = 9;
  let outputTemplate = '[%GAMEID%] %TITLE% (%REGION%)';

  let jobs: Job[] = [
    {
      id: 1,
      fileName: 'Example Game.cue',
      mode: 'convert',
      status: 'pending'
    }
  ];

  function addMockJob() {
    jobs = [
      ...jobs,
      {
        id: Date.now(),
        fileName: mode === 'convert' ? 'New Disc Image.cue' : 'EBOOT.PBP',
        mode,
        status: 'pending'
      }
    ];
  }

  function clearQueue() {
    jobs = [];
  }

  function runAll() {
    jobs = jobs.map((job) => ({ ...job, status: 'done' }));
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

        <button class="drop-zone" on:click={addMockJob}>
          <span class="drop-title">
            {mode === 'convert' ? 'Drop ISO / BIN+CUE here' : 'Drop EBOOT.PBP here'}
          </span>
          <span class="drop-subtitle">Click to add a mock job for now</span>
        </button>
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
        </section>
      {:else}
        <section class="panel">
          <h2>Extract Options</h2>

          <p class="muted">
            Extract mode will unpack an existing EBOOT.PBP back into a disc image.
          </p>
        </section>
      {/if}
    </aside>

    <section class="main-panel">
      <section class="panel queue-panel">
        <div class="panel-header">
          <h2>Queue</h2>

          <div class="actions">
            <button on:click={clearQueue}>Clear</button>
            <button class="primary" on:click={runAll}>Run All</button>
          </div>
        </div>

        {#if jobs.length === 0}
          <div class="empty">
            No jobs yet. Add files from the input panel.
          </div>
        {:else}
          <div class="jobs">
            {#each jobs as job}
              <article class="job">
                <div>
                  <strong>{job.fileName}</strong>
                  <span>{job.mode}</span>
                </div>

                <span class:done={job.status === 'done'} class="status">
                  {job.status}
                </span>
              </article>
            {/each}
          </div>
        {/if}
      </section>

      <section class="panel log-panel">
        <h2>Log</h2>
        <pre>[ready] PopForge initialized.
[info] Conversion engine will be connected next.</pre>
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
