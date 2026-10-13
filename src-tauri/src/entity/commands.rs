use ontara_core::{
    CanonStateChange, Entity, EntityFilter, EntityId, Note, NoteId, PropertyValue, PropertyValueId,
    RecordRef, ReferenceCount, TypeKey,
};
use tauri::State;

use super::{EntityDetail, EntityEdit, EntitySummary, NewEntity, NoteInput, PropertyValueInput};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn list_entities(
    workspace: State<'_, Workspace>,
    filter: EntityFilter,
) -> Result<Vec<EntitySummary>, CommandError> {
    super::list(&workspace, &filter)
}

#[tauri::command(async)]
pub fn get_entity(
    workspace: State<'_, Workspace>,
    id: EntityId,
) -> Result<EntityDetail, CommandError> {
    super::detail(&workspace, id)
}

#[tauri::command(async)]
pub fn create_entity(
    workspace: State<'_, Workspace>,
    entity: NewEntity,
) -> Result<Entity, CommandError> {
    super::create(&workspace, entity)
}

#[tauri::command(async)]
pub fn update_entity(
    workspace: State<'_, Workspace>,
    edit: EntityEdit,
) -> Result<Entity, CommandError> {
    super::update(&workspace, edit)
}

#[tauri::command(async)]
pub fn entity_references(
    workspace: State<'_, Workspace>,
    id: EntityId,
) -> Result<Vec<ReferenceCount>, CommandError> {
    super::references(&workspace, &id)
}

#[tauri::command(async)]
pub fn remove_entity(
    workspace: State<'_, Workspace>,
    id: EntityId,
    remove_dependents: bool,
) -> Result<Vec<ReferenceCount>, CommandError> {
    super::remove(&workspace, &id, remove_dependents)
}

#[tauri::command(async)]
pub fn change_canon_state(
    workspace: State<'_, Workspace>,
    target: RecordRef,
    to: TypeKey,
    reason: Option<String>,
) -> Result<Option<CanonStateChange>, CommandError> {
    super::change_canon_state(&workspace, target, to, reason)
}

#[tauri::command(async)]
pub fn save_property_value(
    workspace: State<'_, Workspace>,
    value: PropertyValueInput,
) -> Result<PropertyValue, CommandError> {
    super::save_property_value(&workspace, value)
}

#[tauri::command(async)]
pub fn remove_property_value(
    workspace: State<'_, Workspace>,
    id: PropertyValueId,
) -> Result<(), CommandError> {
    super::remove_property_value(&workspace, &id)
}

#[tauri::command(async)]
pub fn save_note(workspace: State<'_, Workspace>, note: NoteInput) -> Result<Note, CommandError> {
    super::save_note(&workspace, note)
}

#[tauri::command(async)]
pub fn remove_note(workspace: State<'_, Workspace>, id: NoteId) -> Result<(), CommandError> {
    super::remove_note(&workspace, &id)
}
