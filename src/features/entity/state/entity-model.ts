import type { EntityEdit, EntityDetail, EntityName, NameKind } from '../api/entity-api';

/** The kinds of alternative name, in the order they are offered. */
export const NAME_KINDS: readonly NameKind[] = [
  'alias',
  'epithet',
  'nickname',
  'title',
  'birth_name',
  'former_name',
  'translation',
];

/** What an author edits of an entity, as a plain value that can be compared and reset. */
export interface EntityDraft {
  name: string;
  sort_name: string;
  summary: string;
  description: string;
  tags: string[];
  names: EntityName[];
  archived: boolean;
}

export function draftOf(detail: EntityDetail): EntityDraft {
  const { entity } = detail;
  return {
    name: entity.name,
    sort_name: entity.sort_name ?? '',
    summary: entity.summary,
    description: entity.description,
    tags: [...entity.tags],
    names: entity.names.map((name) => ({ ...name })),
    archived: entity.archived,
  };
}

export function sameDraft(a: EntityDraft, b: EntityDraft): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function editOf(detail: EntityDetail, draft: EntityDraft): EntityEdit {
  return {
    id: detail.entity.id,
    name: draft.name.trim(),
    sort_name: draft.sort_name.trim() === '' ? null : draft.sort_name.trim(),
    summary: draft.summary,
    description: draft.description,
    tags: draft.tags,
    names: draft.names.filter((name) => name.name.trim() !== ''),
    archived: draft.archived,
  };
}

export function newAlternativeName(): EntityName {
  return {
    name: '',
    kind: 'alias',
    language: null,
    is_preferred: false,
    period: { kind: 'unconstrained' },
    note: null,
  };
}
