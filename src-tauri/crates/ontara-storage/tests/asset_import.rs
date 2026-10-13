//! Bringing files into a project, through a real project folder.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;
use std::fs;

use ontara_core::{Timestamp, TypeKey};
use ontara_storage::Project;

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);

fn asset_type() -> TypeKey {
    TypeKey::new("asset").expect("literal test keys are valid")
}

fn project() -> Result<(tempfile::TempDir, Project), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = Project::create(&directory.path().join("Aurea.ontara"), "Aurea", T0)?;
    Ok((directory, project))
}

#[test]
fn an_imported_file_is_copied_into_the_project_and_recorded() -> TestResult {
    let (directory, mut project) = project()?;
    let source = directory.path().join("Ayla Portrait.PNG");
    fs::write(&source, b"png bytes")?;

    let imported = project.import_asset(&source, &asset_type(), None, T0)?;

    assert!(!imported.already_imported);
    assert_eq!(imported.file.relative_path, "assets/Ayla-Portrait.png");
    assert_eq!(imported.file.media_type.as_deref(), Some("image/png"));
    assert_eq!(imported.file.size_bytes, 9);
    assert_eq!(imported.entity.name, "Ayla-Portrait");
    assert_eq!(
        fs::read(project.root().join("assets/Ayla-Portrait.png"))?,
        b"png bytes"
    );
    assert!(source.exists(), "the original file is left where it was");
    assert!(project.missing_assets()?.is_empty());
    Ok(())
}

#[test]
fn importing_the_same_content_twice_returns_the_first_asset() -> TestResult {
    let (directory, mut project) = project()?;
    let first = directory.path().join("map.png");
    let copy = directory.path().join("map-copy.png");
    fs::write(&first, b"same")?;
    fs::write(&copy, b"same")?;

    let original = project.import_asset(&first, &asset_type(), Some("World map"), T0)?;
    let again = project.import_asset(&copy, &asset_type(), None, T0)?;

    assert!(again.already_imported);
    assert_eq!(again.entity.id, original.entity.id);
    assert_eq!(fs::read_dir(project.root().join("assets"))?.count(), 1);
    Ok(())
}

#[test]
fn different_files_with_the_same_name_do_not_overwrite_each_other() -> TestResult {
    let (directory, mut project) = project()?;
    let one = directory.path().join("one");
    let two = directory.path().join("two");
    fs::create_dir_all(&one)?;
    fs::create_dir_all(&two)?;
    fs::write(one.join("sketch.png"), b"first")?;
    fs::write(two.join("sketch.png"), b"second")?;

    let first = project.import_asset(&one.join("sketch.png"), &asset_type(), None, T0)?;
    let second = project.import_asset(&two.join("sketch.png"), &asset_type(), None, T0)?;

    assert_eq!(first.file.relative_path, "assets/sketch.png");
    assert_eq!(second.file.relative_path, "assets/sketch-2.png");
    Ok(())
}

#[test]
fn a_failed_import_leaves_no_file_behind() -> TestResult {
    let (directory, mut project) = project()?;
    let source = directory.path().join("note.txt");
    fs::write(&source, b"text")?;
    let not_an_asset_type = TypeKey::new("character")?;

    let failed = project.import_asset(&source, &not_an_asset_type, None, T0);

    assert!(failed.is_err());
    assert_eq!(fs::read_dir(project.root().join("assets"))?.count(), 0);
    Ok(())
}

#[test]
fn a_missing_source_is_an_error() -> TestResult {
    let (directory, mut project) = project()?;

    let failed = project.import_asset(&directory.path().join("nope.png"), &asset_type(), None, T0);

    assert!(failed.is_err());
    Ok(())
}
