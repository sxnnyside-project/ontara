import { expect, test } from 'bun:test';
import {
  PALETTE,
  assignStyles,
  edgeKey,
  legendOf,
  loadOverrides,
  saveOverrides,
} from '@/features/graph/state/edge-style';

const relation = (type: string) =>
  ({
    source: 'a',
    target: 'b',
    via: 'relationship',
    relationship: 'r',
    relationship_type: type,
  }) as const;

test('each relationship type keeps one style, and styles stay apart past the palette', () => {
  const order = Array.from({ length: PALETTE.length + 2 }, (_, index) => `t${index}`);

  const styles = assignStyles(order, {});

  const first = styles.get('t0');
  const wrapped = styles.get(`t${PALETTE.length}`);
  expect(first?.color).toBe(wrapped?.color);
  expect(first?.mark).not.toBe(wrapped?.mark);
  const looks = new Set([...styles.values()].map((style) => JSON.stringify(style)));
  expect(looks.size).toBe(styles.size);
});

test("the author's choice wins over the default, and events have their own styles", () => {
  const styles = assignStyles(['mentor_of'], { mentor_of: { color: '#112233' } });

  expect(styles.get('mentor_of')?.color).toBe('#112233');
  expect(styles.get('participation')?.pattern).toBe('dotted');
  expect(styles.get('location')?.pattern).toBe('dashed');
});

test('the legend counts the kinds of link in a graph, most numerous first', () => {
  const edges = [
    relation('mentor_of'),
    relation('parent_of'),
    relation('parent_of'),
    { source: 'a', target: 'b', via: 'location' } as const,
  ];

  expect(edgeKey(edges[3] as never)).toBe('location');
  expect(legendOf(edges)).toEqual([
    { key: 'parent_of', count: 2 },
    { key: 'location', count: 1 },
    { key: 'mentor_of', count: 1 },
  ]);
});

test('styles are remembered per project and a broken store is harmless', () => {
  const data = new Map<string, string>();
  const storage = {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => void data.set(key, value),
  };

  saveOverrides(storage, 'one', { owns: { color: '#fff000' } });
  saveOverrides(storage, 'two', {});

  expect(loadOverrides(storage, 'one')).toEqual({ owns: { color: '#fff000' } });
  expect(loadOverrides(storage, 'three')).toEqual({});
  expect(loadOverrides({ getItem: () => '{bad' }, 'one')).toEqual({});
  expect(loadOverrides(undefined, 'one')).toEqual({});
});
