import { call } from '@/shared/backend';
import type {
  EntityId,
  EntityRef,
  NoteId,
  PropertyDefinition,
  PropertyValue,
  RecordRef,
  ReferenceCount,
  Temporal,
  Timestamp,
  TypeKey,
} from '@/shared/model';

export type NameKind =
  | 'alias'
  | 'epithet'
  | 'birth_name'
  | 'former_name'
  | 'title'
  | 'translation'
  | 'nickname';

/** A name an entity has had or is known by. Mirrors `EntityName` in `ontara-core`. */
export interface EntityName {
  name: string;
  kind: NameKind;
  language: EntityId | null;
  is_preferred: boolean;
  period: Temporal;
  note: string | null;
}

/** An identified element of a universe. Mirrors `Entity` in `ontara-core`. */
export interface Entity {
  id: EntityId;
  entity_type: TypeKey;
  name: string;
  sort_name: string | null;
  names: EntityName[];
  summary: string;
  description: string;
  tags: string[];
  canon_state: TypeKey;
  archived: boolean;
  origin: string;
  created_at: Timestamp;
  updated_at: Timestamp;
}

/** One row of an entity list. Mirrors `EntitySummary` in `src-tauri/src/entity.rs`. */
export interface EntitySummary extends EntityRef {
  sort_name: string | null;
  summary: string;
  tags: string[];
  updated_at: Timestamp;
}

export interface Note {
  id: NoteId;
  entity: EntityId;
  title: string;
  body: string;
  created_at: Timestamp;
  updated_at: Timestamp;
}

export interface CanonStateChange {
  id: string;
  target: RecordRef;
  from: TypeKey;
  to: TypeKey;
  reason: string | null;
  origin: string;
  changed_at: Timestamp;
}

/** Everything shown when one entity is open. Mirrors `EntityDetail`. */
export interface EntityDetail {
  entity: Entity;
  type_label: string;
  /** The entity's type followed by its ancestors, nearest first. */
  type_chain: TypeKey[];
  definitions: PropertyDefinition[];
  values: PropertyValue[];
  notes: Note[];
  canon_history: CanonStateChange[];
}

/** What narrows an entity list. Mirrors `EntityFilter` in `ontara-core`. */
export interface EntityFilter {
  /** This type or any of its subtypes. */
  entity_type?: TypeKey | null;
  role?: 'generic' | 'event' | 'editorial_unit' | 'asset' | null;
  canon_state?: TypeKey | null;
  tag?: string | null;
  text?: string | null;
  include_archived?: boolean;
}

export interface NewEntity {
  entity_type: TypeKey;
  name: string;
  summary?: string;
  description?: string;
  tags?: string[];
  canon_state?: TypeKey | null;
}

/** The fields of an entity an author edits directly. Mirrors `EntityEdit`. */
export interface EntityEdit {
  id: EntityId;
  name: string;
  sort_name: string | null;
  summary: string;
  description: string;
  tags: string[];
  names: EntityName[];
  archived: boolean;
}

export interface NoteInput {
  id: NoteId | null;
  entity: EntityId;
  title: string;
  body: string;
}

export function listEntities(filter: EntityFilter = {}): Promise<EntitySummary[]> {
  return call('list_entities', { filter });
}

export function getEntity(id: EntityId): Promise<EntityDetail> {
  return call('get_entity', { id });
}

export function createEntity(entity: NewEntity): Promise<Entity> {
  return call('create_entity', { entity });
}

export function updateEntity(edit: EntityEdit): Promise<Entity> {
  return call('update_entity', { edit });
}

export function entityReferences(id: EntityId): Promise<ReferenceCount[]> {
  return call('entity_references', { id });
}

/** Removes an entity and reports what pointed at it. */
export function removeEntity(id: EntityId, removeDependents: boolean): Promise<ReferenceCount[]> {
  return call('remove_entity', { id, removeDependents });
}

export function changeCanonState(
  target: RecordRef,
  to: TypeKey,
  reason: string | null,
): Promise<CanonStateChange | null> {
  return call('change_canon_state', { target, to, reason });
}

export function saveNote(note: NoteInput): Promise<Note> {
  return call('save_note', { note });
}

export function removeNote(id: NoteId): Promise<void> {
  return call('remove_note', { id });
}
