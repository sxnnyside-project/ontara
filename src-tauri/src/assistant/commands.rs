use tauri::State;

use super::{Assistant, ProviderInput, ProviderView, RequestPreview, TaskOutcome, TaskRequest};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn list_providers(assistant: State<'_, Assistant>) -> Result<Vec<ProviderView>, CommandError> {
    assistant.providers()
}

#[tauri::command(async)]
pub fn save_provider(
    assistant: State<'_, Assistant>,
    provider: ProviderInput,
) -> Result<ProviderView, CommandError> {
    assistant.save_provider(&provider)
}

#[tauri::command(async)]
pub fn remove_provider(assistant: State<'_, Assistant>, id: String) -> Result<(), CommandError> {
    assistant.remove_provider(&id)
}

#[tauri::command(async)]
pub fn test_provider(assistant: State<'_, Assistant>, id: String) -> Result<String, CommandError> {
    assistant.test_provider(&id)
}

#[tauri::command(async)]
pub fn preview_task(
    assistant: State<'_, Assistant>,
    workspace: State<'_, Workspace>,
    task: TaskRequest,
) -> Result<RequestPreview, CommandError> {
    assistant.preview(&workspace, &task)
}

#[tauri::command(async)]
pub fn run_task(
    assistant: State<'_, Assistant>,
    workspace: State<'_, Workspace>,
    task: TaskRequest,
) -> Result<TaskOutcome, CommandError> {
    assistant.run(&workspace, &task)
}

#[tauri::command(async)]
pub fn mcp_connection(
    workspace: State<'_, Workspace>,
) -> Result<crate::mcp::Connection, CommandError> {
    crate::mcp::connection(&workspace)
}
