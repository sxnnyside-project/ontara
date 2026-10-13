import { expect, mock, test } from 'bun:test';

const invoke = mock(async (_command: string, _args?: Record<string, unknown>) => ({}));
mock.module('@tauri-apps/api/core', () => ({ invoke }));

const api = await import('@/features/catalog/api/catalog-api');

test('removing an entry names the catalog and the key', async () => {
  await api.removeCatalogEntry({ catalog: 'entity_type', key: 'guild' });

  expect(invoke).toHaveBeenCalledWith('remove_catalog_entry', {
    entry: { catalog: 'entity_type', key: 'guild' },
  });
});

test('defining an entry sends the record tagged with its catalog', async () => {
  await api.defineCatalogEntry({
    catalog: 'canon_state',
    key: 'alternate_timeline',
    label: 'Alternate timeline',
    description: '',
    counts_as_canon: false,
    sort_order: 6,
    builtin: false,
  });

  const [command, args] = invoke.mock.calls.at(-1) ?? [];
  expect(command).toBe('define_catalog_entry');
  expect(args).toMatchObject({ record: { catalog: 'canon_state', key: 'alternate_timeline' } });
});
