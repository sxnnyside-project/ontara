//! A small universe to start from, so a new author sees what the application does with real
//! material before writing their own. It contains one deliberate contradiction, so the
//! consistency checks have something true to show.

use std::collections::BTreeSet;

use ontara_core::{
    Appearance, BeingProfile, EditError, EditorialNode, Entity, EntityId, EventDetail,
    EventParticipant, Instant, ModelEditor, ModelReaderExt, Note, Relationship, Temporal,
    Timestamp, TypeKey, UniverseProfile,
};
use ontara_storage::{SqliteModel, StorageError};
use serde::Serialize;

use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// What loading the example created.
///
/// Mirrored by `src/features/project/api/example-api.ts`.
#[derive(Debug, Serialize)]
pub struct ExampleLoaded {
    pub entities: usize,
}

/// The words of the example in one language.
struct Words {
    premise: &'static str,
    ayla: (&'static str, &'static str),
    bren: (&'static str, &'static str),
    orin: (&'static str, &'static str),
    aurea: (&'static str, &'static str),
    lighthouse: (&'static str, &'static str),
    lamplighters: (&'static str, &'static str),
    lantern: (&'static str, &'static str),
    flood: (&'static str, &'static str),
    pact: (&'static str, &'static str),
    book: &'static str,
    chapter_one: &'static str,
    chapter_two: &'static str,
    scenes: [&'static str; 3],
    note_title: &'static str,
    note_body: &'static str,
}

const ENGLISH: Words = Words {
    premise: "A sunken empire whose lighthouses still keep watch.",
    ayla: ("Ayla", "Keeper of the drowned lighthouse."),
    bren: ("Bren", "Ayla's son, learning the lamps."),
    orin: ("Orin", "The old navigator who taught Ayla the coast."),
    aurea: ("Aurea", "The capital that went under the sea."),
    lighthouse: (
        "The Drowned Lighthouse",
        "A tower that still turns its light below the waves.",
    ),
    lamplighters: ("The Lamplighters", "Keepers of every light on the coast."),
    lantern: (
        "The Ember Lantern",
        "A lantern that never goes out, passed from keeper to keeper.",
    ),
    flood: ("The Flood", "The night Aurea went under."),
    pact: (
        "The Pact of Lamps",
        "The keepers swear to keep every light burning.",
    ),
    book: "Embers",
    chapter_one: "The ferry",
    chapter_two: "The pact",
    scenes: ["Arrival", "The ledger", "Lamps lit"],
    note_title: "Why this one is red",
    note_body: "In this example Bren takes part in the Pact (day 1200) but is born on day 1500. \
                Open Checks to see it, then fix his birth date in Details and the finding goes away.",
};

const SPANISH: Words = Words {
    premise: "Un imperio hundido cuyos faros siguen vigilando.",
    ayla: ("Ayla", "Guardiana del faro hundido."),
    bren: ("Bren", "Hijo de Ayla, aprende el oficio de las lámparas."),
    orin: ("Orin", "El viejo navegante que le enseñó la costa a Ayla."),
    aurea: ("Aurea", "La capital que quedó bajo el mar."),
    lighthouse: (
        "El Faro Hundido",
        "Una torre que aún gira su luz bajo las olas.",
    ),
    lamplighters: ("Los Encendedores", "Guardianes de cada luz de la costa."),
    lantern: (
        "La Linterna de Brasas",
        "Una linterna que nunca se apaga, de guardián en guardián.",
    ),
    flood: ("La Inundación", "La noche en que Aurea se hundió."),
    pact: (
        "El Pacto de las Lámparas",
        "Los guardianes juran mantener encendida cada luz.",
    ),
    book: "Brasas",
    chapter_one: "El transbordador",
    chapter_two: "El pacto",
    scenes: ["La llegada", "El libro de cuentas", "Lámparas encendidas"],
    note_title: "Por qué este está en rojo",
    note_body: "En este ejemplo Bren participa en el Pacto (día 1200) pero nace el día 1500. \
                Abre Revisión para verlo, y corrige su fecha de nacimiento en Datos: el hallazgo desaparece.",
};

type Failure = EditError<StorageError>;

fn key(raw: &str) -> Result<TypeKey, Failure> {
    Ok(TypeKey::new(raw)?)
}

fn on_day(day: i64) -> Temporal {
    Temporal::At {
        instant: Instant::at_day(day),
    }
}

fn entity(
    editor: &mut ModelEditor<'_, SqliteModel<'_>>,
    entity_type: &str,
    text: (&str, &str),
    now: Timestamp,
) -> Result<EntityId, Failure> {
    let mut entity = Entity::new(key(entity_type)?, text.0, now);
    text.1.clone_into(&mut entity.summary);
    entity.tags = BTreeSet::new();
    Ok(editor.add_entity(entity)?.id)
}

fn build(
    editor: &mut ModelEditor<'_, SqliteModel<'_>>,
    words: &Words,
    now: Timestamp,
) -> Result<usize, Failure> {
    editor.update_universe_profile(
        UniverseProfile {
            summary: words.premise.to_owned(),
            description: String::new(),
            updated_at: None,
        },
        now,
    )?;

    let ayla = entity(editor, "character", words.ayla, now)?;
    let bren = entity(editor, "character", words.bren, now)?;
    let orin = entity(editor, "character", words.orin, now)?;
    let aurea = entity(editor, "city", words.aurea, now)?;
    let lighthouse = entity(editor, "stronghold", words.lighthouse, now)?;
    let lamplighters = entity(editor, "faction", words.lamplighters, now)?;
    let lantern = entity(editor, "item", words.lantern, now)?;
    let flood = entity(editor, "event", words.flood, now)?;
    let pact = entity(editor, "event", words.pact, now)?;

    for (being, birth) in [(orin, -20_000), (ayla, -9_000), (bren, 1_500)] {
        let mut profile = BeingProfile::new(being);
        profile.birth = Some(Instant::at_day(birth));
        editor.set_being_profile(profile)?;
    }

    for (source, kind, target) in [
        (orin, "mentor_of", ayla),
        (ayla, "parent_of", bren),
        (bren, "affiliated_with", lamplighters),
        (ayla, "resides_in", lighthouse),
        (lighthouse, "located_within", aurea),
        (ayla, "owns", lantern),
    ] {
        editor.add_relationship(Relationship::new(key(kind)?, source, target, now))?;
    }

    let mut detail = EventDetail::new(flood);
    detail.period = on_day(0);
    detail.location = Some(aurea);
    detail.participants = vec![EventParticipant::new(ayla), EventParticipant::new(orin)];
    editor.set_event_detail(detail)?;
    let mut detail = EventDetail::new(pact);
    detail.period = on_day(1_200);
    detail.location = Some(lighthouse);
    detail.participants = vec![EventParticipant::new(ayla), EventParticipant::new(bren)];
    editor.set_event_detail(detail)?;
    editor.add_note(Note::new(pact, words.note_title, words.note_body, now))?;

    let book = entity(editor, "volume", (words.book, ""), now)?;
    let first = entity(editor, "chapter", (words.chapter_one, ""), now)?;
    let second = entity(editor, "chapter", (words.chapter_two, ""), now)?;
    let scenes = [
        entity(editor, "scene", (words.scenes[0], ""), now)?,
        entity(editor, "scene", (words.scenes[1], ""), now)?,
        entity(editor, "scene", (words.scenes[2], ""), now)?,
    ];
    let node = |entity, level: &str, parent, position| -> Result<EditorialNode, Failure> {
        Ok(EditorialNode {
            entity,
            level: key(level)?,
            parent,
            position,
        })
    };
    editor.place_editorial_node(node(book, "volume", None, 0)?)?;
    editor.place_editorial_node(node(first, "chapter", Some(book), 0)?)?;
    editor.place_editorial_node(node(second, "chapter", Some(book), 1)?)?;
    editor.place_editorial_node(node(scenes[0], "scene", Some(first), 0)?)?;
    editor.place_editorial_node(node(scenes[1], "scene", Some(first), 1)?)?;
    editor.place_editorial_node(node(scenes[2], "scene", Some(second), 0)?)?;

    for (who, scene) in [
        (orin, scenes[0]),
        (ayla, scenes[0]),
        (ayla, scenes[1]),
        (bren, scenes[1]),
        (ayla, scenes[2]),
        (bren, scenes[2]),
    ] {
        editor.add_appearance(Appearance::new(who, scene, key("physical_presence")?, now))?;
    }

    Ok(15)
}

/// Fills an empty project with the example, in the language asked for.
pub fn load(workspace: &Workspace, locale: &str) -> Result<ExampleLoaded, CommandError> {
    let words = if locale.starts_with("es") {
        &SPANISH
    } else {
        &ENGLISH
    };
    let now = Timestamp::now();
    workspace.record("example.create", |project| {
        if !project.reader().entities()?.is_empty() {
            return Err(CommandError::new(
                "not_empty",
                "The example can only be loaded into a universe with nothing in it yet.",
            ));
        }
        let entities = project.edit(|editor| build(editor, words, now))?;
        Ok(ExampleLoaded { entities })
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::consistency;
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn the_example_loads_once_and_shows_one_contradiction() {
        let (_directory, workspace) = workspace_with_project();

        let loaded = load(&workspace, "en").expect("loads");
        let report = consistency::run(&workspace).expect("checks");
        let again = load(&workspace, "en").expect_err("not empty any more");

        assert_eq!(loaded.entities, 15);
        assert_eq!(report.counts.errors, 1);
        assert_eq!(again.code, "not_empty");
    }

    #[test]
    fn the_example_speaks_the_language_asked_for() {
        let (_directory, workspace) = workspace_with_project();

        load(&workspace, "es").expect("loads");
        let names: Vec<String> = workspace
            .read(|project| Ok(project.reader().entities()?))
            .expect("reads")
            .into_iter()
            .map(|entity| entity.name)
            .collect();

        assert!(names.iter().any(|name| name == "El Faro Hundido"));
    }
}
