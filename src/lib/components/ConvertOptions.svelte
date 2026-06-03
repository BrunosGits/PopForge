<script lang="ts">
  let {
    gameName = $bindable(),
    gameId = $bindable(),
    compression = $bindable(),
    outputTemplate = $bindable(),
    outputFolder = $bindable(),
    isRunning,
    onChooseOutputFolder,
    onAutoGameId
  }: {
    gameName: string;
    gameId: string;
    compression: number;
    outputTemplate: string;
    outputFolder: string;
    isRunning: boolean;
    onChooseOutputFolder: () => void;
    onAutoGameId: () => void;
  } = $props();
</script>

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
      <button type="button" onclick={onAutoGameId}>Auto</button>
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
      <button type="button" onclick={onChooseOutputFolder} disabled={isRunning}>
        Browse
      </button>
    </div>
  </label>
</section>

<style>
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

  label {
    display: grid;
    gap: 6px;
    margin-bottom: 12px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  input,
  select {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-tertiary);
    color: var(--text);
    padding: 9px 10px;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .inline {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  button {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-hover);
    color: var(--text);
    padding: 8px 12px;
    cursor: pointer;
  }

  button:hover {
    border-color: var(--accent);
    color: var(--accent-hover);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
</style>
