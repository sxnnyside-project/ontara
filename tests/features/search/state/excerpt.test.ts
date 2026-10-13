import { expect, test } from 'bun:test';
import { excerptParts } from '@/features/search/state/excerpt';

test('an excerpt is split into plain parts and the words that matched', () => {
  expect(excerptParts('Keeper of the \u0002drowned\u0003 \u0002lighthouse\u0003…')).toEqual([
    { text: 'Keeper of the ', matched: false },
    { text: 'drowned', matched: true },
    { text: ' ', matched: false },
    { text: 'lighthouse', matched: true },
    { text: '…', matched: false },
  ]);
});

test('an excerpt with no marks is one plain part, and an empty one has none', () => {
  expect(excerptParts('plain')).toEqual([{ text: 'plain', matched: false }]);
  expect(excerptParts('')).toEqual([]);
});
