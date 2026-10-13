//! Exploring the neighborhood of an entity as a graph.

use ontara_core::{EntityId, GraphEdge, GraphQuery, MAX_GRAPH_NODES, explore};
use serde::Serialize;

use crate::entity::{EntityRef, resolve, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// An entity of the graph and how many steps it is from the center.
///
/// Mirrored by `src/features/graph/api/graph-api.ts`.
#[derive(Debug, Serialize)]
pub struct GraphNodeView {
    #[serde(flatten)]
    pub entity: EntityRef,
    pub distance: u8,
}

#[derive(Debug, Serialize)]
pub struct GraphView {
    pub center: EntityId,
    pub nodes: Vec<GraphNodeView>,
    pub edges: Vec<GraphEdge>,
    pub truncated: bool,
    /// The most entities a graph shows.
    pub limit: usize,
}

pub fn neighborhood(workspace: &Workspace, query: &GraphQuery) -> Result<GraphView, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        // The center must exist; asking for the graph of a removed entity is a mistake.
        resolve(&reader, &labels, &query.center)?;
        let graph = explore(&reader, query)?;
        let nodes = graph
            .nodes
            .iter()
            .map(|node| {
                Ok(GraphNodeView {
                    entity: resolve(&reader, &labels, &node.entity)?,
                    distance: node.distance,
                })
            })
            .collect::<Result<Vec<_>, CommandError>>()?;
        Ok(GraphView {
            center: graph.center,
            nodes,
            edges: graph.edges,
            truncated: graph.truncated,
            limit: MAX_GRAPH_NODES,
        })
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use ontara_core::{GraphEdgeKind, Lifecycle, Temporal, TypeKey};

    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::relationship::{RelationshipInput, save};
    use crate::workspace::fixture::workspace_with_project;

    fn entity(workspace: &Workspace, entity_type: &str, name: &str) -> EntityId {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new(entity_type).expect("a valid key"),
                name: name.to_owned(),
                summary: String::new(),
                description: String::new(),
                tags: std::collections::BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates")
        .id
    }

    #[test]
    fn the_graph_resolves_each_entity_and_filters_by_relationship_type() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let orin = entity(&workspace, "character", "Orin");
        let mara = entity(&workspace, "character", "Mara");
        for (kind, target) in [("mentor_of", orin), ("friend_of", mara)] {
            let _ = save(
                &workspace,
                RelationshipInput {
                    id: None,
                    relationship_type: TypeKey::new(kind).expect("a valid key"),
                    source: ayla,
                    target,
                    qualifier: None,
                    context: None,
                    lifecycle: Lifecycle::Active,
                    period: Temporal::Unconstrained,
                    note: None,
                },
            );
        }

        let mut query = GraphQuery::around(ayla);
        query
            .relationship_types
            .insert(TypeKey::new("mentor_of").expect("a valid key"));
        let graph = neighborhood(&workspace, &query).expect("explores");

        let names: Vec<_> = graph
            .nodes
            .iter()
            .map(|node| node.entity.name.as_str())
            .collect();
        assert_eq!(names, ["Ayla", "Orin"]);
        assert!(matches!(
            graph.edges[0].kind,
            GraphEdgeKind::Relationship { .. }
        ));
    }

    #[test]
    fn exploring_an_entity_that_does_not_exist_is_a_not_found_failure() {
        let (_directory, workspace) = workspace_with_project();

        let error = neighborhood(&workspace, &GraphQuery::around(EntityId::generate()))
            .expect_err("nothing there");

        assert_eq!(error.code, "not_found");
    }
}
