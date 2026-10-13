use ontara_core::{Appearance, AppearanceId, EntityId, MatrixQuery, TypeKey};
use tauri::State;

use super::{AppearanceInput, MatrixView};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_appearance_matrix(
    workspace: State<'_, Workspace>,
    query: MatrixQuery,
) -> Result<MatrixView, CommandError> {
    super::matrix(&workspace, &query)
}

#[tauri::command(async)]
pub fn save_appearance(
    workspace: State<'_, Workspace>,
    appearance: AppearanceInput,
) -> Result<Appearance, CommandError> {
    super::save(&workspace, appearance)
}

#[tauri::command(async)]
pub fn set_appearance_kind(
    workspace: State<'_, Workspace>,
    entity: EntityId,
    context: EntityId,
    kind: Option<TypeKey>,
) -> Result<Option<Appearance>, CommandError> {
    super::set_kind(&workspace, entity, context, kind)
}

#[tauri::command(async)]
pub fn remove_appearance(
    workspace: State<'_, Workspace>,
    id: AppearanceId,
) -> Result<(), CommandError> {
    super::remove(&workspace, &id)
}
