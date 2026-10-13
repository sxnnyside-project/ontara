use tauri::State;

use super::HistoryState;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_history(workspace: State<'_, Workspace>) -> Result<HistoryState, CommandError> {
    super::get(&workspace)
}

#[tauri::command(async)]
pub fn undo(workspace: State<'_, Workspace>) -> Result<HistoryState, CommandError> {
    super::undo(&workspace)
}

#[tauri::command(async)]
pub fn redo(workspace: State<'_, Workspace>) -> Result<HistoryState, CommandError> {
    super::redo(&workspace)
}
