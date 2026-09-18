<script lang="ts">
  import { fly } from 'svelte/transition';
  import { i18n } from '../../i18n/index.js';

  interface Props {
    message: string;
    undoText?: string;
    onUndo?: () => void;
  }

  let {
    message,
    undoText = i18n.t.common.undo,
    onUndo
  }: Props = $props();
</script>

{#if message}
  <div
    class="ui-toast"
    role="status"
    aria-live="polite"
    transition:fly={{ y: 12, duration: 150 }}
  >
    <span class="toast-message">{message}</span>
    {#if onUndo}
      <button type="button" class="toast-undo-btn" onclick={onUndo}>
        {undoText}
      </button>
    {/if}
  </div>
{/if}

<style>
  .ui-toast {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: var(--bg-surface-raised, #18191C);
    color: var(--text-primary, #F4F4F5);
    border: 1px solid var(--border-color, #2A2D35);
    padding: 8px 14px;
    border-radius: var(--radius-sm, 4px);
    font-size: 13px;
    font-weight: 500;
    box-shadow: var(--shadow-lg, 0 12px 32px rgba(0,0,0,0.18));
    z-index: 1000;
    display: flex;
    align-items: center;
    gap: 12px;
    user-select: none;
  }

  .toast-message {
    font-size: 13px;
    line-height: 18px;
  }

  .toast-undo-btn {
    border: none;
    background: var(--accent-primary, #FF9500);
    color: var(--accent-foreground, #FFFFFF);
    font-size: 12px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: var(--radius-xs, 2px);
    cursor: pointer;
    transition: background 100ms ease;
  }

  .toast-undo-btn:hover {
    background: var(--accent-hover, #E08500);
  }
</style>
