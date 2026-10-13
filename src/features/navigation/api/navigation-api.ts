import { call } from '@/shared/backend';
import type { EntityId, EntityRef, TypeKey } from '@/shared/model';

/** How two entities are connected, seen from the entity being looked at. Mirrors `ConnectionKind`. */
export type ConnectionKind =
  | { via: 'relationship'; relationship: string; relationship_type: TypeKey; outgoing: boolean }
  | { via: 'appears_in'; appearance: string; kind: TypeKey }
  | { via: 'hosts'; appearance: string; kind: TypeKey }
  | { via: 'participates'; role: string }
  | { via: 'has_participant'; role: string }
  | { via: 'takes_place_at' }
  | { via: 'hosts_event' }
  | { via: 'has_asset'; attachment: string }
  | { via: 'attached_to'; attachment: string }
  | { via: 'property_of'; value: string; definition: string }
  | { via: 'inside_section' }
  | { via: 'contains_section' };

/** One link of an entity, with the entity at the other end resolved. Mirrors `ConnectionView`. */
export type Connection = { other: EntityRef } & ConnectionKind;

export function getConnections(entity: EntityId): Promise<Connection[]> {
  return call('get_connections', { entity });
}
