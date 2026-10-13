use tauri::State;

use super::SearchResult;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn search_entities(
    workspace: State<'_, Workspace>,
    query: String,
    include_archived: bool,
) -> Result<Vec<SearchResult>, CommandError> {
    super::find(&workspace, &query, include_archived)
}
