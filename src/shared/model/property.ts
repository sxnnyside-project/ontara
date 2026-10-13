import type { Instant } from './temporal';
import type {
  EntityId,
  PropertyDefinitionId,
  PropertyValueId,
  RelationshipId,
  Timestamp,
  TypeKey,
} from './ids';

export interface ChoiceOption {
  key: TypeKey;
  label: string;
}

/** Mirrors `PropertyDataType` in `ontara-core`. */
export type PropertyDataType =
  | { type: 'text' }
  | { type: 'integer' }
  | { type: 'real' }
  | { type: 'boolean' }
  | { type: 'date' }
  | { type: 'entity_reference'; entity_type: TypeKey | null }
  | { type: 'choice'; options: ChoiceOption[] };

/** Mirrors `PropertyOwner` in `ontara-core`. */
export type PropertyOwner =
  | { kind: 'entity_type'; key: TypeKey }
  | { kind: 'relationship_type'; key: TypeKey };

/** A trait, characteristic or attribute that records of a type can carry. */
export interface PropertyDefinition {
  id: PropertyDefinitionId;
  owner: PropertyOwner;
  key: TypeKey;
  label: string;
  description: string;
  group: TypeKey | null;
  data_type: PropertyDataType;
  required: boolean;
  multiple: boolean;
  sort_order: number;
  builtin: boolean;
}

export interface PropertyGroup {
  key: TypeKey;
  label: string;
  sort_order: number;
}

/** Mirrors `PropertyContent` in `ontara-core`. */
export type PropertyContent =
  | { type: 'text'; value: string }
  | { type: 'integer'; value: number }
  | { type: 'real'; value: number }
  | { type: 'boolean'; value: boolean }
  | { type: 'date'; value: Instant }
  | { type: 'entity_reference'; value: EntityId }
  | { type: 'choice'; value: TypeKey };

export type PropertyValueOwner =
  | { kind: 'entity'; id: EntityId }
  | { kind: 'relationship'; id: RelationshipId };

/** One recorded value of a property. */
export interface PropertyValue {
  id: PropertyValueId;
  owner: PropertyValueOwner;
  definition: PropertyDefinitionId;
  content: PropertyContent;
  valid_from: Instant | null;
  valid_until: Instant | null;
  note: string | null;
  position: number;
  canon_state: TypeKey;
  origin: string;
  created_at: Timestamp;
  updated_at: Timestamp;
}

/** What the interface sends to add or update a value. Mirrors `PropertyValueInput`. */
export interface PropertyValueInput {
  id: PropertyValueId | null;
  owner: PropertyValueOwner;
  definition: PropertyDefinitionId;
  content: PropertyContent;
  valid_from: Instant | null;
  valid_until: Instant | null;
  note: string | null;
  position: number;
}
