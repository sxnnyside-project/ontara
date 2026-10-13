import { call } from '@/shared/backend';
import type { EntityId, EntityRef } from '@/shared/model';
import type { SectionShape } from '../state/dossier-schema';

/** A record of any section, as the backend stores it. The section says which fields it has. */
export type DossierRecord = Record<string, unknown>;

export interface SectionRecords {
  key: string;
  shape: SectionShape;
  records: DossierRecord[];
}

/**
 * What is recorded about an entity, by section, and the entities the records mention. Mirrors
 * `Dossier` in `src-tauri/src/dossier.rs`.
 */
export interface Dossier {
  sections: SectionRecords[];
  entities: Record<string, EntityRef>;
}

export function getDossier(entity: EntityId): Promise<Dossier> {
  return call('get_dossier', { entity });
}

/** Every record of one section, whoever it concerns. */
export function getDossierSection(section: string): Promise<Dossier> {
  return call('get_dossier_section', { section });
}

/** Adds the record, or replaces it when it already has an identifier. */
export function saveDossierRecord(section: string, record: DossierRecord): Promise<DossierRecord> {
  return call('save_dossier_record', { section, record });
}

export function removeDossierRecord(section: string, record: DossierRecord): Promise<void> {
  return call('remove_dossier_record', { section, record });
}
