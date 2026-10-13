use ontara_core::EntityId;
use serde_json::Value;
use tauri::State;

use super::Dossier;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_dossier(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<Dossier, CommandError> {
    super::get(&workspace, entity)
}

#[tauri::command(async)]
pub fn save_dossier_record(
    workspace: State<'_, Workspace>,
    section: String,
    record: Value,
) -> Result<Value, CommandError> {
    super::save(&workspace, &section, record)
}

#[tauri::command(async)]
pub fn remove_dossier_record(
    workspace: State<'_, Workspace>,
    section: String,
    record: Value,
) -> Result<(), CommandError> {
    super::remove(&workspace, &section, record)
}

#[tauri::command(async)]
pub fn get_dossier_section(
    workspace: State<'_, Workspace>,
    section: String,
) -> Result<Dossier, CommandError> {
    super::all(&workspace, &section)
}
