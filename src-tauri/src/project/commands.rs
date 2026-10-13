use tauri::State;

use super::{ProjectSummary, UniverseCounts};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn create_project(
    workspace: State<'_, Workspace>,
    path: String,
    name: String,
) -> Result<ProjectSummary, CommandError> {
    super::create(&workspace, &path, &name)
}

#[tauri::command(async)]
pub fn open_project(
    workspace: State<'_, Workspace>,
    path: String,
    allow_migration: bool,
) -> Result<ProjectSummary, CommandError> {
    super::open(&workspace, &path, allow_migration)
}

#[tauri::command(async)]
pub fn close_project(workspace: State<'_, Workspace>) -> Result<(), CommandError> {
    workspace.close()
}

#[tauri::command(async)]
pub fn current_project(
    workspace: State<'_, Workspace>,
) -> Result<Option<ProjectSummary>, CommandError> {
    super::current(&workspace)
}

#[tauri::command(async)]
pub fn rename_project(
    workspace: State<'_, Workspace>,
    name: String,
) -> Result<ProjectSummary, CommandError> {
    super::rename(&workspace, &name)
}

#[tauri::command(async)]
pub fn update_universe(
    workspace: State<'_, Workspace>,
    summary: String,
    description: String,
) -> Result<ProjectSummary, CommandError> {
    super::update_universe(&workspace, summary, description)
}

#[tauri::command(async)]
pub fn create_project_backup(workspace: State<'_, Workspace>) -> Result<String, CommandError> {
    super::create_backup(&workspace)
}

#[tauri::command(async)]
pub fn universe_counts(workspace: State<'_, Workspace>) -> Result<UniverseCounts, CommandError> {
    super::counts(&workspace)
}
