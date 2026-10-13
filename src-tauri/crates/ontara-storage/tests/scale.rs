//! A universe of a size a long series reaches: the operations an author waits on stay fast.
//!
//! The budgets are generous on purpose, since tests run unoptimized on any machine; they exist to
//! catch a change that makes something quadratic, not to benchmark. Run with `--nocapture` to
//! see the timings.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;
use std::time::{Duration, Instant as Clock};

use ontara_core::{
    Appearance, EditorialNode, Entity, EntityFilter, EntityId, GraphQuery, MatrixQuery,
    ModelQueries, ModelReaderExt, Note, Relationship, TimelineQuery, Timestamp, TypeKey,
    appearance_matrix, check_consistency, explore, find_entities, timeline,
};
use ontara_storage::Project;

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);
const CHARACTERS: usize = 1_200;
const PLACES: usize = 200;
const SCENES: usize = 300;
const BUDGET: Duration = Duration::from_secs(3);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn timed<T>(
    what: &str,
    work: impl FnOnce() -> Result<T, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    let started = Clock::now();
    let value = work()?;
    let elapsed = started.elapsed();
    println!("{what}: {elapsed:?}");
    assert!(elapsed < BUDGET, "{what} took {elapsed:?}, over {BUDGET:?}");
    Ok(value)
}

#[test]
// One story told top to bottom: splitting it would hide the order the author's work happens in.
#[allow(clippy::too_many_lines)]
fn a_large_universe_stays_responsive() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut project = Project::create(&directory.path().join("Big.ontara"), "Big", T0)?;

    let mut characters: Vec<EntityId> = Vec::new();
    let mut places: Vec<EntityId> = Vec::new();
    let mut scenes: Vec<EntityId> = Vec::new();

    timed("populate", || {
        for batch in 0..(CHARACTERS / 100) {
            let ids = project.edit(|editor| {
                let mut ids = Vec::new();
                for index in 0..100 {
                    let number = batch * 100 + index;
                    let mut entity =
                        Entity::new(key("character"), format!("Character {number}"), T0);
                    entity.summary = format!("Keeper number {number} of the northern marches");
                    ids.push(editor.add_entity(entity)?.id);
                }
                Ok(ids)
            })?;
            characters.extend(ids);
        }
        places.extend(project.edit(|editor| {
            (0..PLACES)
                .map(|number| {
                    Ok(editor
                        .add_entity(Entity::new(key("city"), format!("City {number}"), T0))?
                        .id)
                })
                .collect::<Result<Vec<_>, _>>()
        })?);
        let book = project.edit(|editor| {
            Ok(editor
                .add_entity(Entity::new(key("volume"), "Book", T0))?
                .id)
        })?;
        scenes.extend(project.edit(|editor| {
            editor.place_editorial_node(EditorialNode {
                entity: book,
                level: key("volume"),
                parent: None,
                position: 0,
            })?;
            (0..SCENES)
                .map(|number| {
                    let scene = editor
                        .add_entity(Entity::new(key("scene"), format!("Scene {number}"), T0))?
                        .id;
                    editor.place_editorial_node(EditorialNode {
                        entity: scene,
                        level: key("scene"),
                        parent: Some(book),
                        position: i32::try_from(number).unwrap_or(0),
                    })?;
                    Ok(scene)
                })
                .collect::<Result<Vec<_>, _>>()
        })?);

        project.edit(|editor| {
            for (index, &character) in characters.iter().enumerate() {
                let hub = characters[index % 12];
                if hub != character {
                    editor.add_relationship(Relationship::new(
                        key("mentor_of"),
                        hub,
                        character,
                        T0,
                    ))?;
                }
                editor.add_relationship(Relationship::new(
                    key("resides_in"),
                    character,
                    places[index % PLACES],
                    T0,
                ))?;
                editor.add_appearance(Appearance::new(
                    character,
                    scenes[index % SCENES],
                    key("physical_presence"),
                    T0,
                ))?;
                if index % 5 == 0 {
                    editor.add_note(Note::new(
                        character,
                        "Margin",
                        "Remembers the salt road",
                        T0,
                    ))?;
                }
            }
            Ok(())
        })?;
        Ok(())
    })?;

    let reader = project.reader();
    let listed = timed("list every entity", || {
        Ok(find_entities(&reader, &EntityFilter::default())?)
    })?;
    assert_eq!(listed.len(), CHARACTERS + PLACES + 1 + SCENES);

    let hits = timed("full-text search", || {
        Ok(reader.search_text("keeper marches", false, 30)?)
    })?;
    assert_eq!(hits.len(), 30);
    let notes = timed("search a note", || {
        Ok(reader.search_text("salt road", false, 30)?)
    })?;
    assert!(!notes.is_empty());

    let hub = characters[0];
    let graph = timed("graph, three steps from a hub", || {
        Ok(explore(
            &reader,
            &GraphQuery {
                depth: 3,
                ..GraphQuery::around(hub)
            },
        )?)
    })?;
    assert!(graph.nodes.len() > 50);

    timed("appearance matrix", || {
        Ok(appearance_matrix(&reader, &MatrixQuery::whole_work())?)
    })?;
    timed("timeline", || {
        Ok(timeline(&reader, &TimelineQuery::default())?)
    })?;
    timed("consistency checks", || Ok(check_consistency(&reader)?))?;
    drop(reader);

    timed("undo a batch of 100 entities", || {
        project.set_change_context("entity.create", T0);
        project.edit(|editor| {
            for number in 0..100 {
                editor.add_entity(Entity::new(key("character"), format!("Late {number}"), T0))?;
            }
            Ok(())
        })?;
        project.undo(T0)?.expect("something to undo");
        Ok(())
    })?;
    assert_eq!(
        project.reader().entities()?.len(),
        CHARACTERS + PLACES + 1 + SCENES
    );

    timed("verified backup", || Ok(project.create_backup(T0)?))?;
    Ok(())
}
