use ontara_core::EntityId;
use tauri::State;

use super::ConnectionView;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_connections(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<Vec<ConnectionView>, CommandError> {
    super::connections(&workspace, &entity)
}
