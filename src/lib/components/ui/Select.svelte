<script lang="ts">
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { i18n } from '../../i18n/index.js';

  interface SelectOption {
    value: string;
    label: string;
  }

  interface Props {
    value: string;
    options: SelectOption[];
    placeholder?: string;
    ariaLabel?: string;
    disabled?: boolean;
    onChange?: (val: string) => void;
  }

  let {
    value = $bindable(''),
    options = [],
    placeholder = i18n.t.common.selectPlaceholder,
    ariaLabel = i18n.t.common.selectAriaLabel,
    disabled = false,
    onChange = () => {}
  }: Props = $props();

  let open = $state(false);
  let openUpwards = $state(false);
  let root: HTMLDivElement | undefined;
  let menuContainer = $state<HTMLDivElement | undefined>(undefined);

  let highlightedIndex = $state(-1);

  const selectedOption = () => options.find((option) => option.value === value);

  const close = () => {
    open = false;
    highlightedIndex = -1;
  };

  const toggleOpen = () => {
    if (disabled) return;
    if (!open && root) {
      const rect = root.getBoundingClientRect();
      const neededHeight = Math.min(200, Math.max(70, options.length * 30 + 8));
      const distFromBottom = window.innerHeight - rect.bottom;
      openUpwards = distFromBottom < neededHeight && rect.top > distFromBottom;
      const currentIdx = options.findIndex(o => o.value === value);
      highlightedIndex = currentIdx >= 0 ? currentIdx : 0;
    }
    open = !open;
  };

  const choose = (option: SelectOption) => {
    value = option.value;
    onChange(option.value);
    close();
  };

  const scrollHighlightedIntoView = (idx: number) => {
    if (!menuContainer) return;
    const items = menuContainer.querySelectorAll<HTMLButtonElement>('.select-option');
    if (items[idx]) {
      items[idx].scrollIntoView({ block: 'nearest' });
    }
  };

  const handleButtonKeydown = (event: KeyboardEvent) => {
    if (disabled) return;

    if (event.key === 'Escape') {
      event.preventDefault();
      close();
      return;
    }

    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (!open) {
        toggleOpen();
      } else if (options.length > 0) {
        highlightedIndex = (highlightedIndex + 1) % options.length;
        scrollHighlightedIntoView(highlightedIndex);
      }
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (!open) {
        toggleOpen();
      } else if (options.length > 0) {
        highlightedIndex = (highlightedIndex - 1 + options.length) % options.length;
        scrollHighlightedIntoView(highlightedIndex);
      }
      return;
    }

    if (event.key === 'Home' && open && options.length > 0) {
      event.preventDefault();
      highlightedIndex = 0;
      scrollHighlightedIntoView(0);
      return;
    }

    if (event.key === 'End' && open && options.length > 0) {
      event.preventDefault();
      highlightedIndex = options.length - 1;
      scrollHighlightedIntoView(options.length - 1);
      return;
    }

    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      if (open && highlightedIndex >= 0 && options[highlightedIndex]) {
        choose(options[highlightedIndex]);
      } else {
        toggleOpen();
      }
    }
  };

  $effect(() => {
    if (!open) return;

    const handlePointerDown = (event: PointerEvent) => {
      if (root && !root.contains(event.target as Node)) {
        close();
      }
    };

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        close();
      }
    };

    window.addEventListener('pointerdown', handlePointerDown, true);
    window.addEventListener('keydown', handleKeydown, true);

    return () => {
      window.removeEventListener('pointerdown', handlePointerDown, true);
      window.removeEventListener('keydown', handleKeydown, true);
    };
  });
</script>

<div class="ui-select" class:is-open={open} bind:this={root}>
  <button
    type="button"
    class="select-trigger"
    class:open
    class:disabled
    disabled={disabled}
    aria-label={ariaLabel}
    aria-haspopup="listbox"
    aria-expanded={open}
    title={selectedOption()?.label || placeholder}
    onclick={toggleOpen}
    onkeydown={handleButtonKeydown}
  >
    <span class="selected-text">
      <span class="selected-label" title={selectedOption()?.label || placeholder}>{selectedOption()?.label || placeholder}</span>
    </span>
    <svg class="chevron" viewBox="0 0 24 24" aria-hidden="true">
      <path d="m6 9 6 6 6-6" />
    </svg>
  </button>

  {#if open}
    <div
      bind:this={menuContainer}
      class="select-menu"
      class:open-upwards={openUpwards}
      role="listbox"
      tabindex="-1"
      aria-label={ariaLabel}
      transition:fly={{ y: openUpwards ? 6 : -6, duration: 240, easing: cubicOut }}
    >
      {#if options.length === 0}
        <div class="select-empty">Список пуст</div>
      {:else}
        {#each options as option, index (option.value + '-' + index)}
          <button
            type="button"
            class="select-option"
            class:active={option.value === value}
            class:highlighted={index === highlightedIndex}
            title={option.label}
            role="option"
            aria-selected={option.value === value}
            onclick={() => choose(option)}
            onmouseenter={() => { highlightedIndex = index; }}
          >
            <div class="option-content">
              <span class="option-main" title={option.label}>{option.label}</span>
            </div>
            {#if option.value === value}
              <svg class="check-icon" viewBox="0 0 24 24" aria-hidden="true">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            {/if}
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .ui-select {
    position: relative;
    display: inline-flex;
    justify-content: flex-end;
    width: auto;
    max-width: 100%;
  }

  .ui-select.is-open {
    z-index: 200;
  }

  .select-trigger {
    width: 100%;
    max-width: 100%;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 0 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs, 4px);
    background: var(--bg-control, rgba(0, 0, 0, 0.03));
    color: var(--text-primary);
    font-family: var(--font-sans, inherit);
    font-size: 12px;
    font-weight: 450;
    text-align: left;
    cursor: pointer;
    outline: none;
    box-sizing: border-box;
    transition: background 100ms ease, border-color 100ms ease;
  }

  .select-trigger:hover:not(.disabled) {
    background: var(--hover-bg, rgba(0, 0, 0, 0.06));
    border-color: var(--border-color);
  }

  .select-trigger.open {
    border-color: var(--border-strong);
    background: var(--bg-surface, #FFFFFF);
  }

  :global(.theme-dark) .select-trigger {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
  }

  :global(.theme-dark) .select-trigger:hover:not(.disabled) {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  :global(.theme-dark) .select-trigger.open {
    border-color: var(--border-strong);
    background: var(--bg-surface, #121214);
  }

  .select-trigger:focus {
    outline: none;
  }

  .select-trigger:focus-visible {
    outline: none;
    border-color: var(--border-focus);
  }

  .select-trigger.disabled {
    opacity: 0.5;
    cursor: not-allowed;
    background: var(--bg-control);
    color: var(--text-tertiary);
  }

  .selected-text {
    display: flex;
    flex-direction: column;
    justify-content: center;
    overflow: hidden;
    min-width: 0;
  }

  .selected-label,
  .option-main {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .selected-label,
  .option-main {
    color: var(--text-primary, #FFFFFF);
    font-weight: 400;
    font-size: 13px;
    line-height: 1.2;
  }

  .chevron {
    width: 10px;
    height: 10px;
    color: var(--text-tertiary, #9BA1AA);
    opacity: 0.45;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
    transform: rotate(0deg);
    transform-origin: center center;
    transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.15s ease, color 0.15s ease;
    flex-shrink: 0;
  }

  .select-trigger:hover .chevron,
  .select-trigger.open .chevron {
    opacity: 0.9;
    color: var(--text-primary);
  }

  .select-trigger.open .chevron {
    transform: rotate(180deg);
  }

  .check-icon {
    width: 13px;
    height: 13px;
    stroke: var(--accent-primary, #FF9500);
    stroke-width: 2.2;
    fill: none;
    flex-shrink: 0;
  }

  .select-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    left: auto;
    width: max-content;
    min-width: 100%;
    max-width: 320px;
    z-index: 1000;
    max-height: 220px;
    overflow-y: auto;
    padding: 4px;
    border: 1px solid var(--border-color, #23262E);
    border-radius: var(--radius-card, 4px);
    background: var(--bg-floating, #181A1F);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.45);
    box-sizing: border-box;
  }

  :global(.theme-light) .select-menu {
    background: #FFFFFF;
    border-color: #E2E4E8;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12), 0 2px 6px rgba(0, 0, 0, 0.04);
  }

  .select-menu.open-upwards {
    top: auto;
    bottom: calc(100% + 4px);
  }

  .select-empty {
    padding: 8px 10px;
    font-size: 13px;
    color: var(--text-secondary, #8A8F98);
    text-align: center;
  }

  .select-option {
    width: 100%;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 0 10px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    font-family: var(--font-sans, inherit);
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    outline: none;
    box-sizing: border-box;
    transition: background-color 80ms ease, color 80ms ease;
  }

  :global(.theme-light) .selected-label,
  :global(.theme-light) .option-main {
    color: #15171A;
  }

  :global(.theme-light) .select-option:hover {
    background: #F0F2F5;
  }

  :global(.theme-light) .select-option.active {
    background: var(--accent-soft, rgba(255, 149, 0, 0.12));
  }

  .option-content {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    flex: 1;
  }

  .select-option:hover,
  .select-option.highlighted {
    background: var(--hover-surface, rgba(255, 255, 255, 0.04));
  }

  .select-option.active {
    background: var(--accent-subtle, var(--accent-soft));
  }

  .select-option.active .option-main {
    color: var(--text-primary, #E7E9ED);
    font-weight: 500;
  }

</style>
