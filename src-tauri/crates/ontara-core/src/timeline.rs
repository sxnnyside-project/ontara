use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::appearance::Appearance;
use crate::editorial_node::{EditorialNode, reading_order};
use crate::entity::Entity;
use crate::event::{CausalityKind, EventCausality};
use crate::event_detail::{EventDetail, EventScale};
use crate::identifier::EntityId;
use crate::repository::{ModelReader, Reader};
use crate::temporal::Temporal;

/// What to show of the chronology of a universe.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineQuery {
    /// Only events this entity is part of: it takes part in them, hosts them, shows up in them,
    /// or is the event itself.
    pub involving: Option<EntityId>,
    /// Only dated events that end on or after this day.
    pub from_day: Option<i64>,
    /// Only dated events that start on or before this day.
    pub to_day: Option<i64>,
}

/// One event, placed on the in-universe timeline and in the reading order of the work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimelineEntry {
    pub entity: EntityId,
    pub period: Temporal,
    pub start_day: Option<i64>,
    pub end_day: Option<i64>,
    pub location: Option<EntityId>,
    pub scale: Option<EventScale>,
    pub participants: usize,
    /// The sections of the work where the event itself shows up, in reading order.
    pub presented_in: Vec<EntityId>,
    /// Where the first of those sections falls among all sections, from 0.
    pub narrative_rank: Option<usize>,
    /// The event is told after another that happens later: a flashback or a jump back.
    pub out_of_order: bool,
}

/// A cause and its effect, both among the events shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimelineLink {
    pub cause: EntityId,
    pub effect: EntityId,
    pub kind: CausalityKind,
}

/// Events in in-universe order, apart from those that have no known day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Timeline {
    pub dated: Vec<TimelineEntry>,
    pub undated: Vec<TimelineEntry>,
    pub links: Vec<TimelineLink>,
}

/// The records a timeline is built from, so it can be computed without a repository.
pub struct TimelineSource<'a> {
    pub entities: &'a [Entity],
    pub events: &'a [EventDetail],
    pub appearances: &'a [Appearance],
    pub nodes: &'a [EditorialNode],
    pub causalities: &'a [EventCausality],
}

/// The sort key of a dated event: start, then end, falling back to the other when one is open.
fn day_range(period: &Temporal) -> (Option<i64>, Option<i64>) {
    let (start, end) = (period.start_day(), period.end_day());
    (start.or(end), end.or(start))
}

fn involves(
    detail: &EventDetail,
    entity: &EntityId,
    hosted: &BTreeMap<EntityId, BTreeSet<EntityId>>,
) -> bool {
    detail.entity == *entity
        || detail.location.as_ref() == Some(entity)
        || detail
            .participants
            .iter()
            .any(|participant| participant.entity == *entity)
        || hosted
            .get(&detail.entity)
            .is_some_and(|inside| inside.contains(entity))
}

/// Builds the timeline: dated events by start day, then end day, then name; the rest as
/// undated. Archived events are left out. `out_of_order` marks the events the work tells after
/// something that happens later.
#[must_use]
pub fn build_timeline(source: &TimelineSource<'_>, query: &TimelineQuery) -> Timeline {
    let entities: BTreeMap<EntityId, &Entity> = source
        .entities
        .iter()
        .map(|entity| (entity.id, entity))
        .collect();
    let order: BTreeMap<EntityId, usize> = reading_order(source.nodes)
        .iter()
        .enumerate()
        .map(|(index, step)| (step.entity, index))
        .collect();

    let (hosted, presented) = index_appearances(source.appearances, &order);

    let mut entries: Vec<TimelineEntry> = source
        .events
        .iter()
        .filter(|detail| {
            entities
                .get(&detail.entity)
                .is_some_and(|entity| !entity.archived)
        })
        .filter(|detail| {
            query
                .involving
                .as_ref()
                .is_none_or(|entity| involves(detail, entity, &hosted))
        })
        .map(|detail| {
            let (start_day, end_day) = day_range(&detail.period);
            let mut sections: Vec<EntityId> = presented
                .get(&detail.entity)
                .into_iter()
                .flatten()
                .copied()
                .collect();
            sections.sort_by_key(|section| (order.get(section).copied(), *section));
            TimelineEntry {
                entity: detail.entity,
                period: detail.period.clone(),
                start_day,
                end_day,
                location: detail.location,
                scale: detail.scale,
                participants: detail.participants.len(),
                narrative_rank: sections
                    .first()
                    .and_then(|section| order.get(section))
                    .copied(),
                presented_in: sections,
                out_of_order: false,
            }
        })
        .collect();

    let name_of = |entity: &EntityId| {
        entities
            .get(entity)
            .map(|found| found.name.to_lowercase())
            .unwrap_or_default()
    };
    entries.sort_by_cached_key(|entry| {
        (
            entry.start_day,
            entry.end_day,
            name_of(&entry.entity),
            entry.entity,
        )
    });

    let (mut dated, undated): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|entry| entry.start_day.is_some());
    dated.retain(|entry| {
        let after_start = query.from_day.is_none_or(|from| {
            entry
                .end_day
                .or(entry.start_day)
                .is_some_and(|end| end >= from)
        });
        let before_end = query
            .to_day
            .is_none_or(|to| entry.start_day.is_some_and(|start| start <= to));
        after_start && before_end
    });
    mark_out_of_order(&mut dated);

    let links = links_among(source.causalities, dated.iter().chain(&undated));
    Timeline {
        dated,
        undated,
        links,
    }
}

/// The two indexes the timeline needs from the appearances: which entities show up inside each
/// context, and which sections of the work each entity itself shows up in.
fn index_appearances(
    appearances: &[Appearance],
    order: &BTreeMap<EntityId, usize>,
) -> (
    BTreeMap<EntityId, BTreeSet<EntityId>>,
    BTreeMap<EntityId, BTreeSet<EntityId>>,
) {
    let mut hosted: BTreeMap<EntityId, BTreeSet<EntityId>> = BTreeMap::new();
    let mut presented: BTreeMap<EntityId, BTreeSet<EntityId>> = BTreeMap::new();
    for appearance in appearances {
        hosted
            .entry(appearance.context)
            .or_default()
            .insert(appearance.entity);
        if order.contains_key(&appearance.context) {
            presented
                .entry(appearance.entity)
                .or_default()
                .insert(appearance.context);
        }
    }
    (hosted, presented)
}

/// The causal links whose two ends are both among the events shown.
fn links_among<'a>(
    causalities: &[EventCausality],
    shown: impl Iterator<Item = &'a TimelineEntry>,
) -> Vec<TimelineLink> {
    let shown: BTreeSet<EntityId> = shown.map(|entry| entry.entity).collect();
    let mut links: Vec<TimelineLink> = causalities
        .iter()
        .filter(|link| shown.contains(&link.cause) && shown.contains(&link.effect))
        .map(|link| TimelineLink {
            cause: link.cause,
            effect: link.effect,
            kind: link.kind,
        })
        .collect();
    links.sort_by_key(|link| (link.cause, link.effect));
    links
}

/// Flags the events told after another that happens later. Walking the events in the order the
/// work tells them, an event is out of order when something later in time was already told.
fn mark_out_of_order(dated: &mut [TimelineEntry]) {
    let mut told: Vec<(usize, usize)> = dated
        .iter()
        .enumerate()
        .filter_map(|(chronology, entry)| entry.narrative_rank.map(|rank| (rank, chronology)))
        .collect();
    told.sort_unstable();
    let mut latest_told: Option<usize> = None;
    for (_, chronology) in told {
        let after_a_later_event = latest_told.is_some_and(|latest| chronology < latest);
        if let Some(entry) = dated.get_mut(chronology).filter(|_| after_a_later_event) {
            entry.out_of_order = true;
        }
        latest_told = Some(latest_told.map_or(chronology, |latest| latest.max(chronology)));
    }
}

/// The timeline of a universe, read from the repository.
pub fn timeline<R: ModelReader>(reader: &R, query: &TimelineQuery) -> Result<Timeline, R::Error> {
    let entities = Reader::<Entity>::list(reader)?;
    let events = Reader::<EventDetail>::list(reader)?;
    let appearances = Reader::<Appearance>::list(reader)?;
    let nodes = Reader::<EditorialNode>::list(reader)?;
    let causalities = Reader::<EventCausality>::list(reader)?;
    Ok(build_timeline(
        &TimelineSource {
            entities: &entities,
            events: &events,
            appearances: &appearances,
            nodes: &nodes,
            causalities: &causalities,
        },
        query,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::EntityId;
    use crate::temporal::Instant;
    use crate::timestamp::Timestamp;
    use crate::type_key::TypeKey;

    fn key(raw: &str) -> TypeKey {
        TypeKey::new(raw).unwrap_or_else(|error| panic!("{error}"))
    }

    fn entity(name: &str) -> Entity {
        Entity::new(key("event"), name, Timestamp::from_unix_millis(0))
    }

    fn event(entity: &Entity, day: Option<i64>) -> EventDetail {
        let mut detail = EventDetail::new(entity.id);
        if let Some(day) = day {
            detail.period = Temporal::At {
                instant: Instant::at_day(day),
            };
        }
        detail
    }

    fn scene(entity: EntityId, position: i32) -> EditorialNode {
        EditorialNode {
            entity,
            level: key("scene"),
            parent: None,
            position,
        }
    }

    fn appearance(entity: EntityId, context: EntityId) -> Appearance {
        Appearance::new(
            entity,
            context,
            key("physical_presence"),
            Timestamp::from_unix_millis(0),
        )
    }

    #[test]
    fn events_are_ordered_by_day_and_the_undated_are_kept_apart() {
        let (war, peace, rumor) = (entity("War"), entity("Peace"), entity("Rumor"));
        let events = [
            event(&peace, Some(900)),
            event(&rumor, None),
            event(&war, Some(100)),
        ];
        let entities = [war.clone(), peace.clone(), rumor.clone()];

        let timeline = build_timeline(
            &TimelineSource {
                entities: &entities,
                events: &events,
                appearances: &[],
                nodes: &[],
                causalities: &[],
            },
            &TimelineQuery::default(),
        );

        let dated: Vec<_> = timeline.dated.iter().map(|entry| entry.entity).collect();
        assert_eq!(dated, vec![war.id, peace.id]);
        assert_eq!(timeline.undated.len(), 1);
        assert_eq!(timeline.undated[0].entity, rumor.id);
    }

    #[test]
    fn an_event_told_before_one_that_happened_earlier_is_marked_as_out_of_order() {
        let (founding, siege, ending) = (entity("Founding"), entity("Siege"), entity("Ending"));
        let (first, second, third) = (
            EntityId::generate(),
            EntityId::generate(),
            EntityId::generate(),
        );
        // Chronology: founding (10), siege (20), ending (30). The work tells siege, ending, then
        // founding as a flashback.
        let events = [
            event(&founding, Some(10)),
            event(&siege, Some(20)),
            event(&ending, Some(30)),
        ];
        let entities = [founding.clone(), siege.clone(), ending.clone()];
        let nodes = [scene(first, 0), scene(second, 1), scene(third, 2)];
        let appearances = [
            appearance(siege.id, first),
            appearance(ending.id, second),
            appearance(founding.id, third),
        ];

        let timeline = build_timeline(
            &TimelineSource {
                entities: &entities,
                events: &events,
                appearances: &appearances,
                nodes: &nodes,
                causalities: &[],
            },
            &TimelineQuery::default(),
        );

        let flags: Vec<_> = timeline
            .dated
            .iter()
            .map(|entry| (entry.entity, entry.out_of_order, entry.narrative_rank))
            .collect();
        assert_eq!(
            flags,
            vec![
                (founding.id, true, Some(2)),
                (siege.id, false, Some(0)),
                (ending.id, false, Some(1)),
            ]
        );
    }

    #[test]
    fn the_range_and_the_involved_entity_narrow_the_timeline() {
        let (war, peace) = (entity("War"), entity("Peace"));
        let witness = EntityId::generate();
        let mut at_war = event(&war, Some(100));
        at_war
            .participants
            .push(crate::event_detail::EventParticipant::new(witness));
        let events = [at_war, event(&peace, Some(900))];
        let entities = [war.clone(), peace.clone()];
        let source = TimelineSource {
            entities: &entities,
            events: &events,
            appearances: &[],
            nodes: &[],
            causalities: &[],
        };

        let by_entity = build_timeline(
            &source,
            &TimelineQuery {
                involving: Some(witness),
                ..TimelineQuery::default()
            },
        );
        let by_range = build_timeline(
            &source,
            &TimelineQuery {
                from_day: Some(500),
                ..TimelineQuery::default()
            },
        );

        assert_eq!(by_entity.dated.len(), 1);
        assert_eq!(by_entity.dated[0].entity, war.id);
        assert_eq!(by_range.dated.len(), 1);
        assert_eq!(by_range.dated[0].entity, peace.id);
    }
}
