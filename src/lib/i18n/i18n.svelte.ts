import {
  type UiLanguage,
  type ResolvedLanguage,
  translations,
} from './translations.js';

function resolveLanguage(pref: UiLanguage): ResolvedLanguage {
  if (pref === 'ru') return 'ru';
  if (pref === 'en') return 'en';

  if (typeof navigator !== 'undefined') {
    const sys = (navigator.language || '').toLowerCase();
    if (sys.startsWith('ru')) return 'ru';
  }
  return 'en';
}

class I18nStore {
  pref = $state<UiLanguage>('system');
  resolved = $derived(resolveLanguage(this.pref));
  t = $derived(translations[this.resolved]);

  setLanguage(pref: UiLanguage) {
    this.pref = pref;
  }
}

export const i18n = new I18nStore();
