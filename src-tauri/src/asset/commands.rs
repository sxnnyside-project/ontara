use ontara_core::{Attachment, AttachmentId, EntityId};
use tauri::State;
use tauri::ipc::Response;

use super::{AssetView, AttachmentView, ImportedAsset, NewAttachment};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn import_asset(
    workspace: State<'_, Workspace>,
    path: String,
    name: Option<String>,
) -> Result<ImportedAsset, CommandError> {
    super::import(&workspace, &path, name.as_deref())
}

#[tauri::command(async)]
pub fn list_assets(workspace: State<'_, Workspace>) -> Result<Vec<AssetView>, CommandError> {
    super::list(&workspace)
}

#[tauri::command(async)]
pub fn list_attachments(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<Vec<AttachmentView>, CommandError> {
    super::attachments_of(&workspace, &entity)
}

#[tauri::command(async)]
pub fn attach_asset(
    workspace: State<'_, Workspace>,
    attachment: NewAttachment,
) -> Result<Attachment, CommandError> {
    super::attach(&workspace, attachment)
}

#[tauri::command(async)]
pub fn detach_asset(workspace: State<'_, Workspace>, id: AttachmentId) -> Result<(), CommandError> {
    super::detach(&workspace, &id)
}

#[tauri::command(async)]
pub fn read_asset(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<Response, CommandError> {
    super::read(&workspace, &entity).map(Response::new)
}
