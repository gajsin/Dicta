import { parseHistoryJson, serializeHistory, type HistoryItem } from './storage.js';
import { i18n } from '../i18n/index.js';

const HISTORY_STORAGE_KEY = 'dicta_history';

interface HistoryControllerOptions {
  onNotice: (message: string) => void;
  getSaveHistory: () => boolean;
}

function createHistoryId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export class HistoryController {
  items = $state<HistoryItem[]>([]);
  selectedId = $state<string | null>(null);
  lastDeletedItems = $state<HistoryItem[] | null>(null);

  readonly #onNotice: (message: string) => void;
  readonly #getSaveHistory: () => boolean;

  constructor({ onNotice, getSaveHistory }: HistoryControllerOptions) {
    this.#onNotice = onNotice;
    this.#getSaveHistory = getSaveHistory;
  }

  get canUndo(): boolean {
    return Boolean(this.lastDeletedItems?.length);
  }

  load(): void {
    try {
      const stored = localStorage.getItem(HISTORY_STORAGE_KEY);
      if (!stored) return;
      this.items = parseHistoryJson(stored);
    } catch (error) {
      console.error('[Dicta] Failed to load history:', error);
      this.items = [];
    }
  }

  add(text: string, durationSec: number, rawText = text): void {
    // Disabling history pauses collection. Existing entries and explicit undo
    // remain available; no unsaved recordings can be persisted by a later undo.
    if (!this.#getSaveHistory()) return;
    const now = new Date();
    const item: HistoryItem = {
      id: createHistoryId(),
      timestamp: now.toISOString(),
      duration: Math.max(0, durationSec),
      processedText: text,
      rawText,
    };

    this.items = [item, ...this.items];
    this.selectedId = null;
    this.#persist();
  }

  delete(id: string): void {
    const item = this.items.find((candidate) => candidate.id === id);
    if (!item) return;

    this.lastDeletedItems = [item];
    this.items = this.items.filter((candidate) => candidate.id !== id);
    if (this.selectedId === id) {
      this.selectedId = this.items[0]?.id ?? null;
    }
    this.#persist();
    this.#onNotice(i18n.t.history.itemDeleted);
  }

  clear(): void {
    if (this.items.length === 0) return;
    this.lastDeletedItems = [...this.items];
    this.items = [];
    this.selectedId = null;
    this.#persist();
    this.#onNotice(i18n.t.history.historyCleared);
  }

  undo(): void {
    if (!this.lastDeletedItems?.length) return;

    const restored = this.lastDeletedItems;
    this.items = [...restored, ...this.items];
    this.selectedId = null;
    this.lastDeletedItems = null;
    this.#persist();
    this.#onNotice(
      restored.length === 1 ? i18n.t.history.itemRestored : i18n.t.history.historyRestored,
    );
  }

  #persist(): void {
    try {
      localStorage.setItem(HISTORY_STORAGE_KEY, serializeHistory(this.items));
    } catch (error) {
      console.error('[Dicta] Failed to persist history:', error);
      this.#onNotice(i18n.t.history.failedToPersist);
    }
  }
}
