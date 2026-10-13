use std::path::PathBuf;

use thiserror::Error;

/// A persistence operation failed. Each message says what failed and what to do next.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("`{0}` is not an Ontara project: it has no ontara.project.json")]
    NotAProject(PathBuf),

    #[error("the project database is missing at {0}; restore a backup if one exists")]
    MissingDatabase(PathBuf),

    #[error("a project already exists at {0}")]
    ProjectAlreadyExists(PathBuf),

    #[error("{0} exists and is not an empty folder; choose a new or empty location")]
    LocationNotEmpty(PathBuf),

    #[error(
        "this project uses format version {found}, but this version of Ontara only supports \
         up to {supported}; update Ontara to open it"
    )]
    UnsupportedFormat { found: u32, supported: u32 },

    #[error(
        "this project uses schema version {found}, but this version of Ontara only supports \
         up to {supported}; update Ontara to open it"
    )]
    UnsupportedSchema { found: u32, supported: u32 },

    #[error(
        "this project uses schema version {found} and must be upgraded to {required}; \
         a backup is taken before the upgrade, open it again allowing migration"
    )]
    MigrationRequired { found: u32, required: u32 },

    #[error("the project failed its integrity check: {0}; restore the latest backup")]
    IntegrityCheckFailed(String),

    #[error("the backup at {0} is not usable: {1}")]
    InvalidBackup(PathBuf, String),

    #[error("record {id} in {table} is corrupt: {reason}")]
    CorruptRecord {
        table: &'static str,
        id: String,
        reason: String,
    },

    #[error("a record could not be encoded for storage: {0}")]
    Encoding(String),

    #[error("the project manifest is invalid: {0}")]
    InvalidManifest(String),

    #[error("file system error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl StorageError {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
