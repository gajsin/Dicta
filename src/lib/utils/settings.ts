import { invoke } from '@tauri-apps/api/core';

import {
  createDefaultSettings,
  normalizeSettings,
  settingsForLocalStorage,
  type AppSettings,
  type AppSettingsPatch,
} from '../settings/model.js';

const STORAGE_KEY = 'dicta.settings';

function hasTauriRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.__TAURI_INTERNALS__?.invoke === 'function'
  );
}

export async function loadSettingsFromServer(): Promise<AppSettings | null> {
  if (!hasTauriRuntime()) {
    return null;
  }

  const loaded = await invoke<AppSettingsPatch | null>('load_settings');
  return loaded ? normalizeSettings(loaded) : null;
}

export function loadSettings(): AppSettings {
  if (typeof window === 'undefined' || hasTauriRuntime()) {
    return createDefaultSettings();
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return createDefaultSettings();
    }

    return normalizeSettings(JSON.parse(raw) as AppSettingsPatch);
  } catch (error) {
    console.error('[Dicta] Failed to read local settings:', error);
    return createDefaultSettings();
  }
}

export function saveSettings(settings: AppSettings): void {
  if (typeof window === 'undefined' || hasTauriRuntime()) {
    return;
  }

  window.localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify(settingsForLocalStorage(settings)),
  );
}

export async function loadSettingsWithRetry<T>(
  load: () => Promise<T>,
  retryDelaysMs: readonly number[] = [50, 150, 300, 600],
): Promise<T> {
  let lastError: unknown;

  for (let attempt = 0; attempt <= retryDelaysMs.length; attempt += 1) {
    try {
      return await load();
    } catch (error) {
      lastError = error;
      if (attempt === retryDelaysMs.length) break;

      const delayMs = retryDelaysMs[attempt];
      if (delayMs > 0) {
        await new Promise((resolve) => setTimeout(resolve, delayMs));
      }
    }
  }

  throw lastError;
}
