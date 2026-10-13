import { tDynamic } from '@/shared/i18n';

/** What an edit was, in words, from the machine name the backend gave it. */
export function changeText(label: string): string {
  return tDynamic(
    `historyLabels.${label.replace('.', '_')}`,
    tDynamic('historyLabels.edit', label),
  );
}

/** When an edit was made, for a list. Edits with no recorded time say nothing. */
export function timeText(at: number, locale: string): string {
  if (at <= 0) return '';
  return new Date(at).toLocaleString(locale, { dateStyle: 'medium', timeStyle: 'short' });
}
