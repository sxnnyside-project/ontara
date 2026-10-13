import { call } from '@/shared/backend';
import type { EntityId, EntityRef, TypeKey } from '@/shared/model';

/** A section of the work. Mirrors `Section` in `src-tauri/src/editorial.rs`. */
export interface Section {
  entity: EntityRef;
  level: TypeKey;
  parent: EntityId | null;
  position: number;
}

/** What restructuring or removing a section would touch. Mirrors `EditorialImpact`. */
export interface EditorialImpact {
  descendants: number;
  appearances: number;
  context_uses: number;
}

export interface NewSection {
  entity_type: TypeKey;
  level: TypeKey;
  name: string;
  parent: EntityId | null;
}

export function getEditorialTree(): Promise<Section[]> {
  return call('get_editorial_tree');
}

/** Creates the entity and places it last among its siblings. */
export function addSection(section: NewSection): Promise<{ id: EntityId }> {
  return call('add_section', { section });
}

export function reorderSection(
  entity: EntityId,
  parent: EntityId | null,
  index: number,
): Promise<void> {
  return call('reorder_section', { entity, parent, index });
}

export function sectionImpact(entity: EntityId): Promise<EditorialImpact> {
  return call('section_impact', { entity });
}

/** Takes the section out of the structure; with `deleteEntity` the entity goes too. */
export function removeSection(entity: EntityId, deleteEntity: boolean): Promise<void> {
  return call('remove_section', { entity, deleteEntity });
}
