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
    <button class="about-btn" onclick={onAbout}>About</button>
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
    margin-bottom: 16px;
  }

  .title-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .about-btn {
    border: 1px solid #333;
    border-radius: 6px;
    background: transparent;
    color: #a0a0a0;
    padding: 4px 10px;
    font-size: 12px;
    cursor: pointer;
    margin-top: 8px;
  }

  .about-btn:hover {
    border-color: #5b9cf6;
    color: #5b9cf6;
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

  button.active {
    border-color: #5b9cf6;
    background: #5b9cf6;
    color: #08111f;
  }

  @media (max-width: 880px) {
    .topbar {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
