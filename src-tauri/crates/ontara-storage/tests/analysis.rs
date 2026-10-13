//! Exploring and checking a universe: the graph around an entity, who appears where, the
//! timeline and the consistency checks, through a real project.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;

use ontara_core::{
    Appearance, BeingProfile, EditorialNode, Entity, EntityFilter, EntityId, EventDetail,
    EventParticipant, GraphEdgeKind, GraphQuery, Instant, MatrixColumns, MatrixQuery, Relationship,
    Rule, Temporal, TimelineQuery, Timestamp, TypeKey, appearance_matrix, check_consistency,
    explore, timeline,
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

fn place(
    project: &mut Project,
    entity: EntityId,
    level: &str,
    parent: Option<EntityId>,
    position: i32,
) -> Result<(), Box<dyn Error>> {
    project.edit(|editor| {
        editor.place_editorial_node(EditorialNode {
            entity,
            level: key(level),
            parent,
            position,
        })?;
        Ok(())
    })?;
    Ok(())
}

fn on_day(day: i64) -> Temporal {
    Temporal::At {
        instant: Instant::at_day(day),
    }
}

#[test]
fn the_graph_walks_relationships_and_events_up_to_the_depth_asked() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let orin = add(&mut project, "character", "Orin")?;
    let mara = add(&mut project, "character", "Mara")?;
    let ashmere = add(&mut project, "city", "Ashmere")?;
    let battle = add(&mut project, "event", "Battle of the ford")?;
    project.edit(|editor| {
        editor.add_relationship(Relationship::new(key("mentor_of"), ayla, orin, T0))?;
        editor.add_relationship(Relationship::new(key("mentor_of"), orin, mara, T0))?;
        let mut detail = EventDetail::new(battle);
        detail.location = Some(ashmere);
        detail.participants.push(EventParticipant::new(ayla));
        editor.set_event_detail(detail)?;
        Ok(())
    })?;
    let reader = project.reader();

    let near = explore(&reader, &GraphQuery::around(ayla))?;
    let far = explore(
        &reader,
        &GraphQuery {
            depth: 3,
            ..GraphQuery::around(ayla)
        },
    )?;
    let only_mentors = explore(
        &reader,
        &GraphQuery {
            depth: 3,
            include_events: false,
            ..GraphQuery::around(ayla)
        },
    )?;

    let ids = |graph: &ontara_core::Graph| -> Vec<EntityId> {
        graph.nodes.iter().map(|node| node.entity).collect()
    };
    assert_eq!(near.nodes[0].entity, ayla);
    assert!(ids(&near).contains(&orin) && ids(&near).contains(&battle));
    assert!(!ids(&near).contains(&mara), "mara is two steps away");
    assert!(ids(&far).contains(&mara) && ids(&far).contains(&ashmere));
    assert!(!ids(&only_mentors).contains(&battle));
    assert!(far.edges.iter().any(|edge| edge.source == battle
        && edge.target == ashmere
        && matches!(edge.kind, GraphEdgeKind::Location)));
    assert!(!far.truncated);
    Ok(())
}

#[test]
fn the_matrix_has_sections_in_reading_order_and_only_what_was_recorded() -> TestResult {
    let (_directory, mut project) = project()?;
    let book = add(&mut project, "volume", "Embers")?;
    let first = add(&mut project, "chapter", "The ferry")?;
    let second = add(&mut project, "chapter", "The ford")?;
    place(&mut project, book, "volume", None, 0)?;
    place(&mut project, second, "chapter", Some(book), 1)?;
    place(&mut project, first, "chapter", Some(book), 0)?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let orin = add(&mut project, "character", "Orin")?;
    add(&mut project, "character", "Nobody yet")?;
    project.edit(|editor| {
        editor.add_appearance(Appearance::new(ayla, first, key("physical_presence"), T0))?;
        editor.add_appearance(Appearance::new(ayla, second, key("dialogue"), T0))?;
        editor.add_appearance(Appearance::new(orin, second, key("confirmed_absence"), T0))?;
        Ok(())
    })?;
    let reader = project.reader();

    let matrix = appearance_matrix(
        &reader,
        &MatrixQuery {
            rows: EntityFilter {
                entity_type: Some(key("character")),
                ..EntityFilter::default()
            },
            columns: MatrixColumns::Structure {
                section: None,
                level: None,
            },
            include_empty_rows: false,
        },
    )?;
    let with_empty = appearance_matrix(
        &reader,
        &MatrixQuery {
            rows: EntityFilter {
                entity_type: Some(key("character")),
                ..EntityFilter::default()
            },
            include_empty_rows: true,
            ..MatrixQuery::whole_work()
        },
    )?;

    let columns: Vec<_> = matrix.columns.iter().map(|column| column.entity).collect();
    assert_eq!(columns, vec![first, second]);
    assert_eq!(matrix.rows, vec![ayla, orin]);
    assert_eq!(matrix.cells.len(), 3);
    let absence = matrix
        .cells
        .iter()
        .find(|cell| cell.entity == orin)
        .expect("orin has a cell");
    assert!(!absence.is_presence);
    assert_eq!(with_empty.rows.len(), 3);
    Ok(())
}

#[test]
fn the_timeline_orders_by_day_and_flags_what_the_work_tells_out_of_order() -> TestResult {
    let (_directory, mut project) = project()?;
    let book = add(&mut project, "volume", "Embers")?;
    let first = add(&mut project, "scene", "Opening")?;
    let second = add(&mut project, "scene", "Flashback")?;
    place(&mut project, book, "volume", None, 0)?;
    place(&mut project, first, "scene", Some(book), 0)?;
    place(&mut project, second, "scene", Some(book), 1)?;
    let coronation = add(&mut project, "event", "Coronation")?;
    let founding = add(&mut project, "event", "Founding")?;
    let rumor = add(&mut project, "event", "A rumor")?;
    project.edit(|editor| {
        for (event, period) in [
            (coronation, on_day(500)),
            (founding, on_day(10)),
            (rumor, Temporal::Unknown),
        ] {
            let mut detail = EventDetail::new(event);
            detail.period = period;
            editor.set_event_detail(detail)?;
        }
        editor.add_appearance(Appearance::new(
            coronation,
            first,
            key("physical_presence"),
            T0,
        ))?;
        editor.add_appearance(Appearance::new(
            founding,
            second,
            key("physical_presence"),
            T0,
        ))?;
        Ok(())
    })?;
    let reader = project.reader();

    let result = timeline(&reader, &TimelineQuery::default())?;

    let dated: Vec<_> = result.dated.iter().map(|entry| entry.entity).collect();
    assert_eq!(dated, vec![founding, coronation]);
    assert_eq!(result.undated.len(), 1);
    assert!(
        result.dated[0].out_of_order,
        "the founding is told after the coronation"
    );
    assert!(!result.dated[1].out_of_order);
    Ok(())
}

#[test]
fn consistency_checks_find_what_contradicts_and_leave_the_model_untouched() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let feast = add(&mut project, "event", "The feast")?;
    project.edit(|editor| {
        let mut profile = BeingProfile::new(ayla);
        profile.death = Some(Instant::at_day(100));
        editor.set_being_profile(profile)?;
        let mut detail = EventDetail::new(feast);
        detail.period = on_day(300);
        detail.participants.push(EventParticipant::new(ayla));
        editor.set_event_detail(detail)?;
        Ok(())
    })?;
    let before = project.reader();

    let report = check_consistency(&before)?;
    let again = check_consistency(&project.reader())?;

    assert_eq!(report.counts.errors, 1);
    assert_eq!(report.findings[0].rule, Rule::ParticipantAfterDeath);
    assert_eq!(report.findings[0].subject, feast);
    assert_eq!(report.findings[0].related, vec![ayla]);
    assert_eq!(report, again, "the same data gives the same report");
    Ok(())
}
