import { call } from '@/shared/backend';
import type { EntityId, EntityRef, TypeKey } from '@/shared/model';

/** Which entities are rows. Mirrors `EntityFilter` in `ontara-core`. */
export interface RowFilter {
  entity_type?: TypeKey | null;
  text?: string | null;
}

/** What the columns are. Mirrors `MatrixColumns`. */
export type MatrixColumns =
  | { source: 'structure'; section: EntityId | null; level: TypeKey | null }
  | { source: 'events' };

/** Mirrors `MatrixQuery`. */
export interface MatrixQuery {
  rows: RowFilter;
  columns: MatrixColumns;
  include_empty_rows: boolean;
}

/** A column: a section of the work or an event. Mirrors `MatrixColumnView`. */
export type MatrixColumn = EntityRef & {
  level: TypeKey | null;
  depth: number;
  parent: EntityId | null;
};

/** One recorded appearance. Mirrors `MatrixCell` in `ontara-core`. */
export interface MatrixCell {
  entity: EntityId;
  context: EntityId;
  appearance: string;
  kind: TypeKey;
  is_presence: boolean;
  viewpoint: EntityId | null;
  canon_state: TypeKey;
  position: number;
}

/** Mirrors `MatrixView` in `src-tauri/src/matrix.rs`. */
export interface Matrix {
  columns: MatrixColumn[];
  rows: EntityRef[];
  cells: MatrixCell[];
}

/** What the interface sends to add or update an appearance. Mirrors `AppearanceInput`. */
export interface AppearanceInput {
  id: string | null;
  entity: EntityId;
  context: EntityId;
  kind: TypeKey;
  viewpoint: EntityId | null;
  note: string | null;
  position?: number | null;
  canon_state?: TypeKey | null;
}

export function getAppearanceMatrix(query: MatrixQuery): Promise<Matrix> {
  return call('get_appearance_matrix', { query });
}

export function saveAppearance(appearance: AppearanceInput): Promise<unknown> {
  return call('save_appearance', { appearance });
}

/** Sets how an entity is present in a context; `null` takes it out of the context. */
export function setAppearanceKind(
  entity: EntityId,
  context: EntityId,
  kind: TypeKey | null,
): Promise<unknown> {
  return call('set_appearance_kind', { entity, context, kind });
}

export function removeAppearance(id: string): Promise<void> {
  return call('remove_appearance', { id });
}
