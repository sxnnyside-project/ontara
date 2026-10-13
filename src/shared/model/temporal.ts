/** How exact a moment is. */
export type Precision = 'day' | 'month' | 'year' | 'decade' | 'century' | 'millennium';

/**
 * A moment an author tells. The label is what they say; the day, when they know it, places the
 * moment on the shared timeline. Mirrors `Instant` in `ontara-core`.
 */
export interface Instant {
  day: number | null;
  precision: Precision;
  earliest: number | null;
  latest: number | null;
  calendar: string | null;
  label: string | null;
}

/** When something holds. Mirrors `Temporal` in `ontara-core`. */
export type Temporal =
  | { kind: 'unknown' }
  | { kind: 'unconstrained' }
  | { kind: 'at'; instant: Instant }
  | { kind: 'span'; start: Instant | null; end: Instant | null };

/** A moment the author tells in words only. */
export function toldMoment(label: string): Instant {
  return { day: null, precision: 'day', earliest: null, latest: null, calendar: null, label };
}

/** A moment placed on the timeline, optionally with the words the author uses for it. */
export function momentOnDay(day: number, label: string | null = null): Instant {
  return { day, precision: 'day', earliest: null, latest: null, calendar: null, label };
}

/** A short text for a moment: its words, its day, or both. */
export function describeMoment(instant: Instant): string {
  const day = instant.day === null ? null : `D+${instant.day}`;
  if (instant.label && day) return `${instant.label} (${day})`;
  return instant.label ?? day ?? 'Unspecified';
}

/** A short text for a period. */
export function describeTemporal(temporal: Temporal): string {
  switch (temporal.kind) {
    case 'unknown':
      return 'Date unknown';
    case 'unconstrained':
      return 'Not tied to the timeline';
    case 'at':
      return describeMoment(temporal.instant);
    case 'span': {
      const start = temporal.start ? describeMoment(temporal.start) : 'always';
      const end = temporal.end ? describeMoment(temporal.end) : 'still holds';
      return `${start} to ${end}`;
    }
  }
}
