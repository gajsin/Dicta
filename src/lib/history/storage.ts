export interface HistoryItem {
  id: string;
  timestamp: string; // ISO date string
  duration: number; // in seconds
  processedText: string;
  rawText?: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function parseItem(value: unknown): HistoryItem | null {
  if (!isRecord(value)) return null;
  if (
    typeof value.id !== 'string' ||
    !value.id ||
    typeof value.timestamp !== 'string' ||
    Number.isNaN(Date.parse(value.timestamp)) ||
    typeof value.processedText !== 'string' ||
    typeof value.duration !== 'number' ||
    !Number.isFinite(value.duration)
  ) {
    return null;
  }

  const item: HistoryItem = {
    id: value.id,
    timestamp: value.timestamp,
    duration: Math.max(0, value.duration),
    processedText: value.processedText,
    rawText:
      typeof value.rawText === 'string' ? value.rawText : value.processedText,
  };
  return item;
}

export function parseHistoryJson(raw: string): HistoryItem[] {
  const parsed: unknown = JSON.parse(raw);
  if (!Array.isArray(parsed)) return [];
  return parsed
    .map(parseItem)
    .filter((item): item is HistoryItem => item !== null);
}

export function serializeHistory(items: HistoryItem[]): string {
  return JSON.stringify(items);
}

export function getItemTitle(item: HistoryItem, fallbackTitle = 'Пустая запись'): string {
  const t = (item.processedText || item.rawText || '').trim();
  if (!t) return fallbackTitle;
  const singleLine = t.replace(/\r?\n+/g, ' ').trim();
  return singleLine || fallbackTitle;
}
