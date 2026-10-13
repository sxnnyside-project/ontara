import { expect, test } from 'bun:test';
import { describeMoment, describeTemporal, momentOnDay, toldMoment } from '@/shared/model';

test('a moment told in words has no day', () => {
  const moment = toldMoment('the winter of the long siege');

  expect(moment.day).toBeNull();
  expect(describeMoment(moment)).toBe('the winter of the long siege');
});

test('a moment on the timeline shows its day, with its words when it has them', () => {
  expect(describeMoment(momentOnDay(1420))).toBe('D+1420');
  expect(describeMoment(momentOnDay(1420, 'the coronation'))).toBe('the coronation (D+1420)');
});

test('periods read in plain words', () => {
  expect(describeTemporal({ kind: 'unknown' })).toBe('Date unknown');
  expect(describeTemporal({ kind: 'unconstrained' })).toBe('Not tied to the timeline');
  expect(describeTemporal({ kind: 'at', instant: momentOnDay(10) })).toBe('D+10');
  expect(describeTemporal({ kind: 'span', start: momentOnDay(10), end: null })).toBe(
    'D+10 to still holds',
  );
});
