import { emit } from '@tauri-apps/api/event';
import type { ThemeMode } from '../settings/model.js';

export interface AccentThemeChange {
  color: string;
  opacity: number;
}

type AccentThemeTokens = Record<`--${string}`, string>;

const DEFAULT_ACCENT_COLOR = '#FF9500';

function normalizeAccentColor(hex: string): string {
  return hex && /^#[0-9A-Fa-f]{6}$/.test(hex)
    ? hex.toUpperCase()
    : DEFAULT_ACCENT_COLOR;
}

function normalizeAccentOpacity(opacity: number): number {
  return Number.isFinite(opacity)
    ? Math.min(100, Math.max(0, Math.round(opacity)))
    : 100;
}

function formatAlpha(alpha: number): string {
  return String(Math.round(alpha * 1000) / 1000);
}

// Helper to convert hex to rgba
function hexToRgba(hex: string, alpha: number): string {
  const num = parseInt(hex.slice(1), 16);
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${formatAlpha(alpha)})`;
}

// Adjust hex brightness for hover/active
function adjustHex(hex: string, percent: number): string {
  const num = parseInt(hex.slice(1), 16);
  let r = (num >> 16) + Math.round(255 * (percent / 100));
  let g = ((num >> 8) & 0x00FF) + Math.round(255 * (percent / 100));
  let b = (num & 0x0000FF) + Math.round(255 * (percent / 100));

  r = Math.min(255, Math.max(0, r));
  g = Math.min(255, Math.max(0, g));
  b = Math.min(255, Math.max(0, b));

  return '#' + ((r << 16) | (g << 8) | b).toString(16).padStart(6, '0');
}

// Helper to compute WCAG relative luminance
function getLuminance(hex: string): number {
  const c = hex.slice(1);
  const r = parseInt(c.substring(0, 2), 16) / 255;
  const g = parseInt(c.substring(2, 4), 16) / 255;
  const b = parseInt(c.substring(4, 6), 16) / 255;

  const aR = r <= 0.03928 ? r / 12.92 : Math.pow((r + 0.055) / 1.055, 2.4);
  const aG = g <= 0.03928 ? g / 12.92 : Math.pow((g + 0.055) / 1.055, 2.4);
  const aB = b <= 0.03928 ? b / 12.92 : Math.pow((b + 0.055) / 1.055, 2.4);

  return 0.2126 * aR + 0.7152 * aG + 0.0722 * aB;
}

let currentAppliedTheme = '';

export function createAccentThemeTokens(
  hex: string,
  opacity: number = 100,
): AccentThemeTokens {
  const validHex = normalizeAccentColor(hex);
  const normalizedOpacity = normalizeAccentOpacity(opacity);
  const alpha = normalizedOpacity / 100;
  const hoverHex = adjustHex(validHex, -8);
  const isLightAccent = getLuminance(validHex) > 0.45;
  const foreground = isLightAccent ? '#0F172A' : '#FFFFFF';

  return {
    '--accent-color': hexToRgba(validHex, alpha),
    '--accent-primary': hexToRgba(validHex, alpha),
    '--accent-hover': hexToRgba(hoverHex, alpha),
    '--accent-hover-color': hexToRgba(hoverHex, alpha),
    '--accent-light': hexToRgba(validHex, 0.1 * alpha),
    '--accent-soft': hexToRgba(validHex, 0.1 * alpha),
    '--accent-subtle': hexToRgba(validHex, 0.14 * alpha),
    '--accent-subtle-token': hexToRgba(validHex, 0.14 * alpha),
    '--accent-glow': hexToRgba(validHex, 0.2 * alpha),
    '--accent-border': hexToRgba(validHex, 0.2 * alpha),
    '--accent-foreground': foreground,
    '--border-focus': hexToRgba(validHex, alpha),
  };
}

/**
 * Dynamically apply custom accent color CSS variables to the document root
 */
export function applyAccentColor(
  hex: string,
  opacity: number = 100,
  broadcast: boolean = true,
): void {
  if (typeof document === 'undefined') return;
  const validHex = normalizeAccentColor(hex);
  const normalizedOpacity = normalizeAccentOpacity(opacity);
  const themeKey = `${validHex}:${normalizedOpacity}`;
  const root = document.documentElement;
  const tokens = createAccentThemeTokens(validHex, normalizedOpacity);
  if (
    currentAppliedTheme === themeKey &&
    root.style.getPropertyValue('--accent-color') === tokens['--accent-color']
  ) {
    return;
  }
  currentAppliedTheme = themeKey;

  for (const [name, value] of Object.entries(tokens)) {
    root.style.setProperty(name, value);
  }

  if (broadcast && typeof window !== 'undefined') {
    const tauriInternals = window.__TAURI_INTERNALS__;
    if (tauriInternals?.invoke) {
      const change: AccentThemeChange = {
        color: validHex,
        opacity: normalizedOpacity,
      };
      emit('accent-color-changed', change).catch(() => {});
    }
  }
}

/**
 * Compute actual dark/light state from ThemeMode
 */
export function resolveActualTheme(mode: ThemeMode): 'dark' | 'light' {
  if (mode === 'system') {
    if (typeof window !== 'undefined' && window.matchMedia) {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return 'dark';
  }
  return mode;
}
