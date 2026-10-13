import { expect, test } from 'bun:test';
import {
  applicableSections,
  blankRecord,
  cleanRecord,
  describeRecord,
  missingFields,
  ownerField,
  sectionOf,
} from '@/features/dossier/state/dossier-model';
import { CHOICES, SECTIONS } from '@/features/dossier/state/dossier-schema';
import { dossierTexts } from '@/shared/i18n/texts/dossier';

const labels = {
  entity: (id: string) => `<${id}>`,
  choice: (group: string, value: string) => `${group}:${value}`,
  term: (vocabulary: string, key: string) => `${vocabulary}:${key}`,
  stance: (key: string) => key,
  office: (id: string) => id,
  period: () => 'period',
  moment: () => 'moment',
  yes: 'yes',
  no: 'no',
};

function section(key: string) {
  const found = sectionOf(key);
  if (!found) throw new Error(`no section ${key}`);
  return found;
}

test('every section, field and choice has its words in the interface', () => {
  const fields = dossierTexts.dossierFields as Record<string, unknown>;
  const sections = dossierTexts.dossierSections as Record<string, unknown>;
  const choices = dossierTexts.dossierChoices as Record<string, unknown>;
  for (const entry of SECTIONS) {
    expect(sections[entry.key]).toBeDefined();
    expect(sections[`${entry.key}_hint`]).toBeDefined();
    for (const field of entry.fields) {
      if (field.kind === 'canon') continue;
      expect(fields[field.name]).toBeDefined();
      if (field.kind === 'choice') expect(field.group).toBeDefined();
    }
    for (const name of [...entry.title, ...entry.detail]) {
      expect(entry.fields.some((field) => field.name === name)).toBe(true);
    }
  }
  for (const [group, values] of Object.entries(CHOICES)) {
    for (const value of values) expect(choices[`${group}_${value}`]).toBeDefined();
  }
});

test('section keys are unique', () => {
  const keys = SECTIONS.map((entry) => entry.key);
  expect(new Set(keys).size).toBe(keys.length);
});

test('what applies to an entity follows its type and its ancestors', () => {
  const keys = (chain: string[]) => applicableSections(chain).map((entry) => entry.key);

  expect(keys(['character', 'being'])).toContain('being_trait');
  expect(keys(['character', 'being'])).not.toContain('place_border');
  expect(keys(['city', 'settlement', 'place'])).toContain('place_border');
  expect(keys(['realm', 'polity'])).toContain('office');
  expect(keys(['faction'])).toContain('office');
  expect(keys(['note_type'])).toEqual([]);
});

test('a new record is tied to the entity through the right field', () => {
  const tie = section('family_tie');
  const membership = section('membership');

  expect(ownerField(tie, ['character', 'being'])).toBe('parent');
  expect(ownerField(membership, ['faction'])).toBe('faction');
  expect(ownerField(membership, ['character', 'being'])).toBe('member');
  expect(blankRecord(section('being_profile'), 'e1', ['being'], 'canon').entity).toBe('e1');
  expect(blankRecord(tie, 'e1', ['being'], 'canon').parent).toBe('e1');
});

test('a blank record starts with a sensible period, canon state and first choice', () => {
  const record = blankRecord(section('place_border'), 'p1', ['place'], 'canon');

  expect(record.period).toEqual({ kind: 'unconstrained' });
  expect(record.canon_state).toBe('canon');
  expect(record.kind).toBe('land');
  expect(record.first).toBe('p1');
  expect(record.second).toBe('');
});

test('required fields are reported until they are filled', () => {
  const trait = section('being_trait');
  const record = blankRecord(trait, 'e1', ['being'], 'canon');

  expect(missingFields(trait, record).sort()).toEqual(['kind', 'text']);
  expect(missingFields(trait, { ...record, kind: 'fear', text: '  Deep water ' })).toEqual([]);
});

test('what the form holds is cleaned for the backend', () => {
  const resource = section('place_resource');
  const census = section('census');

  expect(
    cleanRecord(resource, { resource: '  Salt ', abundance: null, note: '   ' }),
  ).toMatchObject({ resource: 'Salt', note: null });
  expect(cleanRecord(census, { population: '1200', is_estimate: false })).toMatchObject({
    population: 1200,
  });
  expect(cleanRecord(census, { population: 'many' })).toMatchObject({ population: null });
});

test('a record reads as what it is and what describes it', () => {
  const line = describeRecord(
    section('family_tie'),
    { parent: 'a', child: 'b', kind: 'mother', period: { kind: 'unconstrained' } },
    labels,
  );

  expect(line.title).toBe('<a> → <b>');
  expect(line.detail).toBe('family_tie_kind:mother');
});
