use ontara_core::{EntityId, Relationship, RelationshipId, TypeKey};
use tauri::State;

use super::{RelationshipDetail, RelationshipInput, RelationshipOption, RelationshipView, Side};
use crate::entity::EntitySummary;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn list_relationships(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<Vec<RelationshipView>, CommandError> {
    super::list_for(&workspace, &entity)
}

#[tauri::command(async)]
pub fn get_relationship(
    workspace: State<'_, Workspace>,
    id: RelationshipId,
) -> Result<RelationshipDetail, CommandError> {
    super::detail(&workspace, &id)
}

#[tauri::command(async)]
pub fn save_relationship(
    workspace: State<'_, Workspace>,
    relationship: RelationshipInput,
) -> Result<Relationship, CommandError> {
    super::save(&workspace, relationship)
}

#[tauri::command(async)]
pub fn remove_relationship(
    workspace: State<'_, Workspace>,
    id: RelationshipId,
) -> Result<(), CommandError> {
    super::remove(&workspace, &id)
}

#[tauri::command(async)]
pub fn relationship_options(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<Vec<RelationshipOption>, CommandError> {
    super::options_for(&workspace, &entity)
}

#[tauri::command(async)]
pub fn relationship_candidates(
    workspace: State<'_, Workspace>,
    relationship_type: TypeKey,
    side: Side,
    text: Option<String>,
) -> Result<Vec<EntitySummary>, CommandError> {
    super::candidates(&workspace, &relationship_type, side, text.as_deref())
}
