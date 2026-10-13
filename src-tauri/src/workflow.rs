//! The primary workflow, end to end through the adapter: the path an author takes from an empty
//! folder to a checked, backed-up and exported universe, and back out of a mistake. If any step
//! of it breaks, the application has stopped being usable for its main purpose.

#![cfg(test)]
#![allow(clippy::expect_used)] // Fixtures fail the test loudly.

use std::collections::BTreeSet;

use ontara_core::{
    BeingProfile, EntityFilter, EntityId, EventDetail, EventParticipant, Instant, Lifecycle,
    Temporal, Timestamp, TypeKey,
};

use crate::editorial::{NewSection, add as add_section};
use crate::entity::{NewEntity, create, list, remove};
use crate::export::{ExportFormat, ExportWords, run as export};
use crate::matrix::{AppearanceInput, save as save_appearance};
use crate::relationship::{RelationshipInput, save as save_relationship};
use crate::timeline::save_event;
use crate::workspace::Workspace;
use crate::{backup, consistency, history, project, search};

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("a valid key")
}

fn new_entity(workspace: &Workspace, entity_type: &str, name: &str, summary: &str) -> EntityId {
    create(
        workspace,
        NewEntity {
            entity_type: key(entity_type),
            name: name.to_owned(),
            summary: summary.to_owned(),
            description: String::new(),
            tags: BTreeSet::new(),
            canon_state: None,
        },
    )
    .expect("creates")
    .id
}

fn section(workspace: &Workspace, level: &str, name: &str, parent: Option<EntityId>) -> EntityId {
    add_section(
        workspace,
        NewSection {
            entity_type: key(level),
            level: key(level),
            name: name.to_owned(),
            parent,
        },
    )
    .expect("adds a section")
    .id
}

fn count(workspace: &Workspace) -> usize {
    list(workspace, &EntityFilter::default())
        .expect("lists")
        .len()
}

/// Entities, a relationship, a work with a scene, who appears in it, and an event that happens
/// before one of its participants is born. Returns the contradicting being and the place.
fn write_the_universe(workspace: &Workspace) -> (EntityId, EntityId) {
    let ayla = new_entity(workspace, "character", "Ayla", "Keeper of the lighthouse");
    let bren = new_entity(workspace, "character", "Bren", "Her son");
    let harbor = new_entity(workspace, "city", "Harbor", "Where the ferry lands");

    save_relationship(
        workspace,
        RelationshipInput {
            id: None,
            relationship_type: key("parent_of"),
            source: ayla,
            target: bren,
            qualifier: None,
            context: None,
            lifecycle: Lifecycle::Active,
            period: Temporal::Unconstrained,
            note: None,
        },
    )
    .expect("relates");

    let book = section(workspace, "volume", "Embers", None);
    let scene = section(workspace, "scene", "Arrival", Some(book));
    save_appearance(
        workspace,
        AppearanceInput {
            id: None,
            entity: ayla,
            context: scene,
            kind: key("physical_presence"),
            viewpoint: None,
            note: None,
            position: None,
            canon_state: None,
        },
    )
    .expect("records an appearance");

    let flood = new_entity(workspace, "event", "The Flood", "");
    let mut detail = EventDetail::new(flood);
    detail.period = Temporal::At {
        instant: Instant::at_day(100),
    };
    detail.location = Some(harbor);
    detail.participants = vec![EventParticipant::new(ayla)];
    save_event(workspace, detail).expect("records the event");

    workspace
        .record("dossier.save", |project| {
            let mut profile = BeingProfile::new(ayla);
            profile.birth = Some(Instant::at_day(500));
            project.edit(|editor| editor.set_being_profile(profile))?;
            Ok(())
        })
        .expect("records the birth");
    (ayla, harbor)
}

#[test]
fn an_author_builds_checks_protects_and_exports_a_universe() {
    let directory = tempfile::tempdir().expect("a temporary folder");
    let path = directory.path().join("Aurea.ontara");
    let path = path.to_string_lossy();
    let workspace = Workspace::default();

    project::create(&workspace, &path, "Aurea").expect("creates the project");
    let (_ayla, harbor) = write_the_universe(&workspace);
    assert_eq!(count(&workspace), 6);

    let report = consistency::run(&workspace).expect("checks");
    assert_eq!(
        report.counts.errors, 1,
        "taking part before being born is found"
    );

    let found = search::find(&workspace, "lighthouse", false).expect("searches");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].entity.name, "Ayla");

    // A mistake, taken back.
    remove(&workspace, &harbor, true).expect("removes");
    assert_eq!(count(&workspace), 5);
    history::undo(&workspace).expect("undoes");
    assert_eq!(count(&workspace), 6);

    // Protected and taken out.
    workspace
        .read(|project| Ok(project.create_backup(Timestamp::now())?))
        .expect("backs up");
    assert!(!backup::list(&workspace).expect("lists").is_empty());
    let exported =
        export(&workspace, ExportFormat::Json, &ExportWords::default()).expect("exports");
    assert_eq!(exported.entities, 6);

    // Closed and opened again: everything is still there.
    workspace.close().expect("closes");
    let reopened = project::open(&workspace, &path, false).expect("opens");
    assert_eq!(reopened.name, "Aurea");
    assert_eq!(count(&workspace), 6);
    assert_eq!(
        consistency::run(&workspace).expect("checks").counts.errors,
        1
    );
}
