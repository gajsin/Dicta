import { emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../settings/model.js';
import type { WidgetPayload, WidgetState } from './status.js';
import { resolveActualTheme } from '../utils/accentTheme.js';
import { getNormalizedErrorInfo } from '../utils/errors.js';
import { i18n } from '../i18n/index.js';

interface TranscriptionResult {
  rawText: string;
  processedText: string;
}

interface RecordingControllerOptions {
  isTauriRuntime: boolean;
  windowLabel: string;
  getSettings: () => AppSettings;
  onHistoryItem: (text: string, durationSec: number, rawText: string) => void;
  onOpenSettings: (section: 'dictation' | 'providers') => void;
}

const BUSY_STATES: WidgetState[] = [
  'finishing',
  'transcribing',
  'refining',
  'inserting',
  'copied',
];

export class RecordingController {
  recording = $state(false);
  processing = $state(false);
  widgetState = $state<WidgetState>('idle');
  widgetMessage = $state('');
  shortLabel = $state('');
  audioLevel = $state(0);
  errorActionType = $state<'retry' | 'settings' | 'close'>('close');
  timerSeconds = $state(0);
  lastRecordingStart = $state(0);
  resolvedTheme = $state<'dark' | 'light'>('dark');

  readonly #isTauriRuntime: boolean;
  readonly #windowLabel: string;
  readonly #getSettings: () => AppSettings;
  readonly #onHistoryItem: RecordingControllerOptions['onHistoryItem'];
  readonly #onOpenSettings: RecordingControllerOptions['onOpenSettings'];
  #errorSettingsSection: 'dictation' | 'providers' = 'dictation';

  #timerId: number | undefined;
  #transitionTimeoutId: number | undefined;
  #lastToggleTime = 0;
  #pasteCooldownUntil = 0;
  #recordingSessionId = '';
  #terminalSuccessEmittedForSession = false;

  constructor(options: RecordingControllerOptions) {
    this.#isTauriRuntime = options.isTauriRuntime;
    this.#windowLabel = options.windowLabel;
    this.#getSettings = options.getSettings;
    this.#onHistoryItem = options.onHistoryItem;
    this.#onOpenSettings = options.onOpenSettings;
  }

  broadcast(): void {
    if (this.#windowLabel === 'main' && this.#isTauriRuntime) {
      const settings = this.#getSettings();
      const resolved = resolveActualTheme(settings.theme);
      this.resolvedTheme = resolved;
      emit('widget-update', {
        state: this.widgetState,
        message: this.widgetMessage,
        shortLabel: this.shortLabel,
        timerSeconds: this.timerSeconds,
        audioLevel: this.audioLevel,
        theme: settings.theme,
        resolvedTheme: resolved,
        accentColor: settings.accentColor,
        accentOpacity: settings.accentOpacity,
        uiLanguage: settings.uiLanguage,
        hotkey: settings.hotkey,
        errorActionType: this.errorActionType,
      }).catch(console.error);
    }
  }

  setAudioLevel(level: number): void {
    if (!this.recording) return;
    this.audioLevel = level;
    this.broadcast();
  }

  applyWidgetUpdate(payload: WidgetPayload): void {
    if (payload.state !== undefined) this.widgetState = payload.state;
    if (payload.message !== undefined) this.widgetMessage = payload.message;
    if (payload.shortLabel !== undefined) this.shortLabel = payload.shortLabel;
    if (payload.timerSeconds !== undefined) {
      this.timerSeconds = payload.timerSeconds;
    }
    if (payload.audioLevel !== undefined) this.audioLevel = payload.audioLevel;
    if (payload.errorActionType !== undefined) {
      this.errorActionType = payload.errorActionType;
    }
    if (payload.resolvedTheme !== undefined) {
      this.resolvedTheme = payload.resolvedTheme;
    }
  }

  async toggle(): Promise<void> {
    if (this.#windowLabel !== 'main') return;

    const now = Date.now();
    if (now < this.#pasteCooldownUntil || now - this.#lastToggleTime < 500) {
      return;
    }
    this.#lastToggleTime = now;

    if (this.processing || BUSY_STATES.includes(this.widgetState)) return;
    this.#clearTransitionTimeout();

    if (!this.recording) {
      await this.#start();
    } else {
      await this.#stopAndTranscribe();
    }
  }

  async cancel(): Promise<void> {
    if (
      this.#windowLabel !== 'main' ||
      (!this.recording && this.widgetState !== 'listening')
    ) {
      return;
    }

    this.recording = false;
    this.processing = false;
    this.#stopTimer();
    this.#clearTransitionTimeout();
    this.widgetState = 'cancelled';
    this.widgetMessage = i18n.t.hud.cancelled;
    this.shortLabel = i18n.t.hud.cancelled;
    this.broadcast();

    if (!this.#isTauriRuntime) {
      this.#scheduleIdle(1000);
      return;
    }

    try {
      await invoke('cancel_recording');
      this.#transitionTimeoutId = window.setTimeout(() => {
        invoke('hide_overlay').catch(console.error);
        this.widgetState = 'idle';
        this.broadcast();
      }, 1000);
    } catch (error) {
      this.#showError(error);
    }
  }

  async retry(): Promise<void> {
    if (this.processing || this.recording || this.widgetState !== 'error' || this.errorActionType !== 'retry') return;

    try {
      this.#clearTransitionTimeout();
      this.processing = true;
      this.widgetState = 'transcribing';
      this.widgetMessage = i18n.t.hud.transcribing;
      this.shortLabel = i18n.t.hud.transcribing;
      this.broadcast();
      this.#scheduleRefiningState();

      if (!this.#isTauriRuntime) {
        this.processing = false;
        this.widgetState = 'idle';
        this.broadcast();
        return;
      }

      const result = await invoke<TranscriptionResult>('retry_transcription', {
        options: this.#transcriptionOptions(),
      });
      await this.#handleSuccess(result, this.timerSeconds);
    } catch (error) {
      this.recording = false;
      this.processing = false;
      this.#stopTimer();
      this.#showError(error);
    }
  }

  sendWidgetAction(action: string): void {
    if (this.#isTauriRuntime) {
      emit('widget-action', action).catch(console.error);
    } else {
      this.handleWidgetAction(action);
    }
  }

  handleWidgetAction(action: string): void {
    if (action === 'stop') {
      void this.toggle();
    } else if (action === 'cancel') {
      void this.cancel();
    } else if (action === 'retry') {
      void this.retry();
    } else if (action === 'open_settings') {
      this.#clearTransitionTimeout();
      this.#onOpenSettings(this.#errorSettingsSection);
      this.widgetState = 'idle';
      this.broadcast();
      if (this.#isTauriRuntime) {
        invoke('reveal_main_window').catch(console.error);
        invoke('hide_overlay').catch(console.error);
      }
    } else if (action === 'close') {
      this.#clearTransitionTimeout();
      if (this.#isTauriRuntime) invoke('hide_overlay').catch(console.error);
      this.widgetState = 'idle';
      this.broadcast();
    }
  }

  handleDeviceError(message: string): void {
    if (
      this.processing ||
      !this.recording ||
      Date.now() - this.lastRecordingStart < 1200
    ) {
      console.warn(
        '[Dicta] Ignored microphone cleanup event during stop/processing:',
        message,
      );
      return;
    }

    console.error('[Dicta] Microphone stream error:', message);
    this.recording = false;
    this.processing = false;
    this.#stopTimer();
    this.widgetState = 'error';
    this.widgetMessage = i18n.t.errors.micNotFound;
    this.shortLabel = i18n.t.hud.micErrorShort;
    this.errorActionType = 'settings';
    this.#errorSettingsSection = 'dictation';
    this.broadcast();
  }

  handleRecordingTimeout(message: string): void {
    console.warn('[Dicta] Recording timeout reached:', message);
    if (this.#isTauriRuntime) {
      void invoke('cancel_recording').catch(console.error);
    }
    this.recording = false;
    this.processing = false;
    this.#stopTimer();
    this.widgetState = 'error';
    this.widgetMessage =
      message || i18n.t.errors.recordingTooLong;
    this.shortLabel = i18n.t.hud.error;
    this.errorActionType = 'close';
    this.broadcast();
  }

  dispose(): void {
    this.#stopTimer();
    this.#clearTransitionTimeout();
  }

  async #start(): Promise<void> {
    try {
      if (!this.#isTauriRuntime) {
        throw new Error(
          i18n.t.errors.desktopOnly,
        );
      }

      this.lastRecordingStart = Date.now();
      this.#recordingSessionId = `rec_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
      this.#terminalSuccessEmittedForSession = false;
      this.recording = true;
      this.#startTimer();
      this.widgetState = 'listening';
      this.widgetMessage = i18n.t.hud.listening;
      this.shortLabel = i18n.t.hud.listening;
      this.broadcast();

      await invoke('show_overlay');
      await invoke('start_recording', {
        deviceLabel: this.#getSettings().selectedDeviceId || null,
      });
    } catch (error) {
      this.recording = false;
      this.#stopTimer();
      this.#showError(error);
    }
  }

  async #stopAndTranscribe(): Promise<void> {
    try {
      this.recording = false;
      const durationSec = this.timerSeconds;
      this.#stopTimer();
      this.processing = true;
      this.widgetState = 'finishing';
      this.widgetMessage = i18n.t.hud.finishing;
      this.shortLabel = i18n.t.hud.finishing;
      this.broadcast();

      if (!this.#isTauriRuntime) return;

      this.widgetState = 'transcribing';
      this.widgetMessage = i18n.t.hud.transcribing;
      this.shortLabel = i18n.t.hud.transcribing;
      this.broadcast();
      this.#scheduleRefiningState();

      const startedAt = Date.now();
      const result = await invoke<TranscriptionResult>(
        'stop_recording_and_transcribe',
        { options: this.#transcriptionOptions() },
      );
      const minimumDelay = 600 - (Date.now() - startedAt);
      if (minimumDelay > 0) {
        await new Promise((resolve) => setTimeout(resolve, minimumDelay));
      }
      await this.#handleSuccess(result, durationSec);
    } catch (error) {
      this.recording = false;
      this.processing = false;
      this.#stopTimer();
      const errorInfo = getNormalizedErrorInfo(error);
      if (errorInfo.kind === 'already_completed') {
        console.warn('[Dicta] Ignored duplicate stop call');
        this.widgetState = 'idle';
      } else {
        this.#showError(error);
        return;
      }
      this.broadcast();
    }
  }

  async #handleSuccess(
    result: TranscriptionResult,
    durationSec: number,
  ): Promise<void> {
    if (this.#terminalSuccessEmittedForSession) {
      console.warn('[Dicta] Terminal success already emitted for session:', this.#recordingSessionId);
      return;
    }

    const text = result.processedText || '';
    const rawText = result.rawText || text;

    if (!text.trim()) {
      this.processing = false;
      this.widgetState = 'error';
      this.widgetMessage = i18n.t.errors.noSpeechHint;
      this.shortLabel = i18n.t.hud.noSpeechShort;
      this.errorActionType = 'close';
      this.broadcast();
      return;
    }

    this.#onHistoryItem(text, durationSec, rawText);

    let pasteSuccess = false;
    try {
      await invoke('copy_and_paste', { text, paste: true });
      pasteSuccess = true;
    } catch (error) {
      console.error('[Dicta] Failed to insert recognized text:', error);
      const errStr = String(error || '');
      if (errStr.includes('вставить')) {
        pasteSuccess = false;
      } else {
        this.processing = false;
        this.widgetState = 'error';
        this.widgetMessage = i18n.t.errors.recognizedButNotInserted;
        this.shortLabel = i18n.t.hud.pasteErrorShort;
        this.errorActionType = 'close';
        this.broadcast();
        return;
      }
    }

    this.#terminalSuccessEmittedForSession = true;
    this.widgetState = pasteSuccess ? 'inserting' : 'copied';
    this.widgetMessage = pasteSuccess ? i18n.t.hud.inserted : i18n.t.hud.copied;
    this.shortLabel = this.widgetMessage;
    this.broadcast();

    this.#pasteCooldownUntil = Date.now() + 1500;
    this.#clearTransitionTimeout();
    this.#transitionTimeoutId = window.setTimeout(() => {
      if (this.#isTauriRuntime) {
        invoke('hide_overlay').catch(console.error);
      }
      this.processing = false;
      this.widgetState = 'idle';
      this.widgetMessage = i18n.t.hud.ready;
      this.shortLabel = '';
      this.broadcast();
    }, 1200);
  }

  #transcriptionOptions() {
    const settings = this.#getSettings();
    return {
      providerApiKeys: settings.providerApiKeys,
      sttProvider: settings.sttProvider,
      llmProvider: settings.llmProvider,
      model: settings.selectedModel,
      language: settings.selectedLanguage,
      postprocessEnabled:
        settings.postprocessEnabled && settings.postprocessMode !== 'raw',
      postprocessMode: settings.postprocessMode,
      postprocessModel: settings.postprocessModel,
    };
  }

  #scheduleRefiningState(): void {
    const settings = this.#getSettings();
    if (
      !settings.postprocessEnabled ||
      settings.postprocessMode === 'raw'
    ) {
      return;
    }

    this.#transitionTimeoutId = window.setTimeout(() => {
      if (this.processing && this.widgetState === 'transcribing') {
        this.widgetState = 'refining';
        this.widgetMessage = i18n.t.hud.refining;
        this.shortLabel = i18n.t.hud.refining;
        this.broadcast();
      }
    }, 750);
  }

  #showError(error: unknown): void {
    this.#clearTransitionTimeout();
    const errorInfo = getNormalizedErrorInfo(error);
    const retryAvailable = typeof error === 'object' && error !== null &&
      'retryAvailable' in error && error.retryAvailable === true;
    this.widgetState = 'error';
    this.widgetMessage = errorInfo.message;
    this.shortLabel = errorInfo.shortLabel;
    this.errorActionType = errorInfo.actionType === 'retry' && !retryAvailable
      ? 'close' : errorInfo.actionType;
    this.#errorSettingsSection = errorInfo.settingsSection ?? 'dictation';
    this.broadcast();
  }

  #startTimer(): void {
    if (this.#timerId !== undefined) clearInterval(this.#timerId);
    this.timerSeconds = 0;
    this.broadcast();
    this.#timerId = window.setInterval(() => {
      this.timerSeconds += 1;
      this.broadcast();
    }, 1000);
  }

  #stopTimer(): void {
    if (this.#timerId !== undefined) {
      clearInterval(this.#timerId);
      this.#timerId = undefined;
    }
  }

  #clearTransitionTimeout(): void {
    if (this.#transitionTimeoutId !== undefined) {
      clearTimeout(this.#transitionTimeoutId);
      this.#transitionTimeoutId = undefined;
    }
  }

  #scheduleIdle(delayMs: number): void {
    this.#transitionTimeoutId = window.setTimeout(() => {
      this.widgetState = 'idle';
      this.broadcast();
    }, delayMs);
  }
}
