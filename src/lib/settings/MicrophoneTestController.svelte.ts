import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { i18n } from '../i18n/index.js';

export class MicrophoneTestController {
  isActive = $state(false);
  volumeLevel = $state(0);
  errorMessage = $state('');

  readonly #getDeviceId: () => string;
  #unlisten: (() => void) | null = null;
  #operation = 0;

  constructor(getDeviceId: () => string) {
    this.#getDeviceId = getDeviceId;
  }

  async toggleTest(): Promise<void> {
    if (this.isActive) {
      await this.stop();
    } else {
      await this.start();
    }
  }

  async start(): Promise<void> {
    const operation = ++this.#operation;
    this.isActive = true;
    this.volumeLevel = 0;
    this.errorMessage = '';

    const tauri = window.__TAURI_INTERNALS__;
    if (!tauri?.invoke || !tauri.transformCallback) {
      console.warn(
        '[Dicta] Microphone test is unavailable outside the desktop app',
      );
      this.#reset();
      return;
    }

    try {
      const unlisten = await listen<number>('loopback-audio-level', (event) => {
        this.volumeLevel = Math.min(
          100,
          Math.max(0, Math.round(event.payload * 100)),
        );
      });
      if (operation !== this.#operation) {
        unlisten();
        return;
      }
      this.#unlisten = unlisten;
      await invoke('start_loopback', {
        deviceLabel: this.#getDeviceId() || null,
      });
      if (operation !== this.#operation) {
        await invoke('stop_loopback');
      }
    } catch (error) {
      if (operation !== this.#operation) return;
      console.error('[Dicta] Failed to start microphone test:', error);
      this.#reset();
      this.errorMessage =
        error instanceof Error
          ? error.message
          : String(error || i18n.t.settings.testMicFailed);
    }
  }

  async stop(): Promise<void> {
    this.#operation += 1;
    this.#reset();
    this.errorMessage = '';
    if (!window.__TAURI_INTERNALS__?.invoke) return;

    try {
      await invoke('stop_loopback');
    } catch (error) {
      console.error('[Dicta] Failed to stop microphone test:', error);
    }
  }

  dispose(): void {
    this.#operation += 1;
    this.#reset();
    if (window.__TAURI_INTERNALS__?.invoke) {
      void invoke('stop_loopback').catch(console.error);
    }
  }

  #reset(): void {
    this.isActive = false;
    this.volumeLevel = 0;
    this.#unlisten?.();
    this.#unlisten = null;
  }
}
