<script lang="ts">
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { PROVIDERS_LIST, type ProviderId } from '../../settings/model.js';
  import type { ProviderVerificationController } from '../../settings/ProviderVerificationController.svelte';
  import { i18n } from '../../i18n/index.js';
  import MorphIcon from '../ui/MorphIcon.svelte';

  interface Props {
    providerVerification: ProviderVerificationController;
    onSaveKey: (providerId: ProviderId, key: string) => Promise<void>;
    onClearKey: (providerId: ProviderId) => Promise<void>;
  }

  let {
    providerVerification,
    onSaveKey,
    onClearKey,
  }: Props = $props();

  let expandedProvider = $state<ProviderId | null>(null);
  let editingKeyVal = $state('');
  let copiedKeyProvider = $state<string | null>(null);
  let isDirtyMap = $state<Record<string, boolean>>({});
  let showKeyMap = $state<Record<string, boolean>>({
    openai: false,
    openrouter: false,
    groq: false,
    polza: false,
  });

  const toggleExpand = (providerId: ProviderId) => {
    if (expandedProvider === providerId) {
      expandedProvider = null;
    } else {
      expandedProvider = providerId;
      editingKeyVal = providerVerification.keys[providerId] || '';
      isDirtyMap[providerId] = false;
    }
  };

  const handleCopyKey = async (text: string, providerId: string) => {
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      copiedKeyProvider = providerId;
      setTimeout(() => {
        if (copiedKeyProvider === providerId) copiedKeyProvider = null;
      }, 1500);
    } catch {
      // fallback
    }
  };

  const handleSave = async (providerId: ProviderId) => {
    await onSaveKey(providerId, editingKeyVal);
    isDirtyMap[providerId] = false;
  };

  const handleClear = async (providerId: ProviderId) => {
    await onClearKey(providerId);
    editingKeyVal = '';
    isDirtyMap[providerId] = false;
  };

  const handleTest = async (providerId: ProviderId) => {
    await providerVerification.verify(providerId);
  };
</script>

<div class="settings-page-wrapper">
  <section class="settings-section">
    <div class="section-header-block">
      <h3 class="section-heading">{i18n.t.settings.providersHeading}</h3>
    </div>

    <div class="providers-list-card">
      {#each PROVIDERS_LIST as provider}
        {@const keyVal = providerVerification.keys[provider.id] || ''}
        {@const hasKey = Boolean(keyVal.trim())}
        {@const stInfo = providerVerification.statuses[provider.id] || { state: 'idle' }}
        {@const isChecking = stInfo.state === 'checking'}
        {@const isValid = stInfo.state === 'valid' || providerVerification.verifiedProviders.includes(provider.id)}
        {@const isError = stInfo.state === 'invalid_key' || stInfo.state === 'error'}
        {@const isExpanded = expandedProvider === provider.id}

        <div class="provider-item-wrap" class:expanded={isExpanded}>
          <!-- Row Button -->
          <button
            type="button"
            class="provider-header-row"
            onclick={() => toggleExpand(provider.id)}
            aria-expanded={isExpanded}
          >
            <!-- Column 1: Logo & Name -->
            <div class="provider-brand-slot">
              <div class="provider-logo-box">
                {#if provider.id === 'openai'}
                  <svg class="provider-logo-svg" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M22.28 9.82a5.98 5.98 0 0 0-.51-4.91 6.05 6.05 0 0 0-6.51-2.9A6.06 6.06 0 0 0 4.98 4.18a5.98 5.98 0 0 0-4 2.9 6.05 6.05 0 0 0 .74 7.1 5.98 5.98 0 0 0 .51 4.91 6.05 6.05 0 0 0 6.51 2.9A5.98 5.98 0 0 0 13.26 24a6.05 6.05 0 0 0 5.77-4.2 5.99 5.99 0 0 0 4-2.9 6.05 6.05 0 0 0-.75-7.08zm-9.02 12.6a4.48 4.48 0 0 1-2.88-1.04l.14-.08 4.78-2.76c.24-.14.39-.4.39-.68v-6.74l2.02 1.17c.02.01.04.03.04.05v5.58a4.5 4.5 0 0 1-4.49 4.5zm-9.66-4.12a4.47 4.47 0 0 1-.54-3.01l.15.08 4.78 2.76c.24.14.54.14.78 0l5.84-3.37v2.33a.08.08 0 0 1-.03.06L9.74 19.95a4.5 4.5 0 0 1-6.14-1.65zM2.34 7.9a4.48 4.48 0 0 1 2.37-1.98V11.6c0 .28.15.53.39.68l5.81 3.35-2.02 1.17a.08.08 0 0 1-.07 0l-4.83-2.79A4.5 4.5 0 0 1 2.34 7.87zm16.6 3.85L13.1 8.36l2.02-1.16a.08.08 0 0 1 .07 0l4.83 2.79a4.49 4.49 0 0 1-.68 8.1v-5.67a.79.79 0 0 0-.4-.67zm2.01-3.02l-.14-.09-4.77-2.78a.78.78 0 0 0-.79 0L9.41 9.23V6.9a.07.07 0 0 1 .03-.06l4.83-2.79a4.5 4.5 0 0 1 6.68 4.66zM8.31 12.86l-2.02-1.16a.08.08 0 0 1-.04-.06V6.07a4.5 4.5 0 0 1 7.38-3.45l-.15.08-4.78 2.76a.8.8 0 0 0-.39.68zm1.1-2.37l2.6-1.5 2.6 1.5v3l-2.6 1.5-2.6-1.5z"/>
                  </svg>
                {:else if provider.id === 'openrouter'}
                  <svg class="provider-logo-svg" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M18.654 3.87a5.087 5.087 0 1 1 0 10.174L23.7 19.09c.64.641.187 1.737-.72 1.737H8.48a8.479 8.479 0 0 1 0-16.958h10.175zM8.479 7.26a5.087 5.087 0 1 0 0 10.176 5.087 5.087 0 0 0 0-10.175z"/>
                  </svg>
                {:else if provider.id === 'groq'}
                  <svg class="provider-logo-svg" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12.036 2c-3.853-.035-7 3-7.036 6.781-.035 3.782 3.055 6.872 6.908 6.907h2.42v-2.566h-2.292c-2.407.028-4.38-1.866-4.408-4.23-.029-2.362 1.901-4.298 4.308-4.326h.1c2.407 0 4.358 1.915 4.365 4.278v6.305c0 2.342-1.944 4.25-4.323 4.279a4.375 4.375 0 01-3.033-1.252l-1.851 1.818A7 7 0 0012.029 22h.092c3.803-.056 6.858-3.083 6.879-6.816v-6.5C18.907 4.963 15.817 2 12.036 2z"/>
                  </svg>
                {:else if provider.id === 'polza'}
                  <svg class="provider-logo-svg" viewBox="0 0 718 718" fill="currentColor">
                    <path d="M288.512 0C271.821 0 258.291 13.5305 258.291 30.2211V115.917C258.291 124.182 261.676 132.086 267.658 137.789L338.145 204.998C349.818 216.128 368.175 216.13 379.85 205.002L450.369 137.79C450.522 137.644 450.673 137.497 450.823 137.348C456.524 131.682 459.74 123.969 459.74 115.913V115.805V30.2211C459.74 13.5304 446.209 0 429.519 0H288.512Z"/>
                    <path d="M55.3166 155.003C43.5164 166.804 43.5143 185.935 55.3121 197.738L115.865 258.317C121.709 264.164 129.692 267.361 137.956 267.165L235.355 264.853C251.484 264.47 264.468 251.486 264.851 235.358L267.163 137.957C267.359 129.694 264.162 121.71 258.315 115.866L197.737 55.3125C185.934 43.5147 166.803 43.5167 155.003 55.317L55.3166 155.003Z"/>
                    <path d="M0 429.522C0 446.212 13.5303 459.743 30.2209 459.743H115.913C124.179 459.743 132.085 456.356 137.789 450.372L205 379.853C216.128 368.178 216.126 349.822 204.996 338.148L137.788 267.66C132.085 261.678 124.181 258.293 115.916 258.293H30.2208C13.5303 258.293 0 271.824 0 288.514V429.522Z"/>
                    <path d="M154.998 662.711C166.8 674.516 185.937 674.517 197.74 662.714L258.32 602.133C264.163 596.29 267.359 588.308 267.163 580.046L264.851 482.643C264.468 466.514 251.484 453.53 235.355 453.147L137.956 450.835C129.692 450.639 121.709 453.836 115.865 459.683L55.3092 520.265C43.5126 532.067 43.5133 551.196 55.3109 562.997L154.998 662.711Z"/>
                    <path d="M429.519 718C446.209 718 459.74 704.47 459.74 687.779V602.087C459.74 593.82 456.353 585.914 450.369 580.21L379.85 512.998C368.175 501.87 349.818 501.872 338.145 513.002L267.658 580.211C261.676 585.914 258.291 593.818 258.291 602.083V687.779C258.291 704.47 271.821 718 288.512 718H429.519Z"/>
                    <path d="M662.711 563.001C674.513 551.199 674.514 532.064 662.712 520.262L602.128 459.678C596.285 453.835 588.303 450.639 580.042 450.835L482.639 453.147C466.51 453.53 453.526 466.514 453.144 482.643L450.831 580.046C450.635 588.308 453.831 596.29 459.674 602.133L520.257 662.717C532.059 674.519 551.194 674.519 562.996 662.717L662.711 563.001Z"/>
                    <path d="M717.994 288.514C717.994 271.824 704.464 258.293 687.773 258.293H602.157H602.078C594.04 258.293 586.343 261.495 580.68 267.173C580.52 267.334 580.362 267.496 580.206 267.66L512.998 338.148C501.868 349.822 501.866 368.178 512.994 379.853L580.205 450.372C585.909 456.356 593.815 459.743 602.081 459.743H687.773C704.464 459.743 717.994 446.212 717.994 429.522V288.514Z"/>
                    <path d="M267.959 359.128C267.959 308.778 308.776 267.961 359.125 267.961C409.475 267.961 450.292 308.778 450.292 359.128C450.292 409.478 409.475 450.295 359.125 450.295C308.776 450.295 267.959 409.478 267.959 359.128Z"/>
                    <path d="M459.74 115.805V115.913C459.74 123.969 456.524 131.682 450.823 137.348C450.824 137.551 450.827 137.754 450.831 137.957L453.144 235.358C453.526 251.486 466.51 264.47 482.639 264.853L580.042 267.165C580.255 267.17 580.468 267.173 580.68 267.173C586.343 261.495 594.04 258.293 602.078 258.293H602.157L662.709 197.741C674.512 185.938 674.51 166.8 662.706 154.999L562.992 55.3112C551.191 43.5136 532.062 43.5129 520.26 55.3096L459.74 115.805Z"/>
                  </svg>
                {/if}
              </div>
              <span class="provider-title">{provider.name}</span>
            </div>

            <!-- Column 2: Status Badge -->
            <div class="provider-status-badge">
              {#if isChecking}
                <span class="status-indicator checking">
                  <span class="status-dot pulse"></span>
                  <span>{i18n.t.settings.testingProvider}</span>
                </span>
              {:else if isValid}
                <span class="status-indicator connected">
                  <span>{i18n.t.settings.providerConnected}</span>
                </span>
              {:else if isError}
                <span class="status-indicator error">
                  <span class="status-dot red"></span>
                  <span>{i18n.t.settings.providerError}</span>
                </span>
              {:else if hasKey}
                <span class="status-indicator saved">
                  <span>{i18n.t.settings.keySaved}</span>
                </span>
              {:else}
                <span class="status-indicator not-configured">
                  <span>{i18n.t.settings.notConfigured}</span>
                </span>
              {/if}
            </div>

            <!-- Column 3: Chevron -->
            <div class="provider-chevron-slot">
              <svg class="chevron-arrow" class:open={isExpanded} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </div>
          </button>

          <!-- Expanded Key Drawer (180ms cubic-bezier curve) -->
          {#if isExpanded}
            <div class="provider-expanded-drawer" transition:slide={{ duration: 180, easing: cubicOut }}>
              <div class="drawer-inner-grid">
                <div class="key-direct-input-box">
                  <input
                    type={showKeyMap[provider.id] ? 'text' : 'password'}
                    class="key-text-input"
                    placeholder={i18n.t.settings.enterApiKey(provider.name)}
                    bind:value={editingKeyVal}
                    oninput={() => { isDirtyMap[provider.id] = true; }}
                    onkeydown={(e) => {
                      if (e.key === 'Enter') void handleSave(provider.id);
                    }}
                  />

                  <div class="key-inline-tools">
                    {#if editingKeyVal}
                      <button
                        type="button"
                        class="key-tool-btn"
                        onclick={() => void handleCopyKey(editingKeyVal, provider.id)}
                        title={copiedKeyProvider === provider.id ? i18n.t.history.copied : i18n.t.history.copyTooltip}
                      >
                        <MorphIcon active={copiedKeyProvider === provider.id} size={13} />
                      </button>

                      <button
                        type="button"
                        class="key-tool-btn"
                        onclick={() => { showKeyMap[provider.id] = !showKeyMap[provider.id]; }}
                        title={showKeyMap[provider.id] ? i18n.t.settings.hideKey : i18n.t.settings.showKey}
                      >
                        <svg class="tool-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                          {#if showKeyMap[provider.id]}
                            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"></path>
                            <line x1="1" y1="1" x2="23" y2="23"></line>
                          {:else}
                            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                            <circle cx="12" cy="12" r="3"></circle>
                          {/if}
                        </svg>
                      </button>

                      <button
                        type="button"
                        class="key-tool-btn danger"
                        onclick={() => void handleClear(provider.id)}
                        title={i18n.t.settings.clearKey}
                      >
                        <svg class="tool-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
                          <polyline points="3 6 5 6 21 6"></polyline>
                          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                        </svg>
                      </button>
                    {/if}
                  </div>
                </div>

                <div class="key-actions-row">
                  {#if isDirtyMap[provider.id]}
                    <button
                      type="button"
                      class="action-btn primary"
                      onclick={() => void handleSave(provider.id)}
                    >
                      {i18n.t.settings.saveKey}
                    </button>
                  {/if}

                  {#if hasKey}
                    <button
                      type="button"
                      class="action-btn secondary"
                      disabled={isChecking}
                      onclick={() => void handleTest(provider.id)}
                    >
                      {isChecking ? i18n.t.settings.testingProvider : i18n.t.settings.testProvider}
                    </button>
                  {/if}
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
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
    gap: 10px;
  }

  .section-header-block {
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
    padding-left: 2px;
  }

  /* Providers List */
  .providers-list-card {
    background: transparent;
    border: none;
    border-radius: 0;
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  :global(.theme-dark) .providers-list-card {
    background: transparent;
    border: none;
  }

  .provider-item-wrap {
    position: relative;
    border-radius: var(--radius-xs, 4px);
  }

  .provider-item-wrap + .provider-item-wrap {
    border-top: none;
  }

  :global(.theme-dark) .provider-item-wrap + .provider-item-wrap {
    border-top: none;
  }

  .provider-item-wrap.expanded {
    background: transparent;
  }

  :global(.theme-dark) .provider-item-wrap.expanded {
    background: transparent;
  }

  .provider-header-row {
    width: 100%;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    outline: none;
    transition: background 100ms ease;
    box-sizing: border-box;
    border-radius: var(--radius-xs, 4px);
  }

  .provider-header-row:hover {
    background: var(--hover-bg, rgba(0, 0, 0, 0.02));
  }

  .provider-brand-slot {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;
  }

  .provider-logo-box {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .provider-logo-svg {
    width: 18px;
    height: 18px;
  }

  .provider-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
  }

  /* Status Badge */
  .provider-status-badge {
    margin-right: 8px;
    flex-shrink: 0;
  }

  .status-indicator {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    font-weight: 450;
  }

  .status-indicator.connected {
    color: var(--text-secondary);
  }

  .status-indicator.error {
    color: #EF4444;
  }

  .status-indicator.saved {
    color: var(--text-secondary);
  }

  .status-indicator.not-configured {
    color: var(--text-tertiary);
  }

  .status-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
  }

  .status-dot.red {
    background: #EF4444;
  }

  .status-dot.pulse {
    background: #F59E0B;
    animation: pulse 1s infinite;
  }

  .provider-chevron-slot {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .chevron-arrow {
    width: 12px;
    height: 12px;
    transition: transform 0.5s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .chevron-arrow.open {
    transform: rotate(180deg);
  }

  /* Expanded Drawer */
  .provider-expanded-drawer {
    padding: 6px 4px 12px 4px;
    background: transparent;
    border-top: none;
    border-left: none;
  }

  :global(.theme-dark) .provider-expanded-drawer {
    background: transparent;
    border-top: none;
  }

  .drawer-inner-grid {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 520px;
  }

  .key-direct-input-box {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 32px;
    background: var(--bg-surface-raised, rgba(0, 0, 0, 0.02));
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xs, 4px);
    padding: 0 4px 0 10px;
    box-sizing: border-box;
    transition: border-color 150ms ease;
  }

  .key-direct-input-box:focus-within {
    border-color: var(--text-tertiary);
  }

  .key-text-input {
    flex: 1;
    height: 100%;
    border: none;
    background: transparent;
    outline: none;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    min-width: 0;
  }

  .key-inline-tools {
    display: flex;
    align-items: center;
    gap: 2px;
    flex-shrink: 0;
  }

  .key-tool-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: var(--radius-xs, 4px);
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color 120ms ease, background 120ms ease;
    padding: 0;
  }

  .key-tool-btn:hover {
    color: var(--text-primary);
    background: var(--hover-bg, rgba(0, 0, 0, 0.04));
  }

  .key-tool-btn.danger:hover {
    color: #EF4444;
    background: rgba(239, 68, 68, 0.08);
  }

  .tool-svg {
    width: 13px;
    height: 13px;
  }

  .key-actions-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .action-btn {
    height: 26px;
    padding: 0 10px;
    border-radius: var(--radius-xs, 4px);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    transition: all 100ms ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .action-btn.primary {
    background: var(--text-primary);
    color: var(--bg-surface);
    border: 1px solid var(--text-primary);
  }

  .action-btn.primary:hover {
    opacity: 0.9;
  }

  .action-btn.secondary {
    height: 24px;
    padding: 0 10px;
    background: var(--bg-control, rgba(0, 0, 0, 0.03));
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-xs, 4px);
    font-size: 11.5px;
    font-weight: 500;
  }

  .action-btn.secondary:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--hover-bg, rgba(0, 0, 0, 0.06));
    border-color: var(--border-strong, var(--text-tertiary));
  }

  .action-btn.secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
