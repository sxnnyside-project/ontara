import { en } from './locales/en';
import { es } from './locales/es';
import { workspaceEn } from './locales/workspace-en';
import { workspaceEs } from './locales/workspace-es';
import type { SupportedLocale, TranslationKey } from './types';

type Dictionary = Record<string, Record<string, string>>;

/** Joins the sections of two dictionaries; a section present in both keeps the keys of each. */
function merge(...parts: Dictionary[]): Dictionary {
  const joined: Dictionary = {};
  for (const part of parts) {
    for (const [section, fields] of Object.entries(part)) {
      joined[section] = { ...joined[section], ...fields };
    }
  }
  return joined;
}

const locales: Record<SupportedLocale, Dictionary> = {
  en: merge(en as unknown as Dictionary, workspaceEn),
  es: merge(es as unknown as Dictionary, workspaceEs as unknown as Dictionary),
};

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

export function t(key: TranslationKey, params?: Record<string, string | number>): string {
  const [section, field] = key.split('.') as [string, string];
  const sectionObj = locales[currentLocale][section];

  if (!sectionObj || !(field in sectionObj)) {
    return key;
  }

  let value = sectionObj[field] ?? key;

  if (params) {
    for (const [paramKey, paramValue] of Object.entries(params)) {
      value = value.replaceAll(`{${paramKey}}`, String(paramValue));
    }
  }

  return value;
}
