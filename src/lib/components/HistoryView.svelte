<script lang="ts">
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import type { HistoryItem } from '../history/storage.js';
  import { getItemTitle } from '../history/storage.js';
  import { i18n } from '../i18n/index.js';
  import MorphIcon from './ui/MorphIcon.svelte';
  import { computeWordDiff } from '../utils/diff.js';
  import { formatDuration, formatTime, formatDateHeader } from '../utils/dateTime.js';

  interface Props {
    historyList?: HistoryItem[];
    selectedId?: string | null;
    searchQuery?: string;
    saveHistory?: boolean;
    onSearchChange?: (q: string) => void;
    onDelete?: (id: string) => void;
    onCopyText?: (text: string) => void;
    onOpenPrivacySettings?: () => void;
  }

  let {
    historyList = [],
    selectedId = $bindable(null),
    searchQuery = '',
    saveHistory = true,
    onSearchChange = () => {},
    onDelete = () => {},
    onCopyText = () => {},
    onOpenPrivacySettings = () => {}
  }: Props = $props();

  let expandedId = $state<string | null>(null);
  let activeTabMap = $state<Record<string, 'processed' | 'raw' | 'diff'>>({});
  let copiedId = $state<string | null>(null);
  let lastExternalSelectedId: string | null = null;

  // Synchronize expandedId with selectedId ONLY when an external selection changes
  $effect(() => {
    if (selectedId && selectedId !== lastExternalSelectedId) {
      expandedId = selectedId;
      lastExternalSelectedId = selectedId;
    } else if (!selectedId) {
      lastExternalSelectedId = null;
    }
  });

  const filteredList = $derived.by(() => {
    const list = historyList || [];
    const q = (searchQuery || '').trim().toLowerCase();
    if (!q) return list;

    return list.filter(item =>
      (item.processedText && item.processedText.toLowerCase().includes(q)) ||
      (item.rawText && item.rawText.toLowerCase().includes(q))
    );
  });

  interface DateGroup {
    key: string;
    labelPrefix: string;
    labelDate: string;
    items: HistoryItem[];
  }

  const groupedItems = $derived.by<DateGroup[]>(() => {
    const groups: DateGroup[] = [];
    const now = new Date();
    const todayMidnight = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const yesterdayMidnight = todayMidnight - 86400000;
    const isRu = i18n.resolved === 'ru';

    for (const item of filteredList) {
      const d = new Date(item.timestamp);
      const itemMidnight = isNaN(d.getTime())
        ? 0
        : new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();

      let prefix = '';
      let dateStr = '';
      const formattedDate = formatDateHeader(d, isRu);

      if (itemMidnight === todayMidnight) {
        prefix = i18n.t.history.today;
        dateStr = formattedDate;
      } else if (itemMidnight === yesterdayMidnight) {
        prefix = i18n.t.history.yesterday;
        dateStr = formattedDate;
      } else if (itemMidnight > 0) {
        dateStr = formattedDate;
      } else {
        dateStr = i18n.t.history.earlier;
      }

      const key = `${prefix}-${dateStr}`;
      let group = groups.find(g => g.key === key);
      if (!group) {
        group = { key, labelPrefix: prefix, labelDate: dateStr, items: [] };
        groups.push(group);
      }
      group.items.push(item);
    }

    return groups;
  });

  const toggleExpand = (id: string) => {
    if (expandedId === id) {
      expandedId = null;
      selectedId = null;
      lastExternalSelectedId = null;
    } else {
      expandedId = id;
      selectedId = id;
      lastExternalSelectedId = id;
    }
  };

  const handleCopy = (text: string, id: string) => {
    onCopyText(text);
    copiedId = id;
    setTimeout(() => {
      if (copiedId === id) copiedId = null;
    }, 2000);
  };
</script>

<div class="history-container">
  {#if !saveHistory}
    <div class="history-disabled-banner">
      <div class="disabled-title">{i18n.t.history.historyDisabledTitle}</div>
      <div class="disabled-desc">{i18n.t.history.historyDisabledSubtitle}</div>
      <button type="button" class="enable-history-btn" onclick={onOpenPrivacySettings}>
        {i18n.t.history.enableHistory}
      </button>
    </div>
  {/if}
  {#if historyList.length === 0 && saveHistory}
    <div class="empty-history">
      <div class="empty-title">
        {i18n.t.history.emptyTitle}
      </div>
      <div class="empty-desc">
        {i18n.t.history.emptySubtitle}
      </div>
    </div>
  {:else if historyList.length > 0}
    <div class="history-feed">
      <div class="history-toolbar-sticky">
        <div class="search-input-box">
          <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <input
            type="text"
            class="history-search-input"
            placeholder={i18n.t.topbar.searchPlaceholder}
            value={searchQuery}
            oninput={(e) => onSearchChange((e.currentTarget as HTMLInputElement).value)}
          />
          {#if searchQuery}
            <button
              type="button"
              class="search-clear-btn"
              onclick={() => onSearchChange('')}
              title={i18n.t.common.cancel}
              aria-label={i18n.t.common.cancel}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          {/if}
        </div>
      </div>

      {#if filteredList.length === 0}
        <div class="empty-history">
          <div class="empty-title">
            {i18n.t.history.searchEmptyTitle}
          </div>
          <div class="empty-desc">
            {i18n.t.history.searchEmptySubtitle(searchQuery)}
          </div>
        </div>
      {:else}
        {#each groupedItems as group}
        <div class="history-date-group">
          <div class="date-group-header">
            {#if group.labelPrefix}
              <span class="group-prefix">{group.labelPrefix}</span>
              <span class="group-dot">•</span>
            {/if}
            <span class="group-date">{group.labelDate}</span>
            <span class="group-dot">•</span>
            <span class="group-count">{i18n.t.history.itemsCount(group.items.length)}</span>
          </div>

          <div class="group-items-list">
            {#each group.items as item (item.id)}
              {@const isExpanded = expandedId === item.id}
              {@const activeTab = activeTabMap[item.id] || 'processed'}
              {@const displayedText = activeTab === 'processed' ? item.processedText : (item.rawText || item.processedText)}

              <div class="history-item-card" class:expanded={isExpanded}>
                <!-- Header Row with Hover Quick Actions (Whole row toggles expand) -->
                <div
                  class="item-header-row"
                  onclick={() => toggleExpand(item.id)}
                  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleExpand(item.id); }}
                  role="button"
                  tabindex="0"
                  aria-expanded={isExpanded}
                >
                  <div class="item-header-main">
                    <span class="item-time">{formatTime(item.timestamp)}</span>

                    <span class="item-summary-title">
                      {getItemTitle(item, i18n.t.history.emptyRecordTitle)}
                    </span>
                  </div>

                  <div class="item-meta-right">
                    {#if !isExpanded}
                      <!-- Hover Quick Actions (Raycast / Linear style) -->
                      <div class="hover-quick-actions">
                        <button
                          type="button"
                          class="quick-action-btn"
                          class:copied={copiedId === item.id}
                          onclick={(e) => { e.stopPropagation(); handleCopy(item.processedText || item.rawText || '', item.id); }}
                          title={copiedId === item.id ? i18n.t.history.copied : i18n.t.history.copyTooltip}
                        >
                          <MorphIcon active={copiedId === item.id} size={12} />
                        </button>

                        <button
                          type="button"
                          class="quick-action-btn delete-btn"
                          onclick={(e) => { e.stopPropagation(); onDelete(item.id); }}
                          title={i18n.t.history.deleteTooltip}
                        >
                          <svg class="quick-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                          </svg>
                        </button>
                      </div>
                    {/if}

                    <span class="item-duration">{formatDuration(item.duration)}</span>
                  </div>
                </div>

                <!-- Expanded Content Panel (Linear clean drawer) -->
                {#if isExpanded}
                  <div class="item-expanded-body" transition:slide={{ duration: 180, easing: cubicOut }}>
                    <div class="expanded-tabs-row">
                      <div class="tab-pill-group" role="tablist">
                        <button
                          type="button"
                          class="pill-tab"
                          class:active={activeTab === 'processed'}
                          onclick={() => { activeTabMap[item.id] = 'processed'; }}
                          role="tab"
                          aria-selected={activeTab === 'processed'}
                        >
                          {i18n.t.history.tabResult}
                        </button>

                        {#if item.rawText && item.rawText !== item.processedText}
                          <button
                            type="button"
                            class="pill-tab"
                            class:active={activeTab === 'raw'}
                            onclick={() => { activeTabMap[item.id] = 'raw'; }}
                            role="tab"
                            aria-selected={activeTab === 'raw'}
                          >
                            {i18n.t.history.tabRaw}
                          </button>

                          <button
                            type="button"
                            class="pill-tab"
                            class:active={activeTab === 'diff'}
                            onclick={() => { activeTabMap[item.id] = 'diff'; }}
                            role="tab"
                            aria-selected={activeTab === 'diff'}
                          >
                            {i18n.t.history.tabDiff}
                          </button>
                        {/if}
                      </div>

                      <button
                        type="button"
                        class="action-copy-btn"
                        class:copied={copiedId === item.id}
                        onclick={() => handleCopy(displayedText, item.id)}
                        title={copiedId === item.id ? i18n.t.history.copied : i18n.t.history.copyTooltip}
                      >
                        <MorphIcon active={copiedId === item.id} size={11} />
                        <span>{copiedId === item.id ? i18n.t.history.copied : i18n.t.common.copy}</span>
                      </button>
                    </div>

                    {#if activeTab === 'diff'}
                      <div class="expanded-diff-content">
                        {#each computeWordDiff(item.rawText || '', item.processedText || '') as token}
                          {#if token.type === 'same'}
                            <span class="diff-same">{token.value}</span>
                          {:else if token.type === 'added'}
                            <ins class="diff-ins" title={i18n.t.history.diffAdded}>{token.value}</ins>
                          {:else if token.type === 'removed'}
                            <del class="diff-del" title={i18n.t.history.diffRemoved}>{token.value}</del>
                          {/if}
                        {/each}
                      </div>
                    {:else}
                      <div class="expanded-text-content">
                        {displayedText}
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .history-container {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    box-sizing: border-box;
    padding: 0 16px 28px 16px;
    scrollbar-gutter: stable;
  }

  .history-feed {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
    max-width: 100%;
    margin: 0;
  }

  .history-toolbar-sticky {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 4px 0;
    width: 100%;
    background: var(--bg-app);
    box-sizing: border-box;
    margin-bottom: 8px;
  }

  .search-input-box {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 240px;
    height: 28px;
    padding: 0 8px;
    background: var(--hover-bg, rgba(0, 0, 0, 0.04));
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    transition: background 100ms ease, border-color 100ms ease;
    box-sizing: border-box;
  }

  :global(.theme-dark) .search-input-box {
    background: rgba(255, 255, 255, 0.05);
  }

  .search-input-box:focus-within {
    background: var(--bg-surface, #FFFFFF);
    border-color: var(--border-subtle);
  }

  .search-icon {
    width: 13px;
    height: 13px;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .history-search-input {
    background: transparent;
    border: none;
    outline: none;
    font-size: 12px;
    color: var(--text-primary);
    width: 100%;
    padding: 2px 0;
    min-width: 0;
  }

  .history-search-input::placeholder {
    color: var(--text-tertiary);
  }

  .search-clear-btn {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    padding: 0;
    color: var(--text-tertiary);
    cursor: pointer;
    border-radius: 50%;
    flex-shrink: 0;
    transition: color 100ms ease;
  }

  .search-clear-btn:hover {
    color: var(--text-primary);
  }

  .search-clear-btn svg {
    width: 10px;
    height: 10px;
  }

  .history-date-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .date-group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-left: 8px;
    user-select: none;
  }

  .group-prefix,
  .group-date,
  .group-count,
  .group-dot {
    font-size: 11.5px;
    font-weight: 500;
    letter-spacing: -0.01em;
    color: var(--text-secondary);
  }

  .group-prefix {
    font-weight: 600;
    color: var(--text-primary);
  }

  .group-count {
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .group-dot {
    color: var(--text-tertiary);
    font-size: 8px;
  }

  .group-items-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: transparent;
    border: none;
    box-shadow: none;
  }

  :global(.theme-dark) .group-items-list {
    border: none;
    background: transparent;
    box-shadow: none;
  }

  .history-item-card {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-sm, 4px);
    background: transparent;
    transition: background 100ms ease, border-color 100ms ease;
    position: relative;
    overflow: hidden;
    border: 1px solid transparent;
  }

  .history-item-card.expanded {
    background: var(--bg-surface-raised, #F7F7F8);
    border-color: var(--border-subtle);
  }

  :global(.theme-dark) .history-item-card.expanded {
    background: #131315;
    border-color: rgba(255, 255, 255, 0.08);
  }

  .item-header-row {
    display: flex;
    align-items: center;
    height: 34px;
    padding: 0 10px;
    cursor: pointer;
    user-select: none;
    border-radius: var(--radius-sm, 4px);
    transition: background 80ms ease;
  }

  .item-header-row:focus-visible {
    outline: 1px solid var(--accent-primary, #FF9500);
    outline-offset: -1px;
  }

  .history-item-card:hover .item-header-row {
    background: var(--bg-hover, rgba(0, 0, 0, 0.04));
  }

  :global(.theme-dark) .history-item-card:hover .item-header-row {
    background: rgba(255, 255, 255, 0.05);
  }

  .item-header-main {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    height: 100%;
  }

  .item-time {
    width: 38px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .item-summary-title {
    flex: 1;
    min-width: 0;
    font-size: 12.5px;
    color: var(--text-primary);
    font-weight: 450;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-right: 8px;
  }

  .history-item-card.expanded .item-summary-title {
    font-weight: 600;
  }

  .item-meta-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .item-duration {
    width: 38px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  /* Quick Actions on Hover (Raycast / Linear style) */
  .hover-quick-actions {
    display: flex;
    align-items: center;
    gap: 3px;
    opacity: 0;
    pointer-events: none;
    transition: opacity 80ms ease;
  }

  .history-item-card:hover .hover-quick-actions {
    opacity: 1;
    pointer-events: auto;
  }

  .quick-action-btn {
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-xs, 3px);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
    transition: all 80ms ease;
  }

  .quick-action-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface);
    border-color: var(--border-color);
  }

  .quick-action-btn.delete-btn:hover {
    color: #EF4444;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .quick-icon {
    width: 12px;
    height: 12px;
  }



  /* Expanded Body */
  .item-expanded-body {
    padding: 6px 10px 10px 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: transparent;
  }

  .expanded-tabs-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0;
    margin-bottom: 2px;
  }

  .tab-pill-group {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    background: rgba(0, 0, 0, 0.04);
    padding: 2px;
    border-radius: var(--radius-sm, 4px);
  }

  :global(.theme-dark) .tab-pill-group {
    background: rgba(255, 255, 255, 0.06);
  }

  .pill-tab {
    height: 22px;
    padding: 0 8px;
    background: transparent;
    border: none;
    border-radius: var(--radius-xs, 2px);
    font-family: inherit;
    font-size: 11px;
    font-weight: 450;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: all 100ms ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .pill-tab:hover:not(.active) {
    color: var(--text-primary);
  }

  .pill-tab.active {
    background: var(--bg-surface, #FFFFFF);
    color: var(--text-primary);
    font-weight: 550;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
  }

  :global(.theme-dark) .pill-tab.active {
    background: #1C1C1F;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  }

  .action-copy-btn {
    height: 24px;
    padding: 0 8px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: var(--hover-bg, rgba(0, 0, 0, 0.04));
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    color: var(--text-secondary);
    font-size: 11px;
    font-family: inherit;
    font-weight: 450;
    cursor: pointer;
    transition: all 100ms ease;
  }

  :global(.theme-dark) .action-copy-btn {
    background: rgba(255, 255, 255, 0.05);
    color: #D1D5DB;
    border-color: rgba(255, 255, 255, 0.1);
  }

  .action-copy-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover, rgba(0, 0, 0, 0.07));
    border-color: var(--border-strong);
  }

  :global(.theme-dark) .action-copy-btn:hover {
    background: rgba(255, 255, 255, 0.09);
    color: #FFFFFF;
  }

  .action-copy-btn.copied {
    color: var(--accent-primary);
    border-color: var(--accent-primary);
  }

  .expanded-text-content,
  .expanded-diff-content {
    font-size: 12.5px;
    line-height: 1.55;
    color: var(--text-primary);
    padding: 2px 2px 2px 2px;
    white-space: pre-wrap;
    word-break: break-word;
    user-select: text;
  }

  .diff-ins {
    background: rgba(16, 185, 129, 0.16);
    color: #059669;
    text-decoration: none;
    padding: 1px 2px;
    border-radius: 2px;
    font-weight: 500;
  }

  :global(.theme-dark) .diff-ins {
    background: rgba(16, 185, 129, 0.22);
    color: #34D399;
  }

  .diff-del {
    background: rgba(239, 68, 68, 0.12);
    color: #DC2626;
    text-decoration: line-through;
    opacity: 0.8;
    padding: 1px 2px;
    border-radius: 2px;
    margin-right: 1px;
  }

  :global(.theme-dark) .diff-del {
    background: rgba(239, 68, 68, 0.22);
    color: #F87171;
  }

  .diff-same {
    color: var(--text-primary);
  }

  /* Empty and Disabled States */
  .empty-history,
  .history-disabled-banner {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    text-align: center;
  }

  .empty-title,
  .disabled-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 4px;
  }

  .empty-desc,
  .disabled-desc {
    font-size: 12.5px;
    color: var(--text-secondary);
    max-width: 360px;
    line-height: 1.45;
  }

  .enable-history-btn {
    margin-top: 14px;
    height: 28px;
    padding: 0 12px;
    background: var(--text-primary);
    color: var(--bg-surface);
    border: 1px solid var(--text-primary);
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 100ms ease;
  }

  .enable-history-btn:hover {
    opacity: 0.9;
  }
</style>
