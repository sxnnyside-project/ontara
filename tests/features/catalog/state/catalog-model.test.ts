import { expect, test } from 'bun:test';
import type { EntityType } from '@/features/catalog/api/catalog-api';
import {
  creatableTypes,
  isValidKey,
  keyFromLabel,
  labelOf,
  typeChain,
} from '@/features/catalog/state/catalog-model';

function entityType(key: string, parent: string | null, isAbstract = false): EntityType {
  return {
    key,
    label: key.toUpperCase(),
    description: '',
    parent,
    is_abstract: isAbstract,
    role: 'generic',
    sort_order: 0,
    builtin: true,
  };
}

const types = [
  entityType('place', null, true),
  entityType('settlement', 'place', true),
  entityType('city', 'settlement'),
];

test('a type chain lists the type and its ancestors, nearest first', () => {
  expect(typeChain(types, 'city')).toEqual(['city', 'settlement', 'place']);
  expect(typeChain(types, 'unknown_type')).toEqual(['unknown_type']);
});

test('a loop in the data cannot hang the chain', () => {
  const loop = [entityType('a', 'b'), entityType('b', 'a')];

  expect(typeChain(loop, 'a')).toEqual(['a', 'b']);
});

test('only concrete types can be created', () => {
  expect(creatableTypes(types).map((entry) => entry.key)).toEqual(['city']);
});

test('labels fall back to the key', () => {
  expect(labelOf(types, 'city')).toBe('CITY');
  expect(labelOf(types, 'missing')).toBe('missing');
});

test('keys are made from labels the way the backend accepts them', () => {
  expect(keyFromLabel('Hunter Guild')).toBe('hunter_guild');
  expect(keyFromLabel('Señor de la Ceniza')).toBe('senor_de_la_ceniza');
  expect(keyFromLabel('3rd Age')).toBe('t_3rd_age');
  expect(keyFromLabel('  ')).toBe('');
});

test('key validation mirrors the backend rule', () => {
  expect(isValidKey('guild_2')).toBe(true);
  for (const bad of ['', 'Guild', '2fast', 'has space', 'dash-ed', 'a'.repeat(65)]) {
    expect(isValidKey(bad)).toBe(false);
  }
});

test('built-in labels are shown translated until renamed, and stored as the default again', async () => {
  const { setLocale, storedDefault, tDefault } = await import('@/shared/i18n');
  const { emptyCatalog, mapCatalogLabels, storeCatalogLabels } = await import(
    '@/features/catalog/state/catalog-model'
  );
  const catalog = emptyCatalog();
  catalog.entity_types = [
    {
      key: 'character',
      label: 'Character',
      description: '',
      parent: null,
      is_abstract: false,
      role: 'generic',
      sort_order: 0,
      builtin: true,
    },
    {
      key: 'guild',
      label: 'Guild',
      description: '',
      parent: null,
      is_abstract: false,
      role: 'generic',
      sort_order: 1,
      builtin: false,
    },
    {
      key: 'city',
      label: 'Metropolis',
      description: '',
      parent: null,
      is_abstract: false,
      role: 'generic',
      sort_order: 2,
      builtin: true,
    },
  ];

  setLocale('es');
  const shown = mapCatalogLabels(catalog, tDefault).entity_types.map((entry) => entry.label);
  const first = catalog.entity_types[0];
  if (!first) throw new Error('the fixture has an entity type');
  const back = storeCatalogLabels(
    { catalog: 'entity_type', ...first, label: 'Personaje' },
    storedDefault,
  );

  expect(shown).toEqual(['Personaje', 'Guild', 'Metropolis']);
  expect('label' in back && back.label).toBe('Character');
  setLocale('en');
});

test('creatable types are grouped by what they specialize, with loners gathered last', async () => {
  const { groupCreatableTypes } = await import('@/features/catalog/state/catalog-model');
  const type = (
    key: string,
    parent: string | null,
    options: { abstract?: boolean; role?: 'generic' | 'event' | 'editorial_unit' } = {},
  ): EntityType => ({
    key,
    label: key,
    description: '',
    parent,
    is_abstract: options.abstract ?? false,
    role: options.role ?? 'generic',
    sort_order: 0,
    builtin: true,
  });
  const types = [
    type('being', null, { abstract: true }),
    type('character', 'being'),
    type('deity', 'being'),
    type('item', null),
    type('event', null, { role: 'event' }),
    type('chapter', null, { role: 'editorial_unit' }),
  ];

  const groups = groupCreatableTypes(types, ['generic', 'event'], 'Other');

  expect(groups.map((group) => group.label)).toEqual(['being', 'Other']);
  expect(groups[0]?.types.map((entry) => entry.key)).toEqual(['character', 'deity']);
  expect(groups[1]?.types.map((entry) => entry.key)).toEqual(['item', 'event']);
});
