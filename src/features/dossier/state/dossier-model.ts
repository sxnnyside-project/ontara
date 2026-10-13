import type { EntityId, Instant, Temporal } from '@/shared/model';
import type { DossierRecord } from '../api/dossier-api';
import { CHOICES, type FieldDef, SECTIONS, type SectionDef } from './dossier-schema';

export function sectionOf(key: string): SectionDef | undefined {
  return SECTIONS.find((section) => section.key === key);
}

export function fieldOf(section: SectionDef, name: string): FieldDef | undefined {
  return section.fields.find((field) => field.name === name);
}

/** The field a new record holds the entity in: the first one whose kinds the entity matches. */
export function ownerField(section: SectionDef, typeChain: readonly string[]): string | null {
  for (const owner of section.owners) {
    if (owner.classes.length === 0 || owner.classes.some((kind) => typeChain.includes(kind))) {
      return owner.field;
    }
  }
  return null;
}

/** Whether the record of a link or an item can still change what it ties together. */
export function isKeyField(section: SectionDef, field: FieldDef): boolean {
  return section.shape === 'links' && section.owners.some((owner) => owner.field === field.name);
}

/** A record with nothing filled in, tied to the entity, ready for the form. */
export function blankRecord(
  section: SectionDef,
  entity: EntityId,
  typeChain: readonly string[],
  canonState: string,
): DossierRecord {
  const record: DossierRecord = {};
  for (const field of section.fields) {
    record[field.name] = blankValue(field, canonState);
  }
  if (section.shape === 'profile') {
    record.entity = entity;
  } else {
    const owner = ownerField(section, typeChain);
    if (owner) record[owner] = entity;
  }
  return record;
}

function blankValue(field: FieldDef, canonState: string): unknown {
  switch (field.kind) {
    case 'period':
      return { kind: 'unconstrained' } satisfies Temporal;
    case 'canon':
      return canonState;
    case 'flag':
      return field.required ? false : null;
    case 'choice':
      return field.required && field.group ? (CHOICES[field.group][0] ?? null) : null;
    case 'text':
    case 'longtext':
      return field.required ? '' : null;
    case 'term':
    case 'stance':
    case 'office':
      return field.required ? '' : null;
    case 'entity':
      return field.required ? '' : null;
    default:
      return null;
  }
}

function isEmpty(value: unknown): boolean {
  return (
    value === null || value === undefined || (typeof value === 'string' && value.trim() === '')
  );
}

/** The required fields that have nothing in them. */
export function missingFields(section: SectionDef, record: DossierRecord): string[] {
  return section.fields
    .filter((field) => field.required && isEmpty(record[field.name]))
    .map((field) => field.name);
}

/** What the form holds, as the backend expects it: no blank strings, whole numbers as numbers. */
export function cleanRecord(section: SectionDef, record: DossierRecord): DossierRecord {
  const cleaned: DossierRecord = { ...record };
  for (const field of section.fields) {
    const value = cleaned[field.name];
    switch (field.kind) {
      case 'text':
      case 'longtext':
        if (typeof value === 'string') {
          const trimmed = value.trim();
          cleaned[field.name] = trimmed === '' && !field.required ? null : trimmed;
        }
        break;
      case 'integer':
      case 'number': {
        if (typeof value === 'string') {
          const parsed = value.trim() === '' ? null : Number(value);
          cleaned[field.name] =
            parsed === null || !Number.isFinite(parsed)
              ? null
              : field.kind === 'integer'
                ? Math.trunc(parsed)
                : parsed;
        }
        break;
      }
      case 'entity':
      case 'term':
      case 'stance':
      case 'office':
        if (value === '' && !field.required) cleaned[field.name] = null;
        break;
      default:
        break;
    }
  }
  return cleaned;
}

/** Words to show a value in a list: the pieces that need the project or the language. */
export interface DossierLabels {
  entity(id: string): string;
  choice(group: string, value: string): string;
  term(vocabulary: string, key: string): string;
  stance(key: string): string;
  office(id: string): string;
  period(temporal: Temporal): string;
  moment(instant: Instant): string;
  yes: string;
  no: string;
}

/** A value, in words. Empty when there is nothing to say. */
export function valueText(field: FieldDef, value: unknown, labels: DossierLabels): string {
  if (isEmpty(value)) return '';
  switch (field.kind) {
    case 'entity':
      return labels.entity(String(value));
    case 'choice':
      return field.group ? labels.choice(field.group, String(value)) : String(value);
    case 'term':
      return field.vocabulary ? labels.term(field.vocabulary, String(value)) : String(value);
    case 'stance':
      return labels.stance(String(value));
    case 'office':
      return labels.office(String(value));
    case 'period': {
      const temporal = value as Temporal;
      return temporal.kind === 'unconstrained' ? '' : labels.period(temporal);
    }
    case 'instant':
      return labels.moment(value as Instant);
    case 'flag':
      return value === true ? labels.yes : labels.no;
    case 'canon':
      return '';
    default:
      return String(value);
  }
}

/** The pieces of a record's line in a list: what it is, and what describes it. */
export function describeRecord(
  section: SectionDef,
  record: DossierRecord,
  labels: DossierLabels,
): { title: string; detail: string } {
  const say = (names: readonly string[], separator: string): string =>
    names
      .map((name) => {
        const field = fieldOf(section, name);
        return field ? valueText(field, record[name], labels) : '';
      })
      .filter((text) => text !== '')
      .join(separator);
  return { title: say(section.title, ' → '), detail: say(section.detail, ' · ') };
}

/** The sections that apply to an entity of a type, given its type and ancestors. */
export function applicableSections(typeChain: readonly string[]): SectionDef[] {
  return SECTIONS.filter((section) =>
    section.owners.some(
      (owner) =>
        owner.classes.length === 0 || owner.classes.some((kind) => typeChain.includes(kind)),
    ),
  );
}
