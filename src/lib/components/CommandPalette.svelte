<script lang="ts">
  import { fade, scale } from 'svelte/transition';
  import type { HistoryItem } from '../history/storage.js';
  import { i18n } from '../i18n/index.js';
  import { formatDuration } from '../utils/dateTime.js';

  interface CommandPaletteProps {
    visible?: boolean;
    historyItems?: HistoryItem[];
    currentTheme?: string;
    hotkey?: string;
    onClose?: () => void;
    onSelectHistory?: (id: string) => void;
    onNavigateView?: (view: 'history' | 'general' | 'dictation' | 'providers') => void;
    onSetMode?: (mode: 'raw' | 'minimal' | 'balanced' | 'business') => void;
    onToggleTheme?: () => void;
    onTriggerRecording?: () => void;
  }

  let {
    visible = false,
    historyItems = [],
    currentTheme = 'light',
    hotkey = 'F9',
    onClose = () => {},
    onSelectHistory = () => {},
    onNavigateView = () => {},
    onSetMode = () => {},
    onToggleTheme = () => {},
    onTriggerRecording = () => {}
  }: CommandPaletteProps = $props();

  let query = $state('');
  let selectedIndex = $state(0);
  let inputRef = $state<HTMLInputElement | null>(null);

  interface CommandItem {
    id: string;
    title: string;
    description?: string;
    shortcut?: string;
    icon: string;
    action: () => void;
  }

  const isRu = $derived(i18n.resolved === 'ru');

  const baseCommands = $derived.by<CommandItem[]>(() => {
    const list: CommandItem[] = [
      {
        id: 'cmd-record',
        title: i18n.t.topbar.record,
        shortcut: hotkey,
        icon: 'mic',
        action: () => { onTriggerRecording(); onClose(); }
      },
      {
        id: 'cmd-nav-history',
        title: i18n.t.nav.history,
        description: i18n.t.topbar.historySubtitle,
        icon: 'history',
        action: () => { onNavigateView('history'); onClose(); }
      },
      {
        id: 'cmd-nav-general',
        title: i18n.t.nav.general,
        description: i18n.t.settings.hotkey,
        icon: 'settings',
        action: () => { onNavigateView('general'); onClose(); }
      },
      {
        id: 'cmd-nav-dictation',
        title: i18n.t.nav.dictation,
        description: i18n.t.topbar.dictationTitle,
        icon: 'mic',
        action: () => { onNavigateView('dictation'); onClose(); }
      },
      {
        id: 'cmd-nav-providers',
        title: i18n.t.nav.providers,
        description: i18n.t.settings.providersTitle,
        icon: 'key',
        action: () => { onNavigateView('providers'); onClose(); }
      },
      {
        id: 'cmd-theme',
        title: currentTheme === 'light'
          ? i18n.t.common.switchToDarkTheme
          : i18n.t.common.switchToLightTheme,
        description: i18n.t.settings.theme,
        icon: currentTheme === 'light' ? 'moon' : 'sun',
        action: () => { onToggleTheme(); onClose(); }
      },
      {
        id: 'cmd-mode-raw',
        title: `${i18n.t.settings.processingMode}: ${i18n.t.modes.raw}`,
        description: isRu ? 'Сохранять дословную транскрипцию без LLM' : 'Exact literal transcription without LLM',
        icon: 'check',
        action: () => { onSetMode('raw'); onClose(); }
      },
      {
        id: 'cmd-mode-minimal',
        title: `${i18n.t.settings.processingMode}: ${i18n.t.modes.minimal}`,
        description: isRu ? 'Исправлять только опечатки и пунктуацию' : 'Fix typos and punctuation only',
        icon: 'check',
        action: () => { onSetMode('minimal'); onClose(); }
      },
      {
        id: 'cmd-mode-balanced',
        title: `${i18n.t.settings.processingMode}: ${i18n.t.modes.balanced}`,
        description: isRu ? 'Убирать повторы и слова-паразиты' : 'Remove filler words and repetitions',
        icon: 'check',
        action: () => { onSetMode('balanced'); onClose(); }
      },
      {
        id: 'cmd-mode-business',
        title: `${i18n.t.settings.processingMode}: ${i18n.t.modes.business}`,
        description: isRu ? 'Преобразовывать в четкий деловой стиль' : 'Transform into crisp professional style',
        icon: 'check',
        action: () => { onSetMode('business'); onClose(); }
      }
    ];
    return list;
  });

  const filteredItems = $derived.by<CommandItem[]>(() => {
    const q = query.trim().toLowerCase();
    const historyCmds: CommandItem[] = historyItems.map(h => ({
      id: `history-${h.id}`,
      title: h.processedText,
      description: `${formatDuration(h.duration)} · ${h.timestamp ? new Date(h.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : ''}`,
      icon: 'document',
      action: () => { onNavigateView('history'); onSelectHistory(h.id); onClose(); }
    }));

    const all = [...baseCommands, ...historyCmds];
    if (!q) return all.slice(0, 10);

    return all.filter(item =>
      item.title.toLowerCase().includes(q) ||
      (item.description && item.description.toLowerCase().includes(q))
    ).slice(0, 12);
  });

  $effect(() => {
    if (visible) {
      query = '';
      selectedIndex = 0;
      setTimeout(() => inputRef?.focus(), 50);
    }
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!visible) return;

    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % Math.max(1, filteredItems.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filteredItems.length) % Math.max(1, filteredItems.length);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = filteredItems[selectedIndex];
      if (item) item.action();
    }
  };
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if visible}
  <div
    class="palette-backdrop"
    transition:fade={{ duration: 120 }}
    onclick={onClose}
    onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}
    role="presentation"
  >
    <div
      class="palette-card"
      transition:scale={{ duration: 150, start: 0.96 }}
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Search input header -->
      <div class="palette-header">
        <svg class="palette-search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          bind:this={inputRef}
          type="text"
          class="palette-input"
          placeholder="{isRu ? 'Поиск по командам и истории' : 'Search commands and history'} (Ctrl+K)…"
          bind:value={query}
        />
        <kbd class="palette-esc-badge">Esc</kbd>
      </div>

      <!-- Command list -->
      <div class="palette-results">
        {#if filteredItems.length === 0}
          <div class="palette-empty">{i18n.t.history.searchEmptyTitle}</div>
        {:else}
          {#each filteredItems as item, idx (item.id)}
            <div
              class="palette-item"
              class:selected={idx === selectedIndex}
              onclick={item.action}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') item.action(); }}
              onmouseenter={() => { selectedIndex = idx; }}
              role="option"
              aria-selected={idx === selectedIndex}
              tabindex="0"
            >
              <div class="item-icon-wrapper">
                {#if item.icon === 'mic'}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="23"></line></svg>
                {:else if item.icon === 'history'}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                {:else if item.icon === 'settings'}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                {:else if item.icon === 'sun'}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="5"></circle><line x1="12" y1="1" x2="12" y2="3"></line><line x1="12" y1="21" x2="12" y2="23"></line><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line><line x1="1" y1="12" x2="3" y2="12"></line><line x1="21" y1="12" x2="23" y2="12"></line><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line></svg>
                {:else if item.icon === 'moon'}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
                {:else}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline></svg>
                {/if}
              </div>

              <div class="item-text-group">
                <span class="item-title">{item.title}</span>
                {#if item.description}
                  <span class="item-desc">{item.description}</span>
                {/if}
              </div>

              {#if item.shortcut}
                <kbd class="item-shortcut">{item.shortcut}</kbd>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    z-index: 1000;
  }

  .palette-card {
    width: 520px;
    max-width: 90vw;
    background: var(--bg-surface, #FFFFFF);
    border: 1px solid var(--border-color, #E2E4E8);
    border-radius: var(--radius-card, 4px);
    box-shadow: 0 12px 36px rgba(0, 0, 0, 0.16);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  :global(.theme-dark) .palette-card {
    box-shadow: 0 16px 44px rgba(0, 0, 0, 0.6);
  }

  .palette-header {
    display: flex;
    align-items: center;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-subtle, #ECEEF2);
    gap: 8px;
    background: var(--bg-surface, #FFFFFF);
  }

  .palette-search-icon {
    width: 15px;
    height: 15px;
    color: var(--text-tertiary, #8C92A0);
    flex-shrink: 0;
  }

  .palette-input {
    flex: 1;
    border: none;
    background: transparent;
    font-size: 13.5px;
    font-family: var(--font-sans, inherit);
    color: var(--text-primary, #111827);
    outline: none;
  }

  .palette-input::placeholder {
    color: var(--text-tertiary, #8C92A0);
  }

  .palette-esc-badge {
    font-size: 10px;
    font-family: var(--font-mono, monospace);
    color: var(--text-tertiary, #8C92A0);
    background: var(--bg-muted, #ECEEF2);
    border: 1px solid var(--border-color, #E2E4E8);
    border-radius: var(--radius-xs, 3px);
    padding: 1px 5px;
  }

  .palette-results {
    max-height: 320px;
    overflow-y: auto;
    padding: 4px;
    scrollbar-width: thin;
    scrollbar-color: var(--border-color) transparent;
  }

  .palette-results::-webkit-scrollbar {
    width: 4px;
  }

  .palette-results::-webkit-scrollbar-thumb {
    background: var(--border-color);
    border-radius: 2px;
  }

  .palette-empty {
    padding: 24px;
    text-align: center;
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .palette-item {
    display: flex;
    align-items: center;
    padding: 7px 10px;
    border-radius: var(--radius-sm, 4px);
    gap: 10px;
    cursor: pointer;
    user-select: none;
    transition: background 80ms ease;
    min-height: 34px;
    box-sizing: border-box;
  }

  .palette-item.selected {
    background: var(--hover-surface, #F0F2F5);
  }

  :global(.theme-dark) .palette-item.selected {
    background: #27272A;
  }

  .palette-item:hover:not(.selected) {
    background: var(--hover-bg, rgba(0, 0, 0, 0.03));
  }

  .item-icon-wrapper {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-xs, 4px);
    background: var(--bg-muted, #ECEEF2);
    border: 1px solid var(--border-color, #E2E4E8);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary, #5A606E);
    flex-shrink: 0;
  }

  .item-icon-wrapper svg {
    width: 12px;
    height: 12px;
  }

  .item-text-group {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .item-title {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-primary, #111827);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .item-desc {
    font-size: 11px;
    color: var(--text-tertiary, #8C92A0);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.3;
  }

  .item-shortcut {
    font-size: 10px;
    font-family: var(--font-mono, monospace);
    font-weight: 500;
    color: var(--text-tertiary, #8C92A0);
    background: var(--bg-muted, #ECEEF2);
    padding: 1px 5px;
    border-radius: var(--radius-xs, 3px);
    border: 1px solid var(--border-color, #E2E4E8);
  }
</style>
