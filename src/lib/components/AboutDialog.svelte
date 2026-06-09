<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();
</script>

<div class="overlay" role="dialog" aria-modal="true">
  <div class="modal">
    <h2>PopForge</h2>
    <p class="version">v0.5.0-alpha</p>
    <p class="description">
      PS1 ↔ PSP disc image conversion tool.<br />
      Converts PS1 ISO, BIN+CUE to PSP EBOOT.PBP and back.
    </p>
    <div class="details">
      <p>Built with Tauri 2 + SvelteKit 2 + Svelte 5</p>
      <p>Uses PSXPackager for disc conversion</p>
    </div>
    <div class="links">
      <a href="https://github.com/BrunosGits/PopForge" target="_blank" rel="noopener">
        GitHub Repository
      </a>
    </div>
    <button class="close" onclick={onClose}>Close</button>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    z-index: 200;
    background: rgba(0, 0, 0, 0.4);
  }

  .modal {
    border: 1px solid var(--modal-border);
    border-radius: 14px;
    background: var(--bg);
    padding: 32px;
    max-width: 420px;
    width: 90%;
    text-align: center;
    box-shadow: 0 8px 24px rgba(16, 24, 40, 0.12);
  }

  h2 {
    margin: 0 0 4px;
    font-size: 24px;
    color: var(--accent);
  }

  .version {
    margin: 0 0 16px;
    color: var(--text-secondary);
    font-size: 13px;
    font-weight: 500;
  }

  .description {
    margin: 0 0 16px;
    color: var(--text);
    font-size: 14px;
    line-height: 1.5;
  }

  .details {
    margin: 0 0 16px;
    padding: 12px;
    border: 1px solid var(--modal-border);
    border-radius: 8px;
    background: var(--bg-tertiary);
  }

  .details p {
    margin: 4px 0;
    color: var(--text-secondary);
    font-size: 12px;
  }

  .links {
    margin: 0 0 20px;
  }

  .links a {
    color: var(--accent);
    text-decoration: none;
    font-size: 13px;
  }

  .links a:hover {
    text-decoration: underline;
  }

  .close {
    border: 1px solid var(--accent);
    border-radius: 8px;
    background: var(--accent);
    color: var(--btn-text);
    padding: 8px 24px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .close:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
  }
</style>
