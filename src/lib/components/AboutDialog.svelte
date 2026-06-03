<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();
</script>

<div class="overlay" role="dialog" aria-modal="true">
  <div class="modal">
    <h2>PopForge</h2>
    <p class="version">v0.4.0-alpha</p>
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
    border: 1px solid #E3E8EF;
    border-radius: 14px;
    background: #FFFFFF;
    padding: 32px;
    max-width: 420px;
    width: 90%;
    text-align: center;
    box-shadow: 0 8px 24px rgba(16, 24, 40, 0.12);
  }

  h2 {
    margin: 0 0 4px;
    font-size: 24px;
    color: #2F7DF6;
  }

  .version {
    margin: 0 0 16px;
    color: #667085;
    font-size: 13px;
    font-weight: 500;
  }

  .description {
    margin: 0 0 16px;
    color: #1E2329;
    font-size: 14px;
    line-height: 1.5;
  }

  .details {
    margin: 0 0 16px;
    padding: 12px;
    border: 1px solid #E3E8EF;
    border-radius: 8px;
    background: #F8FAFC;
  }

  .details p {
    margin: 4px 0;
    color: #667085;
    font-size: 12px;
  }

  .links {
    margin: 0 0 20px;
  }

  .links a {
    color: #2F7DF6;
    text-decoration: none;
    font-size: 13px;
  }

  .links a:hover {
    text-decoration: underline;
  }

  .close {
    border: 1px solid #2F7DF6;
    border-radius: 8px;
    background: #2F7DF6;
    color: #FFFFFF;
    padding: 8px 24px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
  }

  .close:hover {
    background: #1F6FE5;
    border-color: #1F6FE5;
  }
</style>
