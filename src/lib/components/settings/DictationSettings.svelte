<script lang="ts">
  import { onDestroy } from 'svelte';
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import Toggle from '../ui/Toggle.svelte';
  import Select from '../ui/Select.svelte';
  import { i18n } from '../../i18n/index.js';
  import {
    PROVIDERS_LIST,
    STT_MODELS_BY_PROVIDER,
    LLM_MODELS_BY_PROVIDER,
    type ProviderId,
  } from '../../settings/model.js';
  import { MicrophoneTestController } from '../../settings/MicrophoneTestController.svelte.js';

  interface Props {
    selectedDeviceId: string;
    availableDevices: Array<{ name: string; isDefault: boolean }>;
    selectedLanguage: string;
    sttProvider: ProviderId;
    selectedModel: string;
    postprocessEnabled: boolean;
    postprocessMode: 'raw' | 'minimal' | 'balanced' | 'business';
    llmProvider: ProviderId;
    postprocessModel: string;
    onUpdate: (patch: {
      selectedDeviceId?: string;
      selectedLanguage?: string;
      sttProvider?: ProviderId;
      selectedModel?: string;
      postprocessEnabled?: boolean;
      postprocessMode?: 'raw' | 'minimal' | 'balanced' | 'business';
      llmProvider?: ProviderId;
      postprocessModel?: string;
    }) => Promise<void>;
  }

  let {
    selectedDeviceId,
    availableDevices,
    selectedLanguage,
    sttProvider,
    selectedModel,
    postprocessEnabled,
    postprocessMode,
    llmProvider,
    postprocessModel,
    onUpdate,
  }: Props = $props();

  const microphoneTest = new MicrophoneTestController(() => selectedDeviceId);

  onDestroy(() => {
    microphoneTest.dispose();
  });

  const deviceOptions = $derived([
    { value: '', label: i18n.t.settings.microphoneDefault },
    ...availableDevices.map((dev) => ({
      value: dev.name,
      label: `${dev.name}${dev.isDefault ? ` (${i18n.t.common.defaultBadge})` : ''}`,
    })),
  ]);

  const providerOptions = PROVIDERS_LIST.map((p) => ({ value: p.id, label: p.name }));

  const sttModelOptions = $derived(
    STT_MODELS_BY_PROVIDER[sttProvider] || []
  );

  const languageOptions = $derived([
    { value: 'auto', label: i18n.t.languages.auto },
    { value: 'ru', label: i18n.t.languages.ru },
    { value: 'en', label: i18n.t.languages.en },
    { value: 'de', label: i18n.t.languages.de },
    { value: 'fr', label: i18n.t.languages.fr },
    { value: 'es', label: i18n.t.languages.es },
  ]);

  const modeOptions = $derived([
    { value: 'raw', label: i18n.t.modes.raw },
    { value: 'minimal', label: i18n.t.modes.minimal },
    { value: 'balanced', label: i18n.t.modes.balanced },
    { value: 'business', label: i18n.t.modes.business },
  ]);

  const llmModelOptions = $derived(
    LLM_MODELS_BY_PROVIDER[llmProvider] || []
  );

  const handleSttProviderSelect = (val: string) => {
    const nextProvider = val as ProviderId;
    const available = STT_MODELS_BY_PROVIDER[nextProvider] || [];
    const nextModel = available.length > 0 ? available[0].value : '';
    void onUpdate({ sttProvider: nextProvider, selectedModel: nextModel });
  };

  const handleLlmProviderSelect = (val: string) => {
    const nextProvider = val as ProviderId;
    const available = LLM_MODELS_BY_PROVIDER[nextProvider] || [];
    const nextModel = available.length > 0 ? available[0].value : '';
    void onUpdate({ llmProvider: nextProvider, postprocessModel: nextModel });
  };
</script>

<div class="settings-page-wrapper">
  <!-- СЕКЦИЯ 1: Микрофон -->
  <section class="settings-section">
    <h3 class="section-heading">{i18n.t.settings.sectionMicrophone}</h3>
    <div class="settings-card">
      <div class="card-row mic-row">
        <span class="row-label">{i18n.t.settings.microphone}</span>
        <div class="row-controls">
          <div class="mic-controls">
            <div class="mic-select-wrap">
              <Select
                value={selectedDeviceId}
                options={deviceOptions}
                onChange={(val) => void onUpdate({ selectedDeviceId: val })}
                ariaLabel={i18n.t.settings.microphone}
              />
            </div>

            <button
              type="button"
              class="compact-action-btn"
              class:testing={microphoneTest.isActive}
              onclick={() => void microphoneTest.toggleTest()}
              title={microphoneTest.isActive ? i18n.t.topbar.stop : i18n.t.settings.testMic}
              aria-label={microphoneTest.isActive ? i18n.t.topbar.stop : i18n.t.settings.testMic}
            >
              <span class="btn-morph-icon-slot">
                <svg class="morph-icon-svg" viewBox="0 0 16 16">
                  <polygon
                    class="morph-play-shape"
                    class:hidden={microphoneTest.isActive}
                    points="4.5,3 12.5,8 4.5,13"
                    fill="currentColor"
                  />
                  <rect
                    class="morph-stop-shape"
                    class:visible={microphoneTest.isActive}
                    x="3.5"
                    y="3.5"
                    width="9"
                    height="9"
                    rx="1.5"
                    fill="currentColor"
                  />
                </svg>
              </span>
            </button>
          </div>
        </div>
      </div>

      {#if microphoneTest.errorMessage}
        <div class="card-subrow error-subrow" transition:slide={{ duration: 200, easing: cubicOut }}>
          <span class="error-msg">{microphoneTest.errorMessage}</span>
        </div>
      {/if}

      {#if microphoneTest.isActive}
        <div class="card-subrow meter-subrow" transition:slide={{ duration: 240, easing: cubicOut }}>
          <span class="meter-label">{i18n.t.settings.signalLevel}</span>
          <div class="meter-track">
            <div class="meter-fill" style="width: {microphoneTest.volumeLevel}%;"></div>
          </div>
          <span class="meter-val">{microphoneTest.volumeLevel}%</span>
        </div>
      {/if}
    </div>
  </section>

  <!-- СЕКЦИЯ 2: Распознавание речи -->
  <section class="settings-section">
    <h3 class="section-heading">{i18n.t.settings.sectionSpeechRecognition}</h3>
    <div class="settings-card">
      <!-- Провайдер -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.sttProvider}</span>
        <div class="row-controls">
          <Select
            value={sttProvider}
            options={providerOptions}
            onChange={handleSttProviderSelect}
            ariaLabel={i18n.t.settings.sttProvider}
          />
        </div>
      </div>

      <!-- Модель -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.sttModel}</span>
        <div class="row-controls">
          <Select
            value={selectedModel}
            options={sttModelOptions}
            onChange={(val) => void onUpdate({ selectedModel: val })}
            ariaLabel={i18n.t.settings.sttModel}
          />
        </div>
      </div>

      <!-- Язык -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.transcriptionLang}</span>
        <div class="row-controls">
          <Select
            value={selectedLanguage}
            options={languageOptions}
            onChange={(val) => void onUpdate({ selectedLanguage: val })}
            ariaLabel={i18n.t.settings.transcriptionLang}
          />
        </div>
      </div>
    </div>
  </section>

  <!-- СЕКЦИЯ 3: Обработка текста -->
  <section class="settings-section">
    <h3 class="section-heading">{i18n.t.settings.sectionTextProcessing}</h3>
    <div class="settings-card">
      <!-- Постобработка -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.postprocessing}</span>
        <div class="row-controls">
          <Toggle
            checked={postprocessEnabled}
            onchange={(checked) => void onUpdate({ postprocessEnabled: checked })}
          />
        </div>
      </div>

      <div class="sub-settings-group" class:disabled={!postprocessEnabled}>
        <!-- Режим -->
        <div class="card-row">
          <span class="row-label">{i18n.t.settings.processingMode}</span>
          <div class="row-controls">
            <Select
              value={postprocessMode}
              options={modeOptions}
              disabled={!postprocessEnabled}
              onChange={(val) => void onUpdate({ postprocessMode: val as any })}
              ariaLabel={i18n.t.settings.processingMode}
            />
          </div>
        </div>

        <!-- Провайдер LLM -->
        <div class="card-row">
          <span class="row-label">{i18n.t.settings.llmProvider}</span>
          <div class="row-controls">
            <Select
              value={llmProvider}
              options={providerOptions}
              disabled={!postprocessEnabled}
              onChange={handleLlmProviderSelect}
              ariaLabel={i18n.t.settings.llmProvider}
            />
          </div>
        </div>

        <!-- Модель LLM -->
        <div class="card-row">
          <span class="row-label">{i18n.t.settings.llmModel}</span>
          <div class="row-controls">
            <Select
              value={postprocessModel}
              options={llmModelOptions}
              disabled={!postprocessEnabled}
              onChange={(val) => void onUpdate({ postprocessModel: val })}
              ariaLabel={i18n.t.settings.llmModel}
            />
          </div>
        </div>
      </div>
    </div>
  </section>
</div>

<style>
  .settings-page-wrapper {
    max-width: 520px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-heading {
    margin: 0;
    font-size: 11px;
    font-weight: 550;
    color: var(--text-tertiary);
    letter-spacing: 0.03em;
    text-transform: uppercase;
    padding: 0 4px;
  }

  .settings-card {
    background: transparent;
    border: none;
    border-radius: 0;
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 1px;
    overflow: visible;
  }

  :global(.theme-dark) .settings-card {
    background: transparent;
    border: none;
  }

  .card-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 4px;
    box-sizing: border-box;
    position: relative;
    border-radius: var(--radius-xs, 4px);
    transition: background 100ms ease;
  }

  .card-row:hover {
    background: var(--hover-bg, rgba(0, 0, 0, 0.02));
  }

  :global(.theme-dark) .card-row:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .card-row + .card-row,
  .card-row + .sub-settings-group,
  .sub-settings-group .card-row {
    border-top: none;
  }

  :global(.theme-dark) .card-row + .card-row,
  :global(.theme-dark) .card-row + .sub-settings-group,
  :global(.theme-dark) .sub-settings-group .card-row {
    border-top: none;
  }

  .card-row:has(:global(.ui-select.is-open)) {
    z-index: 50;
  }

  .row-label {
    font-size: 12.5px;
    color: var(--text-primary);
    font-weight: 450;
    flex-shrink: 0;
  }

  .row-controls {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    flex: 1;
  }

  .row-controls :global(.ui-select) {
    width: 200px;
    flex-shrink: 0;
  }

  .sub-settings-group {
    display: flex;
    flex-direction: column;
    transition: opacity 150ms ease;
  }

  .sub-settings-group.disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  /* Mic row specific */
  .mic-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 200px;
    max-width: 200px;
    flex: 0 0 200px;
  }

  .mic-select-wrap {
    flex: 1;
    min-width: 0;
  }

  .mic-select-wrap :global(.ui-select) {
    width: 100%;
  }

  .mic-select-wrap :global(.select-trigger) {
    width: 100%;
    box-sizing: border-box;
  }

  .compact-action-btn {
    width: 26px;
    height: 26px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-control, rgba(0, 0, 0, 0.03));
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs, 4px);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 100ms ease;
    flex-shrink: 0;
    box-sizing: border-box;
  }

  .compact-action-btn:hover:not(:disabled) {
    background: var(--hover-bg, rgba(0, 0, 0, 0.06));
    border-color: var(--border-color);
    color: var(--text-primary);
  }

  :global(.theme-dark) .compact-action-btn {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.08);
  }

  :global(.theme-dark) .compact-action-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .btn-morph-icon-slot {
    width: 12px;
    height: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    position: relative;
    flex-shrink: 0;
  }

  .morph-icon-svg {
    width: 12px;
    height: 12px;
    display: block;
    overflow: visible;
  }

  .morph-play-shape {
    transform-origin: center;
    transition: transform 0.45s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.3s ease;
    transform: scale(1) rotate(0deg);
    opacity: 1;
  }

  .morph-play-shape.hidden {
    transform: scale(0.2) rotate(45deg);
    opacity: 0;
    pointer-events: none;
  }

  .morph-stop-shape {
    transform-origin: center;
    transition: transform 0.45s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.3s ease;
    transform: scale(0.2) rotate(-45deg);
    opacity: 0;
    pointer-events: none;
  }

  .morph-stop-shape.visible {
    transform: scale(1) rotate(0deg);
    opacity: 1;
    pointer-events: auto;
  }

  .compact-action-btn:hover {
    color: var(--text-primary);
    border-color: var(--border-strong);
    background: var(--hover-surface, #F4F4F5);
  }

  .compact-action-btn.testing {
    background: var(--danger-color, #EF4444);
    border-color: var(--danger-color, #EF4444);
    color: #FFFFFF;
  }

  /* Subrows (meter, error) */
  .card-subrow {
    padding: 10px 16px;
    background: var(--bg-muted, #F4F4F5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .meter-subrow {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .meter-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .meter-track {
    flex: 1;
    height: 6px;
    background: var(--border-color);
    border-radius: 3px;
    overflow: hidden;
  }

  .meter-fill {
    height: 100%;
    background: var(--accent-primary, #FF9500);
    border-radius: 3px;
    transition: width 60ms linear;
  }

  .meter-val {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    width: 32px;
    text-align: right;
  }

  .error-subrow {
    color: #EF4444;
    font-size: 12px;
  }
</style>
