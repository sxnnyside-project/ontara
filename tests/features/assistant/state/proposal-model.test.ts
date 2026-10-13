import { expect, test } from 'bun:test';
import type { ChangeView, Proposal } from '@/features/assistant/api/proposal-api';
import {
  danglingPositions,
  pendingChanges,
  pendingCount,
  targetName,
} from '@/features/assistant/state/proposal-model';

function change(
  position: number,
  body: ChangeView['change'],
  status: ChangeView['status'] = 'pending',
): ChangeView {
  return { position, change: body, status, error: null, before: null };
}

const proposal: Proposal = {
  id: 'p',
  source: 'ai',
  author: 'model',
  title: 'Harbor',
  rationale: '',
  created_at: 1,
  settled: false,
  entities: {
    ayla: {
      id: 'ayla',
      name: 'Ayla',
      entity_type: 'character',
      type_label: 'Character',
      canon_state: 'canon',
      archived: false,
    },
  },
  changes: [
    change(0, {
      op: 'create_entity',
      entity_type: 'city',
      name: 'Harbor',
      summary: '',
      description: '',
      tags: [],
    }),
    change(1, {
      op: 'add_relationship',
      relationship_type: 'resides_in',
      source: { kind: 'existing', id: 'ayla' },
      target: { kind: 'created', name: 'Harbor' },
      qualifier: null,
    }),
    change(
      2,
      { op: 'add_note', entity: { kind: 'existing', id: 'ayla' }, title: 't', body: 'b' },
      'rejected',
    ),
  ],
};

test('only undecided changes count as pending', () => {
  expect(pendingCount([proposal, proposal])).toBe(4);
  expect(pendingChanges(proposal).map((entry) => entry.position)).toEqual([0, 1]);
});

test('names come from the project, or from the change that creates them', () => {
  expect(targetName({ kind: 'existing', id: 'ayla' }, proposal.entities, '?')).toBe('Ayla');
  expect(targetName({ kind: 'existing', id: 'gone' }, proposal.entities, '?')).toBe('?');
  expect(targetName({ kind: 'created', name: 'Harbor' }, proposal.entities, '?')).toBe('Harbor');
});

test('approving a link without the entity it needs is called out', () => {
  expect(danglingPositions(proposal, new Set([1]))).toEqual([1]);
  expect(danglingPositions(proposal, new Set([0, 1]))).toEqual([]);
});
