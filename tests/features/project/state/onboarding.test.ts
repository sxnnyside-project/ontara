import { expect, test } from 'bun:test';
import { dismiss, isDismissed, onboardingSteps } from '@/features/project/state/onboarding';

const none = { entities: 0, relationships: 0, events: 0, sections: 0, appearances: 0, notes: 0 };

function memory() {
  const data = new Map<string, string>();
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => void data.set(key, value),
  };
}

test('a new universe has done nothing, and each step completes on its own', () => {
  expect(onboardingSteps(none).every((step) => !step.done)).toBe(true);

  const done = (counts: Partial<typeof none>) =>
    onboardingSteps({ ...none, ...counts })
      .filter((step) => step.done)
      .map((step) => step.key);

  expect(done({ entities: 2 })).toEqual([]);
  expect(done({ entities: 3 })).toEqual(['entities']);
  expect(done({ relationships: 1, sections: 4 })).toEqual(['relationships', 'structure']);
  expect(done({ appearances: 1, events: 1 })).toEqual(['appearances', 'events']);
});

test('closing the list is remembered per project', () => {
  const storage = memory();

  expect(isDismissed(storage, 'a')).toBe(false);
  dismiss(storage, 'a');
  dismiss(storage, 'a');

  expect(isDismissed(storage, 'a')).toBe(true);
  expect(isDismissed(storage, 'b')).toBe(false);
  expect(JSON.parse(storage.getItem('ontara.onboarding-dismissed') ?? '[]')).toEqual(['a']);
});

test('a missing or broken storage never throws', () => {
  expect(isDismissed(undefined, 'a')).toBe(false);
  expect(isDismissed({ getItem: () => '{not json' }, 'a')).toBe(false);
  expect(() => dismiss(undefined, 'a')).not.toThrow();
  expect(() =>
    dismiss(
      {
        getItem: () => null,
        setItem: () => {
          throw new Error('full');
        },
      },
      'a',
    ),
  ).not.toThrow();
});
