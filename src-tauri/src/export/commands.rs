use tauri::State;

use super::{ExportFormat, ExportWords, ExportedFile};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn export_universe(
    workspace: State<'_, Workspace>,
    format: ExportFormat,
    words: ExportWords,
) -> Result<ExportedFile, CommandError> {
    super::run(&workspace, format, &words)
}
