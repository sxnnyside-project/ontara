import type { Timeline, TimelineEntry, TimelineLink } from '../api/timeline-api';

export type TimelineOrder = 'chronological' | 'narrative';

/** The label of a day on the shared timeline. */
export function dayLabel(day: number | null): string {
  return day === null ? '—' : `D${day < 0 ? '−' : '+'}${Math.abs(day)}`;
}

/** The day, or the span of days, an event covers. */
export function spanLabel(entry: TimelineEntry): string {
  if (entry.start_day === null) return dayLabel(null);
  if (entry.end_day === null || entry.end_day === entry.start_day) return dayLabel(entry.start_day);
  return `${dayLabel(entry.start_day)} → ${dayLabel(entry.end_day)}`;
}

/**
 * The dated events in the order asked for. Chronological is the order the backend gives;
 * narrative is the order a reader meets them, with events the work does not present last.
 */
export function inOrder(timeline: Timeline, order: TimelineOrder): TimelineEntry[] {
  if (order === 'chronological') return timeline.dated;
  const rank = (entry: TimelineEntry) => entry.narrative_rank ?? Number.POSITIVE_INFINITY;
  return [...timeline.dated].sort((a, b) => rank(a) - rank(b));
}

/** The names of what led to an event, among the events shown. */
export function causesOf(
  entry: TimelineEntry,
  links: readonly TimelineLink[],
  names: ReadonlyMap<string, string>,
): string[] {
  return links
    .filter((link) => link.effect === entry.entity.id)
    .map((link) => names.get(link.cause))
    .filter((name): name is string => name !== undefined);
}

/** How many events are told out of order, a measure of how much the work jumps in time. */
export function countOutOfOrder(timeline: Timeline): number {
  return timeline.dated.filter((entry) => entry.out_of_order).length;
}
