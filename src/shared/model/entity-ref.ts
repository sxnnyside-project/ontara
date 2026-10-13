import { tDefault } from '@/shared/i18n';
import type { EntityId, TypeKey } from './ids';

/**
 * A pointer to an entity that is enough to show it and to navigate to it. Mirrors `EntityRef`
 * in `src-tauri/src/entity.rs`.
 */
export interface EntityRef {
  id: EntityId;
  name: string;
  entity_type: TypeKey;
  type_label: string;
  canon_state: TypeKey;
  archived: boolean;
}

/** How many records of one kind still point at something. Mirrors `ReferenceCount`. */
export interface ReferenceCount {
  kind: string;
  count: number;
}

/** Identifies one record of any kind. Mirrors `RecordRef` in `ontara-core`. */
export interface RecordRef {
  kind: string;
  id: string;
}

/** The name of an entity's type in the interface language, unless the author renamed it. */
export function typeLabelOf(entity: Pick<EntityRef, 'entity_type' | 'type_label'>): string {
  return tDefault(`typeNames.${entity.entity_type}`, entity.type_label);
}
