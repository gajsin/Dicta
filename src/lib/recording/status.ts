export type WidgetState =
  | 'idle'
  | 'listening'
  | 'finishing'
  | 'transcribing'
  | 'refining'
  | 'inserting'
  | 'copied'
  | 'error'
  | 'cancelled';

export interface WidgetPayload {
  state: WidgetState;
  message: string;
  shortLabel?: string;
  timerSeconds: number;
  audioLevel: number;
  theme: string;
  resolvedTheme?: 'light' | 'dark';
  accentColor?: string;
  accentOpacity?: number;
  uiLanguage?: 'system' | 'en' | 'ru';
  hotkey: string;
  errorActionType?: 'retry' | 'settings' | 'close';
}

type RecordingStatusKind =
  | 'idle'
  | 'listening'
  | 'processing'
  | 'success'
  | 'error'
  | 'cancelled';

interface RecordingStatus {
  kind: RecordingStatusKind;
  label: string;
}

export function getRecordingStatus(
  state: WidgetState,
  message: string,
): RecordingStatus {
  switch (state) {
    case 'idle':
      return { kind: 'idle', label: 'Начать запись' };
    case 'listening':
      return { kind: 'listening', label: message || 'Слушаю' };
    case 'finishing':
      return { kind: 'processing', label: message || 'Завершаю…' };
    case 'transcribing':
      return { kind: 'processing', label: message || 'Распознаю…' };
    case 'refining':
      return { kind: 'processing', label: message || 'Улучшаю текст…' };
    case 'inserting':
    case 'copied':
      return { kind: 'success', label: message || (state === 'inserting' ? 'Вставлено' : 'Скопировано') };
    case 'error':
      return { kind: 'error', label: message || 'Произошла ошибка' };
    case 'cancelled':
      return { kind: 'cancelled', label: message || 'Запись отменена' };
    default:
      return { kind: 'idle', label: 'Начать запись' };
  }
}
