import { expect, mock, test } from 'bun:test';

const invoke = mock(async (_command: string, _args?: Record<string, unknown>) => []);
mock.module('@tauri-apps/api/core', () => ({ invoke }));

const api = await import('@/features/entity/api/entity-api');

test('listing entities sends the filter', async () => {
  await api.listEntities({ entity_type: 'place', text: 'ash' });

  expect(invoke).toHaveBeenCalledWith('list_entities', {
    filter: { entity_type: 'place', text: 'ash' },
  });
});

test('arguments use the camelCase names Tauri maps to the Rust parameters', async () => {
  await api.removeEntity('e1', true);

  expect(invoke).toHaveBeenCalledWith('remove_entity', { id: 'e1', removeDependents: true });
});
