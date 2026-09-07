import { i18n } from '../i18n/index.js';

interface ErrorInfo {
  message: string;
  shortLabel: string;
  actionType: 'retry' | 'settings' | 'close';
  settingsSection?: 'dictation' | 'providers';
  kind?: 'already_completed' | 'other';
}

export function getNormalizedErrorInfo(error: unknown): ErrorInfo {
  if (error && typeof error === 'object' && !(error instanceof Error) &&
    'message' in error && typeof error.message === 'string') {
    error = error.message;
  }
  let raw = '';

  if (typeof error === 'string') {
    raw = error.toLowerCase();
  } else if (error instanceof Error) {
    raw = error.message?.toLowerCase() ?? '';
  } else if (error && typeof error === 'object') {
    const errorRecord = error as Record<string, unknown>;
    raw = String(errorRecord.message || errorRecord.error || JSON.stringify(error)).toLowerCase();
  }

  if (!raw) {
    return {
      message: i18n.t.errors.defaultError,
      shortLabel: i18n.t.hud.error,
      actionType: 'retry',
    };
  }

  if (raw.includes('notallowed') || raw.includes('permission') || raw.includes('доступ к микрофону запрещён')) {
    return {
      message: i18n.t.errors.permissionDenied,
      shortLabel: i18n.t.hud.micErrorShort,
      actionType: 'settings',
    };
  }

  if (raw.includes('notfound') || raw.includes('device') || raw.includes('микрофон не найден')) {
    return {
      message: i18n.t.errors.micNotFound,
      shortLabel: i18n.t.hud.micErrorShort,
      actionType: 'settings',
    };
  }

  if (raw.includes('notreadable') || raw.includes('busy') || raw.includes('микрофон занят')) {
    return {
      message: i18n.t.errors.micBusy,
      shortLabel: i18n.t.hud.micErrorShort,
      actionType: 'close',
    };
  }

  if (
    raw.includes('401') ||
    raw.includes('403') ||
    raw.includes('unauthorized') ||
    raw.includes('добавьте api-ключ') ||
    raw.includes('invalid api key') ||
    raw.includes('неавториз')
  ) {
    return {
      message: i18n.t.errors.invalidApiKey,
      shortLabel: i18n.t.hud.invalidKeyShort,
      actionType: 'settings',
      settingsSection: 'providers',
    };
  }

  if (
    raw.includes('сеть') ||
    raw.includes('тайм-аут') ||
    raw.includes('таймаут') ||
    raw.includes('timeout') ||
    raw.includes('connection') ||
    raw.includes('fetch') ||
    raw.includes('network') ||
    raw.includes('failed to fetch')
  ) {
    return {
      message: i18n.t.errors.networkError,
      shortLabel: i18n.t.hud.networkErrorShort,
      actionType: 'retry',
    };
  }

  if (
    raw.includes('500') ||
    raw.includes('502') ||
    raw.includes('503') ||
    raw.includes('504') ||
    raw.includes('unavailable') ||
    raw.includes('провайдер недоступен')
  ) {
    return {
      message: i18n.t.errors.providerUnavailable,
      shortLabel: i18n.t.hud.networkErrorShort,
      actionType: 'retry',
    };
  }

  if (
    raw.includes('максимальная длительность') ||
    raw.includes('слишком длинная') ||
    raw.includes('10 мин')
  ) {
    return {
      message: i18n.t.errors.recordingTooLong,
      shortLabel: i18n.t.hud.error,
      actionType: 'retry',
    };
  }

  if (
    raw.includes('пуст') ||
    raw.includes('empty') ||
    raw.includes('слишком короткая') ||
    raw.includes('0 байт') ||
    raw.includes('не содержит звука')
  ) {
    return {
      message: i18n.t.errors.noSpeech,
      shortLabel: i18n.t.hud.noSpeechShort,
      actionType: 'retry',
    };
  }

  if (raw.includes('clipboard') || raw.includes('буфер обмена') || raw.includes('вставить')) {
    return {
      message: i18n.t.errors.pasteError,
      shortLabel: i18n.t.hud.pasteErrorShort,
      actionType: 'retry',
    };
  }

  if (raw.includes('запись не была начата') || raw.includes('recording not started')) {
    return {
      message: i18n.t.errors.recordingNotStarted,
      shortLabel: i18n.t.hud.error,
      actionType: 'close',
      kind: 'already_completed',
    };
  }

  // Fallback nicely formatted message
  const userMessage =
    typeof error === 'string' && error.trim().length > 0
      ? error.trim()
      : error instanceof Error && error.message
      ? error.message
      : i18n.t.errors.defaultError;

  return {
    message: userMessage,
    shortLabel: i18n.t.hud.error,
    actionType: 'retry',
    kind: 'other',
  };
}
