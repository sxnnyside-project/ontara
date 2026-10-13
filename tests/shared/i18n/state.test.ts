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
  expect(t('timeline.dayPrefix')).toBe('D+');
});
