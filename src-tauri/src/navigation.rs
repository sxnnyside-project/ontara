//! Moving through the universe by its links.

use ontara_core::{ConnectionKind, EntityId, connections_of};
use serde::Serialize;

use crate::entity::{EntityRef, resolve, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// One link of an entity, with the entity at the other end resolved.
///
/// Mirrored by `src/features/navigation/api/navigation-api.ts`.
#[derive(Debug, Serialize)]
pub struct ConnectionView {
    pub other: EntityRef,
    #[serde(flatten)]
    pub kind: ConnectionKind,
}

pub fn connections(
    workspace: &Workspace,
    entity: &EntityId,
) -> Result<Vec<ConnectionView>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        connections_of(&reader, entity)?
            .into_iter()
            .map(|connection| {
                Ok(ConnectionView {
                    other: resolve(&reader, &labels, &connection.other)?,
                    kind: connection.kind,
                })
            })
            .collect()
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use ontara_core::{Lifecycle, Temporal, TypeKey};

    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::relationship::{RelationshipInput, save};
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn links_are_followed_in_both_directions_with_the_other_end_resolved() {
        let (_directory, workspace) = workspace_with_project();
        let entity = |entity_type: &str, name: &str| {
            create(
                &workspace,
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
        };
        let ayla = entity("character", "Ayla");
        let orin = entity("character", "Orin");
        save(
            &workspace,
            RelationshipInput {
                id: None,
                relationship_type: TypeKey::new("mentor_of").expect("a valid key"),
                source: ayla.id,
                target: orin.id,
                qualifier: None,
                context: None,
                lifecycle: Lifecycle::Active,
                period: Temporal::Unconstrained,
                note: None,
            },
        )
        .expect("relates");

        let from_orin = connections(&workspace, &orin.id).expect("follows");

        assert_eq!(from_orin.len(), 1);
        assert_eq!(from_orin[0].other.name, "Ayla");
        assert!(matches!(
            from_orin[0].kind,
            ConnectionKind::Relationship {
                outgoing: false,
                ..
            }
        ));
    }
}
