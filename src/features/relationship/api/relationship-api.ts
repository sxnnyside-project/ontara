import { call } from '@/shared/backend';
import type {
  EntityId,
  EntityRef,
  PropertyDefinition,
  PropertyValue,
  RelationshipId,
  Temporal,
  Timestamp,
  TypeKey,
} from '@/shared/model';

export type Lifecycle = 'active' | 'historical';

/** A first-class, typed connection between two entities. Mirrors `Relationship`. */
export interface Relationship {
  id: RelationshipId;
  relationship_type: TypeKey;
  source: EntityId;
  target: EntityId;
  qualifier: string | null;
  context: EntityId | null;
  lifecycle: Lifecycle;
  period: Temporal;
  note: string | null;
  canon_state: TypeKey;
  origin: string;
  created_at: Timestamp;
  updated_at: Timestamp;
}

/** A relationship with both ends resolved. Mirrors `RelationshipView`. */
export interface RelationshipView {
  relationship: Relationship;
  source: EntityRef;
  target: EntityRef;
  context: EntityRef | null;
}

/** Mirrors `RelationshipDetail`: the view plus the properties its type defines. */
export interface RelationshipDetail extends RelationshipView {
  definitions: PropertyDefinition[];
  values: PropertyValue[];
}

/** What the interface sends to add or update a relationship. Mirrors `RelationshipInput`. */
export interface RelationshipInput {
  id: RelationshipId | null;
  relationship_type: TypeKey;
  source: EntityId;
  target: EntityId;
  qualifier: string | null;
  context: EntityId | null;
  lifecycle: Lifecycle;
  period: Temporal;
  note: string | null;
}

export function listRelationships(entity: EntityId): Promise<RelationshipView[]> {
  return call('list_relationships', { entity });
}

export function getRelationship(id: RelationshipId): Promise<RelationshipDetail> {
  return call('get_relationship', { id });
}

export function saveRelationship(relationship: RelationshipInput): Promise<Relationship> {
  return call('save_relationship', { relationship });
}

export function removeRelationship(id: RelationshipId): Promise<void> {
  return call('remove_relationship', { id });
}

/** A way an entity can take part in a relationship. Mirrors `RelationshipOption`. */
export interface RelationshipOption {
  relationship_type: TypeKey;
  /** The entity is the source of the relationship; otherwise it is the target. */
  outgoing: boolean;
}

export type RelationshipSide = 'source' | 'target';

/** The relationship types an entity may take part in, and in which role. */
export function relationshipOptions(entity: EntityId): Promise<RelationshipOption[]> {
  return call('relationship_options', { entity });
}

/** The entities that may sit at one end of a relationship of the given type. */
export function relationshipCandidates(
  relationshipType: TypeKey,
  side: RelationshipSide,
  text: string | null,
): Promise<EntitySummaryLike[]> {
  return call('relationship_candidates', { relationshipType, side, text });
}

/** The part of an entity summary a candidate list needs. Mirrors `EntitySummary`. */
export interface EntitySummaryLike extends EntityRef {
  summary: string;
}
