/**
 * Identifiers and keys as the backend serializes them. They are plain strings on the wire; the
 * aliases only say which kind of string a signature expects.
 */

/** A UUID. Entities are identified by it, never by name. */
export type EntityId = string;
export type RelationshipId = string;
export type PropertyDefinitionId = string;
export type PropertyValueId = string;
export type NoteId = string;
export type AttachmentId = string;

/** The stable snake_case machine name of a type, state, level or term. */
export type TypeKey = string;

/** Milliseconds since the Unix epoch. */
export type Timestamp = number;
