import type { TypeKey } from '@/shared/model';
import type { Catalog, CatalogRecord, EntityType } from '../api/catalog-api';

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

/** Concrete types that share the same root, so a long list can be offered in a few groups. */
export interface TypeGroup {
  /** The root type's key; empty for the group of types that stand alone. */
  root: TypeKey;
  label: string;
  types: EntityType[];
}

/**
 * The types an author can create, grouped by the type they all specialize: characters and
 * deities under "Being", cities and regions under "Place". Types with no relatives are gathered
 * in one last group so the list never turns into a column of groups of one.
 */
export function groupCreatableTypes(
  types: readonly EntityType[],
  roles: readonly string[] = ['generic', 'event'],
  otherLabel = 'Other',
): TypeGroup[] {
  const byKey = new Map(types.map((entry) => [entry.key, entry]));
  const concrete = types.filter((entry) => !entry.is_abstract && roles.includes(entry.role));
  const groups = new Map<TypeKey, EntityType[]>();
  for (const entry of concrete) {
    const root = typeChain(types, entry.key).at(-1) ?? entry.key;
    groups.set(root, [...(groups.get(root) ?? []), entry]);
  }
  const grouped: TypeGroup[] = [];
  const alone: EntityType[] = [];
  for (const [root, members] of groups) {
    const rootType = byKey.get(root);
    const related = members.length > 1 || (rootType?.is_abstract ?? false);
    if (related) {
      grouped.push({
        root,
        label: rootType?.label ?? root,
        types: [...members].sort((a, b) => a.sort_order - b.sort_order),
      });
    } else {
      alone.push(...members);
    }
  }
  grouped.sort(
    (a, b) => (byKey.get(a.root)?.sort_order ?? 0) - (byKey.get(b.root)?.sort_order ?? 0),
  );
  if (alone.length > 0) {
    grouped.push({
      root: '',
      label: otherLabel,
      types: alone.sort((a, b) => a.sort_order - b.sort_order),
    });
  }
  return grouped;
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

/** Turns a stored label into the one shown, or the shown one back into the stored. */
export type LabelMapper = (key: string, label: string) => string;

/** The dictionary keys of the texts of a built-in entry: its label, inverse label, description. */
function defaultKeys(record: CatalogRecord): { label?: string; inverse?: string; hint?: string } {
  switch (record.catalog) {
    case 'entity_type':
      return { label: `typeNames.${record.key}` };
    case 'relationship_type':
      return {
        label: `relationshipNames.${record.key}`,
        inverse: `relationshipInverses.${record.key}`,
      };
    case 'canon_state':
      return { label: `canonNames.${record.key}`, hint: `canonHints.${record.key}` };
    case 'editorial_level':
      return { label: `levelNames.${record.key}` };
    case 'appearance_kind':
      return { label: `appearanceNames.${record.key}` };
    case 'diplomatic_stance':
      return { label: `stanceNames.${record.key}` };
    case 'property_group':
      return { label: `groupNames.${record.key}` };
    case 'property_definition': {
      const owner = record.owner.key;
      return { label: `propertyNames.${owner}_${record.key}` };
    }
    case 'vocabulary_term':
      return { label: `termNames.${record.key}` };
  }
}

/** Applies a label mapper to the texts of a built-in entry; entries the author made are kept. */
function mapRecord<T extends CatalogRecord>(record: T, map: LabelMapper): T {
  // Property groups carry no built-in flag; a translation only applies to the English default.
  if (record.catalog !== 'property_group' && !record.builtin) return record;
  const keys = defaultKeys(record);
  const mapped: Record<string, unknown> = { ...record };
  if (keys.label) mapped.label = map(keys.label, record.label);
  if (keys.inverse && 'inverse_label' in record && record.inverse_label !== null) {
    mapped.inverse_label = map(keys.inverse, record.inverse_label);
  }
  if (keys.hint && 'description' in record && record.description !== '') {
    mapped.description = map(keys.hint, record.description);
  }
  return mapped as T;
}

/** The catalog with the labels of built-in entries mapped, such as translated for display. */
export function mapCatalogLabels(catalog: Catalog, map: LabelMapper): Catalog {
  const each = <C extends CatalogRecord['catalog'], E extends object>(kind: C, entries: E[]): E[] =>
    entries.map((entry) => {
      const record = { catalog: kind, ...entry } as unknown as CatalogRecord;
      const { catalog: _tag, ...rest } = mapRecord(record, map) as CatalogRecord & { catalog: C };
      return rest as unknown as E;
    });
  return {
    entity_types: each('entity_type', catalog.entity_types),
    relationship_types: each('relationship_type', catalog.relationship_types),
    canon_states: each('canon_state', catalog.canon_states),
    editorial_levels: each('editorial_level', catalog.editorial_levels),
    appearance_kinds: each('appearance_kind', catalog.appearance_kinds),
    diplomatic_stances: each('diplomatic_stance', catalog.diplomatic_stances),
    property_groups: each('property_group', catalog.property_groups),
    property_definitions: each('property_definition', catalog.property_definitions),
    vocabulary_terms: each('vocabulary_term', catalog.vocabulary_terms),
  };
}

/** The record to send to the backend: a default shown translated goes back as its default. */
export function storeCatalogLabels(record: CatalogRecord, map: LabelMapper): CatalogRecord {
  return mapRecord(record, map);
}
