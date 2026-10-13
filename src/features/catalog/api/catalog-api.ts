import { call } from '@/shared/backend';
import type {
  PropertyDefinition,
  PropertyDefinitionId,
  PropertyGroup,
  ReferenceCount,
  TypeKey,
} from '@/shared/model';

/** The structural job of an entity type. Mirrors `EntityRole` in `ontara-core`. */
export type EntityRole = 'generic' | 'event' | 'editorial_unit' | 'asset';

export interface EntityType {
  key: TypeKey;
  label: string;
  description: string;
  parent: TypeKey | null;
  is_abstract: boolean;
  role: EntityRole;
  sort_order: number;
  builtin: boolean;
}

export interface RelationshipType {
  key: TypeKey;
  label: string;
  inverse_label: string | null;
  description: string;
  directionality: 'directed' | 'undirected';
  acyclic: boolean;
  allowed_sources: TypeKey[];
  allowed_targets: TypeKey[];
  sort_order: number;
  builtin: boolean;
}

export interface CanonState {
  key: TypeKey;
  label: string;
  description: string;
  counts_as_canon: boolean;
  sort_order: number;
  builtin: boolean;
}

export interface EditorialLevel {
  key: TypeKey;
  label: string;
  depth: number;
  builtin: boolean;
}

export interface AppearanceKind {
  key: TypeKey;
  label: string;
  description: string;
  is_presence: boolean;
  sort_order: number;
  builtin: boolean;
}

export interface DiplomaticStance {
  key: TypeKey;
  label: string;
  description: string;
  is_hierarchical: boolean;
  sort_order: number;
  builtin: boolean;
}

/** The sets of terms an author may extend. Mirrors `Vocabulary` in `ontara-core`. */
export type Vocabulary =
  | 'trait_kind'
  | 'containment_kind'
  | 'family_tie_kind'
  | 'partnership_kind'
  | 'office_acquisition'
  | 'item_acquisition'
  | 'citizenship_status';

export interface VocabularyTerm {
  vocabulary: Vocabulary;
  key: TypeKey;
  label: string;
  description: string;
  sort_order: number;
  builtin: boolean;
}

/** Everything an author can configure. Mirrors `Catalog` in `src-tauri/src/catalog.rs`. */
export interface Catalog {
  entity_types: EntityType[];
  relationship_types: RelationshipType[];
  canon_states: CanonState[];
  editorial_levels: EditorialLevel[];
  appearance_kinds: AppearanceKind[];
  diplomatic_stances: DiplomaticStance[];
  property_groups: PropertyGroup[];
  property_definitions: PropertyDefinition[];
  vocabulary_terms: VocabularyTerm[];
}

/** A catalog entry to define or update, tagged with its catalog. Mirrors `CatalogRecord`. */
export type CatalogRecord =
  | ({ catalog: 'entity_type' } & EntityType)
  | ({ catalog: 'relationship_type' } & RelationshipType)
  | ({ catalog: 'canon_state' } & CanonState)
  | ({ catalog: 'editorial_level' } & EditorialLevel)
  | ({ catalog: 'appearance_kind' } & AppearanceKind)
  | ({ catalog: 'diplomatic_stance' } & DiplomaticStance)
  | ({ catalog: 'property_group' } & PropertyGroup)
  | ({ catalog: 'property_definition' } & PropertyDefinition)
  | ({ catalog: 'vocabulary_term' } & VocabularyTerm);

/** Names one catalog entry. Mirrors `CatalogEntry` in `ontara-core`. */
export type CatalogEntry =
  | { catalog: 'entity_type'; key: TypeKey }
  | { catalog: 'relationship_type'; key: TypeKey }
  | { catalog: 'canon_state'; key: TypeKey }
  | { catalog: 'editorial_level'; key: TypeKey }
  | { catalog: 'appearance_kind'; key: TypeKey }
  | { catalog: 'property_group'; key: TypeKey }
  | { catalog: 'property_definition'; id: PropertyDefinitionId }
  | { catalog: 'diplomatic_stance'; key: TypeKey }
  | { catalog: 'vocabulary_term'; vocabulary: Vocabulary; key: TypeKey };

export function getCatalog(): Promise<Catalog> {
  return call('get_catalog');
}

export function defineCatalogEntry(record: CatalogRecord): Promise<Catalog> {
  return call('define_catalog_entry', { record });
}

export function updateCatalogEntry(record: CatalogRecord): Promise<Catalog> {
  return call('update_catalog_entry', { record });
}

/** What still uses an entry, so the author knows why it cannot be removed. */
export function catalogEntryUsage(entry: CatalogEntry): Promise<ReferenceCount[]> {
  return call('catalog_entry_usage', { entry });
}

export function removeCatalogEntry(entry: CatalogEntry): Promise<Catalog> {
  return call('remove_catalog_entry', { entry });
}

/** Removes a property together with every value recorded for it. */
export function removePropertyWithValues(id: PropertyDefinitionId): Promise<Catalog> {
  return call('remove_property_with_values', { id });
}
