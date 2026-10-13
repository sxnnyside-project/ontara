import type { SupportedLocale, TextGroup } from './define';
import { groups } from './texts';
import type { TranslationKey } from './types';

/** The groups joined by section, so a lookup is `texts[section][key][locale]`. */
const texts: TextGroup = {};
for (const group of groups) {
  for (const [section, entries] of Object.entries(group as TextGroup)) {
    texts[section] = { ...texts[section], ...entries };
  }
}

let currentLocale = $state<SupportedLocale>('es');

export function getLocale(): SupportedLocale {
  return currentLocale;
}

export function setLocale(locale: SupportedLocale): void {
  currentLocale = locale;
}

export function toggleLocale(): void {
  currentLocale = currentLocale === 'es' ? 'en' : 'es';
}

/** A text of the interface, with `{name}` placeholders replaced by `params`. */
export function t(key: TranslationKey, params?: Record<string, string | number>): string {
  const [section, field] = key.split('.') as [string, string];
  const translation = texts[section]?.[field];
  if (!translation) return key;

  let value = translation[currentLocale];
  if (params) {
    for (const [name, replacement] of Object.entries(params)) {
      value = value.replaceAll(`{${name}}`, String(replacement));
    }
  }
  return value;
}

/**
 * The text a built-in catalog entry shows. While the stored label is still the English default
 * it is shown in the interface language; once the author renamed it, their words win.
 */
export function tDefault(key: string, stored: string): string {
  const [section, field] = key.split('.') as [string, string];
  const translation = texts[section]?.[field];
  return translation && translation.en === stored ? translation[currentLocale] : stored;
}

/**
 * The label to store for a built-in entry: the English default when what the author sees is the
 * default in any language, so switching language keeps translating it, and their own words
 * otherwise.
 */
export function storedDefault(key: string, shown: string): string {
  const [section, field] = key.split('.') as [string, string];
  const translation = texts[section]?.[field];
  if (!translation) return shown;
  return Object.values(translation).includes(shown) ? translation.en : shown;
}
