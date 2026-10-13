export interface Point {
  x: number;
  y: number;
}

export interface LayoutNode {
  id: string;
  distance: number;
}

export interface LayoutEdge {
  source: string;
  target: string;
}

/** Room each node needs along its ring, in pixels. */
const NODE_ARC = 210;
/** Smallest gap between two rings, in pixels. */
const RING_GAP = 260;

/**
 * Places the nodes on concentric rings around the center. Each node sits at the angle of the
 * branch it hangs from, and a branch gets room in proportion to how many nodes it holds, so
 * neighbors stay together and edges rarely cross. The same input always gives the same layout.
 */
export function radialLayout(
  nodes: readonly LayoutNode[],
  edges: readonly LayoutEdge[],
): Map<string, Point> {
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const center = nodes.find((node) => node.distance === 0);
  const positions = new Map<string, Point>();
  if (!center) return positions;

  // Each node hangs from its nearest-to-center neighbor, ties broken by the order given.
  const order = new Map(nodes.map((node, index) => [node.id, index]));
  const neighbors = new Map<string, string[]>();
  for (const { source, target } of edges) {
    if (!byId.has(source) || !byId.has(target) || source === target) continue;
    neighbors.set(source, [...(neighbors.get(source) ?? []), target]);
    neighbors.set(target, [...(neighbors.get(target) ?? []), source]);
  }
  const children = new Map<string, string[]>();
  for (const node of nodes) {
    if (node.distance === 0) continue;
    const parent = (neighbors.get(node.id) ?? [])
      .filter((other) => (byId.get(other)?.distance ?? Number.POSITIVE_INFINITY) < node.distance)
      .sort((a, b) => (order.get(a) ?? 0) - (order.get(b) ?? 0))[0];
    const hangsFrom = parent ?? center.id;
    children.set(hangsFrom, [...(children.get(hangsFrom) ?? []), node.id]);
  }

  const weight = new Map<string, number>();
  const measure = (id: string): number => {
    const below = children.get(id) ?? [];
    const total = below.length === 0 ? 1 : below.reduce((sum, child) => sum + measure(child), 0);
    weight.set(id, total);
    return total;
  };
  measure(center.id);

  const perRing = new Map<number, number>();
  for (const node of nodes) perRing.set(node.distance, (perRing.get(node.distance) ?? 0) + 1);
  const radii: number[] = [0];
  const deepest = Math.max(...nodes.map((node) => node.distance));
  for (let ring = 1; ring <= deepest; ring += 1) {
    const needed = ((perRing.get(ring) ?? 0) * NODE_ARC) / (2 * Math.PI);
    radii.push(Math.max((radii[ring - 1] ?? 0) + RING_GAP, needed));
  }

  const place = (id: string, from: number, to: number, distance: number) => {
    const angle = (from + to) / 2;
    const radius = radii[distance] ?? 0;
    positions.set(id, { x: Math.cos(angle) * radius, y: Math.sin(angle) * radius });
    const below = children.get(id) ?? [];
    const total = below.reduce((sum, child) => sum + (weight.get(child) ?? 1), 0);
    let cursor = from;
    for (const child of below) {
      const share = ((to - from) * (weight.get(child) ?? 1)) / total;
      place(child, cursor, cursor + share, byId.get(child)?.distance ?? distance + 1);
      cursor += share;
    }
  };
  place(center.id, -Math.PI / 2, (3 * Math.PI) / 2, 0);
  positions.set(center.id, { x: 0, y: 0 });
  return positions;
}
