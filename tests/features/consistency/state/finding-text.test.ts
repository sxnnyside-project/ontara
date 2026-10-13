import { expect, test } from 'bun:test';
import type { Finding } from '@/features/consistency/api/consistency-api';
import {
  explanationParams,
  filterFindings,
  titleKey,
} from '@/features/consistency/state/finding-text';

const ref = (name: string) => ({
  id: name,
  name,
  entity_type: 'character',
  type_label: 'Character',
  canon_state: 'established',
  archived: false,
});

const findings: Finding[] = [
  {
    fingerprint: 'a',
    rule: 'participant_after_death',
    code: 'CC-01',
    severity: 'error',
    subject: ref('The feast'),
    related: [ref('Ayla')],
    facts: { death_day: '100', event_day: '300' },
  },
  {
    fingerprint: 'b',
    rule: 'duplicate_name',
    code: 'CC-14',
    severity: 'warning',
    subject: ref('Ayla'),
    related: [ref('Ayla')],
    facts: {},
  },
];

test('the message of a finding is built from its facts and the names it involves', () => {
  const first = findings[0];
  if (!first) throw new Error('fixture');

  expect(explanationParams(first)).toEqual({
    death_day: '100',
    event_day: '300',
    subject: 'The feast',
    related: 'Ayla',
  });
  expect(titleKey('duplicate_name')).toBe('consistency.rule_duplicate_name');
});

test('filters narrow by severity and by rule, and an empty filter keeps everything', () => {
  expect(filterFindings(findings, '', '')).toHaveLength(2);
  expect(filterFindings(findings, 'error', '')).toHaveLength(1);
  expect(filterFindings(findings, '', 'duplicate_name')[0]?.fingerprint).toBe('b');
  expect(filterFindings(findings, 'error', 'duplicate_name')).toHaveLength(0);
});
