//! Files that belong to the project, and their association with the universe.

use std::fs;

use ontara_core::{
    AssetFile, Attachment, AttachmentId, EntityId, ModelQueries, ModelReaderExt, Reader, Timestamp,
    TypeKey,
};
use serde::{Deserialize, Serialize};

use crate::entity::{EntityRef, found, reference, resolve, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// Largest file the interface previews. Anything bigger is still imported and attached.
pub const MAX_PREVIEW_BYTES: u64 = 25 * 1024 * 1024;

/// A file registered in the project, with whether it is still on disk.
///
/// Mirrored by `src/features/asset/api/asset-api.ts`.
#[derive(Debug, Serialize)]
pub struct AssetView {
    pub entity: EntityRef,
    pub file: AssetFile,
    pub missing: bool,
    /// What the asset is attached to.
    pub attached_to: Vec<EntityRef>,
}

/// What an import did.
#[derive(Debug, Serialize)]
pub struct ImportedAsset {
    pub entity: EntityRef,
    pub file: AssetFile,
    /// The same content was already in the project, so nothing was copied.
    pub already_imported: bool,
}

/// An asset attached to an entity.
#[derive(Debug, Serialize)]
pub struct AttachmentView {
    pub attachment: Attachment,
    pub asset: EntityRef,
    pub file: Option<AssetFile>,
    pub missing: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewAttachment {
    pub owner: EntityId,
    pub asset: EntityId,
    pub purpose: Option<String>,
    pub note: Option<String>,
}

pub fn import(
    workspace: &Workspace,
    path: &str,
    name: Option<&str>,
) -> Result<ImportedAsset, CommandError> {
    workspace.write(|project| {
        let imported = project.import_asset(
            std::path::Path::new(path),
            &TypeKey::new("asset")?,
            name.filter(|name| !name.trim().is_empty()),
            Timestamp::now(),
        )?;
        let labels = type_labels(&project.reader())?;
        Ok(ImportedAsset {
            entity: reference(&imported.entity, &labels),
            file: imported.file,
            already_imported: imported.already_imported,
        })
    })
}

pub fn list(workspace: &Workspace) -> Result<Vec<AssetView>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let attachments = reader.attachments()?;
        let mut views = Vec::new();
        for file in reader.asset_files()? {
            let entity = resolve(&reader, &labels, &file.entity)?;
            let attached_to = attachments
                .iter()
                .filter(|attachment| attachment.asset == file.entity)
                .map(|attachment| resolve(&reader, &labels, &attachment.owner))
                .collect::<Result<Vec<_>, _>>()?;
            let missing = !project.root().join(&file.relative_path).is_file();
            views.push(AssetView {
                entity,
                file,
                missing,
                attached_to,
            });
        }
        views.sort_by_key(|view| view.entity.name.to_lowercase());
        Ok(views)
    })
}

pub fn attachments_of(
    workspace: &Workspace,
    entity: &EntityId,
) -> Result<Vec<AttachmentView>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        reader
            .attachments_of(entity)?
            .into_iter()
            .filter(|attachment| attachment.owner == *entity)
            .map(|attachment| {
                let file = Reader::<AssetFile>::get(&reader, &attachment.asset)?;
                let missing = file
                    .as_ref()
                    .is_none_or(|file| !project.root().join(&file.relative_path).is_file());
                Ok(AttachmentView {
                    asset: resolve(&reader, &labels, &attachment.asset)?,
                    attachment,
                    file,
                    missing,
                })
            })
            .collect()
    })
}

pub fn attach(workspace: &Workspace, input: NewAttachment) -> Result<Attachment, CommandError> {
    workspace.write(|project| {
        let mut attachment = Attachment::new(input.owner, input.asset, Timestamp::now());
        attachment.purpose = input.purpose;
        attachment.note = input.note;
        Ok(project.edit(|editor| editor.attach_asset(attachment))?)
    })
}

pub fn detach(workspace: &Workspace, id: &AttachmentId) -> Result<(), CommandError> {
    workspace.write(|project| Ok(project.edit(|editor| editor.detach_asset(id))?))
}

/// The bytes of an asset, for previewing it.
pub fn read(workspace: &Workspace, entity: &EntityId) -> Result<Vec<u8>, CommandError> {
    workspace.read(|project| {
        let file = found(
            Reader::<AssetFile>::get(&project.reader(), entity)?,
            "asset",
            entity,
        )?;
        let path = project.root().join(&file.relative_path);
        let size = fs::metadata(&path)
            .map_err(|error| CommandError::new("asset_missing", error.to_string()))?
            .len();
        if size > MAX_PREVIEW_BYTES {
            return Err(CommandError::new(
                "asset_too_large",
                format!("{} is too large to preview", file.relative_path),
            ));
        }
        fs::read(&path).map_err(|error| CommandError::new("asset_missing", error.to_string()))
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn an_imported_file_is_attached_listed_and_readable() {
        let (directory, workspace) = workspace_with_project();
        let source = directory.path().join("ayla.png");
        std::fs::write(&source, b"bytes").expect("writes the file");
        let character = create(
            &workspace,
            NewEntity {
                entity_type: TypeKey::new("character").expect("a valid key"),
                name: "Ayla".to_owned(),
                summary: String::new(),
                description: String::new(),
                tags: std::collections::BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates");

        let imported =
            import(&workspace, &source.to_string_lossy(), Some("Ayla portrait")).expect("imports");
        assert!(!imported.already_imported);
        assert_eq!(imported.entity.name, "Ayla portrait");

        attach(
            &workspace,
            NewAttachment {
                owner: character.id,
                asset: imported.entity.id,
                purpose: Some("portrait".to_owned()),
                note: None,
            },
        )
        .expect("attaches");

        let attached = attachments_of(&workspace, &character.id).expect("lists");
        assert_eq!(attached.len(), 1);
        assert_eq!(attached[0].asset.name, "Ayla portrait");
        assert!(!attached[0].missing);

        let assets = list(&workspace).expect("lists assets");
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].attached_to[0].name, "Ayla");
        assert_eq!(read(&workspace, &imported.entity.id), Ok(b"bytes".to_vec()));
    }

    #[test]
    fn a_deleted_file_is_reported_missing() {
        let (directory, workspace) = workspace_with_project();
        let source = directory.path().join("map.png");
        std::fs::write(&source, b"bytes").expect("writes the file");
        let imported = import(&workspace, &source.to_string_lossy(), None).expect("imports");

        workspace
            .read(|project| {
                std::fs::remove_file(project.root().join(&imported.file.relative_path))
                    .expect("removes the copy");
                Ok(())
            })
            .expect("reads");

        assert!(list(&workspace).expect("lists")[0].missing);
        assert_eq!(
            read(&workspace, &imported.entity.id).map_err(|error| error.code),
            Err("asset_missing")
        );
    }
}
