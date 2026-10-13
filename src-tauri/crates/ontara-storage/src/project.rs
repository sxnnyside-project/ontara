use std::fs;
use std::path::{Path, PathBuf};

use ontara_core::{
    AssetFile, Calendar, CalendarId, EditError, Entity, ModelEditor, ModelReader, ModelReaderExt,
    Timestamp, TypeKey, UniverseProfile, Writer, builtin_appearance_kinds, builtin_canon_states,
    builtin_diplomatic_stances, builtin_editorial_levels, builtin_entity_types,
    builtin_property_set, builtin_relationship_types, builtin_vocabulary_terms,
    gregorian_definition,
};
use rusqlite::Connection;

use crate::asset_import::{self, ImportedAsset};
use crate::backup;
use crate::error::StorageError;
use crate::integrity::{self, IntegrityReport};
use crate::layout::ProjectLayout;
use crate::manifest::ProjectManifest;
use crate::migration::{current_schema_version, latest_schema_version, migrate};
use crate::sqlite_model::SqliteModel;

/// What to do when a project was written by an older schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationPolicy {
    /// Fail with `MigrationRequired` and leave every file untouched.
    Refuse,
    /// Take a backup, then upgrade the schema.
    Apply,
}

/// An open, portable Ontara project.
///
/// Dropping a project closes its database; [`Project::close`] does the same and reports failures.
pub struct Project {
    layout: ProjectLayout,
    manifest: ProjectManifest,
    connection: Connection,
}

impl Project {
    /// Creates a project in a new or empty folder.
    ///
    /// The manifest is written last, so a folder without one is never mistaken for a project.
    pub fn create(root: &Path, name: &str, now: Timestamp) -> Result<Self, StorageError> {
        if name.trim().is_empty() {
            return Err(StorageError::InvalidManifest(
                "the project name must not be blank".into(),
            ));
        }
        let layout = ProjectLayout::new(root);
        if layout.manifest().exists() {
            return Err(StorageError::ProjectAlreadyExists(root.to_path_buf()));
        }
        let created_root = prepare_location(root)?;

        Self::initialize(layout, name, now).inspect_err(|_| discard(root, created_root))
    }

    /// Opens an existing project.
    ///
    /// Validates the format, the schema version and the database's integrity. Nothing is rewritten
    /// unless `policy` allows a migration, and then a backup is taken first.
    pub fn open(
        root: &Path,
        policy: MigrationPolicy,
        now: Timestamp,
    ) -> Result<Self, StorageError> {
        let layout = ProjectLayout::new(root);
        if !layout.manifest().is_file() {
            return Err(StorageError::NotAProject(root.to_path_buf()));
        }
        let manifest = ProjectManifest::read(&layout.manifest())?;
        if !layout.database().is_file() {
            return Err(StorageError::MissingDatabase(layout.database()));
        }

        let mut connection = open_connection(&layout.database())
            .map_err(|error| StorageError::IntegrityCheckFailed(error.to_string()))?;
        let report = integrity::check(&connection, false)
            .map_err(|error| StorageError::IntegrityCheckFailed(error.to_string()))?;
        if !report.is_sound() {
            return Err(StorageError::IntegrityCheckFailed(
                report.problems.join("; "),
            ));
        }

        let found = current_schema_version(&connection)?;
        let required = latest_schema_version();
        if found > required {
            return Err(StorageError::UnsupportedSchema {
                found,
                supported: required,
            });
        }
        if found < required {
            if policy == MigrationPolicy::Refuse {
                return Err(StorageError::MigrationRequired { found, required });
            }
            backup::create(
                &connection,
                &layout,
                now,
                &format!("pre-migration-v{found}"),
            )?;
            migrate(&mut connection)?;
        }

        Ok(Self {
            layout,
            manifest,
            connection,
        })
    }

    #[must_use]
    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        self.layout.root()
    }

    #[must_use]
    pub fn layout(&self) -> &ProjectLayout {
        &self.layout
    }

    /// Changes the project's display name. The folder is not renamed.
    pub fn rename(&mut self, name: &str) -> Result<(), StorageError> {
        if name.trim().is_empty() {
            return Err(StorageError::InvalidManifest(
                "the project name must not be blank".into(),
            ));
        }
        let mut renamed = self.manifest.clone();
        name.clone_into(&mut renamed.name);
        renamed.write(&self.layout.manifest())?;
        self.manifest = renamed;
        Ok(())
    }

    /// Read-only access to the project's records.
    #[must_use]
    pub fn reader(&self) -> impl ModelReader<Error = StorageError> + '_ {
        SqliteModel::new(&self.connection)
    }

    /// Runs a domain edit in one transaction: it commits only if the closure succeeds, so a
    /// failed multi-record edit leaves nothing behind.
    pub fn edit<T>(
        &mut self,
        operation: impl FnOnce(
            &mut ModelEditor<'_, SqliteModel<'_>>,
        ) -> Result<T, EditError<StorageError>>,
    ) -> Result<T, EditError<StorageError>> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| EditError::Repository(StorageError::from(error)))?;

        let outcome = {
            let mut model = SqliteModel::new(&transaction);
            let outcome = {
                let mut editor = ModelEditor::new(&mut model);
                operation(&mut editor)
            };
            if outcome.is_ok() {
                model.prune_instants().map_err(EditError::Repository)?;
            }
            outcome
        };

        match outcome {
            Ok(value) => {
                transaction
                    .commit()
                    .map_err(|error| EditError::Repository(StorageError::from(error)))?;
                Ok(value)
            }
            Err(error) => Err(error), // Dropping the transaction rolls it back.
        }
    }

    /// Brings a file into the project: copies it under `assets/`, and records an entity of
    /// `asset_type` that stands for it, so it can be attached to anything in the universe.
    ///
    /// The file is never moved or linked: the project stays portable. A file whose content is
    /// already in the project is not copied again; the existing asset is returned. `name` is how
    /// the asset is called in the universe, by default the file's name.
    pub fn import_asset(
        &mut self,
        source: &Path,
        asset_type: &TypeKey,
        name: Option<&str>,
        now: Timestamp,
    ) -> Result<ImportedAsset, EditError<StorageError>> {
        let repository = EditError::Repository;
        let metadata =
            fs::metadata(source).map_err(|error| repository(StorageError::io(source, error)))?;
        if !metadata.is_file() {
            return Err(repository(StorageError::io(
                source,
                std::io::Error::other("only files can be imported"),
            )));
        }
        let hash = asset_import::content_hash(source).map_err(repository)?;

        if let Some(existing) = self.asset_with_hash(&hash).map_err(repository)? {
            return Ok(existing);
        }

        let (relative_path, destination) =
            asset_import::copy_into(source, &self.layout.assets_directory(), self.layout.root())
                .map_err(repository)?;

        let file_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("asset");
        let display_name = name.map_or_else(
            || {
                destination
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("asset")
                    .to_owned()
            },
            str::to_owned,
        );
        let media_type = asset_import::media_type_of(file_name).map(str::to_owned);
        let size_bytes = metadata.len();

        let recorded = self.edit(|editor| {
            let entity = editor.add_entity(Entity::new(asset_type.clone(), display_name, now))?;
            let file = editor.register_asset_file(AssetFile {
                entity: entity.id,
                relative_path: relative_path.clone(),
                media_type,
                size_bytes,
                content_hash: Some(hash),
                imported_at: now,
            })?;
            Ok(ImportedAsset {
                entity,
                file,
                already_imported: false,
            })
        });
        if recorded.is_err() {
            let _ = fs::remove_file(&destination);
        }
        recorded
    }

    fn asset_with_hash(&self, hash: &str) -> Result<Option<ImportedAsset>, StorageError> {
        let reader = SqliteModel::new(&self.connection);
        let Some(file) = reader
            .asset_files()?
            .into_iter()
            .find(|file| file.content_hash.as_deref() == Some(hash))
        else {
            return Ok(None);
        };
        Ok(reader.entity(&file.entity)?.map(|entity| ImportedAsset {
            entity,
            file,
            already_imported: true,
        }))
    }

    /// Checks the whole database, not only the quick check done when opening.
    pub fn check_integrity(&self) -> Result<IntegrityReport, StorageError> {
        integrity::check(&self.connection, true)
    }

    /// Asset files registered in the project whose file is not on disk.
    ///
    /// A missing file never prevents the rest of the project from opening; it is reported here.
    pub fn missing_assets(&self) -> Result<Vec<ontara_core::AssetFile>, StorageError> {
        let files = ontara_core::Reader::<ontara_core::AssetFile>::list(&SqliteModel::new(
            &self.connection,
        ))?;
        Ok(files
            .into_iter()
            .filter(|file| !self.layout.root().join(&file.relative_path).is_file())
            .collect())
    }

    /// Writes a verified backup into `backups/` and returns its path.
    pub fn create_backup(&self, now: Timestamp) -> Result<PathBuf, StorageError> {
        backup::create(&self.connection, &self.layout, now, "backup")
    }

    /// Closes the project, surfacing any failure to flush the database.
    pub fn close(self) -> Result<(), StorageError> {
        self.connection
            .close()
            .map_err(|(_, error)| StorageError::from(error))
    }

    fn initialize(layout: ProjectLayout, name: &str, now: Timestamp) -> Result<Self, StorageError> {
        for directory in [
            layout.data_directory(),
            layout.assets_directory(),
            layout.exports_directory(),
            layout.backups_directory(),
        ] {
            fs::create_dir_all(&directory)
                .map_err(|source| StorageError::io(&directory, source))?;
        }

        let mut connection = open_connection(&layout.database())?;
        migrate(&mut connection)?;
        {
            let transaction = connection.transaction()?;
            seed(&transaction, now)?;
            transaction.commit()?;
        }

        let manifest = ProjectManifest::new(name, now);
        manifest.write(&layout.manifest())?;
        Ok(Self {
            layout,
            manifest,
            connection,
        })
    }
}

/// Fills a new project with the catalogs every project starts from.
fn seed(connection: &Connection, now: Timestamp) -> Result<(), StorageError> {
    let mut model = SqliteModel::new(connection);
    for state in builtin_canon_states() {
        model.insert(&state)?;
    }
    for entity_type in builtin_entity_types() {
        model.insert(&entity_type)?;
    }
    for relationship_type in builtin_relationship_types() {
        model.insert(&relationship_type)?;
    }
    let properties = builtin_property_set();
    for group in &properties.groups {
        model.insert(group)?;
    }
    for definition in &properties.definitions {
        model.insert(definition)?;
    }
    for kind in builtin_appearance_kinds() {
        model.insert(&kind)?;
    }
    for term in builtin_vocabulary_terms() {
        model.insert(&term)?;
    }
    for stance in builtin_diplomatic_stances() {
        model.insert(&stance)?;
    }
    for level in builtin_editorial_levels() {
        model.insert(&level)?;
    }
    let mut calendar = Calendar::new("Gregorian", gregorian_definition(), now);
    calendar.id = CalendarId::builtin_default();
    calendar.is_default = true;
    model.insert(&calendar)?;
    model.insert(&UniverseProfile::default())?;
    Ok(())
}

fn open_connection(database: &Path) -> Result<Connection, StorageError> {
    let connection = Connection::open(database)?;
    connection.pragma_update(None, "foreign_keys", true)?;
    // Write-ahead logging keeps readers from blocking the writer; the log is checkpointed and
    // removed when the last connection closes, so a closed project is a single database file.
    connection.pragma_update(None, "journal_mode", "WAL")?;
    Ok(connection)
}

/// Ensures `root` is a usable empty location. Returns whether this call created it.
fn prepare_location(root: &Path) -> Result<bool, StorageError> {
    if root.exists() {
        let mut entries = fs::read_dir(root).map_err(|source| StorageError::io(root, source))?;
        if !root.is_dir() || entries.next().is_some() {
            return Err(StorageError::LocationNotEmpty(root.to_path_buf()));
        }
        Ok(false)
    } else {
        fs::create_dir_all(root).map_err(|source| StorageError::io(root, source))?;
        Ok(true)
    }
}

/// Best-effort removal of a half-created project; the original error is what gets reported.
fn discard(root: &Path, created_root: bool) {
    if created_root {
        let _ = fs::remove_dir_all(root);
    } else if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let _ = if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            };
        }
    }
}

/// Reads the schema version of a project without opening it for use.
pub fn schema_version_of(root: &Path) -> Result<u32, StorageError> {
    let layout = ProjectLayout::new(root);
    let connection = Connection::open_with_flags(
        layout.database(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    current_schema_version(&connection)
}
