import { t } from './state.svelte';
import type { TranslationKey } from './types';

/**
 * Translates a key that is only known at run time, such as the name of a table in the backend.
 * Returns `fallback` when the dictionary has no entry for it.
 */
export function tDynamic(key: string, fallback: string): string {
  const text = t(key as TranslationKey);
  return text === key ? fallback : text;
}
