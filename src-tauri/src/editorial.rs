//! The editorial structure: how the work is divided into volumes, parts, chapters and scenes.

use ontara_core::{
    EditorialImpact, EditorialNode, Entity, EntityId, ModelReaderExt, RemovalStrategy, Timestamp,
    TypeKey,
};
use serde::{Deserialize, Serialize};

use crate::entity::{EntityRef, reference, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// A section of the work, with the entity that stands for it.
///
/// Mirrored by `src/features/editorial/api/editorial-api.ts`.
#[derive(Debug, Serialize)]
pub struct Section {
    pub entity: EntityRef,
    pub level: TypeKey,
    pub parent: Option<EntityId>,
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewSection {
    pub entity_type: TypeKey,
    pub level: TypeKey,
    pub name: String,
    pub parent: Option<EntityId>,
}

pub fn tree(workspace: &Workspace) -> Result<Vec<Section>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let entities = reader.entities()?;
        let mut sections: Vec<Section> = reader
            .editorial_nodes()?
            .into_iter()
            .filter_map(|node| {
                let entity = entities.iter().find(|entity| entity.id == node.entity)?;
                Some(Section {
                    entity: reference(entity, &labels),
                    level: node.level,
                    parent: node.parent,
                    position: node.position,
                })
            })
            .collect();
        sections.sort_by_key(|section| section.position);
        Ok(sections)
    })
}

/// Creates the entity and places it in the structure in one step, last among its siblings.
pub fn add(workspace: &Workspace, input: NewSection) -> Result<Entity, CommandError> {
    workspace.write(|project| {
        let now = Timestamp::now();
        Ok(project.edit(|editor| {
            let entity = editor.add_entity(Entity::new(input.entity_type, input.name, now))?;
            editor.place_editorial_node(EditorialNode {
                entity: entity.id,
                level: input.level,
                parent: input.parent,
                position: i32::MAX,
            })?;
            editor.reorder_editorial_node(&entity.id, input.parent, usize::MAX)?;
            Ok(entity)
        })?)
    })
}

pub fn reorder(
    workspace: &Workspace,
    entity: &EntityId,
    parent: Option<EntityId>,
    index: usize,
) -> Result<(), CommandError> {
    workspace.write(|project| {
        project.edit(|editor| editor.reorder_editorial_node(entity, parent, index))?;
        Ok(())
    })
}

pub fn impact(workspace: &Workspace, entity: &EntityId) -> Result<EditorialImpact, CommandError> {
    workspace.write(|project| Ok(project.edit(|editor| editor.editorial_impact(entity))?))
}

/// Takes the section out of the structure. With `delete_entity` the entity goes too, and what
/// still points at it is removed with it.
pub fn remove(
    workspace: &Workspace,
    entity: &EntityId,
    delete_entity: bool,
) -> Result<(), CommandError> {
    workspace.write(|project| {
        project.edit(|editor| {
            editor.remove_editorial_node(entity)?;
            if delete_entity {
                editor.remove_entity(entity, RemovalStrategy::RemoveDependents)?;
            }
            Ok(())
        })?;
        Ok(())
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::workspace::fixture::workspace_with_project;

    fn section(
        workspace: &Workspace,
        entity_type: &str,
        level: &str,
        name: &str,
        parent: Option<EntityId>,
    ) -> EntityId {
        add(
            workspace,
            NewSection {
                entity_type: TypeKey::new(entity_type).expect("a valid key"),
                level: TypeKey::new(level).expect("a valid key"),
                name: name.to_owned(),
                parent,
            },
        )
        .expect("adds the section")
        .id
    }

    fn order(workspace: &Workspace, parent: Option<EntityId>) -> Vec<String> {
        tree(workspace)
            .expect("reads the tree")
            .into_iter()
            .filter(|section| section.parent == parent)
            .map(|section| section.entity.name)
            .collect()
    }

    #[test]
    fn sections_are_added_in_order_and_can_be_reordered() {
        let (_directory, workspace) = workspace_with_project();
        let volume = section(&workspace, "volume", "volume", "Embers", None);
        let first = section(&workspace, "chapter", "chapter", "The ferry", Some(volume));
        section(&workspace, "chapter", "chapter", "The ford", Some(volume));
        let third = section(&workspace, "chapter", "chapter", "The city", Some(volume));
        assert_eq!(
            order(&workspace, Some(volume)),
            ["The ferry", "The ford", "The city"]
        );

        reorder(&workspace, &third, Some(volume), 0).expect("moves the chapter first");
        assert_eq!(
            order(&workspace, Some(volume)),
            ["The city", "The ferry", "The ford"]
        );

        reorder(&workspace, &first, Some(volume), 99).expect("moves the chapter last");
        assert_eq!(
            order(&workspace, Some(volume)),
            ["The city", "The ford", "The ferry"]
        );
    }

    #[test]
    fn a_level_cannot_sit_inside_a_deeper_one() {
        let (_directory, workspace) = workspace_with_project();
        let volume = section(&workspace, "volume", "volume", "Embers", None);
        let scene = section(&workspace, "scene", "scene", "The crossing", Some(volume));

        let error = add(
            &workspace,
            NewSection {
                entity_type: TypeKey::new("volume").expect("a valid key"),
                level: TypeKey::new("volume").expect("a valid key"),
                name: "Inside a scene".to_owned(),
                parent: Some(scene),
            },
        )
        .expect_err("a volume cannot be inside a scene");

        assert_eq!(error.code, "rejected");
    }

    #[test]
    fn a_section_with_children_is_not_removed_and_its_impact_is_known() {
        let (_directory, workspace) = workspace_with_project();
        let volume = section(&workspace, "volume", "volume", "Embers", None);
        let chapter = section(&workspace, "chapter", "chapter", "The ferry", Some(volume));

        let refused = remove(&workspace, &volume, false).expect_err("it has a chapter");
        assert_eq!(refused.code, "editorial_children_remain");
        assert_eq!(impact(&workspace, &volume).expect("impact").descendants, 1);

        remove(&workspace, &chapter, true).expect("removes the chapter and its entity");
        assert_eq!(order(&workspace, Some(volume)), Vec::<String>::new());
        remove(&workspace, &volume, false).expect("the volume is now empty");
    }
}
