import { expect, test } from 'bun:test';
import { forget, loadRecent, remember, saveRecent } from '@/features/project/state/recent-projects';

test('the most recent project goes first and is never listed twice', () => {
  let recent = remember([], { name: 'Aurea', path: '/p/aurea' });
  recent = remember(recent, { name: 'Brume', path: '/p/brume' });
  recent = remember(recent, { name: 'Aurea', path: '/p/aurea' });

  expect(recent.map((entry) => entry.path)).toEqual(['/p/aurea', '/p/brume']);
});

test('the list stays short', () => {
  let recent: ReturnType<typeof remember> = [];
  for (let index = 0; index < 20; index += 1) {
    recent = remember(recent, { name: `P${index}`, path: `/p/${index}` });
  }

  expect(recent).toHaveLength(8);
  expect(recent[0]?.path).toBe('/p/19');
});

test('a project can be forgotten without touching the others', () => {
  const recent = forget(
    [
      { name: 'A', path: '/a' },
      { name: 'B', path: '/b' },
    ],
    '/a',
  );

  expect(recent).toEqual([{ name: 'B', path: '/b' }]);
});

test('unreadable storage yields an empty list instead of failing', () => {
  expect(loadRecent(undefined)).toEqual([]);
  expect(loadRecent({ getItem: () => 'not json' })).toEqual([]);
  expect(loadRecent({ getItem: () => '[{"name":1}]' })).toEqual([]);
  expect(() =>
    saveRecent(
      {
        setItem: () => {
          throw new Error('blocked');
        },
      },
      [],
    ),
  ).not.toThrow();
});
