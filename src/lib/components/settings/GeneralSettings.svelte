<script lang="ts">
  import { slide } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { invoke } from '@tauri-apps/api/core';
  import Toggle from '../ui/Toggle.svelte';
  import Select from '../ui/Select.svelte';
  import ConfirmModal from '../ui/ConfirmModal.svelte';
  import { i18n, type UiLanguage } from '../../i18n/index.js';
  import type { ThemeMode } from '../../settings/model.js';
  import { hotkeyFromKeyboardEvent } from '../../settings/hotkey.js';
  import { applyAccentColor } from '../../utils/accentTheme.js';

  interface Props {
    hotkey: string;
    autostart: boolean;
    uiLanguage: UiLanguage;
    theme: ThemeMode;
    accentColor: string;
    saveHistory: boolean;
    onClearHistory: () => void;
    onUpdate: (patch: {
      hotkey?: string;
      autostart?: boolean;
      uiLanguage?: UiLanguage;
      theme?: ThemeMode;
      accentColor?: string;
      saveHistory?: boolean;
    }) => Promise<void>;
  }

  let {
    hotkey,
    autostart,
    uiLanguage,
    theme,
    accentColor,
    saveHistory,
    onClearHistory,
    onUpdate,
  }: Props = $props();

  let isCapturingHotkey = $state(false);
  let hotkeyErrorMsg = $state('');
  let showConfirmClear = $state(false);

  const ACCENT_PRESETS = $derived([
    { color: '#FF9500', name: i18n.t.settings.colorOrange },
    { color: '#3B82F6', name: i18n.t.settings.colorBlue },
    { color: '#10B981', name: i18n.t.settings.colorEmerald },
    { color: '#8B5CF6', name: i18n.t.settings.colorPurple },
    { color: '#EC4899', name: i18n.t.settings.colorPink },
    { color: '#18181B', name: i18n.t.settings.colorMonochrome },
  ]);

  const handleHotkeyKeyDown = (e: KeyboardEvent) => {
    if (!isCapturingHotkey) return;
    e.preventDefault();
    e.stopPropagation();

    const result = hotkeyFromKeyboardEvent(e);
    if (result.kind === 'pending') return;
    if (result.kind === 'error') {
      hotkeyErrorMsg = i18n.t.settings.unsupportedHotkey(result.key || e.key || e.code);
      return;
    }

    const next = result.hotkey;
    isCapturingHotkey = false;
    hotkeyErrorMsg = '';

    void onUpdate({ hotkey: next }).catch((error) => {
      hotkeyErrorMsg = String(error || 'Failed to apply hotkey');
    });
  };

  const handleSelectAccent = (color: string) => {
    applyAccentColor(color, 100, true);
    void onUpdate({ accentColor: color });
  };

  let isCustomColorDrawerOpen = $state(false);
  let isPickingColor = $state(false);

  let colorInputRef: HTMLInputElement | null = $state(null);

  const pickColorWithEyeDropper = async () => {
    if (isPickingColor) return;
    isPickingColor = true;
    if (typeof document !== 'undefined') {
      document.body.style.cursor = 'crosshair';
    }
    try {
      // 1. Native Windows GDI pixel sampler across all displays
      const picked = await invoke<string | null>('pick_screen_color');
      if (picked) {
        handleSelectAccent(picked.toUpperCase());
        return;
      }
    } catch (err) {
      console.warn('[Dicta] pick_screen_color failed or unavailable, using fallback:', err);
      // 2. Web EyeDropper API fallback if available
      if (typeof window !== 'undefined' && 'EyeDropper' in window) {
        try {
          const eyeDropper = new (window as any).EyeDropper();
          const result = await eyeDropper.open();
          if (result?.sRGBHex) {
            handleSelectAccent(result.sRGBHex.toUpperCase());
            return;
          }
        } catch (e: any) {
          if (e?.name === 'AbortError') return;
        }
      }
      // 3. Native system color dialog fallback
      if (colorInputRef) {
        colorInputRef.click();
      }
    } finally {
      if (typeof document !== 'undefined') {
        document.body.style.cursor = '';
      }
      isPickingColor = false;
    }
  };

  function hexToHsl(hex: string): { h: number; s: number; l: number } {
    let c = hex.replace('#', '');
    if (c.length === 3) c = c.split('').map((x) => x + x).join('');
    const num = parseInt(c, 16) || 0;
    const r = ((num >> 16) & 255) / 255;
    const g = ((num >> 8) & 255) / 255;
    const b = (num & 255) / 255;

    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    let h = 0;
    let s = 0;
    const l = (max + min) / 2;

    if (max !== min) {
      const d = max - min;
      s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
      switch (max) {
        case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
        case g: h = ((b - r) / d + 2) / 6; break;
        case b: h = ((r - g) / d + 4) / 6; break;
      }
    }
    return {
      h: Math.round(h * 360),
      s: Math.round(s * 100),
      l: Math.round(l * 100),
    };
  }

  function hslToHex(h: number, s: number, l: number): string {
    const sNorm = Math.max(0, Math.min(100, s)) / 100;
    const lNorm = Math.max(0, Math.min(100, l)) / 100;
    const a = sNorm * Math.min(lNorm, 1 - lNorm);
    const f = (n: number) => {
      const k = (n + h / 30) % 12;
      const color = lNorm - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
      return Math.round(255 * color).toString(16).padStart(2, '0');
    };
    return `#${f(0)}${f(8)}${f(4)}`.toUpperCase();
  }

  let customH = $state(35);
  let customS = $state(100);
  let customL = $state(50);
  let customHexInput = $state('');

  $effect(() => {
    const hsl = hexToHsl(accentColor);
    customH = hsl.h;
    customS = hsl.s;
    customL = hsl.l;
    customHexInput = accentColor.toUpperCase();
  });

  const handleHueChange = (hue: number) => {
    customH = hue;
    const sat = Math.max(75, customS);
    const lit = Math.max(25, Math.min(75, customL));
    const newHex = hslToHex(customH, sat, lit);
    customHexInput = newHex;
    handleSelectAccent(newHex);
  };

  const handleLightnessChange = (lit: number) => {
    customL = lit;
    const sat = Math.max(75, customS);
    const newHex = hslToHex(customH, sat, customL);
    customHexInput = newHex;
    handleSelectAccent(newHex);
  };

  const handleHexInputChange = (e: Event) => {
    const target = e.currentTarget as HTMLInputElement;
    let val = target.value.trim();
    if (!val.startsWith('#')) val = '#' + val;
    customHexInput = val;
    if (/^#[0-9A-Fa-f]{6}$/.test(val)) {
      handleSelectAccent(val);
    }
  };

  const isCustomAccent = $derived(
    !ACCENT_PRESETS.some((p) => p.color.toUpperCase() === accentColor.toUpperCase())
  );

  const themeOptions = $derived([
    { value: 'system', label: i18n.t.settings.themeSystem },
    { value: 'light', label: i18n.t.settings.themeLight },
    { value: 'dark', label: i18n.t.settings.themeDark },
  ]);

  const uiLanguageOptions = $derived([
    { value: 'system', label: i18n.t.languages.system },
    { value: 'en', label: 'English' },
    { value: 'ru', label: 'Русский' },
  ]);
</script>

<svelte:window onkeydown={handleHotkeyKeyDown} />

<div class="settings-page-wrapper">
  {#if hotkeyErrorMsg}
    <div class="settings-error-banner">
      {hotkeyErrorMsg}
    </div>
  {/if}

  <!-- СЕКЦИЯ 1: Приложение -->
  <section class="settings-section">
    <h3 class="section-heading">{i18n.t.settings.sectionApp}</h3>
    <div class="settings-card">
      <!-- 1. Язык интерфейса -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.interfaceLang}</span>
        <div class="row-controls">
          <Select
            value={uiLanguage}
            options={uiLanguageOptions}
            onChange={(val) => void onUpdate({ uiLanguage: val as UiLanguage })}
            ariaLabel={i18n.t.settings.interfaceLang}
          />
        </div>
      </div>

      <!-- 2. Горячая клавиша -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.hotkey}</span>
        <div class="row-controls">
          <button
            type="button"
            class="hotkey-recorder-button"
            class:capturing={isCapturingHotkey}
            onclick={() => {
              isCapturingHotkey = !isCapturingHotkey;
              hotkeyErrorMsg = '';
            }}
            title={isCapturingHotkey ? i18n.t.settings.cancelHotkey : i18n.t.settings.changeHotkey}
            aria-label={i18n.t.settings.hotkey}
          >
            {#if isCapturingHotkey}
              <div class="hotkey-capturing-wrap">
                <span class="hotkey-capturing-pulse"></span>
                <span class="hotkey-capturing-text">{i18n.t.settings.pressKey}</span>
              </div>
            {:else}
              <kbd class="hotkey-keycap">{hotkey}</kbd>
            {/if}
          </button>
        </div>
      </div>

      <!-- 3. Запуск с Windows -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.startWithWindows}</span>
        <div class="row-controls">
          <Toggle
            checked={autostart}
            onchange={(checked) => void onUpdate({ autostart: checked })}
          />
        </div>
      </div>
    </div>
  </section>

  <!-- СЕКЦИЯ 2: Внешний вид -->
  <section class="settings-section">
    <h3 class="section-heading">{i18n.t.settings.sectionAppearance}</h3>
    <div class="settings-card">
      <!-- 4. Тема оформления -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.theme}</span>
        <div class="row-controls">
          <Select
            value={theme}
            options={themeOptions}
            onChange={(val) => void onUpdate({ theme: val as ThemeMode })}
            ariaLabel={i18n.t.settings.theme}
          />
        </div>
      </div>

      <!-- 5. Акцентный цвет -->
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.accentColor}</span>
        <div class="row-controls">
          <div class="accent-swatches-strip">
            {#each ACCENT_PRESETS as p}
              {@const isSelected = accentColor.toUpperCase() === p.color.toUpperCase()}
              <button
                type="button"
                class="swatch-btn"
                class:selected={isSelected}
                style="background-color: {p.color};"
                title={p.name}
                onclick={() => {
                  isCustomColorDrawerOpen = false;
                  handleSelectAccent(p.color);
                }}
                aria-label={p.name}
              >
                {#if isSelected}
                  <svg class="swatch-check-svg" viewBox="0 0 16 16" fill="none">
                    <polyline points="3.5 8.5 6.5 11.5 12.5 4.5" stroke="#FFFFFF" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" />
                  </svg>
                {/if}
              </button>
            {/each}

            <!-- Custom In-App Color Picker Toggle Button with Morph Icon -->
            <button
              type="button"
              class="swatch-btn custom-color-picker-btn"
              class:selected={isCustomAccent || isCustomColorDrawerOpen}
              style="background-color: {isCustomAccent ? accentColor : 'transparent'};"
              title={i18n.t.settings.customRgbPicker}
              onclick={() => { isCustomColorDrawerOpen = !isCustomColorDrawerOpen; }}
              aria-label={i18n.t.settings.customRgbPicker}
            >
              <div class="morph-toggle-icon-slot">
                <!-- Pipette icon (visible when closed) -->
                <svg
                  class="custom-color-icon pipette-shape"
                  class:hidden={isCustomColorDrawerOpen}
                  class:active-custom={isCustomAccent}
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <path d="M12 19l7-7 3 3-7 7-3-3z"></path>
                  <path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"></path>
                  <path d="M2 2l7.586 7.586"></path>
                  <circle cx="11" cy="11" r="2"></circle>
                </svg>

                <!-- Chevron-up icon (visible when open) -->
                <svg
                  class="custom-color-icon chevron-shape"
                  class:visible={isCustomColorDrawerOpen}
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2.5"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                >
                  <polyline points="18 15 12 9 6 15"></polyline>
                </svg>
              </div>
            </button>
          </div>
        </div>
      </div>

      <!-- In-App Animated Visual Color Picker Drawer -->
      {#if isCustomColorDrawerOpen}
        <div class="custom-color-drawer" transition:slide={{ duration: 380, easing: cubicOut }}>
          <!-- Rainbow Hue Spectrum Slider -->
          <div class="spectrum-slider-row">
            <span class="spectrum-slider-label">{i18n.t.settings.colorHue}</span>
            <input
              type="range"
              min="0"
              max="360"
              class="spectrum-range-slider hue-rainbow-slider"
              value={customH}
              oninput={(e) => handleHueChange(Number(e.currentTarget.value))}
            />
          </div>

          <!-- Tone / Brightness Slider -->
          <div class="spectrum-slider-row">
            <span class="spectrum-slider-label">{i18n.t.settings.colorTone}</span>
            <input
              type="range"
              min="20"
              max="80"
              class="spectrum-range-slider tone-slider"
              style="--tone-gradient: linear-gradient(to right, hsl({customH}, 90%, 20%), hsl({customH}, 90%, 50%), hsl({customH}, 90%, 80%));"
              value={customL}
              oninput={(e) => handleLightnessChange(Number(e.currentTarget.value))}
            />
          </div>

          <!-- Header / Info Bar with Hex input, EyeDropper, and preview swatch -->
          <div class="drawer-header-row">
            <div class="drawer-preview-swatch" style="background-color: {accentColor};"></div>
            <div class="drawer-hex-box">
              <span class="drawer-hex-prefix">#</span>
              <input
                type="text"
                class="drawer-hex-input"
                maxlength="7"
                value={customHexInput.replace('#', '')}
                oninput={handleHexInputChange}
                spellcheck="false"
              />
            </div>

            <button
              type="button"
              class="drawer-eyedropper-btn"
              class:picking={isPickingColor}
              onclick={pickColorWithEyeDropper}
              title={isPickingColor ? i18n.t.settings.eyedropperPickingTooltip : i18n.t.settings.eyedropperTooltip}
              aria-label={i18n.t.settings.eyedropperTooltip}
            >
              <svg class="eyedropper-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 19l7-7 3 3-7 7-3-3z"></path>
                <path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"></path>
                <path d="M2 2l7.586 7.586"></path>
                <circle cx="11" cy="11" r="2"></circle>
              </svg>
            </button>
            <input
              bind:this={colorInputRef}
              type="color"
              class="visually-hidden-color-input"
              value={accentColor}
              oninput={(e) => handleSelectAccent(e.currentTarget.value.toUpperCase())}
            />

            <span class="drawer-rgb-summary">{customHexInput}</span>
          </div>
        </div>
      {/if}
    </div>
  </section>

  <!-- СЕКЦИЯ 3: История -->
  <section class="settings-section">
    <h3 class="section-heading">{i18n.t.settings.sectionHistory}</h3>
    <div class="settings-card">
      <div class="card-row">
        <span class="row-label" id="save-history-label">{i18n.t.settings.saveHistory}</span>
        <div class="row-controls">
          <Toggle
            checked={saveHistory}
            ariaLabel={i18n.t.settings.saveHistory}
            onchange={(checked) => void onUpdate({ saveHistory: checked })}
          />
        </div>
      </div>
      <div class="card-row">
        <span class="row-label">{i18n.t.settings.historyCountLabel}</span>
        <div class="row-controls">
          <button
            type="button"
            class="history-clear-action-btn"
            onclick={() => { showConfirmClear = true; }}
            title={i18n.t.settings.clearHistoryBtn}
            aria-label={i18n.t.settings.clearHistoryBtn}
          >
            <span>{i18n.t.settings.clearHistoryBtn}</span>
          </button>
        </div>
      </div>
    </div>
  </section>
</div>

{#if showConfirmClear}
  <ConfirmModal
    title={i18n.t.settings.clearHistoryModalTitle}
    description={i18n.t.settings.clearHistoryModalDesc}
    confirmText={i18n.t.settings.clearHistoryBtn}
    cancelText={i18n.t.common.cancel}
    onConfirm={() => {
      showConfirmClear = false;
      onClearHistory();
    }}
    onCancel={() => { showConfirmClear = false; }}
  />
{/if}

<style>
  .settings-page-wrapper {
    max-width: 520px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .settings-error-banner {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #EF4444;
    padding: 8px 12px;
    border-radius: var(--radius-sm, 4px);
    font-size: 12px;
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

  .card-row + .card-row {
    border-top: none;
  }

  :global(.theme-dark) .card-row + .card-row {
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


  /* Desktop Hotkey Recorder Button */
  .hotkey-recorder-button {
    width: auto;
    height: 22px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: var(--radius-xs, 4px);
    display: inline-flex;
    align-items: center;
    justify-content: flex-end;
    cursor: pointer;
    outline: none;
    box-sizing: border-box;
  }

  .hotkey-recorder-button.capturing {
    height: 22px;
    padding: 0 8px;
    background: var(--accent-soft, rgba(255, 149, 0, 0.1));
    border: 1px solid var(--accent-primary);
    border-radius: var(--radius-xs, 4px);
  }

  .hotkey-capturing-wrap {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .hotkey-keycap {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
    background: var(--bg-control, rgba(0, 0, 0, 0.03));
    border: 1px solid var(--border-subtle);
    padding: 2px 7px;
    border-radius: var(--radius-xs, 4px);
    line-height: 1.2;
    transition: background 100ms ease, border-color 100ms ease;
  }

  .hotkey-recorder-button:hover:not(.capturing) .hotkey-keycap {
    background: var(--hover-bg, rgba(0, 0, 0, 0.06));
    border-color: var(--border-color);
  }

  :global(.theme-dark) .hotkey-recorder-button:hover:not(.capturing) .hotkey-keycap {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .history-clear-action-btn {
    width: 200px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-control, rgba(0, 0, 0, 0.03));
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-xs, 4px);
    color: var(--text-secondary);
    font-family: inherit;
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    transition: all 120ms ease;
    box-sizing: border-box;
  }

  .history-clear-action-btn:hover {
    background: rgba(239, 68, 68, 0.08);
    border-color: rgba(239, 68, 68, 0.25);
    color: #EF4444;
  }

  :global(.theme-dark) .history-clear-action-btn {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary);
  }

  :global(.theme-dark) .history-clear-action-btn:hover {
    background: rgba(239, 68, 68, 0.12);
    border-color: rgba(239, 68, 68, 0.35);
    color: #F87171;
  }

  :global(.theme-dark) .hotkey-keycap {
    background: var(--bg-muted, rgba(0, 0, 0, 0.06));
    border: 1px solid var(--border-subtle);
    box-shadow: none;
  }


  .hotkey-capturing-pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent-primary);
    animation: hotkey-pulse 1s infinite alternate ease-in-out;
  }

  @keyframes hotkey-pulse {
    from { transform: scale(0.8); opacity: 0.6; }
    to { transform: scale(1.2); opacity: 1; }
  }

  .hotkey-capturing-text {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--accent-primary);
  }

  /* Accent Swatches */
  .accent-swatches-strip {
    width: 200px;
    height: 26px;
    display: inline-flex;
    align-items: center;
    justify-content: space-between;
    box-sizing: border-box;
  }

  .swatch-btn {
    width: 18px;
    height: 18px;
    border-radius: var(--radius-xs, 4px);
    border: 1px solid rgba(0, 0, 0, 0.12);
    cursor: pointer;
    padding: 0;
    outline: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: transform 100ms ease;
    box-shadow: none;
    flex-shrink: 0;
  }

  .swatch-btn:hover {
    transform: scale(1.1);
  }

  .swatch-btn.selected {
    box-shadow: none;
  }

  .swatch-check-svg {
    width: 10px;
    height: 10px;
    pointer-events: none;
  }

  .custom-color-picker-btn {
    position: relative;
    border: 1px solid var(--border-color);
    background: var(--bg-control, rgba(0, 0, 0, 0.03));
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    overflow: hidden;
    transition: all 120ms ease;
  }

  .custom-color-picker-btn.selected {
    box-shadow: none;
    border-color: transparent;
  }

  .custom-color-picker-btn:hover {
    border-color: var(--text-primary);
    transform: scale(1.1);
  }

  /* Morph toggle icon slot (Pipette <-> Chevron) */
  .morph-toggle-icon-slot {
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .pipette-shape {
    position: absolute;
    width: 12px;
    height: 12px;
    transition: transform 0.45s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease;
    transform: scale(1) rotate(0deg);
    opacity: 1;
    color: var(--text-secondary);
  }

  .pipette-shape.active-custom {
    color: #FFFFFF;
    filter: drop-shadow(0 1px 1.5px rgba(0, 0, 0, 0.5));
  }

  .pipette-shape.hidden {
    transform: scale(0.2) rotate(-90deg);
    opacity: 0;
    pointer-events: none;
  }

  .chevron-shape {
    position: absolute;
    width: 12px;
    height: 12px;
    transition: transform 0.45s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease;
    transform: scale(0.2) rotate(90deg);
    opacity: 0;
    pointer-events: none;
    color: var(--text-primary);
  }

  .chevron-shape.visible {
    transform: scale(1) rotate(0deg);
    opacity: 1;
    pointer-events: auto;
  }

  /* In-App Animated Visual Color Picker Drawer */
  .custom-color-drawer {
    padding: 14px;
    background: var(--bg-muted, #18181B);
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-sizing: border-box;
  }

  :global(.theme-dark) .custom-color-drawer {
    background: #141416;
    border-top-color: rgba(255, 255, 255, 0.05);
  }

  /* Spectrum Sliders */
  .spectrum-slider-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .spectrum-slider-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    width: 32px;
    flex-shrink: 0;
  }

  .spectrum-range-slider {
    flex: 1;
    height: 8px;
    -webkit-appearance: none;
    appearance: none;
    border-radius: 4px;
    outline: none;
    cursor: pointer;
    margin: 0;
  }

  .hue-rainbow-slider {
    background: linear-gradient(
      to right,
      #ff0000 0%,
      #ff8000 12.5%,
      #ffff00 25%,
      #80ff00 37.5%,
      #00ff00 50%,
      #00ffff 62.5%,
      #0000ff 75%,
      #8000ff 87.5%,
      #ff0000 100%
    );
  }

  .tone-slider {
    background: var(--tone-gradient);
  }

  .spectrum-range-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: #FFFFFF;
    border: 1.5px solid rgba(0, 0, 0, 0.35);
    cursor: pointer;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
    transition: transform 0.15s ease;
  }

  .spectrum-range-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .spectrum-range-slider::-moz-range-thumb {
    width: 15px;
    height: 15px;
    border-radius: 50%;
    background: #FFFFFF;
    border: 1.5px solid rgba(0, 0, 0, 0.35);
    cursor: pointer;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  }

  .drawer-header-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding-top: 2px;
  }

  .drawer-preview-swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.15);
    flex-shrink: 0;
  }

  .drawer-hex-box {
    display: inline-flex;
    align-items: center;
    height: 24px;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    padding: 0 6px;
  }

  .drawer-hex-prefix {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    color: var(--text-tertiary);
  }

  .drawer-hex-input {
    background: transparent;
    border: none;
    outline: none;
    font-family: var(--font-mono, monospace);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-primary);
    width: 60px;
    padding: 0 2px;
    text-transform: uppercase;
  }

  .drawer-eyedropper-btn {
    width: 24px;
    height: 24px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm, 4px);
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0;
    transition: all 0.15s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .drawer-eyedropper-btn:hover {
    color: var(--text-primary);
    border-color: var(--text-primary);
    background: var(--hover-bg, rgba(255, 255, 255, 0.08));
    transform: scale(1.08);
  }

  .drawer-eyedropper-btn.picking {
    border-color: var(--accent-primary);
    background: var(--accent-soft, rgba(255, 149, 0, 0.15));
    color: var(--accent-primary);
    animation: pipette-pulse 1s infinite alternate ease-in-out;
  }

  @keyframes pipette-pulse {
    0% { transform: scale(1); box-shadow: 0 0 0 0 var(--accent-soft); }
    100% { transform: scale(1.08); box-shadow: 0 0 0 3px var(--accent-soft); }
  }

  .eyedropper-icon {
    width: 12px;
    height: 12px;
  }

  .visually-hidden-color-input {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
    pointer-events: none;
    border: none;
    padding: 0;
    margin: 0;
  }

  .drawer-rgb-summary {
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
    margin-left: auto;
  }
</style>
