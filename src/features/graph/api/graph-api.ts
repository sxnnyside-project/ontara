import { call } from '@/shared/backend';
import type { EntityId, EntityRef, TypeKey } from '@/shared/model';

/** What to explore. Mirrors `GraphQuery` in `ontara-core`. */
export interface GraphQuery {
  center: EntityId;
  /** Steps away from the center, 1 to 4. */
  depth: number;
  /** Only follow these relationship types; empty follows all of them. */
  relationship_types: TypeKey[];
  include_events: boolean;
}

/** What an edge stands for. Mirrors `GraphEdgeKind`. */
export type GraphEdgeKind =
  | { via: 'relationship'; relationship: string; relationship_type: TypeKey }
  | { via: 'participation'; role: string }
  | { via: 'location' };

/** A directed edge between two entities of the graph. Mirrors `GraphEdge`. */
export type GraphEdge = { source: EntityId; target: EntityId } & GraphEdgeKind;

/** An entity of the graph and its distance from the center. Mirrors `GraphNodeView`. */
export type GraphNode = EntityRef & { distance: number };

/** Mirrors `GraphView` in `src-tauri/src/graph.rs`. */
export interface Graph {
  center: EntityId;
  nodes: GraphNode[];
  edges: GraphEdge[];
  truncated: boolean;
  limit: number;
}

export function exploreGraph(query: GraphQuery): Promise<Graph> {
  return call('explore_graph', { query });
}
