use std::path::{Path, PathBuf};

pub const MANIFEST_FILE_NAME: &str = "ontara.project.json";
const DATA_DIRECTORY: &str = "data";
const DATABASE_FILE_NAME: &str = "ontara.sqlite";
const ASSETS_DIRECTORY: &str = "assets";
const EXPORTS_DIRECTORY: &str = "exports";
const BACKUPS_DIRECTORY: &str = "backups";

/// Where each part of a portable project lives, relative to its root folder.
///
/// Only relative layout is stored anywhere; the root is whatever folder the user opened.
#[derive(Debug, Clone)]
pub struct ProjectLayout {
    root: PathBuf,
}

impl ProjectLayout {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn manifest(&self) -> PathBuf {
        self.root.join(MANIFEST_FILE_NAME)
    }

    #[must_use]
    pub fn data_directory(&self) -> PathBuf {
        self.root.join(DATA_DIRECTORY)
    }

    #[must_use]
    pub fn database(&self) -> PathBuf {
        self.data_directory().join(DATABASE_FILE_NAME)
    }

    #[must_use]
    pub fn assets_directory(&self) -> PathBuf {
        self.root.join(ASSETS_DIRECTORY)
    }

    #[must_use]
    pub fn exports_directory(&self) -> PathBuf {
        self.root.join(EXPORTS_DIRECTORY)
    }

    #[must_use]
    pub fn backups_directory(&self) -> PathBuf {
        self.root.join(BACKUPS_DIRECTORY)
    }
}
