use tauri::State;

use super::ReportView;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn check_consistency(workspace: State<'_, Workspace>) -> Result<ReportView, CommandError> {
    super::run(&workspace)
}
