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
