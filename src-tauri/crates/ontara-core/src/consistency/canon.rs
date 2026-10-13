//! Rules about canon resting on what is not canon.

use super::{Context, Finding, Rule};
use crate::identifier::EntityId;

pub(super) fn run(context: &Context<'_>, findings: &mut Vec<Finding>) {
    let name_of = |id: &EntityId| {
        context
            .entities
            .get(id)
            .map(|entity| entity.canon_state.to_string())
            .unwrap_or_default()
    };
    // A canon record may only stand on entities that exist, are visible and are canon. A missing
    // entity is the repository's business; here only existing entities that are not canon count.
    let shaky = |id: &EntityId| {
        context
            .entities
            .get(id)
            .is_some_and(|entity| !entity.archived && !context.counts_as_canon(&entity.canon_state))
    };

    for relationship in &context.snapshot.relationships {
        if !context.counts_as_canon(&relationship.canon_state) {
            continue;
        }
        for (subject, other) in [
            (relationship.source, relationship.target),
            (relationship.target, relationship.source),
        ] {
            if shaky(&subject) {
                findings.push(Finding::new(
                    Rule::CanonOnUncanon,
                    other,
                    vec![subject],
                    &[
                        ("record", "relationship".to_owned()),
                        ("state", name_of(&subject)),
                    ],
                ));
            }
        }
    }

    for appearance in &context.snapshot.appearances {
        if !context.counts_as_canon(&appearance.canon_state) {
            continue;
        }
        for (subject, other) in [
            (appearance.entity, appearance.context),
            (appearance.context, appearance.entity),
        ] {
            if shaky(&subject) {
                findings.push(Finding::new(
                    Rule::CanonOnUncanon,
                    other,
                    vec![subject],
                    &[
                        ("record", "appearance".to_owned()),
                        ("state", name_of(&subject)),
                    ],
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixtures::*;
    use super::super::{Rule, check_snapshot};
    use crate::relationship::Relationship;

    #[test]
    fn an_established_relationship_to_a_rejected_entity_is_a_warning() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let ghost = add_entity(&mut snapshot, "character", "Ghost");
        for entity in &mut snapshot.entities {
            if entity.id == ghost {
                entity.canon_state = key("rejected");
            }
        }
        snapshot
            .relationships
            .push(Relationship::new(key("mentor_of"), ayla, ghost, now()));

        let report = check_snapshot(&snapshot);

        assert_eq!(rules_of(&report), vec![Rule::CanonOnUncanon]);
        assert_eq!(report.findings[0].subject, ayla);
    }
}
