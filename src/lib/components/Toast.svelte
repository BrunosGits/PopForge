<script lang="ts">
  import type { ToastNotification } from '$lib/types';

  let {
    notifications,
    onDismiss
  }: {
    notifications: ToastNotification[];
    onDismiss: (id: number) => void;
  } = $props();
</script>

{#if notifications.length > 0}
  <div class="toast-container">
    {#each notifications as toast (toast.id)}
      <div class="toast {toast.type}" role="button" tabindex="0" onclick={() => onDismiss(toast.id)} onkeydown={(e) => e.key === 'Enter' && onDismiss(toast.id)}>
        <span class="toast-icon">
          {#if toast.type === 'success'}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="8" cy="8" r="7" stroke="#25A55F" stroke-width="1.5"/>
              <path d="M5 8l2 2 4-4" stroke="#25A55F" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          {:else if toast.type === 'error'}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="8" cy="8" r="7" stroke="#E5484D" stroke-width="1.5"/>
              <path d="M5.5 5.5l5 5M10.5 5.5l-5 5" stroke="#E5484D" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          {:else}
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <circle cx="8" cy="8" r="7" stroke="#2F7DF6" stroke-width="1.5"/>
              <path d="M8 5v4M8 11v0" stroke="#2F7DF6" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          {/if}
        </span>
        <span class="toast-message">{toast.message}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 58px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 300;
    display: flex;
    flex-direction: column;
    gap: 6px;
    pointer-events: none;
  }

  .toast {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 500;
    line-height: 1.3;
    cursor: pointer;
    pointer-events: auto;
    box-shadow: 0 4px 14px rgba(16, 24, 40, 0.12);
    animation: toast-in 0.2s ease-out;
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .toast.success {
    background: #ECFDF3;
    border: 1px solid #ABF0C6;
    color: #067647;
  }

  .toast.error {
    background: #FEF2F2;
    border: 1px solid #FECACA;
    color: #B42318;
  }

  .toast.info {
    background: #EAF2FF;
    border: 1px solid #BFDBFE;
    color: #1758A6;
  }

  .toast-icon {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .toast-message {
    white-space: nowrap;
  }
</style>
