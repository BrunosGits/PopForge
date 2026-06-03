<script lang="ts">
  import type { Mode, ToolStatus } from '$lib/types';

  let {
    mode,
    isRunning,
    toolchain,
    onAddJobs,
    onRefreshTools,
    onChooseToolPath
  }: {
    mode: Mode;
    isRunning: boolean;
    toolchain: ToolStatus[];
    onAddJobs: () => void;
    onRefreshTools: () => void;
    onChooseToolPath: (name: string) => void;
  } = $props();
</script>

<aside class="sidebar">
  <section class="panel">
    <h2>Input</h2>

    <button class="drop-zone" onclick={onAddJobs} disabled={isRunning}>
      <span class="drop-title">
        {mode === 'convert' ? 'Drop ISO / BIN+CUE here or click to select' : 'Drop EBOOT.PBP here or click to select'}
      </span>
      <span class="drop-subtitle">Click to add files to the queue</span>
    </button>
  </section>

  <section class="panel">
    <div class="panel-header">
      <h2>Toolchain</h2>
      <button type="button" onclick={onRefreshTools} disabled={isRunning}>
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
      <button type="button" onclick={() => onChooseToolPath('psxpackager')} disabled={isRunning}>
        Set PSXPackager
      </button>
    </div>
  </section>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .panel {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--bg);
    padding: 16px;
  }

  h2 {
    margin: 0 0 14px;
    font-size: 15px;
  }

  .drop-zone {
    width: 100%;
    min-height: 150px;
    border: 1px dashed var(--accent);
    background: var(--accent-bg);
    display: grid;
    place-items: center;
    gap: 8px;
    text-align: center;
  }

  .drop-title {
    display: block;
    font-weight: 700;
  }

  .drop-subtitle {
    color: var(--text-secondary);
    font-size: 13px;
  }

  .panel-header,
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
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
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-secondary);
  }

  .tool-row span {
    display: block;
  }

  .tool-row span span {
    color: var(--text-secondary);
    font-size: 12px;
    margin-top: 3px;
  }

  .status {
    color: var(--text-secondary);
    text-transform: uppercase;
    font-size: 12px;
  }

  .status.done {
    color: #6ee785;
  }

  .status.error {
    color: #ff7b7b;
  }
</style>
