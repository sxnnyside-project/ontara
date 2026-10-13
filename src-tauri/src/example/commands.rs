use tauri::State;

use super::ExampleLoaded;
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn load_example_universe(
    workspace: State<'_, Workspace>,
    locale: String,
) -> Result<ExampleLoaded, CommandError> {
    super::load(&workspace, &locale)
}
