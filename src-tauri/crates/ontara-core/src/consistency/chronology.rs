//! Rules about when things happen relative to each other.

use std::collections::{BTreeMap, BTreeSet};

use super::{Context, Finding, Rule};
use crate::identifier::EntityId;

/// The days an event covers, if it has any: start and end, either filled in from the other.
fn days_of(period: &crate::temporal::Temporal) -> Option<(i64, i64)> {
    let (start, end) = (period.start_day(), period.end_day());
    Some((start.or(end)?, end.or(start)?))
}

pub(super) fn run(context: &Context<'_>, findings: &mut Vec<Finding>) {
    participants_and_lifetimes(context, findings);
    parents_and_children(context, findings);
    causes_and_effects(context, findings);
    two_places_at_once(context, findings);
    office_seats(context, findings);
}

fn participants_and_lifetimes(context: &Context<'_>, findings: &mut Vec<Finding>) {
    for detail in &context.snapshot.events {
        if !context.entity_counts(&detail.entity) {
            continue;
        }
        let Some((start, end)) = days_of(&detail.period) else {
            continue;
        };
        let participants: BTreeSet<EntityId> = detail
            .participants
            .iter()
            .map(|participant| participant.entity)
            .collect();
        for participant in participants {
            let Some(profile) = context.profiles.get(&participant) else {
                continue;
            };
            let death = profile.death.as_ref().and_then(|instant| instant.day);
            if let Some(death) = death.filter(|death| start > *death) {
                findings.push(Finding::new(
                    Rule::ParticipantAfterDeath,
                    detail.entity,
                    vec![participant],
                    &[
                        ("event_day", start.to_string()),
                        ("death_day", death.to_string()),
                    ],
                ));
            }
            let birth = profile.birth.as_ref().and_then(|instant| instant.day);
            if let Some(birth) = birth.filter(|birth| end < *birth) {
                findings.push(Finding::new(
                    Rule::ParticipantBeforeBirth,
                    detail.entity,
                    vec![participant],
                    &[
                        ("event_day", end.to_string()),
                        ("birth_day", birth.to_string()),
                    ],
                ));
            }
        }
    }
}

fn parents_and_children(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let birth_of = |entity: &EntityId| {
        context
            .profiles
            .get(entity)
            .and_then(|profile| profile.birth.as_ref())
            .and_then(|instant| instant.day)
    };
    for tie in &context.snapshot.family_ties {
        if !context.counts_as_canon(&tie.canon_state) {
            continue;
        }
        let (Some(parent), Some(child)) = (birth_of(&tie.parent), birth_of(&tie.child)) else {
            continue;
        };
        if parent >= child {
            findings.push(Finding::new(
                Rule::ParentNotOlder,
                tie.parent,
                vec![tie.child],
                &[
                    ("parent_birth_day", parent.to_string()),
                    ("child_birth_day", child.to_string()),
                ],
            ));
        }
    }
}

fn causes_and_effects(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let start_of: BTreeMap<EntityId, i64> = context
        .snapshot
        .events
        .iter()
        .filter_map(|detail| Some((detail.entity, days_of(&detail.period)?.0)))
        .collect();
    for link in &context.snapshot.causalities {
        if !context.counts_as_canon(&link.canon_state) {
            continue;
        }
        let (Some(cause), Some(effect)) = (start_of.get(&link.cause), start_of.get(&link.effect))
        else {
            continue;
        };
        if effect < cause {
            findings.push(Finding::new(
                Rule::EffectBeforeCause,
                link.effect,
                vec![link.cause],
                &[
                    ("cause_day", cause.to_string()),
                    ("effect_day", effect.to_string()),
                ],
            ));
        }
    }
}

/// One participation of an entity in a dated event at a known place.
struct Presence {
    event: EntityId,
    place: EntityId,
    start: i64,
    end: i64,
}

fn two_places_at_once(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let mut by_entity: BTreeMap<EntityId, Vec<Presence>> = BTreeMap::new();
    for detail in &context.snapshot.events {
        let (Some(place), Some((start, end))) = (detail.location, days_of(&detail.period)) else {
            continue;
        };
        if !context.entity_counts(&detail.entity) {
            continue;
        }
        let participants: BTreeSet<EntityId> = detail
            .participants
            .iter()
            .map(|participant| participant.entity)
            .collect();
        for participant in participants {
            by_entity.entry(participant).or_default().push(Presence {
                event: detail.entity,
                place,
                start,
                end,
            });
        }
    }
    for (entity, mut presences) in by_entity {
        presences.sort_by_key(|presence| (presence.start, presence.end, presence.event));
        for (index, earlier) in presences.iter().enumerate() {
            for later in &presences[index + 1..] {
                if later.start > earlier.end {
                    break;
                }
                if later.place != earlier.place {
                    findings.push(Finding::new(
                        Rule::TwoPlacesAtOnce,
                        entity,
                        vec![earlier.event, later.event],
                        &[],
                    ));
                }
            }
        }
    }
}

fn office_seats(context: &Context<'_>, findings: &mut Vec<Finding>) {
    for office in &context.snapshot.offices {
        // Each tenure with a known start opens a seat; an end closes it the day after.
        let mut changes: Vec<(i64, i32)> = Vec::new();
        for tenure in context
            .snapshot
            .tenures
            .iter()
            .filter(|tenure| tenure.office == office.id)
            .filter(|tenure| context.counts_as_canon(&tenure.canon_state))
        {
            let Some(start) = tenure.period.start_day() else {
                continue;
            };
            changes.push((start, 1));
            if let Some(end) = tenure.period.end_day() {
                changes.push((end.saturating_add(1), -1));
            }
        }
        // Closings sort before openings on the same day, so a handover is not an overlap.
        changes.sort_unstable();
        let mut holders = 0;
        let mut most = 0;
        for (_, change) in changes {
            holders += change;
            most = most.max(holders);
        }
        let seats = office.seats.max(1);
        if most > seats {
            findings.push(Finding::new(
                Rule::OfficeOverfilled,
                office.body,
                Vec::new(),
                &[
                    ("office", office.title.clone()),
                    ("seats", seats.to_string()),
                    ("holders", most.to_string()),
                ],
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::{Rule, check_snapshot};
    use crate::being::{BeingProfile, FamilyTie};
    use crate::event::{CausalityKind, EventCausality};
    use crate::polity::{Office, OfficeTenure};
    use crate::temporal::{Instant, Temporal};

    fn lifetime(
        snapshot: &mut super::super::Snapshot,
        entity: crate::identifier::EntityId,
        birth: i64,
        death: i64,
    ) {
        let mut profile = BeingProfile::new(entity);
        profile.birth = Some(Instant::at_day(birth));
        profile.death = Some(Instant::at_day(death));
        snapshot.profiles.push(profile);
    }

    #[test]
    fn taking_part_in_an_event_after_dying_is_an_error() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        lifetime(&mut snapshot, ayla, 0, 50);
        add_event(&mut snapshot, "The coronation", on_day(80), &[ayla]);

        let report = check_snapshot(&snapshot);

        assert_eq!(rules_of(&report), vec![Rule::ParticipantAfterDeath]);
        assert_eq!(
            report.findings[0]
                .facts
                .get("death_day")
                .map(String::as_str),
            Some("50")
        );
    }

    #[test]
    fn taking_part_in_an_event_before_being_born_is_an_error() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        lifetime(&mut snapshot, ayla, 100, 200);
        add_event(&mut snapshot, "The omen", on_day(20), &[ayla]);

        assert_eq!(
            rules_of(&check_snapshot(&snapshot)),
            vec![Rule::ParticipantBeforeBirth]
        );
    }

    #[test]
    fn an_event_that_is_not_canon_is_not_held_to_the_rules() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        lifetime(&mut snapshot, ayla, 0, 50);
        let event = add_event(&mut snapshot, "A draft scene", on_day(80), &[ayla]);
        for entity in &mut snapshot.entities {
            if entity.id == event {
                entity.canon_state = key("provisional");
            }
        }

        assert!(check_snapshot(&snapshot).findings.is_empty());
    }

    #[test]
    fn a_parent_born_after_the_child_is_an_error() {
        let mut snapshot = universe();
        let parent = add_entity(&mut snapshot, "character", "Parent");
        let child = add_entity(&mut snapshot, "character", "Child");
        lifetime(&mut snapshot, parent, 300, 400);
        lifetime(&mut snapshot, child, 100, 200);
        snapshot
            .family_ties
            .push(FamilyTie::new(parent, child, key("biological"), now()));

        assert_eq!(
            rules_of(&check_snapshot(&snapshot)),
            vec![Rule::ParentNotOlder]
        );
    }

    #[test]
    fn an_effect_before_its_cause_is_an_error() {
        let mut snapshot = universe();
        let cause = add_event(&mut snapshot, "The spark", on_day(200), &[]);
        let effect = add_event(&mut snapshot, "The fire", on_day(100), &[]);
        snapshot.causalities.push(EventCausality::new(
            cause,
            effect,
            CausalityKind::Direct,
            now(),
        ));

        assert_eq!(
            rules_of(&check_snapshot(&snapshot)),
            vec![Rule::EffectBeforeCause]
        );
    }

    #[test]
    fn being_in_two_places_during_overlapping_events_is_a_warning() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let north = add_entity(&mut snapshot, "city", "Northmere");
        let south = add_entity(&mut snapshot, "city", "Southmere");
        let siege = add_event(&mut snapshot, "The siege", on_day(100), &[ayla]);
        let feast = add_event(&mut snapshot, "The feast", on_day(100), &[ayla]);
        for detail in &mut snapshot.events {
            if detail.entity == siege {
                detail.location = Some(north);
            } else if detail.entity == feast {
                detail.location = Some(south);
            }
        }

        let report = check_snapshot(&snapshot);

        assert_eq!(rules_of(&report), vec![Rule::TwoPlacesAtOnce]);
        assert_eq!(report.findings[0].subject, ayla);
    }

    #[test]
    fn an_office_with_one_seat_cannot_have_two_holders_at_once() {
        let mut snapshot = universe();
        let throne = add_entity(&mut snapshot, "realm", "Valdria");
        let (first, second) = (
            add_entity(&mut snapshot, "character", "First"),
            add_entity(&mut snapshot, "character", "Second"),
        );
        let office = Office::new(throne, "King", now());
        let office_id = office.id;
        snapshot.offices.push(office);
        for (holder, start, end) in [(first, 10, 50), (second, 40, 90)] {
            let mut tenure = OfficeTenure::new(office_id, holder, now());
            tenure.period = Temporal::Span {
                start: Some(Instant::at_day(start)),
                end: Some(Instant::at_day(end)),
            };
            snapshot.tenures.push(tenure);
        }

        assert_eq!(
            rules_of(&check_snapshot(&snapshot)),
            vec![Rule::OfficeOverfilled]
        );
    }

    #[test]
    fn a_handover_on_consecutive_days_is_not_an_overlap() {
        let mut snapshot = universe();
        let throne = add_entity(&mut snapshot, "realm", "Valdria");
        let (first, second) = (
            add_entity(&mut snapshot, "character", "First"),
            add_entity(&mut snapshot, "character", "Second"),
        );
        let office = Office::new(throne, "King", now());
        let office_id = office.id;
        snapshot.offices.push(office);
        for (holder, start, end) in [(first, 10, 50), (second, 51, 90)] {
            let mut tenure = OfficeTenure::new(office_id, holder, now());
            tenure.period = Temporal::Span {
                start: Some(Instant::at_day(start)),
                end: Some(Instant::at_day(end)),
            };
            snapshot.tenures.push(tenure);
        }

        assert!(check_snapshot(&snapshot).findings.is_empty());
    }
}
