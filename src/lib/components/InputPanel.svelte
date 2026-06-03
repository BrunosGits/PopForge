<script lang="ts">
  import type { Mode } from '$lib/types';

  let {
    mode,
    isRunning,
    onAddJobs
  }: {
    mode: Mode;
    isRunning: boolean;
    onAddJobs: () => void;
  } = $props();
</script>

<section class="panel">
  <h2>Input</h2>

  <button class="drop-zone" onclick={onAddJobs} disabled={isRunning}>
    <span class="drop-title">
      {mode === 'convert' ? 'Drop ISO / BIN+CUE here or click to select' : 'Drop EBOOT.PBP here or click to select'}
    </span>
    <span class="drop-subtitle">Click to add files to the queue</span>
  </button>
</section>

<style>
  .panel {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--bg);
    padding: 18px;
    box-shadow: 0 1px 2px rgba(16, 24, 40, 0.04);
  }

  h2 {
    margin: 0 0 14px;
    font-size: 15px;
    font-weight: 700;
    color: #1E2329;
  }

  .drop-zone {
    width: 100%;
    min-height: 150px;
    border: 1.5px dashed #7CADFF;
    border-radius: 10px;
    background: #F3F6FB;
    display: grid;
    place-items: center;
    gap: 8px;
    text-align: center;
    cursor: pointer;
  }

  .drop-zone:hover {
    background: #EAF2FF;
  }

  .drop-zone:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .drop-title {
    display: block;
    font-size: 15px;
    font-weight: 600;
    color: #1E2329;
  }

  .drop-subtitle {
    display: block;
    font-size: 13px;
    font-weight: 400;
    color: #667085;
  }
</style>
