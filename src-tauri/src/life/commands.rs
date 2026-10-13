use ontara_core::{EntityId, Instant};
use tauri::State;

use super::Life;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_life(workspace: State<'_, Workspace>, entity: EntityId) -> Result<Life, CommandError> {
    super::get(&workspace, &entity)
}

#[tauri::command(async)]
pub fn save_life(
    workspace: State<'_, Workspace>,
    entity: EntityId,
    birth: Option<Instant>,
    death: Option<Instant>,
) -> Result<Life, CommandError> {
    super::save(&workspace, entity, birth, death)
}
