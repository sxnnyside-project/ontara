use std::fs;
use std::io::Write;
use std::path::Path;

use ontara_core::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::StorageError;

/// Version of the manifest and folder layout. The database schema is versioned separately.
pub const FORMAT_VERSION: u32 = 1;

/// The `ontara.project.json` file at the root of every project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub project_id: Uuid,
    pub name: String,
    pub created_at: Timestamp,
    /// Version of Ontara that created the project.
    pub created_with: String,
}

impl ProjectManifest {
    pub(crate) fn new(name: &str, now: Timestamp) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            project_id: Uuid::now_v7(),
            name: name.to_owned(),
            created_at: now,
            created_with: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }

    pub(crate) fn read(path: &Path) -> Result<Self, StorageError> {
        let raw = fs::read_to_string(path).map_err(|source| StorageError::io(path, source))?;
        let manifest: Self = serde_json::from_str(&raw)
            .map_err(|error| StorageError::InvalidManifest(error.to_string()))?;
        if manifest.format_version > FORMAT_VERSION {
            return Err(StorageError::UnsupportedFormat {
                found: manifest.format_version,
                supported: FORMAT_VERSION,
            });
        }
        Ok(manifest)
    }

    /// Writes the manifest through a temporary file so an interruption never leaves a torn file.
    pub(crate) fn write(&self, path: &Path) -> Result<(), StorageError> {
        let temporary = path.with_extension("json.tmp");
        let mut json = serde_json::to_string_pretty(self)
            .map_err(|error| StorageError::InvalidManifest(error.to_string()))?;
        json.push('\n');

        let mut file =
            fs::File::create(&temporary).map_err(|source| StorageError::io(&temporary, source))?;
        file.write_all(json.as_bytes())
            .map_err(|source| StorageError::io(&temporary, source))?;
        file.sync_all()
            .map_err(|source| StorageError::io(&temporary, source))?;
        fs::rename(&temporary, path).map_err(|source| StorageError::io(path, source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_survives_a_write_and_read() -> Result<(), StorageError> {
        let directory = tempfile::tempdir().map_err(|e| StorageError::io("tempdir", e))?;
        let path = directory.path().join("ontara.project.json");
        let manifest = ProjectManifest::new("Aurea", Timestamp::from_unix_millis(5));

        manifest.write(&path)?;

        assert_eq!(ProjectManifest::read(&path)?, manifest);
        assert!(!path.with_extension("json.tmp").exists());
        Ok(())
    }

    #[test]
    fn a_newer_format_is_refused_with_guidance() -> Result<(), StorageError> {
        let directory = tempfile::tempdir().map_err(|e| StorageError::io("tempdir", e))?;
        let path = directory.path().join("ontara.project.json");
        let mut manifest = ProjectManifest::new("Aurea", Timestamp::from_unix_millis(5));
        manifest.format_version = FORMAT_VERSION + 1;
        manifest.write(&path)?;

        let error = ProjectManifest::read(&path);

        assert!(matches!(error, Err(StorageError::UnsupportedFormat { .. })));
        Ok(())
    }
}
