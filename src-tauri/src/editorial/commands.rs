use ontara_core::{EditorialImpact, Entity, EntityId};
use tauri::State;

use super::{NewSection, Section};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_editorial_tree(workspace: State<'_, Workspace>) -> Result<Vec<Section>, CommandError> {
    super::tree(&workspace)
}

#[tauri::command(async)]
pub fn add_section(
    workspace: State<'_, Workspace>,
    section: NewSection,
) -> Result<Entity, CommandError> {
    super::add(&workspace, section)
}

#[tauri::command(async)]
pub fn reorder_section(
    workspace: State<'_, Workspace>,
    entity: EntityId,
    parent: Option<EntityId>,
    index: usize,
) -> Result<(), CommandError> {
    super::reorder(&workspace, &entity, parent, index)
}

#[tauri::command(async)]
pub fn section_impact(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<EditorialImpact, CommandError> {
    super::impact(&workspace, &entity)
}

#[tauri::command(async)]
pub fn remove_section(
    workspace: State<'_, Workspace>,
    entity: EntityId,
    delete_entity: bool,
) -> Result<(), CommandError> {
    super::remove(&workspace, &entity, delete_entity)
}
