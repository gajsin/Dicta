<script lang="ts">
  import type { Snippet } from 'svelte';
  import { i18n } from '../../i18n/index.js';
  import type { AppView } from '../../window/appListeners.js';

  export type SettingsSection = AppView;

  interface Props {
    activeSection: SettingsSection;
    onSelectSection: (section: SettingsSection) => void;
    children?: Snippet;
  }

  let {
    activeSection,
    onSelectSection,
    children,
  }: Props = $props();

  let isCollapsed = $state(false);
</script>

<div class="settings-layout-root">
  <!-- Desktop Collapsible Left Sidebar (144px open / 44px collapsed) with 0.7s smooth motion -->
  <aside class="settings-sidebar" class:collapsed={isCollapsed}>
    <div class="sidebar-nav-group" role="tablist">
      <!-- 1. История (History) -->
      <button
        type="button"
        role="tab"
        class="sidebar-nav-item"
        class:active={activeSection === 'history'}
        aria-selected={activeSection === 'history'}
        title={isCollapsed ? i18n.t.nav.history : undefined}
        onclick={() => onSelectSection('history')}
      >
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10"></circle>
          <polyline points="12 6 12 12 16 14"></polyline>
        </svg>
        <span class="nav-label">{i18n.t.nav.history}</span>
      </button>

      <!-- 2. Основные (General - Crisp Lucide Gear Icon) -->
      <button
        type="button"
        role="tab"
        class="sidebar-nav-item"
        class:active={activeSection === 'general'}
        aria-selected={activeSection === 'general'}
        title={isCollapsed ? i18n.t.nav.general : undefined}
        onclick={() => onSelectSection('general')}
      >
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
        <span class="nav-label">{i18n.t.nav.general}</span>
      </button>

      <!-- 3. Диктовка (Dictation) -->
      <button
        type="button"
        role="tab"
        class="sidebar-nav-item"
        class:active={activeSection === 'dictation'}
        aria-selected={activeSection === 'dictation'}
        title={isCollapsed ? i18n.t.nav.dictation : undefined}
        onclick={() => onSelectSection('dictation')}
      >
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path>
          <path d="M19 10v2a7 7 0 0 1-14 0v-2"></path>
          <line x1="12" y1="19" x2="12" y2="23"></line>
        </svg>
        <span class="nav-label">{i18n.t.nav.dictation}</span>
      </button>

      <!-- 4. Провайдеры (Providers) -->
      <button
        type="button"
        role="tab"
        class="sidebar-nav-item"
        class:active={activeSection === 'providers'}
        aria-selected={activeSection === 'providers'}
        title={isCollapsed ? i18n.t.nav.providers : undefined}
        onclick={() => onSelectSection('providers')}
      >
        <svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"></path>
        </svg>
        <span class="nav-label">{i18n.t.nav.providers}</span>
      </button>
    </div>

    <!-- Bottom Sidebar Toggle (0.5s smooth morphing collapse) -->
    <div class="sidebar-footer">
      <button
        type="button"
        class="collapse-toggle-btn"
        onclick={() => { isCollapsed = !isCollapsed; }}
        title={isCollapsed ? i18n.t.settings.expandSidebar : i18n.t.settings.collapseSidebar}
        aria-label={isCollapsed ? i18n.t.settings.expandSidebar : i18n.t.settings.collapseSidebar}
      >
        <svg
          class="collapse-arrow-icon"
          class:collapsed={isCollapsed}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
        <span class="collapse-label">{isCollapsed ? '' : i18n.t.settings.collapseSidebar}</span>
      </button>
    </div>
  </aside>

  <!-- Content Detail Stage -->
  <div class="settings-detail-stage">
    {#if activeSection === 'history'}
      {#if children}
        {@render children()}
      {/if}
    {:else}
      <div class="settings-detail-scroll">
        {#if children}
          {@render children()}
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-layout-root {
    display: flex;
    width: 100%;
    height: 100%;
    background: var(--bg-app);
    overflow: hidden;
  }

  /* Desktop Settings Sidebar: 144px open / 44px collapsed (180ms transition) */
  .settings-sidebar {
    width: 144px;
    min-width: 144px;
    max-width: 144px;
    height: 100%;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border-subtle);
    padding: 12px 6px 10px 6px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    user-select: none;
    flex-shrink: 0;
    transition: width 180ms cubic-bezier(0.16, 1, 0.3, 1),
                min-width 180ms cubic-bezier(0.16, 1, 0.3, 1),
                max-width 180ms cubic-bezier(0.16, 1, 0.3, 1),
                padding 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .settings-sidebar.collapsed {
    width: 44px;
    min-width: 44px;
    max-width: 44px;
    padding: 12px 6px 10px 6px;
  }

  .sidebar-nav-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
  }

  .sidebar-nav-item {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 450;
    color: var(--text-secondary);
    transition: background 100ms ease, color 100ms ease;
    outline: none;
    text-align: left;
    width: 100%;
    box-sizing: border-box;
    overflow: hidden;
  }

  .settings-sidebar.collapsed .sidebar-nav-item {
    padding: 0;
    justify-content: center;
    width: 30px;
    height: 30px;
    margin: 0 auto;
    gap: 0;
  }

  .settings-sidebar.collapsed .nav-icon {
    margin: 0;
    flex-shrink: 0;
  }

  .sidebar-nav-item:hover:not(.active) {
    color: var(--text-primary);
    background: var(--hover-bg, rgba(0, 0, 0, 0.04));
  }

  .sidebar-nav-item.active {
    background: var(--hover-bg, rgba(0, 0, 0, 0.04));
    color: var(--text-primary);
    font-weight: 500;
    border-color: transparent;
    box-shadow: none;
  }

  :global(.theme-dark) .sidebar-nav-item.active {
    background: rgba(255, 255, 255, 0.06);
    box-shadow: none;
    border-color: transparent;
  }

  .nav-icon {
    width: 14px;
    height: 14px;
    color: var(--text-tertiary);
    flex-shrink: 0;
    transition: color 120ms ease;
  }

  .sidebar-nav-item:hover .nav-icon {
    color: var(--text-primary);
  }

  .sidebar-nav-item.active .nav-icon {
    color: var(--accent-primary);
  }

  .nav-label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: opacity 140ms ease, max-width 180ms cubic-bezier(0.16, 1, 0.3, 1);
    max-width: 120px;
    opacity: 1;
  }

  .settings-sidebar.collapsed .nav-label {
    display: none;
    opacity: 0;
    max-width: 0;
    pointer-events: none;
  }

  /* Sidebar Footer (Collapse Toggle) */
  .sidebar-footer {
    width: 100%;
    padding-top: 6px;
    display: flex;
    justify-content: center;
  }

  .collapse-toggle-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    width: 100%;
    padding: 0 8px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    color: var(--text-tertiary);
    font-size: 11px;
    cursor: pointer;
    transition: background 100ms ease, color 100ms ease;
    outline: none;
    box-sizing: border-box;
    overflow: hidden;
  }

  .settings-sidebar.collapsed .collapse-toggle-btn {
    justify-content: center;
    padding: 0;
    width: 30px;
    height: 30px;
    margin: 0 auto;
    gap: 0;
  }

  .settings-sidebar.collapsed .collapse-arrow-icon {
    margin: 0;
    flex-shrink: 0;
  }

  .collapse-toggle-btn:hover {
    color: var(--text-primary);
    background: var(--hover-bg, rgba(0, 0, 0, 0.04));
  }

  .collapse-arrow-icon {
    width: 12px;
    height: 12px;
    flex-shrink: 0;
    transition: transform 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .collapse-arrow-icon.collapsed {
    transform: rotate(180deg);
  }

  .collapse-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: opacity 140ms ease, max-width 180ms cubic-bezier(0.16, 1, 0.3, 1);
    max-width: 100px;
    opacity: 1;
  }

  .settings-sidebar.collapsed .collapse-label {
    display: none;
    opacity: 0;
    max-width: 0;
    pointer-events: none;
  }

  /* Content Detail Stage */
  .settings-detail-stage {
    flex: 1;
    min-width: 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-app);
  }

  .settings-detail-scroll {
    flex: 1;
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
    padding: 16px 20px 32px 20px;
  }
</style>
