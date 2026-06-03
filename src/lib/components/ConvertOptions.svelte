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
      <button type="button" on:click={onAutoGameId}>Auto</button>
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
      <button type="button" on:click={onChooseOutputFolder} disabled={isRunning}>
        Browse
      </button>
    </div>
  </label>

  <p class="muted">
    CHD inputs are normalized with chdman before PopForge runs PSXPackager.
  </p>
</section>

<style>
  .panel {
    border: 1px solid #333;
    border-radius: 14px;
    background: rgba(36, 36, 36, 0.86);
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

  .muted {
    color: #a0a0a0;
    font-size: 13px;
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
</style>
