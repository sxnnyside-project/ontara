use ontara_core::ProposalId;
use tauri::State;

use super::{PolicyView, ProposalView};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn list_proposals(workspace: State<'_, Workspace>) -> Result<Vec<ProposalView>, CommandError> {
    super::list(&workspace)
}

#[tauri::command(async)]
pub fn decide_proposal(
    workspace: State<'_, Workspace>,
    id: ProposalId,
    approve: Vec<u32>,
    reject: Vec<u32>,
) -> Result<ProposalView, CommandError> {
    super::decide(&workspace, id, &approve, &reject)
}

#[tauri::command(async)]
pub fn discard_proposal(
    workspace: State<'_, Workspace>,
    id: ProposalId,
) -> Result<(), CommandError> {
    super::discard(&workspace, id)
}

#[tauri::command(async)]
pub fn get_ai_policy(workspace: State<'_, Workspace>) -> Result<PolicyView, CommandError> {
    super::policy(&workspace)
}

#[tauri::command(async)]
pub fn set_ai_policy(
    workspace: State<'_, Workspace>,
    cloud_allowed: bool,
    mcp_allowed: bool,
) -> Result<PolicyView, CommandError> {
    super::set_policy(&workspace, cloud_allowed, mcp_allowed)
}
