use ontara_core::GraphQuery;
use tauri::State;

use super::GraphView;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn explore_graph(
    workspace: State<'_, Workspace>,
    query: GraphQuery,
) -> Result<GraphView, CommandError> {
    super::neighborhood(&workspace, &query)
}
