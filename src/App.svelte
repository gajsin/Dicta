<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import './lib/styles/theme.css';
  import './lib/styles/appLayout.css';

  import HistoryView from './lib/components/HistoryView.svelte';
  import GeneralSettings from './lib/components/settings/GeneralSettings.svelte';
  import DictationSettings from './lib/components/settings/DictationSettings.svelte';
  import ProvidersSettings from './lib/components/settings/ProvidersSettings.svelte';
  import FloatingWidget from './lib/components/FloatingWidget.svelte';
  import CommandPalette from './lib/components/CommandPalette.svelte';
  import Toast from './lib/components/ui/Toast.svelte';
  import AppHeader from './lib/components/layout/AppHeader.svelte';
  import SettingsLayout, { type SettingsSection } from './lib/components/settings/SettingsLayout.svelte';

  import { i18n } from './lib/i18n/index.js';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { loadSettings } from './lib/utils/settings.js';
  import { AppSettingsController } from './lib/settings/AppSettingsController.svelte.js';
  import { HistoryController } from './lib/history/HistoryController.svelte.js';
  import { RecordingController } from './lib/recording/RecordingController.svelte.js';
  import { ProviderVerificationController } from './lib/settings/ProviderVerificationController.svelte.js';
  import type { ProviderId, ThemeMode, AppSettingsPatch } from './lib/settings/model.js';
  import { setupAppListeners, hydrateAndRevealMainWindow, type AppView } from './lib/window/appListeners.js';
  import { resolveActualTheme, applyAccentColor } from './lib/utils/accentTheme.js';

  const windowLabel = (() => {
    try {
      return getCurrentWindow().label;
    } catch {
      return 'main';
    }
  })();

  const initialSettings = loadSettings();
  const tauriInternals = typeof window !== 'undefined' ? window.__TAURI_INTERNALS__ : undefined;
  const isTauriRuntime = Boolean(tauriInternals?.invoke && tauriInternals?.transformCallback);

  const appSettings = new AppSettingsController({
    initialSettings,
    isTauriRuntime,
    windowLabel,
  });

  $effect(() => {
    applyAccentColor(appSettings.current.accentColor, appSettings.current.accentOpacity, false);
  });

  $effect(() => {
    i18n.setLanguage(appSettings.current.uiLanguage);
    recording.broadcast();
  });

  let searchQuery = $state('');
  let availableDevices = $state<Array<{ name: string; isDefault: boolean }>>([]);
  let currentView = $state<AppView>('history');

  const handleSelectView = (view: SettingsSection) => {
    currentView = view;
  };

  let showCommandPalette = $state(false);
  let toastTimeoutId = $state<number | undefined>(undefined);
  let notificationToast = $state('');

  const showToast = (msg: string) => {
    notificationToast = msg;
    if (toastTimeoutId !== undefined) clearTimeout(toastTimeoutId);
    toastTimeoutId = window.setTimeout(() => {
      notificationToast = '';
      toastTimeoutId = undefined;
    }, 3500);
  };

  const history = new HistoryController({
    onNotice: showToast,
    getSaveHistory: () => appSettings.current.saveHistory,
  });
  const recording = new RecordingController({
    isTauriRuntime,
    windowLabel,
    getSettings: () => appSettings.current,
    onHistoryItem: (text, durationSec, rawText) => {
      history.add(text, durationSec, rawText);
    },
    onOpenSettings: (section) => {
      currentView = section;
    },
  });

  const providerVerification = new ProviderVerificationController({
    persist: async (): Promise<void> => {
      await appSettings.update({
        providerApiKeys: { ...providerVerification.keys },
        verifiedProviders: [...providerVerification.verifiedProviders],
      });
    },
    deleteKey: async (provider: ProviderId): Promise<void> => {
      if (isTauriRuntime) {
        await invoke('delete_provider_api_key', { provider });
      }
    },
  });

  $effect(() => {
    providerVerification.hydrate(appSettings.current.providerApiKeys, appSettings.current.verifiedProviders);
  });

  $effect(() => {
    if (typeof document !== 'undefined') {
      const actual = resolveActualTheme(appSettings.current.theme);
      document.documentElement.classList.toggle('theme-dark', actual === 'dark');
      document.documentElement.classList.toggle('theme-light', actual === 'light');
    }
  });

  const toggleTheme = async () => {
    const theme = appSettings.current.theme;
    const nextMode: ThemeMode = theme === 'system' ? 'light' : theme === 'light' ? 'dark' : 'system';
    try {
      await appSettings.setTheme(nextMode);
      recording.broadcast();
    } catch (error) {
      console.error('[Dicta] Failed to update theme:', error);
      showToast(i18n.t.common.failedToSaveTheme);
    }
  };

  const refreshDevices = async () => {
    if (isTauriRuntime) {
      try {
        const devices = await invoke<Array<{ name: string; isDefault: boolean }>>('get_microphone_devices');
        availableDevices = devices;
      } catch (err) {
        console.error('[Dicta] Failed to fetch microphones:', err);
      }
    }
  };

  const handleCopyTranscriptText = async (text: string) => {
    try {
      if (isTauriRuntime) {
        await invoke('copy_and_paste', { text, paste: false });
      } else if (navigator.clipboard) {
        await navigator.clipboard.writeText(text);
      } else {
        throw new Error('Clipboard API unavailable');
      }
      showToast(i18n.t.common.textCopiedToast);
    } catch (error) {
      console.error('[Dicta] Clipboard write failed:', error);
      showToast(i18n.t.common.copyFailedToast);
    }
  };

  const handleSaveSettingsFromChild = async (newSettings: AppSettingsPatch) => {
    try {
      await appSettings.update(newSettings);
      recording.broadcast();
    } catch (error) {
      console.error('[Dicta] Settings update failed:', error);
      showToast(i18n.t.common.failedToSaveSettings);
    }
  };

  const handleSaveProviderKey = async (providerId: ProviderId, key: string) => {
    try {
      await providerVerification.save(providerId, key);
      recording.broadcast();
    } catch (error) {
      console.error('[Dicta] Failed to save provider key:', error);
      showToast(i18n.t.common.failedToSaveSettings);
    }
  };

  const handleClearProviderKey = async (providerId: ProviderId) => {
    try {
      await providerVerification.clear(providerId);
      recording.broadcast();
    } catch (error) {
      console.error('[Dicta] Failed to clear provider key:', error);
      showToast(i18n.t.common.failedToSaveSettings);
    }
  };

  const handleGlobalKeyDown = (e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      showCommandPalette = !showCommandPalette;
    }
  };

  onMount(() => {
    let mediaQuery: MediaQueryList | null = null;
    let handleMediaChange: () => void = () => {};
    if (typeof window !== 'undefined' && window.matchMedia) {
      mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      handleMediaChange = () => {
        if (appSettings.current.theme === 'system') {
          const actual = resolveActualTheme('system');
          document.documentElement.classList.toggle('theme-dark', actual === 'dark');
          document.documentElement.classList.toggle('theme-light', actual === 'light');
        }
      };
      mediaQuery.addEventListener('change', handleMediaChange);
    }

    const cleanupListeners = setupAppListeners({
      isTauriRuntime,
      windowLabel,
      recording,
      appSettings,
      onNavigate: (view) => {
        currentView = view;
      },
    });

    history.load();

    void hydrateAndRevealMainWindow(
      () => appSettings.reloadFromServer(),
      windowLabel === 'main' && isTauriRuntime,
    ).catch((error) => {
      console.error('[Dicta] Failed to reveal main window:', error);
    });

    refreshDevices();

    return () => {
      if (mediaQuery && handleMediaChange) {
        mediaQuery.removeEventListener('change', handleMediaChange);
      }
      cleanupListeners();
    };
  });

  onDestroy(() => {
    recording.dispose();
    if (toastTimeoutId !== undefined) {
      clearTimeout(toastTimeoutId);
    }
  });
</script>

<svelte:window onkeydown={handleGlobalKeyDown} />

<svelte:head>
  <title>Dicta</title>
</svelte:head>

<main class="app-root theme-{appSettings.current.theme}" class:overlay-root={windowLabel === 'overlay'}>
  {#if windowLabel !== 'overlay'}
    <div class="window-shell">
      <AppHeader />

      <!-- Unified Desktop Navigation: History, General, Dictation, Providers -->
      <SettingsLayout
        activeSection={currentView}
        onSelectSection={handleSelectView}
      >
        {#if currentView === 'history'}
          <HistoryView
            historyList={history.items}
            bind:selectedId={history.selectedId}
            {searchQuery}
            onSearchChange={(q) => { searchQuery = q; }}
            saveHistory={appSettings.current.saveHistory}
            onDelete={(id) => history.delete(id)}
            onCopyText={handleCopyTranscriptText}
            onOpenPrivacySettings={() => {
              currentView = 'general';
            }}
          />
        {:else if currentView === 'general'}
          <GeneralSettings
            hotkey={appSettings.current.hotkey}
            autostart={appSettings.current.autostart}
            uiLanguage={appSettings.current.uiLanguage}
            theme={appSettings.current.theme}
            accentColor={appSettings.current.accentColor}
            saveHistory={appSettings.current.saveHistory}
            onClearHistory={() => history.clear()}
            onUpdate={handleSaveSettingsFromChild}
          />
        {:else if currentView === 'dictation'}
          <DictationSettings
            selectedDeviceId={appSettings.current.selectedDeviceId}
            {availableDevices}
            selectedLanguage={appSettings.current.selectedLanguage}
            sttProvider={appSettings.current.sttProvider}
            selectedModel={appSettings.current.selectedModel}
            postprocessEnabled={appSettings.current.postprocessEnabled}
            postprocessMode={appSettings.current.postprocessMode}
            llmProvider={appSettings.current.llmProvider}
            postprocessModel={appSettings.current.postprocessModel}
            onUpdate={handleSaveSettingsFromChild}
          />
        {:else if currentView === 'providers'}
          <ProvidersSettings
            {providerVerification}
            onSaveKey={handleSaveProviderKey}
            onClearKey={handleClearProviderKey}
          />
        {/if}
      </SettingsLayout>

      <Toast message={notificationToast} onUndo={history.canUndo ? () => history.undo() : undefined} />

      <CommandPalette
        visible={showCommandPalette}
        historyItems={history.items}
        currentTheme={appSettings.current.theme}
        hotkey={appSettings.current.hotkey}
        onClose={() => { showCommandPalette = false; }}
        onNavigateView={(v) => {
          currentView = v;
        }}
        onSelectHistory={(id) => {
          history.selectedId = id;
          currentView = 'history';
        }}
        onToggleTheme={toggleTheme}
        onTriggerRecording={() => recording.toggle()}
        onSetMode={(m) => {
          void appSettings
            .update({
              postprocessMode: m,
              postprocessEnabled: m !== 'raw',
            })
            .then(() => {
              showToast(i18n.t.common.modeChanged(i18n.t.modes[m]));
            })
            .catch((error) => {
              console.error('[Dicta] Failed to update processing mode:', error);
              showToast(i18n.t.common.failedToSaveMode);
            });
        }}
      />
    </div>
  {:else if windowLabel === 'overlay'}
    <FloatingWidget
      visible={true}
      widgetState={recording.widgetState}
      audioLevel={recording.audioLevel}
      timerSeconds={recording.timerSeconds}
      message={recording.widgetMessage}
      shortLabel={recording.shortLabel}
      hotkeyName={appSettings.current.hotkey}
      resolvedTheme={recording.resolvedTheme}
      errorActionType={recording.errorActionType}
      onStop={() => recording.sendWidgetAction('stop')}
      onCancel={() => recording.sendWidgetAction('cancel')}
      onRetry={() => recording.sendWidgetAction('retry')}
      onOpenSettings={() => recording.sendWidgetAction('open_settings')}
      onClose={() => recording.sendWidgetAction('close')}
    />
  {/if}
</main>
