use tauri::State;

use super::BackupSummary;
use crate::project::ProjectSummary;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn list_backups(workspace: State<'_, Workspace>) -> Result<Vec<BackupSummary>, CommandError> {
    super::list(&workspace)
}

#[tauri::command(async)]
pub fn restore_project_backup(
    workspace: State<'_, Workspace>,
    name: String,
) -> Result<ProjectSummary, CommandError> {
    super::restore(&workspace, &name)
}
