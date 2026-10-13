import { expect, test } from 'bun:test';
import { radialLayout } from '@/features/graph/state/graph-layout';

test('the center sits at the origin and each ring is farther out than the last', () => {
  const positions = radialLayout(
    [
      { id: 'c', distance: 0 },
      { id: 'a', distance: 1 },
      { id: 'b', distance: 1 },
      { id: 'x', distance: 2 },
    ],
    [
      { source: 'c', target: 'a' },
      { source: 'c', target: 'b' },
      { source: 'a', target: 'x' },
    ],
  );

  const radius = (id: string) => {
    const point = positions.get(id);
    return Math.hypot(point?.x ?? 0, point?.y ?? 0);
  };
  expect(positions.get('c')).toEqual({ x: 0, y: 0 });
  expect(radius('a')).toBeGreaterThan(0);
  expect(radius('x')).toBeGreaterThan(radius('a'));
});

test('no two nodes share a position', () => {
  const nodes = [{ id: 'c', distance: 0 }];
  const edges = [];
  for (let index = 0; index < 12; index += 1) {
    nodes.push({ id: `n${index}`, distance: 1 });
    edges.push({ source: 'c', target: `n${index}` });
  }

  const positions = radialLayout(nodes, edges);

  const seen = new Set([...positions.values()].map((p) => `${p.x.toFixed(1)}:${p.y.toFixed(1)}`));
  expect(seen.size).toBe(nodes.length);
});

test('a node stays near the branch it hangs from', () => {
  const positions = radialLayout(
    [
      { id: 'c', distance: 0 },
      { id: 'left', distance: 1 },
      { id: 'right', distance: 1 },
      { id: 'left-child', distance: 2 },
    ],
    [
      { source: 'c', target: 'left' },
      { source: 'c', target: 'right' },
      { source: 'left', target: 'left-child' },
    ],
  );

  const near = (a: string, b: string) => {
    const [p, q] = [positions.get(a), positions.get(b)];
    return Math.hypot((p?.x ?? 0) - (q?.x ?? 0), (p?.y ?? 0) - (q?.y ?? 0));
  };
  expect(near('left-child', 'left')).toBeLessThan(near('left-child', 'right'));
});

test('a graph without a center has no layout', () => {
  expect(radialLayout([{ id: 'a', distance: 1 }], []).size).toBe(0);
});
