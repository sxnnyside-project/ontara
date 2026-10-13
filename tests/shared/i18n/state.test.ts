// Polyfill Svelte runes for Bun test environment
// @ts-expect-error mock Svelte rune in test runner
globalThis.$state = <T>(val: T): T => val;

import { expect, test } from 'bun:test';

const { getLocale, setLocale, t, toggleLocale } = await import('@/shared/i18n');

test('i18n returns correct translations for Spanish and English', () => {
  setLocale('es');
  expect(getLocale()).toBe('es');
  expect(t('common.appName')).toBe('Ontara');
  expect(t('canon.established')).toBe('Establecido');
  expect(t('tabs.graph')).toBe('Grafo');

  setLocale('en');
  expect(getLocale()).toBe('en');
  expect(t('canon.established')).toBe('Established');
  expect(t('tabs.graph')).toBe('Graph');

  toggleLocale();
  expect(getLocale()).toBe('es');
});

test('i18n interpolates parameters if provided', () => {
  setLocale('en');
  expect(t('graphView.steps', { count: 3 })).toBe('3 step(s)');
});

test('every text is written in every language and no key is defined twice', async () => {
  const { groups } = await import('@/shared/i18n/texts');
  const { LOCALES } = await import('@/shared/i18n/define');
  const seen = new Set<string>();
  for (const group of groups) {
    for (const [section, entries] of Object.entries(group)) {
      for (const [key, translation] of Object.entries(entries)) {
        const id = `${section}.${key}`;
        expect(seen.has(id)).toBe(false);
        seen.add(id);
        for (const locale of LOCALES) {
          expect((translation as Record<string, string>)[locale]?.length).toBeGreaterThan(0);
        }
      }
    }
  }
});
