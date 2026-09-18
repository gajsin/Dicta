<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import { i18n } from '../../i18n/index.js';

  interface Props {
    title: string;
    description: string;
    confirmText?: string;
    cancelText?: string;
    onConfirm: () => void;
    onCancel: () => void;
  }

  let {
    title,
    description,
    confirmText = i18n.t.common.confirm,
    cancelText = i18n.t.common.cancel,
    onConfirm,
    onCancel,
  }: Props = $props();
</script>

  <div class="modal-backdrop" transition:fade={{ duration: 120 }} role="dialog" aria-modal="true">
    <div class="modal-card" transition:scale={{ start: 0.95, duration: 160, opacity: 0 }}>
      <h3 class="modal-title">{title}</h3>
      <p class="modal-desc">{description}</p>
      <div class="modal-actions">
        <button type="button" class="modal-btn secondary" onclick={onCancel}>{cancelText}</button>
        <button type="button" class="modal-btn danger-filled" onclick={onConfirm}>{confirmText}</button>
      </div>
    </div>
  </div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    background: var(--bg-surface, #FFFFFF);
    border: 1px solid var(--border-color, #E2E4E8);
    border-radius: var(--radius-card, 4px);
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.18);
    padding: 22px 24px;
    max-width: 400px;
    width: 90%;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  :global(.theme-dark) .modal-card {
    box-shadow: 0 16px 44px rgba(0, 0, 0, 0.6);
  }

  .modal-title {
    font-size: 15px;
    font-weight: 600;
    margin: 0;
    color: var(--text-primary, #111827);
  }

  .modal-desc {
    font-size: 12.5px;
    color: var(--text-secondary, #5A606E);
    margin: 0;
    line-height: 1.45;
  }

  .modal-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 10px;
  }

  .modal-btn {
    height: 32px;
    padding: 0 14px;
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid var(--border-color, #E2E4E8);
    background: var(--bg-surface, #FFFFFF);
    color: var(--text-primary, #111827);
    transition: background 120ms ease, border-color 120ms ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .modal-btn.secondary:hover {
    background: var(--hover-surface, #F4F4F5);
    border-color: var(--border-strong, #CDD0D7);
  }

  .modal-btn.danger-filled {
    background: #EF4444;
    color: #FFFFFF;
    border-color: transparent;
  }

  .modal-btn.danger-filled:hover {
    background: #DC2626;
  }
</style>
