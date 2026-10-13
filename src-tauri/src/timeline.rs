//! The chronology of a universe, and the detail of each event on it.

use ontara_core::{
    CausalityKind, EntityId, EventCausality, EventCausalityId, EventDetail, EventParticipant,
    EventScale, ModelQueries, ModelReaderExt, Origin, Reader, Temporal, TimelineLink,
    TimelineQuery, Timestamp, TypeKey, timeline,
};
use serde::{Deserialize, Serialize};

use crate::entity::{EntityRef, found, resolve, resolve_all, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// An event on the timeline, with the entities it names resolved.
///
/// Mirrored by `src/features/timeline/api/timeline-api.ts`.
#[derive(Debug, Serialize)]
pub struct TimelineEntryView {
    pub entity: EntityRef,
    pub period: Temporal,
    pub start_day: Option<i64>,
    pub end_day: Option<i64>,
    pub location: Option<EntityRef>,
    pub scale: Option<EventScale>,
    pub participants: usize,
    pub presented_in: Vec<EntityRef>,
    pub narrative_rank: Option<usize>,
    pub out_of_order: bool,
}

#[derive(Debug, Serialize)]
pub struct TimelineView {
    pub dated: Vec<TimelineEntryView>,
    pub undated: Vec<TimelineEntryView>,
    pub links: Vec<TimelineLink>,
}

/// A participant of an event with the entity resolved.
#[derive(Debug, Serialize)]
pub struct ParticipantView {
    #[serde(flatten)]
    pub participant: EventParticipant,
    pub entity: EntityRef,
}

/// One cause or effect of an event.
#[derive(Debug, Serialize)]
pub struct CausalityView {
    pub id: EventCausalityId,
    pub other: EntityRef,
    pub kind: CausalityKind,
    pub note: Option<String>,
}

/// What an event is: when, where, who, and what led to it and came of it.
#[derive(Debug, Serialize)]
pub struct EventView {
    /// Absent while nothing has been recorded about the event yet.
    pub detail: Option<EventDetail>,
    pub location: Option<EntityRef>,
    pub participants: Vec<ParticipantView>,
    pub causes: Vec<CausalityView>,
    pub effects: Vec<CausalityView>,
}

#[derive(Debug, Deserialize)]
pub struct CausalityInput {
    pub cause: EntityId,
    pub effect: EntityId,
    pub kind: CausalityKind,
    pub note: Option<String>,
}

pub fn chronology(
    workspace: &Workspace,
    query: &TimelineQuery,
) -> Result<TimelineView, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let result = timeline(&reader, query)?;
        let view = |entries: &[ontara_core::TimelineEntry]| {
            entries
                .iter()
                .map(|entry| {
                    Ok(TimelineEntryView {
                        entity: resolve(&reader, &labels, &entry.entity)?,
                        period: entry.period.clone(),
                        start_day: entry.start_day,
                        end_day: entry.end_day,
                        location: entry
                            .location
                            .as_ref()
                            .map(|place| resolve(&reader, &labels, place))
                            .transpose()?,
                        scale: entry.scale,
                        participants: entry.participants,
                        presented_in: resolve_all(&reader, &labels, &entry.presented_in)?,
                        narrative_rank: entry.narrative_rank,
                        out_of_order: entry.out_of_order,
                    })
                })
                .collect::<Result<Vec<_>, CommandError>>()
        };
        Ok(TimelineView {
            dated: view(&result.dated)?,
            undated: view(&result.undated)?,
            links: result.links,
        })
    })
}

pub fn event(workspace: &Workspace, entity: &EntityId) -> Result<EventView, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        resolve(&reader, &labels, entity)?;
        let detail = Reader::<EventDetail>::get(&reader, entity)?;
        let location = detail
            .as_ref()
            .and_then(|detail| detail.location.as_ref())
            .map(|place| resolve(&reader, &labels, place))
            .transpose()?;
        let participants = detail
            .iter()
            .flat_map(|detail| detail.participants.iter())
            .map(|participant| {
                Ok(ParticipantView {
                    participant: participant.clone(),
                    entity: resolve(&reader, &labels, &participant.entity)?,
                })
            })
            .collect::<Result<Vec<_>, CommandError>>()?;
        let link = |record: EventCausality, other: EntityId| {
            Ok(CausalityView {
                id: record.id,
                other: resolve(&reader, &labels, &other)?,
                kind: record.kind,
                note: record.note,
            })
        };
        let causes = reader
            .event_causality_by_effect(entity)?
            .into_iter()
            .map(|record| {
                let other = record.cause;
                link(record, other)
            })
            .collect::<Result<Vec<_>, CommandError>>()?;
        let effects = reader
            .event_causality_by_cause(entity)?
            .into_iter()
            .map(|record| {
                let other = record.effect;
                link(record, other)
            })
            .collect::<Result<Vec<_>, CommandError>>()?;
        Ok(EventView {
            detail,
            location,
            participants,
            causes,
            effects,
        })
    })
}

pub fn save_event(workspace: &Workspace, detail: EventDetail) -> Result<EventDetail, CommandError> {
    workspace.record("timeline.save_event", |project| {
        Ok(project.edit(|editor| editor.set_event_detail(detail))?)
    })
}

pub fn clear_event(workspace: &Workspace, entity: &EntityId) -> Result<(), CommandError> {
    workspace.record("timeline.clear_event", |project| {
        Ok(project.edit(|editor| editor.clear_event_detail(entity))?)
    })
}

pub fn add_causality(
    workspace: &Workspace,
    input: CausalityInput,
) -> Result<EventCausality, CommandError> {
    workspace.record("timeline.add_causality", |project| {
        let mut record =
            EventCausality::new(input.cause, input.effect, input.kind, Timestamp::now());
        record.note = input.note;
        record.origin = Origin::User;
        Ok(project.edit(|editor| editor.add_event_causality(record))?)
    })
}

pub fn remove_causality(workspace: &Workspace, id: &EventCausalityId) -> Result<(), CommandError> {
    workspace.record("timeline.remove_causality", |project| {
        found(
            Reader::<EventCausality>::get(&project.reader(), id)?,
            "causality",
            id,
        )?;
        Ok(project.edit(|editor| editor.remove_event_causality(id))?)
    })
}

/// Event types the author can create, for choosing the kind of a new event.
pub fn event_type_keys(workspace: &Workspace) -> Result<Vec<TypeKey>, CommandError> {
    workspace.read(|project| {
        Ok(project
            .reader()
            .entity_types()?
            .into_iter()
            .filter(|entity_type| entity_type.role == ontara_core::EntityRole::Event)
            .filter(|entity_type| !entity_type.is_abstract)
            .map(|entity_type| entity_type.key)
            .collect())
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use ontara_core::{Instant, TypeKey};

    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    fn entity(workspace: &Workspace, entity_type: &str, name: &str) -> EntityId {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new(entity_type).expect("a valid key"),
                name: name.to_owned(),
                summary: String::new(),
                description: String::new(),
                tags: std::collections::BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates")
        .id
    }

    fn dated(entity: EntityId, day: i64) -> EventDetail {
        let mut detail = EventDetail::new(entity);
        detail.period = Temporal::At {
            instant: Instant::at_day(day),
        };
        detail
    }

    #[test]
    fn an_event_is_dated_placed_and_linked_to_what_caused_it() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let ashmere = entity(&workspace, "city", "Ashmere");
        let spark = entity(&workspace, "event", "The spark");
        let fire = entity(&workspace, "event", "The fire");
        save_event(&workspace, dated(spark, 10)).expect("dates the spark");
        let mut detail = dated(fire, 20);
        detail.location = Some(ashmere);
        detail.participants.push(EventParticipant::new(ayla));
        save_event(&workspace, detail).expect("dates the fire");
        add_causality(
            &workspace,
            CausalityInput {
                cause: spark,
                effect: fire,
                kind: CausalityKind::Direct,
                note: None,
            },
        )
        .expect("links");

        let timeline = chronology(&workspace, &TimelineQuery::default()).expect("timeline");
        let view = event(&workspace, &fire).expect("event");

        let order: Vec<_> = timeline
            .dated
            .iter()
            .map(|entry| entry.entity.name.as_str())
            .collect();
        assert_eq!(order, ["The spark", "The fire"]);
        assert_eq!(timeline.links.len(), 1);
        assert_eq!(
            view.location.map(|place| place.name),
            Some("Ashmere".to_owned())
        );
        assert_eq!(view.participants[0].entity.name, "Ayla");
        assert_eq!(view.causes[0].other.name, "The spark");
    }

    #[test]
    fn only_events_can_be_given_a_date() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");

        let error = save_event(&workspace, dated(ayla, 5)).expect_err("a character is no event");

        assert_eq!(error.code, "rejected");
    }
}
