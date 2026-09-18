<script lang="ts">
  import WindowControls from '../ui/WindowControls.svelte';
  import DictaLogo from '../DictaLogo.svelte';
</script>

<header class="dicta-main-header" data-tauri-drag-region>
  <div class="header-left-group" data-tauri-drag-region>
    <div class="header-brand-slot" data-tauri-drag-region>
      <DictaLogo size={20} />
    </div>
  </div>

  <div class="header-drag-spacer" data-tauri-drag-region></div>

  <div class="header-right-group" data-tauri-drag-region="false">
    <div class="header-win-controls-slot">
      <WindowControls />
    </div>
  </div>
</header>

<style>
  .dicta-main-header {
    height: 60px;
    width: 100%;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px 0 0;
    box-sizing: border-box;
    user-select: none;
    flex-shrink: 0;
    position: absolute;
    inset: 0 0 auto 0;
    z-index: 50;
  }

  .header-left-group {
    display: flex;
    align-items: center;
    height: 100%;
  }

  .header-brand-slot {
    width: 44px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: default;
    flex-shrink: 0;
    transform: translateX(8px);
    transition: transform 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  :global(.window-shell:has(.settings-sidebar.collapsed)) .header-brand-slot {
    transform: translateX(0);
  }

  .header-drag-spacer {
    flex: 1;
    height: 100%;
  }

  .header-right-group {
    display: flex;
    align-items: center;
    height: 100%;
  }

  .header-win-controls-slot {
    display: flex;
    align-items: center;
  }
</style>
