<script lang="ts">
  import type { Mode } from '$lib/types';

  let { mode = $bindable(), isRunning, onAbout }: { mode: Mode; isRunning: boolean; onAbout: () => void } = $props();
</script>

<header class="topbar">
  <div class="title-row">
    <div>
      <p class="eyebrow">PSX · PSP · Vita</p>
      <h1>PopForge</h1>
    </div>
    <button class="btn-secondary" onclick={onAbout}>About</button>
  </div>

  <div class="mode-toggle">
    <button
      class:active={mode === 'convert'}
      onclick={() => (mode = 'convert')}
      disabled={isRunning}
    >
      Convert
    </button>
    <button
      class:active={mode === 'extract'}
      onclick={() => (mode = 'extract')}
      disabled={isRunning}
    >
      Extract
    </button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 20px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
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

  .btn-secondary:hover {
    background: var(--body-bg);
    border-color: var(--btn-hover-border);
  }

  .eyebrow {
    margin: 0 0 4px;
    color: var(--accent);
    letter-spacing: 0.16em;
    text-transform: uppercase;
    font-size: 13px;
    font-weight: 600;
  }

  h1 {
    margin: 0;
    font-size: 34px;
    font-weight: 700;
    color: var(--text);
  }

  .mode-toggle {
    display: inline-flex;
    align-items: center;
    background: var(--toggle-bg);
    border-radius: 999px;
    padding: 3px;
    gap: 2px;
    height: 30px;
  }

  .mode-toggle button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 24px;
    padding: 0 14px;
    border-radius: 999px;
    border: none;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.01em;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, color 0.15s ease, box-shadow 0.15s ease;
    background: transparent;
    color: var(--text-secondary);
  }

  .mode-toggle button.active {
    background: var(--accent);
    color: var(--btn-text);
    box-shadow: 0 1px 3px rgba(47, 125, 246, 0.30), 0 1px 1px rgba(0, 0, 0, 0.06);
  }

  .mode-toggle button:not(.active):hover {
    color: var(--text);
  }

  .mode-toggle button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
</style>
