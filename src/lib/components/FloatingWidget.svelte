<script lang="ts">
  import AudioVisualizer from './ui/AudioVisualizer.svelte';
  import { getRecordingStatus, type WidgetState } from '../recording/status.js';
  import { formatDuration } from '../utils/dateTime.js';
  import { i18n } from '../i18n/index.js';

  interface Props {
    visible?: boolean;
    widgetState?: WidgetState;
    audioLevel?: number;
    timerSeconds?: number;
    message?: string;
    shortLabel?: string;
    hotkeyName?: string;
    resolvedTheme?: 'light' | 'dark';
    errorActionType?: 'retry' | 'settings' | 'close';
    onStop?: () => void;
    onCancel?: () => void;
    onRetry?: () => void;
    onOpenSettings?: () => void;
    onClose?: () => void;
  }

  let {
    visible = false,
    widgetState = 'idle',
    audioLevel = 0,
    timerSeconds = 0,
    message = '',
    shortLabel = '',
    hotkeyName = 'F9',
    resolvedTheme = 'dark',
    errorActionType = 'close',
    onStop = () => {},
    onCancel = () => {},
    onRetry = () => {},
    onOpenSettings = () => {},
    onClose = () => {}
  }: Props = $props();

  const isListeningState = $derived(widgetState === 'listening');
  const recordingStatus = $derived(getRecordingStatus(widgetState, message));

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!visible) return;
    const isListening = widgetState === 'listening';
    if (e.key === 'Escape') {
      e.preventDefault();
      if (isListening) {
        onCancel();
      } else {
        onClose();
      }
    }
  };
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if visible}
  <div class="overlay-wrapper" data-tauri-drag-region>
    <div
      class="overlay-capsule"
      class:theme-dark={resolvedTheme === 'dark'}
      class:theme-light={resolvedTheme === 'light'}
      class:state-listening={recordingStatus.kind === 'listening'}
      class:state-processing={recordingStatus.kind === 'processing'}
      class:state-success={recordingStatus.kind === 'success'}
      class:state-error={recordingStatus.kind === 'error'}
      class:state-ready={recordingStatus.kind === 'idle'}
      data-tauri-drag-region
    >
      {#if recordingStatus.kind === 'listening'}
        <!-- 2. Recording State (Waveform, Timer, Dedicated Stop Button) -->
        <div
          class="capsule-content recording-content"
          data-tauri-drag-region
        >
          <AudioVisualizer {visible} isListening={isListeningState} {audioLevel} />
          <span class="timer-label" data-tauri-drag-region>{formatDuration(timerSeconds)}</span>
          <button
            type="button"
            class="hud-stop-btn"
            onclick={(e) => { e.stopPropagation(); onStop(); }}
            title={i18n.t.hud.stop}
            aria-label="{i18n.t.hud.stop}"
          >
            <span class="hud-stop-square-icon"></span>
          </button>
        </div>
      {:else if recordingStatus.kind === 'processing'}
        <!-- 3. Processing State (Modern ring spinner in accent color) -->
        <div class="capsule-content processing-content" data-tauri-drag-region>
          <div class="hud-spinner-slot">
            <svg class="modern-ring-spinner" viewBox="0 0 24 24">
              <circle class="spinner-track" cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2.2" fill="none" />
              <circle class="spinner-arc" cx="12" cy="12" r="9" stroke="var(--accent-primary, #FF9500)" stroke-width="2.4" stroke-linecap="round" fill="none" stroke-dasharray="56" stroke-dashoffset="38" />
            </svg>
          </div>
          <span class="status-label processing-label" data-tauri-drag-region>{recordingStatus.label || i18n.t.hud.transcribing}</span>
        </div>
      {:else if recordingStatus.kind === 'success'}
        <!-- 4. Success State (Clean Minimalist Checkmark - No Green Circle) -->
        <div class="capsule-content success-content" data-tauri-drag-region>
          <div class="hud-check-slot" data-tauri-drag-region>
            <svg class="success-check-icon" viewBox="0 0 16 16" fill="none">
              <path class="success-check-path" d="M3.5 8.5L6.5 11.5L12.5 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </div>
          <span class="status-label success-label" data-tauri-drag-region>
            {message || (widgetState === 'copied' ? i18n.t.hud.copied : i18n.t.hud.inserted)}
          </span>
        </div>
      {:else if recordingStatus.kind === 'error'}
        <!-- Error State -->
        <div class="capsule-content error-content" data-tauri-drag-region>
          <svg class="error-status-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <span class="status-label error-text" title={message} data-tauri-drag-region>
            {shortLabel || recordingStatus.label}
          </span>
          <div class="error-actions-group">
            {#if errorActionType === 'retry'}
            <button class="hud-action-btn" onclick={onRetry} title={i18n.t.hud.retry} aria-label={i18n.t.hud.retry}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                <path d="M3 3v5h5"/>
                <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
                <path d="M16 21h5v-5"/>
              </svg>
            </button>
            {:else if errorActionType === 'settings'}
            <button class="hud-action-btn" onclick={onOpenSettings} title={i18n.t.hud.settings} aria-label={i18n.t.hud.settings}>
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <path d="M4 7h16M4 17h16M8 4v6M16 14v6" />
              </svg>
            </button>
            {/if}
            <button class="hud-action-btn" onclick={onClose} title="{i18n.t.common.cancel} (Esc)" aria-label="{i18n.t.common.cancel}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          </div>
        </div>
      {:else if recordingStatus.kind === 'cancelled'}
        <div class="capsule-content" data-tauri-drag-region>
          <span class="status-label" data-tauri-drag-region>{recordingStatus.label}</span>
        </div>
      {:else}
        <!-- 1. Ready State -->
        <div class="capsule-content ready-content" data-tauri-drag-region>
          <svg class="ready-sound-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="4" y1="9" x2="4" y2="15"></line>
            <line x1="8" y1="5" x2="8" y2="19"></line>
            <line x1="12" y1="2" x2="12" y2="22"></line>
            <line x1="16" y1="5" x2="16" y2="19"></line>
            <line x1="20" y1="9" x2="20" y2="15"></line>
          </svg>
          <span class="status-label" data-tauri-drag-region>{i18n.t.hud.ready}</span>
          <kbd class="hotkey-badge">{hotkeyName}</kbd>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay-wrapper {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100vw;
    height: 100vh;
    background: transparent !important;
    pointer-events: none;
    user-select: none;
  }

  /* One Shared HUD Visual Shell: Dynamic width, 6px radius, deep shadow floating surface */
  .overlay-capsule {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 32px;
    width: 148px;
    border-radius: 6px;
    padding: 0 10px;
    pointer-events: auto;
    font-family: var(--font-sans, system-ui, -apple-system, sans-serif);
    background: #121214;
    border: 1px solid #222226;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6), 0 2px 8px rgba(0, 0, 0, 0.4);
    overflow: hidden;
    box-sizing: border-box;
    transition: width 180ms cubic-bezier(0.16, 1, 0.3, 1),
                background-color 180ms cubic-bezier(0.16, 1, 0.3, 1),
                border-color 180ms cubic-bezier(0.16, 1, 0.3, 1),
                box-shadow 180ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .overlay-capsule.state-listening,
  .overlay-capsule.state-processing,
  .overlay-capsule.state-success,
  .overlay-capsule.state-ready {
    width: 154px;
  }

  .overlay-capsule.state-error {
    width: 176px;
  }

  .overlay-capsule.theme-dark {
    background: #121214;
    border: 1px solid #222226;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6), 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .overlay-capsule.theme-light {
    background: rgba(255, 255, 255, 0.96);
    backdrop-filter: blur(20px) saturate(180%);
    border: 1px solid rgba(0, 0, 0, 0.08);
    box-shadow: 0 16px 36px -4px rgba(0, 0, 0, 0.15), 0 4px 12px -2px rgba(0, 0, 0, 0.08), 0 0 0 1px rgba(0, 0, 0, 0.04);
  }

  .overlay-capsule.theme-light .timer-label {
    color: #0F172A;
  }

  .overlay-capsule.theme-light .status-label {
    color: #0F172A;
  }

  .overlay-capsule.theme-light .ready-sound-icon {
    color: #64748B;
  }

  .overlay-capsule.theme-light .hotkey-badge {
    background: #FFFFFF;
    border: 1px solid #CBD5E1;
    color: #0F172A;
    box-shadow: 0 1px 0 #CBD5E1;
  }

  .capsule-content {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    animation: capsule-content-enter 180ms cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes capsule-content-enter {
    from {
      opacity: 0;
      transform: scale(0.92) translateY(1px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  /* 1. Ready State */
  .ready-content {
    width: 100%;
    gap: 8px;
    justify-content: space-between;
  }

  .ready-sound-icon {
    width: 13px;
    height: 13px;
    color: var(--text-secondary, #9BA1AA);
    flex-shrink: 0;
  }

  .status-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary, #E7E9ED);
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .hotkey-badge {
    border: 1px solid var(--border-color, #272C33);
    background: var(--bg-control, #171A20);
    border-radius: 3px;
    padding: 1px 6px;
    font-size: 10px;
    color: var(--text-secondary, #9BA1AA);
    font-family: var(--font-mono, monospace);
    flex-shrink: 0;
  }

  /* 2. Recording State: indicator + waveform + timer */
  .recording-content {
    width: 100%;
    height: 100%;
    gap: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 4px;
  }

  .hud-stop-btn {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
    outline: none;
    transition: background-color 100ms ease;
  }

  .hud-stop-square-icon {
    width: 6px;
    height: 6px;
    border-radius: 1.5px;
    flex-shrink: 0;
    transition: background-color 100ms ease;
  }

  .overlay-capsule.theme-light .hud-stop-btn {
    background: rgba(0, 0, 0, 0.05);
  }

  .overlay-capsule.theme-light .hud-stop-btn:hover {
    background: rgba(239, 68, 68, 0.12);
  }

  .overlay-capsule.theme-light .hud-stop-square-icon {
    background: #374151;
  }

  .overlay-capsule.theme-light .hud-stop-btn:hover .hud-stop-square-icon {
    background: #EF4444;
  }

  .overlay-capsule.theme-dark .hud-stop-btn {
    background: rgba(255, 255, 255, 0.08);
  }

  .overlay-capsule.theme-dark .hud-stop-btn:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  .overlay-capsule.theme-dark .hud-stop-square-icon {
    background: #EDEDED;
  }

  .overlay-capsule.theme-dark .hud-stop-btn:hover .hud-stop-square-icon {
    background: #F87171;
  }

  .timer-label {
    font-size: 11.5px;
    font-weight: 500;
    color: #FFFFFF;
    font-family: var(--font-mono, monospace);
    font-variant-numeric: tabular-nums;
    font-feature-settings: "tnum";
    letter-spacing: 0.02em;
    width: 38px;
    text-align: right;
    flex-shrink: 0;
    user-select: none;
    cursor: default;
  }

  /* 3. Processing State: spinner */
  .processing-content {
    width: 100%;
    gap: 8px;
    align-items: center;
    justify-content: center;
  }

  .processing-label {
    flex: 0 0 auto;
    font-size: 12px;
    color: var(--text-primary, #E7E9ED);
  }

  .hud-spinner-slot {
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .modern-ring-spinner {
    width: 14px;
    height: 14px;
    animation: spin 0.85s linear infinite;
    flex-shrink: 0;
  }

  .spinner-track {
    stroke: rgba(255, 255, 255, 0.12);
  }

  :global(.theme-light) .spinner-track {
    stroke: rgba(0, 0, 0, 0.1);
  }

  .spinner-arc {
    stroke: var(--accent-primary, #FF9500);
    filter: drop-shadow(0 0 3px var(--accent-border, rgba(255, 149, 0, 0.4)));
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* 4. Success State */
  .success-content {
    width: 100%;
    height: 100%;
    gap: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    margin: 0 auto;
  }

  .hud-check-slot {
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--text-primary, #EDEDED);
  }

  :global(.theme-light) .hud-check-slot {
    color: #18181B;
  }

  .success-check-icon {
    width: 14px;
    height: 14px;
    animation: check-pop 0.35s cubic-bezier(0.16, 1, 0.3, 1) both;
  }

  @keyframes check-pop {
    0% { transform: scale(0.6); opacity: 0; }
    100% { transform: scale(1); opacity: 1; }
  }

  .success-check-path {
    stroke-dasharray: 20;
    stroke-dashoffset: 20;
    animation: check-draw 0.35s 0.05s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  @keyframes check-draw {
    to { stroke-dashoffset: 0; }
  }

  .success-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary, #E7E9ED);
    white-space: nowrap;
    text-align: center;
    flex: 0 0 auto;
  }

  /* 5. Error State */
  .error-content {
    width: 100%;
    gap: 6px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .error-status-icon {
    width: 13px;
    height: 13px;
    color: #EF4444;
    flex-shrink: 0;
  }

  .status-label.error-text {
    font-size: 11.5px;
    font-weight: 500;
    color: #EF4444;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .overlay-capsule.theme-dark .status-label.error-text {
    color: #F87171;
  }

  .overlay-capsule.theme-light .status-label.error-text {
    color: #DC2626;
  }

  .error-actions-group {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .hud-action-btn {
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
    outline: none;
    transition: background-color 100ms ease, color 100ms ease;
  }

  .overlay-capsule.theme-light .hud-action-btn {
    color: #64748B;
  }

  .overlay-capsule.theme-light .hud-action-btn:hover {
    color: #0F172A;
    background: rgba(0, 0, 0, 0.06);
  }

  .overlay-capsule.theme-dark .hud-action-btn {
    color: #9BA1AA;
  }

  .overlay-capsule.theme-dark .hud-action-btn:hover {
    color: #EDEDED;
    background: rgba(255, 255, 255, 0.08);
  }

  .hud-action-btn svg {
    width: 11px;
    height: 11px;
  }
</style>
