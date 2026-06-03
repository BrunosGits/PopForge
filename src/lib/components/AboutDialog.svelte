<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();
</script>

<div class="overlay" role="dialog" aria-modal="true">
  <div class="modal">
    <h2>PopForge</h2>
    <p class="version">v0.1.0</p>
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
    background: rgba(0, 0, 0, 0.6);
  }

  .modal {
    border: 1px solid #333;
    border-radius: 14px;
    background: #1a1a1a;
    padding: 32px;
    max-width: 420px;
    width: 90%;
    text-align: center;
  }

  h2 {
    margin: 0 0 4px;
    font-size: 24px;
    color: #5b9cf6;
  }

  .version {
    margin: 0 0 16px;
    color: #a0a0a0;
    font-size: 13px;
  }

  .description {
    margin: 0 0 16px;
    color: #ccc;
    font-size: 14px;
    line-height: 1.5;
  }

  .details {
    margin: 0 0 16px;
    padding: 12px;
    border: 1px solid #2c2c2c;
    border-radius: 8px;
    background: rgba(26, 26, 26, 0.65);
  }

  .details p {
    margin: 4px 0;
    color: #a0a0a0;
    font-size: 12px;
  }

  .links {
    margin: 0 0 20px;
  }

  .links a {
    color: #5b9cf6;
    text-decoration: none;
    font-size: 13px;
  }

  .links a:hover {
    text-decoration: underline;
  }

  .close {
    border: 1px solid #5b9cf6;
    border-radius: 8px;
    background: #5b9cf6;
    color: #08111f;
    padding: 8px 24px;
    font-size: 14px;
    cursor: pointer;
  }

  .close:hover {
    background: #8bbcff;
  }
</style>
