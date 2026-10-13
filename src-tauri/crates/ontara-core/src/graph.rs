use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::connections::{ConnectionKind, connections_of};
use crate::entity::Entity;
use crate::identifier::{EntityId, RelationshipId};
use crate::repository::{ModelReader, Reader};
use crate::type_key::TypeKey;

/// Most entities a graph shows. A larger neighborhood is cut and says so, so a hub never turns
/// into an unreadable cloud.
pub const MAX_GRAPH_NODES: usize = 150;

/// Farthest an exploration walks from its center.
pub const MAX_GRAPH_DEPTH: u8 = 4;

fn default_depth() -> u8 {
    1
}

const fn enabled() -> bool {
    true
}

/// What to explore: the neighborhood of one entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphQuery {
    pub center: EntityId,
    /// Steps away from the center to walk, between 1 and [`MAX_GRAPH_DEPTH`].
    #[serde(default = "default_depth")]
    pub depth: u8,
    /// Only follow these relationship types; empty follows all of them.
    #[serde(default)]
    pub relationship_types: BTreeSet<TypeKey>,
    /// Also follow who took part in an event and where it happened.
    #[serde(default = "enabled")]
    pub include_events: bool,
}

impl GraphQuery {
    #[must_use]
    pub fn around(center: EntityId) -> Self {
        Self {
            center,
            depth: default_depth(),
            relationship_types: BTreeSet::new(),
            include_events: true,
        }
    }
}

/// An entity in the graph and how many steps it is from the center.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphNode {
    pub entity: EntityId,
    pub distance: u8,
}

/// What an edge stands for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "via", rename_all = "snake_case")]
pub enum GraphEdgeKind {
    Relationship {
        relationship: RelationshipId,
        relationship_type: TypeKey,
    },
    /// From the participant to the event.
    Participation { role: String },
    /// From the event to the place it happens at.
    Location,
}

impl GraphEdgeKind {
    fn discriminator(&self) -> String {
        match self {
            Self::Relationship { relationship, .. } => format!("relationship:{relationship}"),
            Self::Participation { role } => format!("participation:{role}"),
            Self::Location => "location".to_owned(),
        }
    }
}

/// A directed edge between two entities of the graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphEdge {
    pub source: EntityId,
    pub target: EntityId,
    #[serde(flatten)]
    pub kind: GraphEdgeKind,
}

/// The neighborhood of an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Graph {
    pub center: EntityId,
    /// The center first, then by distance.
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// More entities were within reach than [`MAX_GRAPH_NODES`] allows.
    pub truncated: bool,
}

/// The edge a connection stands for, seen from `current`, or nothing if it is not part of a graph.
fn edge_of(
    current: EntityId,
    other: EntityId,
    kind: ConnectionKind,
    query: &GraphQuery,
) -> Option<GraphEdge> {
    let (source, target, kind) = match kind {
        ConnectionKind::Relationship {
            relationship,
            relationship_type,
            outgoing,
        } => {
            if !query.relationship_types.is_empty()
                && !query.relationship_types.contains(&relationship_type)
            {
                return None;
            }
            let (source, target) = if outgoing {
                (current, other)
            } else {
                (other, current)
            };
            (
                source,
                target,
                GraphEdgeKind::Relationship {
                    relationship,
                    relationship_type,
                },
            )
        }
        ConnectionKind::Participates { role } if query.include_events => {
            (current, other, GraphEdgeKind::Participation { role })
        }
        ConnectionKind::HasParticipant { role } if query.include_events => {
            (other, current, GraphEdgeKind::Participation { role })
        }
        ConnectionKind::TakesPlaceAt if query.include_events => {
            (current, other, GraphEdgeKind::Location)
        }
        ConnectionKind::HostsEvent if query.include_events => {
            (other, current, GraphEdgeKind::Location)
        }
        _ => return None,
    };
    Some(GraphEdge {
        source,
        target,
        kind,
    })
}

/// Walks outward from the center, breadth first, through relationships and, if asked, through
/// event participation and location. Archived entities are left out. The result is the same for
/// the same data: nodes by distance then identifier, edges by endpoints.
pub fn explore<R: ModelReader>(reader: &R, query: &GraphQuery) -> Result<Graph, R::Error> {
    let depth = query.depth.clamp(1, MAX_GRAPH_DEPTH);
    let mut distance: BTreeMap<EntityId, u8> = BTreeMap::from([(query.center, 0)]);
    let mut edges: BTreeMap<(EntityId, EntityId, String), GraphEdge> = BTreeMap::new();
    let mut archived: BTreeSet<EntityId> = BTreeSet::new();
    let mut truncated = false;
    let mut queue = VecDeque::from([query.center]);

    while let Some(current) = queue.pop_front() {
        let here = distance.get(&current).copied().unwrap_or(depth);
        for connection in connections_of(reader, &current)? {
            let other = connection.other;
            if other == current || archived.contains(&other) {
                continue;
            }
            let Some(edge) = edge_of(current, other, connection.kind, query) else {
                continue;
            };
            if !distance.contains_key(&other) {
                if here >= depth {
                    continue;
                }
                let hidden =
                    Reader::<Entity>::get(reader, &other)?.is_none_or(|entity| entity.archived);
                if hidden {
                    archived.insert(other);
                    continue;
                }
                if distance.len() >= MAX_GRAPH_NODES {
                    truncated = true;
                    continue;
                }
                distance.insert(other, here + 1);
                queue.push_back(other);
            }
            let key = (edge.source, edge.target, edge.kind.discriminator());
            edges.entry(key).or_insert(edge);
        }
    }

    let mut nodes: Vec<GraphNode> = distance
        .into_iter()
        .map(|(entity, distance)| GraphNode { entity, distance })
        .collect();
    nodes.sort_by_key(|node| (node.distance, node.entity));
    Ok(Graph {
        center: query.center,
        nodes,
        edges: edges.into_values().collect(),
        truncated,
    })
}
