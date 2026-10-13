import type { TypeKey } from '@/shared/model';
import type { Catalog, EntityType } from '../api/catalog-api';

/** The type followed by each ancestor, nearest first. Unknown keys yield only themselves. */
export function typeChain(types: readonly EntityType[], key: TypeKey): TypeKey[] {
  const byKey = new Map(types.map((entityType) => [entityType.key, entityType]));
  const chain: TypeKey[] = [];
  let next: TypeKey | null = key;
  while (next !== null && !chain.includes(next)) {
    chain.push(next);
    next = byKey.get(next)?.parent ?? null;
  }
  return chain;
}

/** The types that can be created directly, in the order the project defines them. */
export function creatableTypes(types: readonly EntityType[]): EntityType[] {
  return types.filter((entityType) => !entityType.is_abstract);
}

/** The label of a type, or its key when the project does not define it. */
export function labelOf(entries: readonly { key: TypeKey; label: string }[], key: TypeKey): string {
  return entries.find((entry) => entry.key === key)?.label ?? key;
}

/** Turns what an author typed into a valid key: lowercase, digits and underscores. */
export function keyFromLabel(label: string): string {
  const folded = label
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, 64);
  return /^[a-z]/.test(folded) ? folded : folded === '' ? '' : `t_${folded}`.slice(0, 64);
}

/** Whether a key is acceptable to the backend. */
export function isValidKey(key: string): boolean {
  return /^[a-z][a-z0-9_]{0,63}$/.test(key);
}

export function emptyCatalog(): Catalog {
  return {
    entity_types: [],
    relationship_types: [],
    canon_states: [],
    editorial_levels: [],
    appearance_kinds: [],
    diplomatic_stances: [],
    property_groups: [],
    property_definitions: [],
    vocabulary_terms: [],
  };
}
