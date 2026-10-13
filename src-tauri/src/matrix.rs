//! Who appears where: the appearance matrix and the appearances behind its cells.

use ontara_core::{
    Appearance, AppearanceId, EntityId, MatrixCell, MatrixQuery, ModelQueries, Origin, Reader,
    Timestamp, TypeKey, appearance_matrix,
};
use serde::{Deserialize, Serialize};

use crate::entity::{EntityRef, found, resolve, resolve_all, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// A column of the matrix: a section of the work or an event.
///
/// Mirrored by `src/features/matrix/api/matrix-api.ts`.
#[derive(Debug, Serialize)]
pub struct MatrixColumnView {
    #[serde(flatten)]
    pub entity: EntityRef,
    pub level: Option<TypeKey>,
    pub depth: usize,
    pub parent: Option<EntityId>,
}

#[derive(Debug, Serialize)]
pub struct MatrixView {
    pub columns: Vec<MatrixColumnView>,
    pub rows: Vec<EntityRef>,
    pub cells: Vec<MatrixCell>,
}

/// An appearance as the author edits it.
#[derive(Debug, Deserialize)]
pub struct AppearanceInput {
    /// Present when updating an appearance, absent when adding one.
    pub id: Option<AppearanceId>,
    pub entity: EntityId,
    pub context: EntityId,
    pub kind: TypeKey,
    pub viewpoint: Option<EntityId>,
    pub note: Option<String>,
    pub position: Option<i32>,
    pub canon_state: Option<TypeKey>,
}

pub fn matrix(workspace: &Workspace, query: &MatrixQuery) -> Result<MatrixView, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let result = appearance_matrix(&reader, query)?;
        let columns = result
            .columns
            .iter()
            .map(|column| {
                Ok(MatrixColumnView {
                    entity: resolve(&reader, &labels, &column.entity)?,
                    level: column.level.clone(),
                    depth: column.depth,
                    parent: column.parent,
                })
            })
            .collect::<Result<Vec<_>, CommandError>>()?;
        Ok(MatrixView {
            columns,
            rows: resolve_all(&reader, &labels, &result.rows)?,
            cells: result.cells,
        })
    })
}

pub fn save(workspace: &Workspace, input: AppearanceInput) -> Result<Appearance, CommandError> {
    workspace.write(|project| {
        let now = Timestamp::now();
        let stored = match input.id {
            Some(id) => Some(found(
                Reader::<Appearance>::get(&project.reader(), &id)?,
                "appearance",
                id,
            )?),
            None => None,
        };
        let is_update = stored.is_some();
        let mut appearance = stored.unwrap_or_else(|| {
            Appearance::new(input.entity, input.context, input.kind.clone(), now)
        });
        appearance.entity = input.entity;
        appearance.context = input.context;
        appearance.kind = input.kind;
        appearance.viewpoint = input.viewpoint;
        appearance.note = input.note;
        if let Some(position) = input.position {
            appearance.position = position;
        }
        if let Some(canon_state) = input.canon_state {
            appearance.canon_state = canon_state;
        }
        appearance.origin = Origin::User;
        Ok(project.edit(|editor| {
            if is_update {
                editor.update_appearance(appearance, now)
            } else {
                editor.add_appearance(appearance)
            }
        })?)
    })
}

/// Sets how an entity is present in a context with one gesture: the single appearance becomes
/// `kind`, a missing one is recorded, and `None` takes the entity out of the context.
pub fn set_kind(
    workspace: &Workspace,
    entity: EntityId,
    context: EntityId,
    kind: Option<TypeKey>,
) -> Result<Option<Appearance>, CommandError> {
    let existing: Vec<Appearance> = workspace.read(|project| {
        Ok(project
            .reader()
            .appearances_of(&entity)?
            .into_iter()
            .filter(|appearance| appearance.context == context)
            .collect())
    })?;
    match kind {
        None => {
            workspace.write(|project| {
                Ok(project.edit(|editor| {
                    for appearance in &existing {
                        editor.remove_appearance(&appearance.id)?;
                    }
                    Ok(())
                })?)
            })?;
            Ok(None)
        }
        Some(kind) => {
            let first = existing.first();
            let input = AppearanceInput {
                id: first.map(|appearance| appearance.id),
                entity,
                context,
                kind,
                viewpoint: first.and_then(|appearance| appearance.viewpoint),
                note: first.and_then(|appearance| appearance.note.clone()),
                position: None,
                canon_state: None,
            };
            save(workspace, input).map(Some)
        }
    }
}

pub fn remove(workspace: &Workspace, id: &AppearanceId) -> Result<(), CommandError> {
    workspace.write(|project| Ok(project.edit(|editor| editor.remove_appearance(id))?))
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use ontara_core::{EntityFilter, MatrixColumns};

    use super::*;
    use crate::editorial::{NewSection, add};
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    fn character(workspace: &Workspace, name: &str) -> EntityId {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new("character").expect("a valid key"),
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
    fn one_gesture_places_changes_and_clears_an_appearance() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = character(&workspace, "Ayla");
        let scene = add(
            &workspace,
            NewSection {
                entity_type: TypeKey::new("scene").expect("a valid key"),
                level: TypeKey::new("scene").expect("a valid key"),
                name: "The gate".to_owned(),
                parent: None,
            },
        )
        .expect("adds a scene")
        .id;
        let query = MatrixQuery {
            rows: EntityFilter::default(),
            columns: MatrixColumns::Structure {
                section: None,
                level: None,
            },
            include_empty_rows: false,
        };
        let physical = TypeKey::new("physical_presence").expect("a valid key");
        let dialogue = TypeKey::new("dialogue").expect("a valid key");

        set_kind(&workspace, ayla, scene, Some(physical)).expect("places");
        set_kind(&workspace, ayla, scene, Some(dialogue.clone())).expect("changes");
        let placed = matrix(&workspace, &query).expect("matrix");
        set_kind(&workspace, ayla, scene, None).expect("clears");
        let cleared = matrix(&workspace, &query).expect("matrix");

        assert_eq!(
            placed.cells.len(),
            1,
            "changing the kind reuses the appearance"
        );
        assert_eq!(placed.cells[0].kind, dialogue);
        assert_eq!(placed.rows[0].name, "Ayla");
        assert_eq!(placed.columns[0].entity.name, "The gate");
        assert!(cleared.cells.is_empty());
    }
}
