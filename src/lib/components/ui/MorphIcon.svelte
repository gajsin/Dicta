<script lang="ts">
  interface Props {
    active?: boolean;
    size?: number;
    class?: string;
    ariaLabel?: string;
  }

  let {
    active = false,
    size = 16,
    class: className = '',
    ariaLabel = '',
  }: Props = $props();
</script>

<span
  class="morph-icon-container {className}"
  class:active
  style:width="{size}px"
  style:height="{size}px"
  aria-hidden={ariaLabel ? undefined : 'true'}
  aria-label={ariaLabel || undefined}
>
  <svg
    class="morph-icon icon-copy"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    style:width="{size}px"
    style:height="{size}px"
  >
    <rect width="14" height="14" x="8" y="8" rx="2" ry="2"></rect>
    <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"></path>
  </svg>
  <svg
    class="morph-icon icon-check"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2.2"
    stroke-linecap="round"
    stroke-linejoin="round"
    style:width="{size}px"
    style:height="{size}px"
  >
    <polyline points="20 6 9 17 4 12"></polyline>
  </svg>
</span>

<style>
  .morph-icon-container {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    user-select: none;
    line-height: 0;
  }

  .morph-icon {
    position: absolute;
    inset: 0;
    transition: transform 150ms cubic-bezier(0.16, 1, 0.3, 1),
                opacity 150ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .icon-copy {
    transform: scale(1) rotate(0deg);
    opacity: 1;
  }

  .icon-check {
    transform: scale(0.5) rotate(-20deg);
    opacity: 0;
    color: var(--success-color, #059669);
  }

  :global(.theme-dark) .icon-check {
    color: #10B981;
  }

  .active .icon-copy {
    transform: scale(0.5) rotate(20deg);
    opacity: 0;
  }

  .active .icon-check {
    transform: scale(1) rotate(0deg);
    opacity: 1;
  }
</style>
