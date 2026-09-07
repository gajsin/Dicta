import type { ResolvedLanguage, TranslationDictionary } from './types.js';
import { en } from './locales/en.js';
import { ru } from './locales/ru.js';

export * from './types.js';

export const translations: Record<ResolvedLanguage, TranslationDictionary> = {
  en,
  ru,
};
