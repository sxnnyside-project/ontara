use std::fs;
use std::path::{Path, PathBuf};

use ontara_core::Timestamp;
use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};

use crate::error::StorageError;
use crate::integrity;
use crate::layout::ProjectLayout;
use crate::migration::latest_schema_version;

/// A verified copy of the database in `backups/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupFile {
    pub path: PathBuf,
    pub name: String,
    /// What made it: `backup` (by the author), `auto`, `pre-migration-v1`, `pre-restore`.
    pub label: String,
    /// When it was made, milliseconds since the Unix epoch, as written in its name.
    pub created_at: i64,
    pub size_bytes: u64,
}

/// The backups in `backups/`, newest first. Partial copies and other files are not backups.
pub(crate) fn list(layout: &ProjectLayout) -> Result<Vec<BackupFile>, StorageError> {
    let directory = layout.backups_directory();
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        // A project that never took a backup has no folder for them.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(StorageError::io(&directory, source)),
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".sqlite") else {
            continue;
        };
        let Some((label, millis)) = stem.rsplit_once('-') else {
            continue;
        };
        let Ok(created_at) = millis.parse::<i64>() else {
            continue;
        };
        let size_bytes = entry.metadata().map_or(0, |metadata| metadata.len());
        found.push(BackupFile {
            label: label.to_owned(),
            created_at,
            size_bytes,
            name,
            path,
        });
    }
    found.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(found)
}

/// Deletes the oldest backups with `label`, keeping the `keep` newest. Returns how many went.
pub(crate) fn prune(
    layout: &ProjectLayout,
    label: &str,
    keep: usize,
) -> Result<usize, StorageError> {
    let mut removed = 0;
    for file in list(layout)?
        .into_iter()
        .filter(|file| file.label == label)
        .skip(keep)
    {
        fs::remove_file(&file.path).map_err(|source| StorageError::io(&file.path, source))?;
        removed += 1;
    }
    Ok(removed)
}

/// Writes a verified copy of the project database into `backups/` and returns its path.
///
/// The copy is written under a temporary name, checked, and only then given its final name, so a
/// file in `backups/` is always one that passed verification.
pub(crate) fn create(
    connection: &Connection,
    layout: &ProjectLayout,
    now: Timestamp,
    label: &str,
) -> Result<PathBuf, StorageError> {
    let directory = layout.backups_directory();
    fs::create_dir_all(&directory).map_err(|source| StorageError::io(&directory, source))?;

    let name = format!("{label}-{}.sqlite", now.as_unix_millis());
    let destination = directory.join(&name);
    let partial = directory.join(format!("{name}.partial"));

    copy_database(connection, &partial)?;
    if let Err(error) = verify(&partial) {
        let _ = fs::remove_file(&partial); // The unverified copy is useless; the error is what matters.
        return Err(error);
    }
    fs::rename(&partial, &destination).map_err(|source| StorageError::io(&destination, source))?;
    Ok(destination)
}

/// Replaces the project database with a verified backup.
///
/// The project must be closed. The current database is preserved in `backups/` first, so a
/// restore can itself be reversed.
pub fn restore(root: &Path, backup: &Path, now: Timestamp) -> Result<(), StorageError> {
    let layout = ProjectLayout::new(root);
    if !layout.manifest().is_file() {
        return Err(StorageError::NotAProject(root.to_path_buf()));
    }
    let backup_version = verify(backup)
        .map_err(|error| StorageError::InvalidBackup(backup.to_path_buf(), error.to_string()))?;
    if backup_version > latest_schema_version() {
        return Err(StorageError::UnsupportedSchema {
            found: backup_version,
            supported: latest_schema_version(),
        });
    }

    let database = layout.database();
    preserve_current(&layout, &database, now)?;

    let staged = database.with_extension("sqlite.restore");
    fs::copy(backup, &staged).map_err(|source| StorageError::io(&staged, source))?;
    for sidecar in ["sqlite-wal", "sqlite-shm"] {
        let path = database.with_extension(sidecar);
        if path.exists() {
            fs::remove_file(&path).map_err(|source| StorageError::io(&path, source))?;
        }
    }
    fs::rename(&staged, &database).map_err(|source| StorageError::io(&database, source))
}

fn preserve_current(
    layout: &ProjectLayout,
    database: &Path,
    now: Timestamp,
) -> Result<(), StorageError> {
    if !database.exists() {
        return Ok(());
    }
    let directory = layout.backups_directory();
    fs::create_dir_all(&directory).map_err(|source| StorageError::io(&directory, source))?;
    let millis = now.as_unix_millis();

    let readable = Connection::open_with_flags(database, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(StorageError::from)
        .and_then(|connection| {
            copy_database(
                &connection,
                &directory.join(format!("pre-restore-{millis}.sqlite")),
            )
        });
    if readable.is_err() {
        // Unreadable (for example corrupt): keep the raw bytes for forensics instead of losing them.
        let raw = directory.join(format!("pre-restore-{millis}.damaged"));
        fs::copy(database, &raw).map_err(|source| StorageError::io(&raw, source))?;
    }
    Ok(())
}

fn copy_database(source: &Connection, destination: &Path) -> Result<(), StorageError> {
    let mut target = Connection::open(destination)?;
    let backup = Backup::new(source, &mut target)?;
    backup.run_to_completion(256, std::time::Duration::ZERO, None)?;
    Ok(())
}

/// Opens a database file read-only, checks it and returns its schema version.
fn verify(path: &Path) -> Result<u32, StorageError> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let report = integrity::check(&connection, true)?;
    if !report.is_sound() {
        return Err(StorageError::IntegrityCheckFailed(
            report.problems.join("; "),
        ));
    }
    crate::migration::current_schema_version(&connection)
}
