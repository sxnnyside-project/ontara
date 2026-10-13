use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{AttachmentId, EntityId};
use crate::record::Record;
use crate::timestamp::Timestamp;

/// The file behind an entity of an asset type.
///
/// Only a project-relative path is stored, so the project stays movable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFile {
    pub entity: EntityId,
    /// Forward-slash path relative to the project root, inside `assets/`.
    pub relative_path: String,
    pub media_type: Option<String>,
    pub size_bytes: u64,
    pub content_hash: Option<String>,
    pub imported_at: Timestamp,
}

impl AssetFile {
    pub fn validate(&self) -> Result<(), DomainError> {
        let path = self.relative_path.as_str();
        let escapes = path
            .split('/')
            .any(|part| part == ".." || part == "." || part.is_empty());
        let absolute = path.starts_with('/') || path.contains('\\') || path.contains(':');
        if path.is_empty() || escapes || absolute || !path.starts_with("assets/") {
            return Err(DomainError::InvalidPath(self.relative_path.clone()));
        }
        require_optional_text(self.media_type.as_deref(), "media type")?;
        require_optional_text(self.content_hash.as_deref(), "content hash")
    }
}

impl Record for AssetFile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}

/// The association of an asset with an entity. One asset can serve many entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub owner: EntityId,
    pub asset: EntityId,
    /// Why the asset is attached: "portrait", "map", "reference".
    pub purpose: Option<String>,
    pub position: i32,
    pub note: Option<String>,
    pub created_at: Timestamp,
}

impl Attachment {
    #[must_use]
    pub fn new(owner: EntityId, asset: EntityId, now: Timestamp) -> Self {
        Self {
            id: AttachmentId::generate(),
            owner,
            asset,
            purpose: None,
            position: 0,
            note: None,
            created_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.purpose.as_deref(), "purpose")?;
        require_optional_text(self.note.as_deref(), "note")
    }
}

impl Record for Attachment {
    type Id = AttachmentId;

    fn id(&self) -> AttachmentId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(path: &str) -> AssetFile {
        AssetFile {
            entity: EntityId::generate(),
            relative_path: path.to_owned(),
            media_type: None,
            size_bytes: 0,
            content_hash: None,
            imported_at: Timestamp::from_unix_millis(0),
        }
    }

    #[test]
    fn only_paths_inside_assets_are_accepted() {
        assert!(asset("assets/characters/ayla.png").validate().is_ok());
        for bad in [
            "",
            "/etc/passwd",
            "../outside.png",
            "assets/../outside.png",
            "C:\\x.png",
            "notes/a.png",
            "assets//a.png",
        ] {
            assert!(asset(bad).validate().is_err(), "{bad:?} should be rejected");
        }
    }
}
