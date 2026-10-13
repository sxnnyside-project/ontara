//! Phase A exit condition: a project can be created, edited, closed, reopened, moved and
//! recovered without breaking its domain relationships.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use ontara_core::{
    Entity, EntityId, ModelReaderExt, Relationship, RemovalStrategy, Timestamp, TypeKey,
};
use ontara_storage::{
    MigrationPolicy, Project, StorageError, latest_schema_version, restore_backup,
    schema_version_of,
};
use rusqlite::Connection;

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);
const T1: Timestamp = Timestamp::from_unix_millis(2_000);
const T2: Timestamp = Timestamp::from_unix_millis(3_000);

struct Cast {
    ayla: EntityId,
    bren: EntityId,
    citadel: EntityId,
}

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn new_project(sandbox: &Path) -> Result<(Project, PathBuf), Box<dyn Error>> {
    let root = sandbox.join("Aurea.ontara");
    Ok((Project::create(&root, "Aurea", T0)?, root))
}

/// Two characters and a location; Ayla is Bren's parent and lives in the Citadel.
fn populate(project: &mut Project) -> Result<Cast, Box<dyn Error>> {
    let cast = project.edit(|editor| {
        let ayla = editor.add_entity(Entity::new(key("character"), "Ayla", T0))?;
        let bren = editor.add_entity(Entity::new(key("character"), "Bren", T0))?;
        let citadel = editor.add_entity(Entity::new(key("stronghold"), "The Citadel", T0))?;
        editor.add_relationship(Relationship::new(key("parent_of"), ayla.id, bren.id, T0))?;
        editor.add_relationship(Relationship::new(
            key("located_within"),
            ayla.id,
            citadel.id,
            T0,
        ))?;
        Ok(Cast {
            ayla: ayla.id,
            bren: bren.id,
            citadel: citadel.id,
        })
    })?;
    Ok(cast)
}

fn open(root: &Path) -> Result<Project, StorageError> {
    Project::open(root, MigrationPolicy::Refuse, T1)
}

#[test]
fn a_new_project_starts_with_every_catalog_seeded() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (project, _) = new_project(sandbox.path())?;
    let reader = project.reader();

    let states: Vec<_> = reader
        .canon_states()?
        .into_iter()
        .map(|s| s.key.to_string())
        .collect();
    for expected in [
        "established",
        "provisional",
        "disputed",
        "deprecated",
        "rejected",
        "disestablished",
    ] {
        assert!(
            states.iter().any(|state| state == expected),
            "missing canon state {expected}"
        );
    }
    assert!(reader.entity_types()?.len() >= 30);
    assert!(reader.entity_types()?.iter().any(|t| t.parent.is_some()));
    assert!(reader.relationship_types()?.len() >= 8);
    assert!(reader.vocabulary_terms()?.len() >= 40);
    assert!(reader.diplomatic_stances()?.len() >= 8);
    assert!(
        reader
            .appearance_kinds()?
            .iter()
            .any(|kind| !kind.is_presence)
    );
    assert_eq!(reader.editorial_levels()?.len(), 5);
    assert!(
        reader
            .calendars()?
            .iter()
            .any(|calendar| calendar.is_default)
    );
    assert!(reader.property_definitions()?.len() >= 80);
    Ok(())
}

#[test]
fn a_project_survives_close_move_and_reopen_with_its_relationships_intact() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (mut project, original) = new_project(sandbox.path())?;
    let cast = populate(&mut project)?;
    project.close()?;

    let destination = sandbox.path().join("elsewhere");
    fs::create_dir(&destination)?;
    let moved = destination.join("Renamed Folder.ontara");
    fs::rename(&original, &moved)?;

    let reopened = open(&moved)?;
    let reader = reopened.reader();
    assert_eq!(reopened.manifest().name, "Aurea");
    assert_eq!(reader.entities()?.len(), 3);
    assert_eq!(
        ontara_core::ModelQueries::relationships_of(&reader, &cast.ayla)?.len(),
        2
    );
    assert_eq!(
        ontara_core::ModelQueries::relationships_of(&reader, &cast.bren)?.len(),
        1
    );
    assert_eq!(
        ontara_core::ModelQueries::relationships_of(&reader, &cast.citadel)?.len(),
        1
    );
    assert!(reopened.check_integrity()?.is_sound());
    Ok(())
}

#[test]
fn a_closed_project_keeps_its_database_in_a_single_file() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (mut project, root) = new_project(sandbox.path())?;
    populate(&mut project)?;
    project.close()?;

    let mut names: Vec<String> = fs::read_dir(root.join("data"))?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, _>>()?;
    names.sort();

    assert_eq!(names, ["ontara.sqlite"]);
    Ok(())
}

#[test]
fn renaming_an_entity_keeps_every_relationship() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (mut project, _) = new_project(sandbox.path())?;
    let cast = populate(&mut project)?;

    let mut ayla = project
        .reader()
        .entity(&cast.ayla)?
        .ok_or("Ayla must exist")?;
    ayla.name = "Ayla Prime".to_owned();
    project.edit(|editor| {
        editor.update_entity(ayla, T1)?;
        Ok(())
    })?;

    let reader = project.reader();
    let ayla = reader.entity(&cast.ayla)?.ok_or("Ayla must still exist")?;
    assert_eq!(ayla.name, "Ayla Prime");
    assert_eq!(ayla.created_at, T0);
    assert_eq!(ayla.updated_at, T1);
    assert_eq!(
        ontara_core::ModelQueries::relationships_of(&reader, &cast.ayla)?.len(),
        2
    );
    Ok(())
}

#[test]
fn a_failed_multi_record_edit_leaves_nothing_behind() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (mut project, _) = new_project(sandbox.path())?;

    let outcome = project.edit(|editor| {
        let ayla = editor.add_entity(Entity::new(key("character"), "Ayla", T0))?;
        let ghost = EntityId::generate();
        editor.add_relationship(Relationship::new(key("mentor_of"), ayla.id, ghost, T0))?;
        Ok(())
    });

    assert!(outcome.is_err());
    assert!(project.reader().entities()?.is_empty());
    Ok(())
}

#[test]
fn a_backup_can_be_restored_and_the_replaced_state_is_kept() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (mut project, root) = new_project(sandbox.path())?;
    let cast = populate(&mut project)?;
    let backup = project.create_backup(T1)?;
    project.edit(|editor| {
        editor.remove_entity(&cast.ayla, RemovalStrategy::RemoveDependents)?;
        Ok(())
    })?;
    assert_eq!(project.reader().entities()?.len(), 2);
    project.close()?;

    restore_backup(&root, &backup, T2)?;

    let restored = open(&root)?;
    let reader = restored.reader();
    assert_eq!(reader.entities()?.len(), 3);
    assert_eq!(
        ontara_core::ModelQueries::relationships_of(&reader, &cast.ayla)?.len(),
        2
    );
    let preserved = fs::read_dir(root.join("backups"))?
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("pre-restore-")
        });
    assert!(
        preserved,
        "the state replaced by the restore must be kept in backups/"
    );
    Ok(())
}

#[test]
fn a_schema_newer_than_this_build_is_refused_without_rewriting_anything() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (project, root) = new_project(sandbox.path())?;
    project.close()?;
    let newer = latest_schema_version() + 1;
    Connection::open(root.join("data/ontara.sqlite"))?.pragma_update(
        None,
        "user_version",
        newer,
    )?;

    let error = open(&root).err();

    assert!(matches!(error, Some(StorageError::UnsupportedSchema { found, .. }) if found == newer));
    assert_eq!(schema_version_of(&root)?, newer);
    Ok(())
}

#[test]
fn an_older_schema_requires_explicit_migration_and_is_left_untouched() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (project, root) = new_project(sandbox.path())?;
    project.close()?;
    Connection::open(root.join("data/ontara.sqlite"))?.pragma_update(None, "user_version", 0)?;

    let error = open(&root).err();

    assert!(matches!(
        error,
        Some(StorageError::MigrationRequired { found: 0, .. })
    ));
    assert_eq!(schema_version_of(&root)?, 0);
    assert!(
        fs::read_dir(root.join("backups"))?.next().is_none(),
        "refusing must not write a backup either"
    );
    Ok(())
}

#[test]
fn a_manifest_from_a_newer_format_is_refused() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (project, root) = new_project(sandbox.path())?;
    project.close()?;
    let manifest_path = root.join("ontara.project.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    manifest["format_version"] = serde_json::json!(99);
    fs::write(&manifest_path, serde_json::to_string(&manifest)?)?;

    let error = open(&root).err();

    assert!(matches!(
        error,
        Some(StorageError::UnsupportedFormat { found: 99, .. })
    ));
    Ok(())
}

#[test]
fn a_damaged_database_is_reported_with_recovery_guidance() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let (project, root) = new_project(sandbox.path())?;
    project.close()?;
    fs::write(root.join("data/ontara.sqlite"), vec![0xAB; 4096])?;

    let error = open(&root).err();

    assert!(matches!(error, Some(StorageError::IntegrityCheckFailed(_))));
    Ok(())
}

#[test]
fn creating_a_project_refuses_a_location_that_already_has_content() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    fs::write(sandbox.path().join("notes.txt"), "keep me")?;

    let error = Project::create(sandbox.path(), "Aurea", T0).err();

    assert!(matches!(error, Some(StorageError::LocationNotEmpty(_))));
    assert!(sandbox.path().join("notes.txt").exists());
    Ok(())
}

#[test]
fn a_folder_without_a_manifest_is_not_a_project() -> TestResult {
    let sandbox = tempfile::tempdir()?;

    let error = open(sandbox.path()).err();

    assert!(matches!(error, Some(StorageError::NotAProject(_))));
    Ok(())
}
