import { expect, test } from 'bun:test';
import type { Connection } from '@/features/navigation/api/navigation-api';
import { groupConnections, groupOf } from '@/features/navigation/state/connection-groups';

const other = {
  id: 'x',
  name: 'X',
  entity_type: 'character',
  type_label: 'Character',
  canon_state: 'established',
  archived: false,
};

const relationship: Connection = {
  other,
  via: 'relationship',
  relationship: 'r',
  relationship_type: 'mentor_of',
  outgoing: true,
};
const appearance: Connection = {
  other,
  via: 'appears_in',
  appearance: 'a',
  kind: 'physical_presence',
};
const asset: Connection = { other, via: 'has_asset', attachment: 'z' };

test('each kind of link belongs to one group', () => {
  expect(groupOf(relationship)).toBe('relationships');
  expect(groupOf(appearance)).toBe('appearances');
  expect(groupOf(asset)).toBe('assets');
  expect(groupOf({ other, via: 'takes_place_at' })).toBe('events');
  expect(groupOf({ other, via: 'inside_section' })).toBe('structure');
});

test('groups keep their display order and empty ones are left out', () => {
  const groups = groupConnections([asset, appearance, relationship]);

  expect(groups.map((entry) => entry.group)).toEqual(['relationships', 'appearances', 'assets']);
});
