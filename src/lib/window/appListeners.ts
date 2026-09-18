import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { applyAccentColor, type AccentThemeChange } from '../utils/accentTheme.js';
import { i18n } from '../i18n/index.js';
import type { RecordingController } from '../recording/RecordingController.svelte.js';
import type { AppSettingsController } from '../settings/AppSettingsController.svelte.js';
import type { WidgetPayload } from '../recording/status.js';

export type AppView = 'history' | 'general' | 'dictation' | 'providers';

interface ListenerOptions {
  isTauriRuntime: boolean;
  windowLabel: string;
  recording: RecordingController;
  appSettings: AppSettingsController;
  onNavigate: (view: AppView) => void;
}

export function setupAppListeners({
  isTauriRuntime,
  windowLabel,
  recording,
  appSettings,
  onNavigate,
}: ListenerOptions): () => void {
  if (!isTauriRuntime) {
    return () => {};
  }

  const unlistenPromises: Promise<UnlistenFn>[] = [];

  unlistenPromises.push(
    listen<AccentThemeChange | string>('accent-color-changed', (e) => {
      const change =
        typeof e.payload === 'string'
          ? { color: e.payload, opacity: 100 }
          : e.payload;
      applyAccentColor(change.color, change.opacity, false);
    }),
  );

  unlistenPromises.push(
    listen('settings-updated', () => {
      void appSettings.reloadFromServer().catch((error) => {
        console.error('[Dicta] Failed to reload settings:', error);
      });
    }),
  );

  unlistenPromises.push(
    listen<string>('navigate-view', (event) => {
      if (windowLabel === 'main') {
        const p = event.payload;
        if (
          p === 'general' ||
          p === 'dictation' ||
          p === 'history' ||
          p === 'providers'
        ) {
          onNavigate(p as AppView);
        }
      }
    }),
  );

  if (windowLabel === 'main') {
    unlistenPromises.push(
      listen<number>('audio-level', (e) => {
        recording.setAudioLevel(e.payload);
      }),
      listen('toggle-recording', () => {
        void recording.toggle();
      }),
      listen('request-widget-sync', () => {
        recording.broadcast();
      }),
      listen<string>('recording-device-error', (e) => {
        recording.handleDeviceError(e.payload);
      }),
      listen<string>('recording-timeout', (e) => {
        recording.handleRecordingTimeout(e.payload);
      }),
      listen<string>('widget-action', (e) => {
        recording.handleWidgetAction(e.payload);
      }),
      listen('cancel-recording', () => {
        recording.cancel();
      }),
    );
  }

  if (windowLabel === 'overlay') {
    if (typeof document !== 'undefined') {
      document.documentElement.style.background = 'transparent';
      document.body.style.background = 'transparent';
      document.documentElement.classList.add('theme-dark');
      document.documentElement.classList.remove('theme-light');
    }

    unlistenPromises.push(
      listen<WidgetPayload>('widget-update', (e) => {
        recording.applyWidgetUpdate(e.payload);
        if (
          e.payload.theme === 'light' ||
          e.payload.theme === 'dark' ||
          e.payload.theme === 'system'
        ) {
          appSettings.apply({ theme: e.payload.theme });
        }
        const effectiveTheme = e.payload.resolvedTheme || (e.payload.theme ? (e.payload.theme === 'light' ? 'light' : 'dark') : undefined);
        if (effectiveTheme && typeof document !== 'undefined') {
          document.documentElement.classList.toggle('theme-dark', effectiveTheme === 'dark');
          document.documentElement.classList.toggle('theme-light', effectiveTheme === 'light');
        }
        if (e.payload.hotkey !== undefined) {
          appSettings.apply({ hotkey: e.payload.hotkey });
        }
        if (e.payload.accentColor) {
          applyAccentColor(
            e.payload.accentColor,
            e.payload.accentOpacity ?? 100,
            false,
          );
        }
        if (e.payload.uiLanguage) {
          i18n.setLanguage(e.payload.uiLanguage);
        }
      }),
    );
  }

  return () => {
    unlistenPromises.forEach((promise) => {
      promise.then((unlisten) => unlisten()).catch(console.error);
    });
  };
}

export async function hydrateAndRevealMainWindow(
  reloadSettings: () => Promise<void>,
  shouldReveal: boolean,
): Promise<void> {
  try {
    await reloadSettings();
  } catch (error) {
    console.error('[Dicta] Failed to load settings:', error);
  }
  if (!shouldReveal) return;

  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => resolve());
  });
  await invoke('reveal_main_window');
}
