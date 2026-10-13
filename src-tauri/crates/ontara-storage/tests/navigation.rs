//! Browsing a universe: finding entities and following the links between them, through a real
//! project.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;

use ontara_core::{
    Appearance, AssetFile, Attachment, ConnectionKind, EditorialNode, Entity, EntityFilter,
    EntityId, EntityName, EventDetail, EventParticipant, NameKind, Relationship, Timestamp,
    TypeKey, connections_of, find_entities,
};
use ontara_storage::Project;

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn project() -> Result<(tempfile::TempDir, Project), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = Project::create(&directory.path().join("Aurea.ontara"), "Aurea", T0)?;
    Ok((directory, project))
}

fn add(project: &mut Project, entity_type: &str, name: &str) -> Result<EntityId, Box<dyn Error>> {
    Ok(project.edit(|editor| {
        Ok(editor
            .add_entity(Entity::new(key(entity_type), name, T0))?
            .id)
    })?)
}

fn names(project: &Project, filter: &EntityFilter) -> Result<Vec<String>, Box<dyn Error>> {
    let reader = project.reader();
    Ok(find_entities(&reader, filter)?
        .into_iter()
        .map(|entity| entity.name)
        .collect())
}

#[test]
fn filtering_by_a_type_includes_its_subtypes() -> TestResult {
    let (_directory, mut project) = project()?;
    add(&mut project, "city", "Ashmere")?;
    add(&mut project, "region", "The Reach")?;
    add(&mut project, "character", "Ayla")?;

    let places = names(
        &project,
        &EntityFilter {
            entity_type: Some(key("place")),
            ..EntityFilter::default()
        },
    )?;
    let settlements = names(
        &project,
        &EntityFilter {
            entity_type: Some(key("settlement")),
            ..EntityFilter::default()
        },
    )?;

    assert_eq!(places, ["Ashmere", "The Reach"]);
    assert_eq!(settlements, ["Ashmere"]);
    Ok(())
}

#[test]
fn text_matches_names_aliases_summaries_and_tags_ignoring_case() -> TestResult {
    let (_directory, mut project) = project()?;
    project.edit(|editor| {
        let mut ayla = Entity::new(key("character"), "Ayla", T0);
        ayla.names
            .push(EntityName::new("Magpie", NameKind::Nickname));
        editor.add_entity(ayla)?;
        let mut orin = Entity::new(key("character"), "Orin", T0);
        orin.summary = "A ferryman on the Sable river".to_owned();
        editor.add_entity(orin)?;
        let mut mara = Entity::new(key("character"), "Mara", T0);
        mara.tags.insert("smuggler".to_owned());
        editor.add_entity(mara)?;
        Ok(())
    })?;

    let find = |text: &str| {
        names(
            &project,
            &EntityFilter {
                text: Some(text.to_owned()),
                ..EntityFilter::default()
            },
        )
    };

    assert_eq!(find("MAGPIE")?, ["Ayla"]);
    assert_eq!(find("sable")?, ["Orin"]);
    assert_eq!(find("smug")?, ["Mara"]);
    assert!(find("nothing like this")?.is_empty());
    Ok(())
}

#[test]
fn archived_entities_are_hidden_unless_asked_for_and_the_list_is_sorted() -> TestResult {
    let (_directory, mut project) = project()?;
    project.edit(|editor| {
        editor.add_entity(Entity::new(key("character"), "Zed", T0))?;
        let mut the_wanderer = Entity::new(key("character"), "The Wanderer", T0);
        the_wanderer.sort_name = Some("Wanderer, The".to_owned());
        editor.add_entity(the_wanderer)?;
        let mut gone = Entity::new(key("character"), "Aaron", T0);
        gone.archived = true;
        editor.add_entity(gone)?;
        Ok(())
    })?;

    let visible = names(&project, &EntityFilter::default())?;
    let everything = names(
        &project,
        &EntityFilter {
            include_archived: true,
            ..EntityFilter::default()
        },
    )?;

    assert_eq!(visible, ["The Wanderer", "Zed"]);
    assert_eq!(everything, ["Aaron", "The Wanderer", "Zed"]);
    Ok(())
}

#[test]
fn an_entity_lists_what_links_to_it_and_what_it_links_to() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let orin = add(&mut project, "character", "Orin")?;
    let ashmere = add(&mut project, "city", "Ashmere")?;
    let chapter = add(&mut project, "chapter", "The ferry")?;
    let battle = add(&mut project, "event", "Battle of the ford")?;
    let portrait = add(&mut project, "asset", "Ayla portrait")?;

    project.edit(|editor| {
        editor.add_relationship(Relationship::new(key("mentor_of"), ayla, orin, T0))?;
        editor.add_appearance(Appearance::new(ayla, chapter, key("physical_presence"), T0))?;
        let mut detail = EventDetail::new(battle);
        detail.location = Some(ashmere);
        detail.participants.push(EventParticipant::new(ayla));
        editor.set_event_detail(detail)?;
        editor.register_asset_file(AssetFile {
            entity: portrait,
            relative_path: "assets/ayla.png".to_owned(),
            media_type: Some("image/png".to_owned()),
            size_bytes: 3,
            content_hash: None,
            imported_at: T0,
        })?;
        editor.attach_asset(Attachment::new(ayla, portrait, T0))?;
        Ok(())
    })?;

    let reader = project.reader();
    let from_ayla = connections_of(&reader, &ayla)?;
    let from_orin = connections_of(&reader, &orin)?;
    let from_battle = connections_of(&reader, &battle)?;
    let from_portrait = connections_of(&reader, &portrait)?;

    assert!(from_ayla.iter().any(|link| link.other == orin
        && matches!(
            link.kind,
            ConnectionKind::Relationship { outgoing: true, .. }
        )));
    assert!(
        from_ayla
            .iter()
            .any(|link| link.other == chapter
                && matches!(link.kind, ConnectionKind::AppearsIn { .. }))
    );
    assert!(from_ayla.iter().any(
        |link| link.other == battle && matches!(link.kind, ConnectionKind::Participates { .. })
    ));
    assert!(
        from_ayla
            .iter()
            .any(|link| link.other == portrait
                && matches!(link.kind, ConnectionKind::HasAsset { .. }))
    );
    assert!(from_orin.iter().any(|link| link.other == ayla
        && matches!(
            link.kind,
            ConnectionKind::Relationship {
                outgoing: false,
                ..
            }
        )));
    assert!(
        from_battle
            .iter()
            .any(|link| link.other == ashmere && matches!(link.kind, ConnectionKind::TakesPlaceAt))
    );
    assert!(
        from_portrait.iter().any(
            |link| link.other == ayla && matches!(link.kind, ConnectionKind::AttachedTo { .. })
        )
    );
    Ok(())
}

#[test]
fn sections_link_to_their_parent_and_children() -> TestResult {
    let (_directory, mut project) = project()?;
    let book = add(&mut project, "volume", "Embers")?;
    let chapter = add(&mut project, "chapter", "The ferry")?;
    project.edit(|editor| {
        editor.place_editorial_node(EditorialNode {
            entity: book,
            level: key("volume"),
            parent: None,
            position: 0,
        })?;
        editor.place_editorial_node(EditorialNode {
            entity: chapter,
            level: key("chapter"),
            parent: Some(book),
            position: 0,
        })?;
        Ok(())
    })?;

    let reader = project.reader();
    let from_chapter = connections_of(&reader, &chapter)?;
    let from_book = connections_of(&reader, &book)?;

    assert!(
        from_chapter
            .iter()
            .any(|link| link.other == book && matches!(link.kind, ConnectionKind::InsideSection))
    );
    assert!(
        from_book
            .iter()
            .any(|link| link.other == chapter
                && matches!(link.kind, ConnectionKind::ContainsSection))
    );
    Ok(())
}
