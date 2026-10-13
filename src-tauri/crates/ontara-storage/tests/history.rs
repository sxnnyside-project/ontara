//! Undo and redo of real edits, and the full-text index that follows them, through a project.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;

use ontara_core::{
    Entity, EntityId, ModelQueries, ModelReaderExt, Relationship, RemovalStrategy, Timestamp,
    TypeKey,
};
use ontara_storage::Project;

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);
const T1: Timestamp = Timestamp::from_unix_millis(2_000);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn project() -> Result<(tempfile::TempDir, Project), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = Project::create(&directory.path().join("Aurea.ontara"), "Aurea", T0)?;
    Ok((directory, project))
}

fn add(project: &mut Project, name: &str) -> Result<EntityId, Box<dyn Error>> {
    project.set_change_context("entity.create", T0);
    Ok(project.edit(|editor| {
        Ok(editor
            .add_entity(Entity::new(key("character"), name, T0))?
            .id)
    })?)
}

fn names(project: &Project) -> Result<Vec<String>, Box<dyn Error>> {
    let mut names: Vec<String> = project
        .reader()
        .entities()?
        .into_iter()
        .map(|entity| entity.name)
        .collect();
    names.sort();
    Ok(names)
}

#[test]
fn creating_an_entity_can_be_undone_and_redone() -> TestResult {
    let (_directory, mut project) = project()?;
    add(&mut project, "Ayla")?;

    let undone = project.undo(T1)?.expect("something to undo");
    assert_eq!(undone.label, "entity.create");
    assert!(names(&project)?.is_empty());

    project.redo(T1)?.expect("something to redo");
    assert_eq!(names(&project)?, ["Ayla"]);
    Ok(())
}

#[test]
fn removing_an_entity_with_its_relationships_comes_back_whole() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "Ayla")?;
    let bren = add(&mut project, "Bren")?;
    project.edit(|editor| {
        editor.add_relationship(Relationship::new(key("parent_of"), ayla, bren, T0))?;
        Ok(())
    })?;
    project.set_change_context("entity.remove", T1);
    project.edit(|editor| editor.remove_entity(&ayla, RemovalStrategy::RemoveDependents))?;
    assert_eq!(names(&project)?, ["Bren"]);
    assert!(project.reader().relationships()?.is_empty());

    project.undo(T1)?;

    assert_eq!(names(&project)?, ["Ayla", "Bren"]);
    assert_eq!(project.reader().relationships()?.len(), 1);
    assert!(project.check_integrity()?.is_sound());
    Ok(())
}

#[test]
fn history_survives_closing_and_opening_the_project() -> TestResult {
    let directory = tempfile::tempdir()?;
    let root = directory.path().join("Aurea.ontara");
    let mut project = Project::create(&root, "Aurea", T0)?;
    add(&mut project, "Ayla")?;
    project.close()?;

    let mut reopened = Project::open(&root, ontara_storage::MigrationPolicy::Refuse, T1)?;

    assert_eq!(reopened.history()?.undo.len(), 1);
    reopened.undo(T1)?;
    assert!(names(&reopened)?.is_empty());
    Ok(())
}

#[test]
fn a_failed_edit_leaves_no_history() -> TestResult {
    let (_directory, mut project) = project()?;

    let failed = project.edit(|editor| {
        editor.add_entity(Entity::new(key("character"), "Ayla", T0))?;
        editor.add_entity(Entity::new(key("not_a_type"), "Nobody", T0))?;
        Ok(())
    });

    assert!(failed.is_err());
    assert!(project.history()?.undo.is_empty());
    assert!(names(&project)?.is_empty());
    Ok(())
}

#[test]
fn text_search_finds_names_notes_and_what_an_edit_changed() -> TestResult {
    let (_directory, mut project) = project()?;
    let mut keeper = Entity::new(key("character"), "Ayla", T0);
    keeper.summary = "Keeper of the drowned lighthouse".to_owned();
    project.edit(|editor| {
        editor.add_entity(keeper)?;
        Ok(())
    })?;

    let found = project.reader().search_text("lighthous", false, 10)?;
    assert_eq!(found.len(), 1);

    project.undo(T1)?;
    assert!(
        project
            .reader()
            .search_text("lighthouse", false, 10)?
            .is_empty()
    );
    Ok(())
}
