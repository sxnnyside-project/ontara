import type { Connection } from '../api/navigation-api';

/** The ways entities are linked, in the order they are shown. */
export type ConnectionGroup =
  | 'relationships'
  | 'appearances'
  | 'events'
  | 'assets'
  | 'properties'
  | 'structure';

export const GROUP_ORDER: readonly ConnectionGroup[] = [
  'relationships',
  'appearances',
  'events',
  'assets',
  'properties',
  'structure',
];

export function groupOf(connection: Connection): ConnectionGroup {
  switch (connection.via) {
    case 'relationship':
      return 'relationships';
    case 'appears_in':
    case 'hosts':
      return 'appearances';
    case 'participates':
    case 'has_participant':
    case 'takes_place_at':
    case 'hosts_event':
      return 'events';
    case 'has_asset':
    case 'attached_to':
      return 'assets';
    case 'property_of':
      return 'properties';
    case 'inside_section':
    case 'contains_section':
      return 'structure';
  }
}

/** The connections split by group, keeping only groups that have any, in display order. */
export function groupConnections(
  connections: readonly Connection[],
): { group: ConnectionGroup; items: Connection[] }[] {
  return GROUP_ORDER.map((group) => ({
    group,
    items: connections.filter((connection) => groupOf(connection) === group),
  })).filter((entry) => entry.items.length > 0);
}
