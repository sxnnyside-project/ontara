import { expect, test } from 'bun:test';
import type { EntityDetail } from '@/features/entity/api/entity-api';
import {
  draftOf,
  editOf,
  newAlternativeName,
  sameDraft,
} from '@/features/entity/state/entity-model';

const detail: EntityDetail = {
  entity: {
    id: 'e1',
    entity_type: 'character',
    name: 'Ayla',
    sort_name: null,
    names: [],
    summary: 'A smuggler',
    description: '',
    tags: ['smuggler'],
    canon_state: 'established',
    archived: false,
    origin: 'user',
    created_at: 1,
    updated_at: 1,
  },
  type_label: 'Character',
  type_chain: ['character', 'being'],
  definitions: [],
  values: [],
  notes: [],
  canon_history: [],
};

test('a draft starts equal to what is stored', () => {
  expect(sameDraft(draftOf(detail), draftOf(detail))).toBe(true);
});

test('editing a field makes the draft differ', () => {
  const draft = draftOf(detail);
  draft.summary = 'A retired smuggler';

  expect(sameDraft(draft, draftOf(detail))).toBe(false);
});

test('the edit sent to the backend trims names and drops blank alternatives', () => {
  const draft = draftOf(detail);
  draft.name = '  Ayla of the Reeds ';
  draft.sort_name = '   ';
  draft.names = [{ ...newAlternativeName(), name: 'Magpie' }, newAlternativeName()];

  const edit = editOf(detail, draft);

  expect(edit.name).toBe('Ayla of the Reeds');
  expect(edit.sort_name).toBeNull();
  expect(edit.names.map((name) => name.name)).toEqual(['Magpie']);
});
