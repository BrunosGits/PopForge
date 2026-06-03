<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';

  let {
    icon0Path = $bindable(),
    pic0Path = $bindable(),
    pic1Path = $bindable(),
    isRunning,
    onChooseAsset,
    onResetAsset,
    onPreviewAsset
  }: {
    icon0Path: string;
    pic0Path: string;
    pic1Path: string;
    isRunning: boolean;
    onChooseAsset: (name: string) => void;
    onResetAsset: (name: string) => void;
    onPreviewAsset: (name: string) => void;
  } = $props();

  let previewAsset: string | null = $state(null);

  function getPreviewPath(name: string): string {
    if (name === 'icon0') return icon0Path;
    if (name === 'pic0') return pic0Path;
    return pic1Path;
  }
</script>

<section class="panel">
  <h2>Customize PSP Assets</h2>

  <div class="asset-row">
    <div class="asset-label">ICON0.PNG</div>
    <div class="inline">
      <input
        value={icon0Path}
        readonly
        placeholder="Bundled default"
      />
      <button type="button" onclick={() => onChooseAsset('icon0')} disabled={isRunning}>
        Choose
      </button>
    </div>
    <div class="asset-actions">
      <button type="button" onclick={() => onResetAsset('icon0')} disabled={isRunning}>
        Reset
      </button>
      <button type="button" onclick={() => { previewAsset = previewAsset === 'icon0' ? null : 'icon0'; }} disabled={isRunning}>
        Preview
      </button>
    </div>
  </div>

  <div class="asset-row">
    <div class="asset-label">PIC0.PNG</div>
    <div class="inline">
      <input
        value={pic0Path}
        readonly
        placeholder="Bundled default"
      />
      <button type="button" onclick={() => onChooseAsset('pic0')} disabled={isRunning}>
        Choose
      </button>
    </div>
    <div class="asset-actions">
      <button type="button" onclick={() => onResetAsset('pic0')} disabled={isRunning}>
        Reset
      </button>
      <button type="button" onclick={() => { previewAsset = previewAsset === 'pic0' ? null : 'pic0'; }} disabled={isRunning}>
        Preview
      </button>
    </div>
  </div>

  <div class="asset-row">
    <div class="asset-label">PIC1.PNG</div>
    <div class="inline">
      <input
        value={pic1Path}
        readonly
        placeholder="Bundled default"
      />
      <button type="button" onclick={() => onChooseAsset('pic1')} disabled={isRunning}>
        Choose
      </button>
    </div>
    <div class="asset-actions">
      <button type="button" onclick={() => onResetAsset('pic1')} disabled={isRunning}>
        Reset
      </button>
      <button type="button" onclick={() => { previewAsset = previewAsset === 'pic1' ? null : 'pic1'; }} disabled={isRunning}>
        Preview
      </button>
    </div>
  </div>

  {#if previewAsset}
    {@const previewPath = getPreviewPath(previewAsset)}
    <div class="asset-preview">
      {#if previewPath}
        <img
          src={convertFileSrc(previewPath)}
          alt="{previewAsset.toUpperCase()} preview"
        />
      {:else}
        <p class="muted">No custom image selected. Preview shows bundled default only when set in the temp staging folder.</p>
      {/if}
      <button type="button" onclick={() => (previewAsset = null)}>Close Preview</button>
    </div>
  {/if}
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

  .asset-row {
    display: grid;
    gap: 6px;
    margin-bottom: 14px;
    padding: 10px;
    border: 1px solid #2c2c2c;
    border-radius: 10px;
    background: rgba(26, 26, 26, 0.45);
  }

  .asset-label {
    color: #cfe1ff;
    font-size: 13px;
    font-weight: 600;
  }

  .asset-actions {
    display: flex;
    gap: 8px;
  }

  .asset-preview {
    display: grid;
    gap: 8px;
    margin-top: 6px;
    padding: 10px;
    border: 1px dashed #5b9cf6;
    border-radius: 10px;
    background: rgba(91, 156, 246, 0.05);
  }

  .asset-preview img {
    max-width: 100%;
    max-height: 180px;
    object-fit: contain;
    border-radius: 6px;
    background: #1a1a1a;
  }

  .inline {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  input {
    width: 100%;
    border: 1px solid #333;
    border-radius: 8px;
    background: #1a1a1a;
    color: #f2f2f2;
    padding: 9px 10px;
  }

  input:focus {
    outline: none;
    border-color: #5b9cf6;
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
