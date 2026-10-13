//! Rules about what is left unfinished.

use std::collections::{BTreeMap, BTreeSet};

use super::{Context, Finding, Rule};
use crate::entity_type::EntityRole;
use crate::identifier::{EntityId, PropertyDefinitionId};
use crate::property_definition::PropertyOwner;
use crate::property_value::{PropertyContent, PropertyValueOwner};

pub(super) fn run(context: &Context<'_>, findings: &mut Vec<Finding>) {
    required_properties(context, findings);
    undated_events(context, findings);
    unconnected_entities(context, findings);
    duplicate_names(context, findings);
}

fn required_properties(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let recorded: BTreeSet<(EntityId, PropertyDefinitionId)> = context
        .snapshot
        .values
        .iter()
        .filter_map(|value| match value.owner {
            PropertyValueOwner::Entity(entity) => Some((entity, value.definition)),
            PropertyValueOwner::Relationship(_) => None,
        })
        .collect();
    for entity in &context.snapshot.entities {
        if entity.archived || !context.counts_as_canon(&entity.canon_state) {
            continue;
        }
        let chain: BTreeSet<_> = context
            .type_chain(&entity.entity_type)
            .iter()
            .map(|entity_type| &entity_type.key)
            .collect();
        for definition in &context.snapshot.definitions {
            let applies = match &definition.owner {
                PropertyOwner::EntityType(key) => chain.contains(key),
                PropertyOwner::RelationshipType(_) => false,
            };
            if applies && definition.required && !recorded.contains(&(entity.id, definition.id)) {
                findings.push(Finding::new(
                    Rule::RequiredPropertyMissing,
                    entity.id,
                    Vec::new(),
                    &[
                        ("property", definition.label.clone()),
                        ("key", definition.key.to_string()),
                    ],
                ));
            }
        }
    }
}

fn undated_events(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let details: BTreeMap<EntityId, &crate::event_detail::EventDetail> = context
        .snapshot
        .events
        .iter()
        .map(|detail| (detail.entity, detail))
        .collect();
    for entity in &context.snapshot.entities {
        if entity.archived || context.role_of(entity) != Some(EntityRole::Event) {
            continue;
        }
        let dated = details.get(&entity.id).is_some_and(|detail| {
            detail.period.start_day().is_some()
                || detail.period.end_day().is_some()
                || matches!(detail.period, crate::temporal::Temporal::Unconstrained)
        });
        if !dated {
            findings.push(Finding::new(Rule::EventUndated, entity.id, Vec::new(), &[]));
        }
    }
}

fn unconnected_entities(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let snapshot = context.snapshot;
    let mut connected: BTreeSet<EntityId> = BTreeSet::new();
    for relationship in &snapshot.relationships {
        connected.insert(relationship.source);
        connected.insert(relationship.target);
        connected.extend(relationship.context);
    }
    for appearance in &snapshot.appearances {
        connected.insert(appearance.entity);
        connected.insert(appearance.context);
        connected.extend(appearance.viewpoint);
    }
    for detail in &snapshot.events {
        connected.insert(detail.entity);
        connected.extend(detail.location);
        connected.extend(detail.participants.iter().map(|p| p.entity));
    }
    for attachment in &snapshot.attachments {
        connected.insert(attachment.owner);
        connected.insert(attachment.asset);
    }
    for tenure in &snapshot.tenures {
        connected.insert(tenure.holder);
    }
    for office in &snapshot.offices {
        connected.insert(office.body);
    }
    for tie in &snapshot.family_ties {
        connected.insert(tie.parent);
        connected.insert(tie.child);
    }
    for causality in &snapshot.causalities {
        connected.insert(causality.cause);
        connected.insert(causality.effect);
    }
    for node in &snapshot.nodes {
        connected.insert(node.entity);
        connected.extend(node.parent);
    }
    for value in &snapshot.values {
        if let PropertyContent::EntityReference(target) = value.content {
            connected.insert(target);
            if let PropertyValueOwner::Entity(owner) = value.owner {
                connected.insert(owner);
            }
        }
    }
    for entity in &snapshot.entities {
        if entity.archived
            || context.role_of(entity) != Some(EntityRole::Generic)
            || connected.contains(&entity.id)
        {
            continue;
        }
        findings.push(Finding::new(
            Rule::NoConnections,
            entity.id,
            Vec::new(),
            &[],
        ));
    }
}

fn duplicate_names(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let mut groups: BTreeMap<(String, String), Vec<EntityId>> = BTreeMap::new();
    for entity in context
        .snapshot
        .entities
        .iter()
        .filter(|entity| !entity.archived)
    {
        let name = entity
            .name
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if name.is_empty() {
            continue;
        }
        groups
            .entry((entity.entity_type.to_string(), name))
            .or_default()
            .push(entity.id);
    }
    for (_, mut ids) in groups {
        if ids.len() < 2 {
            continue;
        }
        ids.sort_unstable();
        let subject = ids[0];
        findings.push(Finding::new(
            Rule::DuplicateName,
            subject,
            ids[1..].to_vec(),
            &[],
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::{Rule, check_snapshot};
    use crate::property_definition::{PropertyDataType, PropertyDefinition, PropertyOwner};
    use crate::property_value::{PropertyContent, PropertyValue, PropertyValueOwner};

    fn definition(required: bool) -> PropertyDefinition {
        let mut definition = PropertyDefinition::custom(
            PropertyOwner::EntityType(key("being")),
            key("guild_oath"),
            "Guild oath",
            PropertyDataType::Text,
        );
        definition.required = required;
        definition
    }

    #[test]
    fn a_required_property_inherited_from_a_parent_type_must_have_a_value() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let orin = add_entity(&mut snapshot, "character", "Orin");
        let oath = definition(true);
        snapshot.values.push(PropertyValue::new(
            PropertyValueOwner::Entity(orin),
            oath.id,
            PropertyContent::Text("Never lie".to_owned()),
            now(),
        ));
        snapshot.definitions.push(oath);
        // Both entities are connected so only the missing value is reported.
        snapshot
            .relationships
            .push(crate::relationship::Relationship::new(
                key("mentor_of"),
                ayla,
                orin,
                now(),
            ));

        let report = check_snapshot(&snapshot);

        assert_eq!(rules_of(&report), vec![Rule::RequiredPropertyMissing]);
        assert_eq!(report.findings[0].subject, ayla);
    }

    #[test]
    fn an_optional_property_is_never_reported() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let city = add_entity(&mut snapshot, "city", "Northmere");
        snapshot.definitions.push(definition(false));
        snapshot
            .relationships
            .push(crate::relationship::Relationship::new(
                key("located_within"),
                ayla,
                city,
                now(),
            ));

        assert!(check_snapshot(&snapshot).findings.is_empty());
    }

    #[test]
    fn two_entities_of_one_type_with_the_same_name_are_reported_once() {
        let mut snapshot = universe();
        let first = add_entity(&mut snapshot, "character", "Ayla");
        let second = add_entity(&mut snapshot, "character", "  ayla ");
        let place = add_entity(&mut snapshot, "city", "Ayla");
        for entity in [first, second, place] {
            snapshot
                .relationships
                .push(crate::relationship::Relationship::new(
                    key("located_within"),
                    entity,
                    place,
                    now(),
                ));
        }

        let report = check_snapshot(&snapshot);
        let duplicates: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| finding.rule == Rule::DuplicateName)
            .collect();

        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].related.len(), 1);
    }

    #[test]
    fn an_event_without_a_day_and_an_entity_without_links_are_housekeeping() {
        let mut snapshot = universe();
        add_event(
            &mut snapshot,
            "Someday",
            crate::temporal::Temporal::Unknown,
            &[],
        );
        add_entity(&mut snapshot, "character", "Wanderer");

        let report = check_snapshot(&snapshot);

        assert_eq!(report.counts.infos, 2);
        assert_eq!(report.counts.errors + report.counts.warnings, 0);
    }
}
