<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    label?: string;
    ariaLabel?: string;
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = false,
    disabled = false,
    label = '',
    ariaLabel = label,
    onchange = () => {}
  }: Props = $props();

  const toggle = () => {
    if (disabled) return;
    onchange(!checked);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (disabled) return;
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      onchange(!checked);
    }
  };
</script>

<div
  class="toggle-container"
  class:disabled
  role="switch"
  aria-label={ariaLabel}
  aria-checked={checked}
  tabindex={disabled ? -1 : 0}
  onclick={toggle}
  onkeydown={handleKeyDown}
>
  {#if label}
    <span class="toggle-label">{label}</span>
  {/if}
  <div class="toggle-track" class:checked>
    <div class="toggle-thumb" class:checked></div>
  </div>
</div>

<style>
  .toggle-container {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    user-select: none;
    outline: none;
  }

  .toggle-container:focus-visible .toggle-track {
    box-shadow: 0 0 0 2px var(--bg-app), 0 0 0 3px var(--accent-primary, #FF9500);
  }

  .toggle-container.disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .toggle-label {
    font-size: 12.5px;
    color: var(--text-secondary);
    font-weight: 450;
  }

  .toggle-track {
    width: 26px;
    height: 15px;
    border-radius: 8px;
    background: var(--border-strong, #D1D5DB);
    position: relative;
    transition: background-color 140ms cubic-bezier(0.16, 1, 0.3, 1);
    flex-shrink: 0;
  }

  :global(.theme-dark) .toggle-track {
    background: rgba(255, 255, 255, 0.15);
  }

  .toggle-track.checked {
    background: var(--accent-primary, #FF9500);
  }

  .toggle-thumb {
    width: 11px;
    height: 11px;
    border-radius: 50%;
    background: #FFFFFF;
    position: absolute;
    top: 2px;
    left: 2px;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    transition: transform 160ms cubic-bezier(0.16, 1, 0.3, 1),
                width 100ms ease-out;
  }

  .toggle-thumb.checked {
    transform: translateX(11px);
  }

  /* Compact tactile feedback on press */
  .toggle-container:active:not(.disabled) .toggle-thumb {
    width: 13px;
  }

  .toggle-container:active:not(.disabled) .toggle-thumb.checked {
    transform: translateX(9px);
  }
</style>
