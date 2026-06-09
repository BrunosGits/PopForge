<script lang="ts">
  import type { Mode } from '$lib/types';

  let {
    mode,
    isRunning,
    onAddSingle,
    onAddBatch
  }: {
    mode: Mode;
    isRunning: boolean;
    onAddSingle: () => void;
    onAddBatch: () => void;
  } = $props();
</script>

<section class="panel">
  <h2>Input</h2>

  <button class="drop-zone" onclick={onAddSingle} disabled={isRunning}>
    <span class="drop-title">
      {mode === 'convert' ? 'Drop ISO / BIN+CUE here or click to select' : 'Drop EBOOT.PBP here or click to select'}
    </span>
    <span class="drop-subtitle">Click to add a single file, or drag &amp; drop</span>
  </button>

  <div class="add-actions">
    <button class="btn-add" onclick={onAddBatch} disabled={isRunning}>
      <svg viewBox="0 0 16 16" fill="none" width="14" height="14">
        <path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
      Add Batch
    </button>
  </div>
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
    color: var(--text);
  }

  .drop-zone {
    width: 100%;
    min-height: 120px;
    border: 1.5px dashed var(--drop-border);
    border-radius: 10px;
    background: var(--bg-secondary);
    display: grid;
    place-items: center;
    gap: 8px;
    text-align: center;
    cursor: pointer;
  }

  .drop-zone:hover {
    background: var(--accent-bg);
  }

  .drop-zone:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .drop-title {
    display: block;
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
  }

  .drop-subtitle {
    display: block;
    font-size: 13px;
    font-weight: 400;
    color: var(--text-secondary);
  }

  .add-actions {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }

  .btn-add {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    line-height: 1;
    height: 34px;
    padding: 0 14px;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.01em;
    border: 1px solid var(--btn-border);
    border-radius: 8px;
    background: var(--bg);
    color: var(--btn-text-color);
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .btn-add:hover {
    background: var(--body-bg);
    border-color: var(--btn-hover-border);
  }

  .btn-add:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
</style>
