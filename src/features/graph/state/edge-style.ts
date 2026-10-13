import type { GraphEdge } from '../api/graph-api';

/** The shape drawn at the middle of a link, so a link is told apart without its color. */
export type Mark = 'circle' | 'diamond' | 'square' | 'bar';

export type Pattern = 'solid' | 'dashed' | 'dotted';

/** How one kind of link is drawn. */
export interface EdgeStyle {
  color: string;
  pattern: Pattern;
  mark: Mark;
}

export const MARKS: readonly Mark[] = ['circle', 'diamond', 'square', 'bar'];
export const PATTERNS: readonly Pattern[] = ['solid', 'dashed', 'dotted'];

/** Colors that stay apart from one another and readable on both themes. */
export const PALETTE: readonly string[] = [
  '#4f9cf0',
  '#e8a33d',
  '#3fb68b',
  '#c678dd',
  '#e06c75',
  '#2fb5c4',
  '#d1813f',
  '#8bb85a',
];

/** Links that come from events rather than from relationships. */
export const PARTICIPATION = 'participation';
export const LOCATION = 'location';

const EVENT_STYLES: Record<string, EdgeStyle> = {
  [PARTICIPATION]: { color: '#8c94a3', pattern: 'dotted', mark: 'circle' },
  [LOCATION]: { color: '#8c94a3', pattern: 'dashed', mark: 'square' },
};

/** What kind of link an edge is: its relationship type, or the way an event touches an entity. */
export function edgeKey(edge: GraphEdge): string {
  return edge.via === 'relationship' ? edge.relationship_type : edge.via;
}

/**
 * The style of every kind of link. Each relationship type gets one by its place in the catalog,
 * so it looks the same in every graph: colors first, then a different mark once the palette is
 * used, then a different pattern. An author's own choices win.
 */
export function assignStyles(
  catalogOrder: readonly string[],
  overrides: Readonly<Record<string, Partial<EdgeStyle>>>,
): Map<string, EdgeStyle> {
  const styles = new Map<string, EdgeStyle>();
  catalogOrder.forEach((key, index) => {
    const cycle = Math.floor(index / PALETTE.length);
    styles.set(key, {
      color: PALETTE[index % PALETTE.length] ?? '#8c94a3',
      mark: MARKS[cycle % MARKS.length] ?? 'circle',
      pattern: PATTERNS[Math.floor(cycle / MARKS.length) % PATTERNS.length] ?? 'solid',
      ...overrides[key],
    });
  });
  for (const [key, style] of Object.entries(EVENT_STYLES)) {
    styles.set(key, { ...style, ...overrides[key] });
  }
  return styles;
}

/** One kind of link present in a graph, and how many links of it there are. */
export interface LegendEntry {
  key: string;
  count: number;
}

/** The kinds of link a graph uses, most numerous first. */
export function legendOf(edges: readonly GraphEdge[]): LegendEntry[] {
  const counts = new Map<string, number>();
  for (const edge of edges) {
    const key = edgeKey(edge);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return [...counts.entries()]
    .map(([key, count]) => ({ key, count }))
    .sort((a, b) => b.count - a.count || a.key.localeCompare(b.key));
}

export function dashArray(pattern: Pattern): string | undefined {
  if (pattern === 'dashed') return '7 4';
  if (pattern === 'dotted') return '1.5 3.5';
  return undefined;
}

/** A graph this small can show the words on its links without becoming unreadable. */
export const LABELS_UP_TO = 6;

const STORAGE_KEY = 'ontara.graph-link-styles';

type Overrides = Record<string, Partial<EdgeStyle>>;

function isOverrides(value: unknown): value is Record<string, Overrides> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/** What the author changed for a project; nothing when it cannot be read. */
export function loadOverrides(
  storage: Pick<Storage, 'getItem'> | undefined,
  project: string,
): Overrides {
  try {
    const raw = storage?.getItem(STORAGE_KEY);
    if (!raw) return {};
    const all: unknown = JSON.parse(raw);
    return isOverrides(all) ? (all[project] ?? {}) : {};
  } catch {
    return {};
  }
}

export function saveOverrides(
  storage: Pick<Storage, 'getItem' | 'setItem'> | undefined,
  project: string,
  overrides: Overrides,
): void {
  try {
    const raw = storage?.getItem(STORAGE_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : {};
    const all = isOverrides(parsed) ? parsed : {};
    storage?.setItem(STORAGE_KEY, JSON.stringify({ ...all, [project]: overrides }));
  } catch {
    // A style that cannot be remembered is only forgotten.
  }
}
