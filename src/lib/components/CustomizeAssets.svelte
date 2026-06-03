<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';

  let {
    icon0Path = $bindable(),
    pic0Path = $bindable(),
    pic1Path = $bindable(),
    isRunning,
    collapsed,
    onToggle,
    onChooseAsset,
    onResetAsset,
    onPreviewAsset
  }: {
    icon0Path: string;
    pic0Path: string;
    pic1Path: string;
    isRunning: boolean;
    collapsed: boolean;
    onToggle: () => void;
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
  <button class="section-header" onclick={onToggle}>
    <h2>Customize PSP Assets</h2>
    <svg class="chevron" class:open={!collapsed} viewBox="0 0 16 16" fill="none">
      <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
    </svg>
  </button>

  {#if !collapsed}
    <div class="asset-row">
      <div class="asset-label">ICON0.PNG</div>
      <div class="inline">
        <input
          value={icon0Path}
          readonly
          placeholder="Bundled default"
        />
        <button type="button" class="btn-secondary" onclick={() => onChooseAsset('icon0')} disabled={isRunning}>
          Choose
        </button>
      </div>
      <div class="asset-actions">
        <button type="button" class="button-small" onclick={() => onResetAsset('icon0')} disabled={isRunning}>
          Reset
        </button>
        <button type="button" class="button-small" onclick={() => { previewAsset = previewAsset === 'icon0' ? null : 'icon0'; }} disabled={isRunning}>
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
        <button type="button" class="btn-secondary" onclick={() => onChooseAsset('pic0')} disabled={isRunning}>
          Choose
        </button>
      </div>
      <div class="asset-actions">
        <button type="button" class="button-small" onclick={() => onResetAsset('pic0')} disabled={isRunning}>
          Reset
        </button>
        <button type="button" class="button-small" onclick={() => { previewAsset = previewAsset === 'pic0' ? null : 'pic0'; }} disabled={isRunning}>
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
        <button type="button" class="btn-secondary" onclick={() => onChooseAsset('pic1')} disabled={isRunning}>
          Choose
        </button>
      </div>
      <div class="asset-actions">
        <button type="button" class="button-small" onclick={() => onResetAsset('pic1')} disabled={isRunning}>
          Reset
        </button>
        <button type="button" class="button-small" onclick={() => { previewAsset = previewAsset === 'pic1' ? null : 'pic1'; }} disabled={isRunning}>
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
        <button type="button" class="button-small" onclick={() => (previewAsset = null)}>Close Preview</button>
      </div>
    {/if}
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

  .asset-row {
    display: grid;
    gap: 6px;
    margin-top: 14px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--bg-tertiary);
  }

  .asset-label {
    color: #2F7DF6;
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
    padding: 12px;
    border: 1px dashed #7CADFF;
    border-radius: 10px;
    background: #EAF2FF;
  }

  .asset-preview img {
    max-width: 100%;
    max-height: 180px;
    object-fit: contain;
    border-radius: 6px;
    background: #F8FAFC;
  }

  .inline {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  input {
    width: 100%;
    box-sizing: border-box;
    height: 38px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: #FFFFFF;
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

  input:focus {
    outline: none;
    border-color: #2F7DF6;
    box-shadow: 0 0 0 3px rgba(47, 125, 246, 0.14);
  }

  .muted {
    color: #667085;
    font-size: 13px;
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

  .button-small {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 30px;
    padding: 0 12px;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.01em;
    border: 1px solid #D0D5DD;
    border-radius: 7px;
    background: #FFFFFF;
    color: #344054;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .button-small:hover {
    background: #F5F6F8;
    border-color: #C1C7CF;
  }

  .button-small:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }
</style>
