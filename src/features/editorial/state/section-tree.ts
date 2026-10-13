import type { Section } from '../api/editorial-api';

/** A section with the sections placed directly inside it, in order. */
export interface SectionNode {
  section: Section;
  children: SectionNode[];
}

/** Builds the tree from the flat list the backend sends. Orphans become roots. */
export function buildTree(sections: readonly Section[]): SectionNode[] {
  const nodes = new Map<string, SectionNode>(
    sections.map((section) => [section.entity.id, { section, children: [] }]),
  );
  const roots: SectionNode[] = [];
  for (const node of nodes.values()) {
    const parent = node.section.parent ? nodes.get(node.section.parent) : undefined;
    (parent ? parent.children : roots).push(node);
  }
  const byPosition = (a: SectionNode, b: SectionNode) =>
    a.section.position - b.section.position ||
    a.section.entity.name.localeCompare(b.section.entity.name);
  roots.sort(byPosition);
  for (const node of nodes.values()) node.children.sort(byPosition);
  return roots;
}

/** The sections in reading order, each with how deep it sits. */
export function flatten(
  roots: readonly SectionNode[],
  collapsed: ReadonlySet<string> = new Set(),
): { node: SectionNode; depth: number }[] {
  const rows: { node: SectionNode; depth: number }[] = [];
  const visit = (nodes: readonly SectionNode[], depth: number) => {
    for (const node of nodes) {
      rows.push({ node, depth });
      if (!collapsed.has(node.section.entity.id)) visit(node.children, depth + 1);
    }
  };
  visit(roots, 0);
  return rows;
}
