import { call } from '@/shared/backend';

/** One edit in the history. Mirrors `ChangeSummary` in `ontara-storage`. */
export interface Change {
  /** Stable machine name of the operation, such as `entity.update`. */
  label: string;
  /** Milliseconds since the Unix epoch. */
  at: number;
}

/** What can be undone and redone. Mirrors `HistoryState` in `src-tauri/src/history.rs`. */
export interface HistoryState {
  /** Newest first. */
  undo: Change[];
  /** The next redo first. */
  redo: Change[];
  /** What an undo or redo just reversed or reapplied. */
  applied: Change | null;
}

export function getHistory(): Promise<HistoryState> {
  return call('get_history');
}

export function undoChange(): Promise<HistoryState> {
  return call('undo');
}

export function redoChange(): Promise<HistoryState> {
  return call('redo');
}
