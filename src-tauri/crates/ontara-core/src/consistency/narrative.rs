//! Rules about how the story presents the universe.

use std::collections::{BTreeMap, BTreeSet};

use super::{Context, Finding, Rule};
use crate::entity_type::EntityRole;
use crate::identifier::EntityId;

pub(super) fn run(context: &Context<'_>, findings: &mut Vec<Finding>) {
    presence_and_absence(context, findings);
    viewpoint_absent(context, findings);
    sections_not_placed(context, findings);
}

fn presence_and_absence(context: &Context<'_>, findings: &mut Vec<Finding>) {
    // For each entity in each context: whether it is recorded present, and whether absent.
    let mut recorded: BTreeMap<(EntityId, EntityId), (bool, bool)> = BTreeMap::new();
    for appearance in &context.snapshot.appearances {
        if !context.counts_as_canon(&appearance.canon_state) {
            continue;
        }
        let entry = recorded
            .entry((appearance.entity, appearance.context))
            .or_default();
        if context.is_presence(&appearance.kind) {
            entry.0 = true;
        } else {
            entry.1 = true;
        }
    }
    for ((entity, place), (present, absent)) in recorded {
        if present && absent {
            findings.push(Finding::new(
                Rule::PresenceAndAbsence,
                entity,
                vec![place],
                &[],
            ));
        }
    }
}

fn viewpoint_absent(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let present: BTreeSet<(EntityId, EntityId)> = context
        .snapshot
        .appearances
        .iter()
        .filter(|appearance| context.is_presence(&appearance.kind))
        .map(|appearance| (appearance.entity, appearance.context))
        .collect();
    let mut reported = BTreeSet::new();
    for appearance in &context.snapshot.appearances {
        let Some(viewpoint) = appearance.viewpoint else {
            continue;
        };
        if !context.counts_as_canon(&appearance.canon_state)
            || present.contains(&(viewpoint, appearance.context))
            || !reported.insert((appearance.context, viewpoint))
        {
            continue;
        }
        findings.push(Finding::new(
            Rule::ViewpointAbsent,
            appearance.context,
            vec![viewpoint],
            &[],
        ));
    }
}

fn sections_not_placed(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let placed: BTreeSet<EntityId> = context
        .snapshot
        .nodes
        .iter()
        .map(|node| node.entity)
        .collect();
    for entity in &context.snapshot.entities {
        if entity.archived
            || context.role_of(entity) != Some(EntityRole::EditorialUnit)
            || placed.contains(&entity.id)
        {
            continue;
        }
        findings.push(Finding::new(
            Rule::SectionNotPlaced,
            entity.id,
            Vec::new(),
            &[],
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::{Rule, check_snapshot};
    use crate::editorial_node::EditorialNode;

    #[test]
    fn being_present_and_confirmed_absent_in_one_scene_is_an_error() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let scene = add_entity(&mut snapshot, "scene", "The gate");
        snapshot.nodes.push(EditorialNode {
            entity: scene,
            level: key("scene"),
            parent: None,
            position: 0,
        });
        add_appearance(&mut snapshot, ayla, scene, "physical_presence");
        add_appearance(&mut snapshot, ayla, scene, "confirmed_absence");

        assert_eq!(
            rules_of(&check_snapshot(&snapshot)),
            vec![Rule::PresenceAndAbsence]
        );
    }

    #[test]
    fn a_scene_told_through_someone_who_is_not_in_it_is_a_warning() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let orin = add_entity(&mut snapshot, "character", "Orin");
        let scene = add_entity(&mut snapshot, "scene", "The gate");
        snapshot.nodes.push(EditorialNode {
            entity: scene,
            level: key("scene"),
            parent: None,
            position: 0,
        });
        let mut appearance = add_appearance(&mut snapshot, ayla, scene, "physical_presence");
        appearance.viewpoint = Some(orin);
        snapshot.appearances[0] = appearance;

        let report = check_snapshot(&snapshot);

        assert_eq!(rules_of(&report), vec![Rule::ViewpointAbsent]);
        assert_eq!(report.findings[0].subject, scene);
    }

    #[test]
    fn a_chapter_outside_the_structure_is_reported() {
        let mut snapshot = universe();
        let chapter = add_entity(&mut snapshot, "chapter", "Lost chapter");

        let report = check_snapshot(&snapshot);

        assert_eq!(rules_of(&report), vec![Rule::SectionNotPlaced]);
        assert_eq!(report.findings[0].subject, chapter);
    }
}
