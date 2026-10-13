//! Ontara persistence.
//!
//! Portable project folders, `SQLite` storage behind the domain's repository contracts, and
//! explicit versioned migrations. No domain rule lives here: edits go through `ontara-core`.

mod asset_import;
mod backup;
mod error;
mod integrity;
mod layout;
mod manifest;
mod migration;
mod project;
mod sqlite_model;

pub use asset_import::ImportedAsset;
pub use backup::restore as restore_backup;
pub use error::StorageError;
pub use integrity::IntegrityReport;
pub use layout::{MANIFEST_FILE_NAME, ProjectLayout};
pub use manifest::{FORMAT_VERSION, ProjectManifest};
pub use migration::latest_schema_version;
pub use project::{MigrationPolicy, Project, schema_version_of};
pub use sqlite_model::SqliteModel;
