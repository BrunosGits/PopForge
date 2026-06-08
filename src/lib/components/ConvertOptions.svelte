<script lang="ts">
  import { slide } from 'svelte/transition';

  let {
    mode = $bindable(),
    gameName = $bindable(),
    gameId = $bindable(),
    compression = $bindable(),
    outputTemplate = $bindable(),
    outputFolder = $bindable(),
    subfolderPerGame = $bindable(),
    isRunning,
    collapsed,
    onToggle,
    onChooseOutputFolder,
    onGrabFromFile,
    onFetchMetadata,
  }: {
    mode: string;
    gameName: string;
    gameId: string;
    compression: number;
    outputTemplate: string;
    outputFolder: string;
    subfolderPerGame: boolean;
    isRunning: boolean;
    collapsed: boolean;
    onToggle: () => void;
    onChooseOutputFolder: () => void;
    onGrabFromFile: () => void;
    onFetchMetadata: () => void;
  } = $props();
</script>

<section class="panel">
  <button class="section-header" onclick={onToggle}>
    <h2>Convert Options</h2>
    <svg class="chevron" class:open={!collapsed} viewBox="0 0 16 16" fill="none">
      <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>
  </button>

  {#if !collapsed}
    {#if mode === 'convert'}
      <div transition:slide>
        <label>
          Game ID
          <div class="inline">
            <input bind:value={gameId} />
            <button type="button" class="btn-secondary" onclick={onGrabFromFile}>Grab from File</button>
            <span class="help-icon" data-tooltip="Extracts the serial number from the filename or disc image content. e.g. SCUS-94244">?</span>
          </div>
        </label>

        <label>
          Game Name
          <div class="inline">
            <input bind:value={gameName} placeholder="Tony Hawk's Pro Skater" />
            <button type="button" class="btn-secondary" onclick={onFetchMetadata}>Fetch Metadata</button>
            <span class="help-icon" data-tooltip="Looks up the game title from psxdatacenter.com using the file serial number. e.g. SCUS-94244">?</span>
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
      </div>
    {/if}

    <label class="toggle-row">
      <span>Subfolder per game</span>
      <button
        type="button"
        class="toggle"
        class:active={subfolderPerGame}
        aria-label={subfolderPerGame ? 'Disable subfolder per game' : 'Enable subfolder per game'}
        onclick={() => (subfolderPerGame = !subfolderPerGame)}
        role="switch"
        aria-checked={subfolderPerGame}
      >
        <span class="toggle-knob" class:on={subfolderPerGame}></span>
      </button>
    </label>

    {#if mode === 'convert'}
      <div transition:slide>
        <label>
          Output Filename Template
          <input bind:value={outputTemplate} class="mono" />
        </label>
      </div>
    {/if}

    <label>
      Output Folder
      <div class="inline">
        <input bind:value={outputFolder} readonly />
        <button type="button" class="btn-secondary" onclick={onChooseOutputFolder} disabled={isRunning}>
          Browse
        </button>
      </div>
    </label>

  {/if}
</section>

<style>
  .panel {
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--bg);
    padding: 18px;
    box-shadow: 0 1px 2px rgba(16, 24, 40, 0.04);
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    border: none;
    background: none;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  h2 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
    color: #1E2329;
  }

  .chevron {
    width: 14px;
    color: var(--text-secondary);
    transition: transform 0.15s;
  }

  .chevron.open {
    transform: rotate(90deg);
  }

  label {
    display: grid;
    gap: 6px;
    margin-top: 14px;
    color: #667085;
    font-size: 13px;
    font-weight: 500;
  }

  input,
  select {
    width: 100%;
    box-sizing: border-box;
    height: 38px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: #F8FAFC;
    color: #1E2329;
    padding: 0 12px;
    font-size: 13px;
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

  input::placeholder {
    color: #98A2B3;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: #2F7DF6;
    box-shadow: 0 0 0 3px rgba(47, 125, 246, 0.14);
    background: #FFFFFF;
  }

  .mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
  }

  .inline {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 8px;
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
    border: 1px solid #D0D5DD;
    border-radius: 8px;
    background: #FFFFFF;
    color: #344054;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .btn-secondary:hover {
    background: #F5F6F8;
    border-color: #C1C7CF;
  }

  .btn-secondary:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-direction: row;
  }

  .toggle {
    width: 36px;
    height: 20px;
    border: none;
    border-radius: 999px;
    background: #D0D5DD;
    cursor: pointer;
    padding: 0;
    position: relative;
    transition: background 0.15s ease;
    flex-shrink: 0;
  }

  .toggle.active {
    background: #2F7DF6;
  }

  .toggle-knob {
    display: block;
    width: 16px;
    height: 16px;
    border-radius: 999px;
    background: #FFFFFF;
    position: absolute;
    top: 2px;
    left: 2px;
    transition: transform 0.15s ease;
    box-shadow: 0 1px 2px rgba(16, 24, 40, 0.15);
  }

  .toggle-knob.on {
    transform: translateX(16px);
  }

  .help-icon {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 999px;
    border: 1px solid var(--text-tertiary);
    color: var(--text-tertiary);
    font-size: 11px;
    font-weight: 600;
    cursor: help;
    flex-shrink: 0;
  }

  .help-icon::after {
    content: attr(data-tooltip);
    position: absolute;
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
    white-space: nowrap;
    background: #1E2329;
    color: #fff;
    font-size: 12px;
    font-weight: 400;
    padding: 6px 10px;
    border-radius: 6px;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.1s ease;
  }

  .help-icon:hover::after {
    opacity: 1;
  }

</style>
