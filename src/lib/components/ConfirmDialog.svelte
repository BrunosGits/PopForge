<script lang="ts">
  let { title, message, confirmLabel = 'Clear', onConfirm, onCancel }: {
    title: string;
    message: string;
    confirmLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();
</script>

<div class="overlay" role="alertdialog" aria-modal="true" aria-labelledby="confirm-title">
  <div class="modal">
    <h2 id="confirm-title">{title}</h2>
    <p>{message}</p>
    <div class="actions">
      <button class="btn-cancel" onclick={onCancel}>Cancel</button>
      <button class="btn-confirm" onclick={onConfirm}>{confirmLabel}</button>
    </div>
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
    padding: 28px;
    max-width: 380px;
    width: 90%;
    text-align: center;
    box-shadow: 0 8px 24px rgba(16, 24, 40, 0.12);
  }

  h2 {
    margin: 0 0 8px;
    font-size: 18px;
    color: var(--text);
  }

  p {
    margin: 0 0 24px;
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.5;
  }

  .actions {
    display: flex;
    gap: 10px;
    justify-content: center;
  }

  .btn-cancel {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 36px;
    padding: 0 18px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.01em;
    border: 1px solid var(--btn-border);
    border-radius: 8px;
    background: var(--bg);
    color: var(--btn-text-color);
    cursor: pointer;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .btn-cancel:hover {
    background: var(--body-bg);
    border-color: var(--btn-hover-border);
  }

  .btn-confirm {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    height: 36px;
    padding: 0 18px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.01em;
    border: 1px solid var(--danger);
    border-radius: 8px;
    background: var(--danger);
    color: var(--btn-text);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-confirm:hover {
    background: var(--btn-danger-hover);
  }
</style>
