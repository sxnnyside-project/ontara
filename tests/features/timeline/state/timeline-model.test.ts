import { expect, test } from 'bun:test';
import type { Timeline, TimelineEntry } from '@/features/timeline/api/timeline-api';
import {
  causesOf,
  countOutOfOrder,
  dayLabel,
  inOrder,
  spanLabel,
} from '@/features/timeline/state/timeline-model';

function entry(id: string, start: number | null, rank: number | null, jump = false): TimelineEntry {
  return {
    entity: {
      id,
      name: id,
      entity_type: 'event',
      type_label: 'Event',
      canon_state: 'established',
      archived: false,
    },
    period: { kind: 'unknown' },
    start_day: start,
    end_day: start,
    location: null,
    scale: null,
    participants: 0,
    presented_in: [],
    narrative_rank: rank,
    out_of_order: jump,
  };
}

const timeline: Timeline = {
  dated: [
    entry('founding', 10, 2, true),
    entry('siege', 20, 0),
    entry('ending', 30, 1),
    entry('ghost', 40, null),
  ],
  undated: [],
  links: [{ cause: 'siege', effect: 'ending', kind: 'direct' }],
};

test('days read as offsets from the origin of the shared timeline', () => {
  expect(dayLabel(12)).toBe('D+12');
  expect(dayLabel(-3)).toBe('D−3');
  expect(dayLabel(null)).toBe('—');
});

test('a span shows both ends and a single day shows one', () => {
  expect(spanLabel({ ...entry('a', 5, null), end_day: 9 })).toBe('D+5 → D+9');
  expect(spanLabel(entry('a', 5, null))).toBe('D+5');
});

test('narrative order follows the reading order and leaves untold events last', () => {
  const order = inOrder(timeline, 'narrative').map((item) => item.entity.id);

  expect(order).toEqual(['siege', 'ending', 'founding', 'ghost']);
  expect(inOrder(timeline, 'chronological')[0]?.entity.id).toBe('founding');
});

test('causes are named from the links among the events shown', () => {
  const names = new Map([
    ['siege', 'The siege'],
    ['ending', 'The ending'],
  ]);

  expect(causesOf(entry('ending', 30, 1), timeline.links, names)).toEqual(['The siege']);
  expect(countOutOfOrder(timeline)).toBe(1);
});
