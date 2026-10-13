// @unocss-include
// The icon classes below are written out in full so UnoCSS can find and generate them.
import type { MatrixCell } from '../api/matrix-api';

/** Key of the cell where a row meets a column. */
export function cellKey(entity: string, context: string): string {
  return `${entity}:${context}`;
}

/** Groups the cells by where their row meets their column. */
export function indexCells(cells: readonly MatrixCell[]): Map<string, MatrixCell[]> {
  const index = new Map<string, MatrixCell[]>();
  for (const cell of cells) {
    const key = cellKey(cell.entity, cell.context);
    index.set(key, [...(index.get(key) ?? []), cell]);
  }
  return index;
}

/** In how many columns each row is present, counting presence only. */
export function presenceByRow(cells: readonly MatrixCell[]): Map<string, number> {
  const seen = new Map<string, Set<string>>();
  for (const cell of cells) {
    if (!cell.is_presence) continue;
    const contexts = seen.get(cell.entity) ?? new Set<string>();
    contexts.add(cell.context);
    seen.set(cell.entity, contexts);
  }
  return new Map([...seen].map(([entity, contexts]) => [entity, contexts.size]));
}

/** How many rows are present in each column, counting presence only. */
export function presenceByColumn(cells: readonly MatrixCell[]): Map<string, number> {
  const seen = new Map<string, Set<string>>();
  for (const cell of cells) {
    if (!cell.is_presence) continue;
    const entities = seen.get(cell.context) ?? new Set<string>();
    entities.add(cell.entity);
    seen.set(cell.context, entities);
  }
  return new Map([...seen].map(([context, entities]) => [context, entities.size]));
}

export interface KindLook {
  icon: string;
  text: string;
  background: string;
  border: string;
}

const LOOKS: Record<string, KindLook> = {
  physical_presence: {
    icon: 'i-mingcute:walk-line',
    text: 'text-[var(--color-meridian-blue)]',
    background: 'bg-[var(--color-meridian-blue-bg)]',
    border: 'border-[var(--color-meridian-blue)]/20',
  },
  dialogue: {
    icon: 'i-mingcute:chat-3-line',
    text: 'text-[var(--color-canon-established)]',
    background: 'bg-[var(--color-canon-established-bg)]',
    border: 'border-[var(--color-canon-established)]/20',
  },
  indirect_mention: {
    icon: 'i-mingcute:quill-pen-line',
    text: 'text-[var(--color-canon-provisional)]',
    background: 'bg-[var(--color-canon-provisional-bg)]',
    border: 'border-[var(--color-canon-provisional)]/20',
  },
  flashback: {
    icon: 'i-mingcute:history-line',
    text: 'text-[var(--color-canon-speculative)]',
    background: 'bg-[var(--color-canon-speculative-bg)]',
    border: 'border-[var(--color-canon-speculative)]/20',
  },
  remote_presence: {
    icon: 'i-mingcute:eye-line',
    text: 'text-[var(--color-crown-red-text)]',
    background: 'bg-[var(--color-crown-red-subtle)]',
    border: 'border-[var(--color-crown-red-border)]',
  },
  confirmed_absence: {
    icon: 'i-mingcute:close-line',
    text: 'text-[var(--color-text-muted)]',
    background: 'bg-transparent',
    border: 'border-[var(--color-border-subtle)]',
  },
};

const FALLBACK_PRESENCE: KindLook = {
  icon: 'i-mingcute:user-3-line',
  text: 'text-[var(--color-text-secondary)]',
  background: 'bg-[var(--color-surface-elevated)]',
  border: 'border-[var(--color-border-strong)]',
};

/** How a kind of appearance looks; a project's own kinds fall back to a neutral look. */
export function kindLook(kind: string, isPresence: boolean): KindLook {
  return (
    LOOKS[kind] ?? (isPresence ? FALLBACK_PRESENCE : (LOOKS.confirmed_absence ?? FALLBACK_PRESENCE))
  );
}
