import { expect, mock, test } from 'bun:test';

const invoke = mock(async (_command: string, _args?: Record<string, unknown>) => []);
mock.module('@tauri-apps/api/core', () => ({ invoke }));

const api = await import('@/features/relationship/api/relationship-api');

test('candidates are asked for by relationship type and side', async () => {
  await api.relationshipCandidates('mentor_of', 'target', 'ori');

  expect(invoke).toHaveBeenCalledWith('relationship_candidates', {
    relationshipType: 'mentor_of',
    side: 'target',
    text: 'ori',
  });
});
