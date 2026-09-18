import { parseHistoryJson, serializeHistory, type HistoryItem } from './storage.js';
import { i18n } from '../i18n/index.js';
import { invoke } from '@tauri-apps/api/core';

const HISTORY_STORAGE_KEY = 'dicta_history';

interface HistoryControllerOptions {
  onNotice: (message: string) => void;
  getSaveHistory: () => boolean;
  isTauriRuntime: boolean;
}

type HistoryChange =
  | { type: 'add'; item: HistoryItem }
  | { type: 'delete'; id: string }
  | { type: 'clear' }
  | { type: 'restore'; items: HistoryItem[] };

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
  readonly #isTauriRuntime: boolean;
  #ready: Promise<void> = Promise.resolve();
  #pending: Promise<void> = Promise.resolve();
  #nativeAvailable = true;

  constructor({ onNotice, getSaveHistory, isTauriRuntime }: HistoryControllerOptions) {
    this.#onNotice = onNotice;
    this.#getSaveHistory = getSaveHistory;
    this.#isTauriRuntime = isTauriRuntime;
  }

  get canUndo(): boolean {
    return Boolean(this.lastDeletedItems?.length);
  }

  load(): void {
    let legacyItems: HistoryItem[] = [];
    try {
      const stored = localStorage.getItem(HISTORY_STORAGE_KEY);
      if (stored) legacyItems = parseHistoryJson(stored);
    } catch (error) {
      console.error('[Dicta] Failed to load history:', error);
    }
    this.items = legacyItems;
    if (!this.#isTauriRuntime) return;
    const legacyIds = new Set(legacyItems.map((item) => item.id));
    this.#ready = invoke<HistoryItem[]>('load_history', { legacyItems })
      .then((items) => {
        // A recording can finish while the initial read is in flight.
        const pendingAdds = this.items.filter((item) => !legacyIds.has(item.id));
        this.items = [...pendingAdds, ...items];
        // Remove this origin's copy only after native storage has accepted it.
        try { localStorage.removeItem(HISTORY_STORAGE_KEY); } catch { /* native copy is safe */ }
      })
      .catch((error) => {
        this.#nativeAvailable = false;
        console.error('[Dicta] Failed to load shared history:', error);
        this.#onNotice(i18n.t.history.failedToPersist);
      });
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
    this.#persist({ type: 'add', item });
  }

  delete(id: string): void {
    const item = this.items.find((candidate) => candidate.id === id);
    if (!item) return;

    this.lastDeletedItems = [item];
    this.items = this.items.filter((candidate) => candidate.id !== id);
    if (this.selectedId === id) {
      this.selectedId = this.items[0]?.id ?? null;
    }
    this.#persist({ type: 'delete', id });
    this.#onNotice(i18n.t.history.itemDeleted);
  }

  clear(): void {
    if (this.items.length === 0) return;
    this.lastDeletedItems = [...this.items];
    this.items = [];
    this.selectedId = null;
    this.#persist({ type: 'clear' });
    this.#onNotice(i18n.t.history.historyCleared);
  }

  undo(): void {
    if (!this.lastDeletedItems?.length) return;

    const restored = this.lastDeletedItems;
    this.items = [...restored, ...this.items];
    this.selectedId = null;
    this.lastDeletedItems = null;
    this.#persist({ type: 'restore', items: restored });
    this.#onNotice(
      restored.length === 1 ? i18n.t.history.itemRestored : i18n.t.history.historyRestored,
    );
  }

  #persist(change: HistoryChange): void {
    if (this.#isTauriRuntime) {
      this.#pending = this.#pending.then(async () => {
        await this.#ready;
        if (this.#nativeAvailable) {
          await invoke('change_history', { change });
          return;
        }
        localStorage.setItem(HISTORY_STORAGE_KEY, serializeHistory(this.items));
      }).catch((error) => {
        console.error('[Dicta] Failed to persist history:', error);
        try { localStorage.setItem(HISTORY_STORAGE_KEY, serializeHistory(this.items)); } catch { /* notice below */ }
        this.#onNotice(i18n.t.history.failedToPersist);
      });
      return;
    }
    try {
      localStorage.setItem(HISTORY_STORAGE_KEY, serializeHistory(this.items));
    } catch (error) {
      console.error('[Dicta] Failed to persist history:', error);
      this.#onNotice(i18n.t.history.failedToPersist);
    }
  }
}
