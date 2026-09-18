<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { i18n } from '../../i18n/index.js';

  const windowInstance = (() => {
    try {
      return getCurrentWindow();
    } catch {
      return null;
    }
  })();

  const handleClose = () => {
    windowInstance?.hide();
  };
</script>

<div class="win-controls" data-tauri-drag-region="false">
  <button
    type="button"
    class="win-btn close"
    onclick={handleClose}
    title={i18n.t.common.closeToTray}
    aria-label={i18n.t.common.closeToTray}
  >
    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
      <line x1="4" y1="4" x2="12" y2="12"></line>
      <line x1="12" y1="4" x2="4" y2="12"></line>
    </svg>
  </button>
</div>

<style>
  .win-controls {
    display: flex;
    align-items: center;
    height: 100%;
    flex-shrink: 0;
  }

  .win-btn {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm, 4px);
    color: var(--text-tertiary, #71717A);
    cursor: pointer;
    transition: all 0.15s cubic-bezier(0.16, 1, 0.3, 1);
    outline: none;
    box-sizing: border-box;
  }

  .win-btn svg {
    width: 13px;
    height: 13px;
  }

  .win-btn.close:hover {
    background: rgba(239, 68, 68, 0.12);
    color: #EF4444;
  }

  .win-btn.close:active {
    background: rgba(239, 68, 68, 0.2);
    color: #DC2626;
    transform: scale(0.95);
  }

  :global(.theme-dark) .win-btn {
    color: var(--text-tertiary, #88888E);
  }

  :global(.theme-light) .win-btn {
    color: var(--text-tertiary);
  }

  :global(.theme-light) .win-btn.close:hover {
    background: #FEE2E2;
    color: #DC2626;
  }

  :global(.theme-dark) .win-btn.close:hover {
    background: rgba(239, 68, 68, 0.18);
    color: #F87171;
  }
</style>
