import { expect, test } from 'bun:test';
import type { Section } from '@/features/editorial/api/editorial-api';
import { buildTree, flatten } from '@/features/editorial/state/section-tree';

function section(id: string, parent: string | null, position: number): Section {
  return {
    entity: {
      id,
      name: id,
      entity_type: 'chapter',
      type_label: 'Chapter',
      canon_state: 'established',
      archived: false,
    },
    level: 'chapter',
    parent,
    position,
  };
}

const sections = [
  section('c2', 'v1', 1),
  section('v1', null, 0),
  section('c1', 'v1', 0),
  section('s1', 'c1', 0),
  section('v2', null, 1),
];

test('the tree nests sections under their parent and orders siblings by position', () => {
  const roots = buildTree(sections);

  expect(roots.map((node) => node.section.entity.id)).toEqual(['v1', 'v2']);
  expect(roots[0]?.children.map((node) => node.section.entity.id)).toEqual(['c1', 'c2']);
});

test('flattening gives reading order and depth', () => {
  const rows = flatten(buildTree(sections));

  expect(rows.map((row) => `${row.depth}:${row.node.section.entity.id}`)).toEqual([
    '0:v1',
    '1:c1',
    '2:s1',
    '1:c2',
    '0:v2',
  ]);
});

test('a collapsed section hides what is inside it', () => {
  const rows = flatten(buildTree(sections), new Set(['v1']));

  expect(rows.map((row) => row.node.section.entity.id)).toEqual(['v1', 'v2']);
});

test('a section whose parent is missing is shown at the top', () => {
  const roots = buildTree([section('lost', 'gone', 0)]);

  expect(roots).toHaveLength(1);
});
