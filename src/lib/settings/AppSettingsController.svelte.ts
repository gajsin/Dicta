import { invoke } from '@tauri-apps/api/core';

import {
  loadSettingsFromServer,
  loadSettingsWithRetry,
  saveSettings,
} from '../utils/settings.js';
import {
  mergeSettings,
  normalizeSettings,
  type AppSettings,
  type AppSettingsPatch,
  type ThemeMode,
} from './model.js';

interface ControllerOptions {
  initialSettings: AppSettings;
  isTauriRuntime: boolean;
  windowLabel: string;
}

function cloneSettings(settings: AppSettings): AppSettings {
  return {
    ...settings,
    providerApiKeys: { ...settings.providerApiKeys },
    verifiedProviders: [...settings.verifiedProviders],
  };
}

export class AppSettingsController {
  current = $state<AppSettings>(normalizeSettings());

  readonly #isTauriRuntime: boolean;
  readonly #windowLabel: string;

  constructor({
    initialSettings,
    isTauriRuntime,
    windowLabel,
  }: ControllerOptions) {
    this.current = normalizeSettings(initialSettings);
    this.#isTauriRuntime = isTauriRuntime;
    this.#windowLabel = windowLabel;
  }

  snapshot(): AppSettings {
    return cloneSettings(this.current);
  }

  apply(settings: AppSettingsPatch): void {
    this.current = mergeSettings(this.current, settings);
  }

  async reloadFromServer(): Promise<void> {
    if (!this.#isTauriRuntime) {
      return;
    }

    const serverSettings = await loadSettingsWithRetry(loadSettingsFromServer);
    if (serverSettings) {
      this.current = serverSettings;
      saveSettings(serverSettings);
    }
  }

  async update(settings: AppSettingsPatch): Promise<void> {
    const previous = this.snapshot();
    this.apply(settings);

    try {
      await this.persist();
    } catch (error) {
      this.current = previous;
      throw error;
    }
  }

  async setTheme(theme: ThemeMode): Promise<void> {
    await this.update({ theme });
  }

  async persist(): Promise<void> {
    if (this.#windowLabel !== 'main') {
      return;
    }

    const snapshot = this.snapshot();
    if (this.#isTauriRuntime) {
      await invoke('sync_settings', { settings: snapshot });
    }
    saveSettings(snapshot);
  }
}
