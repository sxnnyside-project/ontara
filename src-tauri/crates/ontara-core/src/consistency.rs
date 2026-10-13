//! Deterministic consistency checks.
//!
//! A check reads the model and reports findings; it never changes a record, and it never decides
//! what is canon. The same data always produces the same findings in the same order. Records that
//! do not count as canon (a provisional idea, a rejected draft) are not held to the rules.

mod canon;
mod chronology;
mod completeness;
mod narrative;

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::appearance::Appearance;
use crate::appearance_kind::AppearanceKind;
use crate::asset::Attachment;
use crate::being::{BeingProfile, FamilyTie};
use crate::canon_state::CanonState;
use crate::editorial_node::EditorialNode;
use crate::entity::Entity;
use crate::entity_type::EntityType;
use crate::event::EventCausality;
use crate::event_detail::EventDetail;
use crate::identifier::EntityId;
use crate::polity::{Office, OfficeTenure};
use crate::property_definition::PropertyDefinition;
use crate::property_value::PropertyValue;
use crate::relationship::Relationship;
use crate::repository::{ModelReader, Reader};
use crate::type_key::TypeKey;

/// How much a finding matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The model contradicts itself.
    Error,
    /// Something is probably wrong or at risk.
    Warning,
    /// Housekeeping: something left unfinished.
    Info,
}

/// One kind of check. The code is stable and is what an author sees next to a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// An event is dated after a participant's death.
    ParticipantAfterDeath,
    /// An event is dated before a participant's birth.
    ParticipantBeforeBirth,
    /// A parent was born on or after the day their child was.
    ParentNotOlder,
    /// An effect happens before its cause.
    EffectBeforeCause,
    /// An entity is both present and confirmed absent in the same context.
    PresenceAndAbsence,
    /// An entity takes part in overlapping events at different places.
    TwoPlacesAtOnce,
    /// A canon record rests on an entity that is not canon.
    CanonOnUncanon,
    /// A required property has no value.
    RequiredPropertyMissing,
    /// A scene is told through someone who is not present in it.
    ViewpointAbsent,
    /// More people hold an office at once than it has seats.
    OfficeOverfilled,
    /// A chapter, scene or other editorial unit is not placed in the structure.
    SectionNotPlaced,
    /// An event has no known day.
    EventUndated,
    /// An entity is not connected to anything.
    NoConnections,
    /// Two entities of one type share a name.
    DuplicateName,
}

impl Rule {
    pub const ALL: [Self; 14] = [
        Self::ParticipantAfterDeath,
        Self::ParticipantBeforeBirth,
        Self::ParentNotOlder,
        Self::EffectBeforeCause,
        Self::PresenceAndAbsence,
        Self::TwoPlacesAtOnce,
        Self::CanonOnUncanon,
        Self::RequiredPropertyMissing,
        Self::ViewpointAbsent,
        Self::OfficeOverfilled,
        Self::SectionNotPlaced,
        Self::EventUndated,
        Self::NoConnections,
        Self::DuplicateName,
    ];

    /// The stable code shown with a finding.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ParticipantAfterDeath => "CC-01",
            Self::ParticipantBeforeBirth => "CC-02",
            Self::ParentNotOlder => "CC-03",
            Self::EffectBeforeCause => "CC-04",
            Self::PresenceAndAbsence => "CC-05",
            Self::TwoPlacesAtOnce => "CC-06",
            Self::CanonOnUncanon => "CC-07",
            Self::RequiredPropertyMissing => "CC-08",
            Self::ViewpointAbsent => "CC-09",
            Self::OfficeOverfilled => "CC-10",
            Self::SectionNotPlaced => "CC-11",
            Self::EventUndated => "CC-12",
            Self::NoConnections => "CC-13",
            Self::DuplicateName => "CC-14",
        }
    }

    #[must_use]
    pub const fn severity(self) -> Severity {
        match self {
            Self::ParticipantAfterDeath
            | Self::ParticipantBeforeBirth
            | Self::ParentNotOlder
            | Self::EffectBeforeCause
            | Self::PresenceAndAbsence => Severity::Error,
            Self::TwoPlacesAtOnce
            | Self::CanonOnUncanon
            | Self::RequiredPropertyMissing
            | Self::ViewpointAbsent
            | Self::OfficeOverfilled
            | Self::SectionNotPlaced
            | Self::DuplicateName => Severity::Warning,
            Self::EventUndated | Self::NoConnections => Severity::Info,
        }
    }
}

/// What a check found: the rule, the entity to open to deal with it, and what else is involved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// Stable for the same finding over the same data, so a finding can be told apart and kept.
    pub fingerprint: String,
    pub rule: Rule,
    pub code: &'static str,
    pub severity: Severity,
    pub subject: EntityId,
    pub related: Vec<EntityId>,
    /// Values the message needs, such as days or titles.
    pub facts: BTreeMap<&'static str, String>,
}

impl Finding {
    pub(crate) fn new(
        rule: Rule,
        subject: EntityId,
        related: Vec<EntityId>,
        facts: &[(&'static str, String)],
    ) -> Self {
        let parts: Vec<String> = [format!("{}:{subject}", rule.code())]
            .into_iter()
            .chain(related.iter().map(ToString::to_string))
            .chain(facts.iter().map(|(key, value)| format!("{key}={value}")))
            .collect();
        let fingerprint = parts.join(":");
        Self {
            fingerprint,
            rule,
            code: rule.code(),
            severity: rule.severity(),
            subject,
            related,
            facts: facts.iter().cloned().collect(),
        }
    }
}

/// How many findings of each severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct SeverityCounts {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

/// How much was examined, so "no findings" can be told from "nothing to check".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Examined {
    pub entities: usize,
    pub relationships: usize,
    pub appearances: usize,
    pub events: usize,
}

/// The result of running every check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsistencyReport {
    pub findings: Vec<Finding>,
    pub counts: SeverityCounts,
    pub examined: Examined,
}

/// The records the checks read, so they can run on a repository or on records made by hand.
#[derive(Debug, Default)]
pub struct Snapshot {
    pub entities: Vec<Entity>,
    pub entity_types: Vec<EntityType>,
    pub canon_states: Vec<CanonState>,
    pub appearance_kinds: Vec<AppearanceKind>,
    pub relationships: Vec<Relationship>,
    pub appearances: Vec<Appearance>,
    pub events: Vec<EventDetail>,
    pub nodes: Vec<EditorialNode>,
    pub causalities: Vec<EventCausality>,
    pub profiles: Vec<BeingProfile>,
    pub family_ties: Vec<FamilyTie>,
    pub offices: Vec<Office>,
    pub tenures: Vec<OfficeTenure>,
    pub definitions: Vec<PropertyDefinition>,
    pub values: Vec<PropertyValue>,
    pub attachments: Vec<Attachment>,
}

impl Snapshot {
    /// Reads everything the checks need.
    pub fn load<R: ModelReader>(reader: &R) -> Result<Self, R::Error> {
        Ok(Self {
            entities: Reader::<Entity>::list(reader)?,
            entity_types: Reader::<EntityType>::list(reader)?,
            canon_states: Reader::<CanonState>::list(reader)?,
            appearance_kinds: Reader::<AppearanceKind>::list(reader)?,
            relationships: Reader::<Relationship>::list(reader)?,
            appearances: Reader::<Appearance>::list(reader)?,
            events: Reader::<EventDetail>::list(reader)?,
            nodes: Reader::<EditorialNode>::list(reader)?,
            causalities: Reader::<EventCausality>::list(reader)?,
            profiles: Reader::<BeingProfile>::list(reader)?,
            family_ties: Reader::<FamilyTie>::list(reader)?,
            offices: Reader::<Office>::list(reader)?,
            tenures: Reader::<OfficeTenure>::list(reader)?,
            definitions: Reader::<PropertyDefinition>::list(reader)?,
            values: Reader::<PropertyValue>::list(reader)?,
            attachments: Reader::<Attachment>::list(reader)?,
        })
    }
}

/// The snapshot with lookups the checks share.
pub(crate) struct Context<'a> {
    pub snapshot: &'a Snapshot,
    pub entities: BTreeMap<EntityId, &'a Entity>,
    pub types: BTreeMap<&'a TypeKey, &'a EntityType>,
    pub profiles: BTreeMap<EntityId, &'a BeingProfile>,
    canon: BTreeMap<&'a TypeKey, bool>,
    presence: BTreeMap<&'a TypeKey, bool>,
}

impl<'a> Context<'a> {
    fn new(snapshot: &'a Snapshot) -> Self {
        Self {
            snapshot,
            entities: snapshot
                .entities
                .iter()
                .map(|entity| (entity.id, entity))
                .collect(),
            types: snapshot
                .entity_types
                .iter()
                .map(|entity_type| (&entity_type.key, entity_type))
                .collect(),
            profiles: snapshot
                .profiles
                .iter()
                .map(|profile| (profile.entity, profile))
                .collect(),
            canon: snapshot
                .canon_states
                .iter()
                .map(|state| (&state.key, state.counts_as_canon))
                .collect(),
            presence: snapshot
                .appearance_kinds
                .iter()
                .map(|kind| (&kind.key, kind.is_presence))
                .collect(),
        }
    }

    /// Whether records in the state are held to the rules. An unknown state counts.
    pub fn counts_as_canon(&self, state: &TypeKey) -> bool {
        self.canon.get(state).copied().unwrap_or(true)
    }

    /// Whether the entity exists, is not archived and is canon.
    pub fn entity_counts(&self, id: &EntityId) -> bool {
        self.entities
            .get(id)
            .is_some_and(|entity| !entity.archived && self.counts_as_canon(&entity.canon_state))
    }

    pub fn is_presence(&self, kind: &TypeKey) -> bool {
        self.presence.get(kind).copied().unwrap_or(true)
    }

    /// The type and its ancestors, nearest first. A broken hierarchy ends the chain.
    pub fn type_chain(&self, key: &TypeKey) -> Vec<&'a EntityType> {
        let mut chain: Vec<&EntityType> = Vec::new();
        let mut next = Some(key);
        while let Some(current) = next {
            let Some(found) = self.types.get(current) else {
                break;
            };
            if chain.iter().any(|seen| seen.key == found.key) {
                break;
            }
            chain.push(found);
            next = found.parent.as_ref();
        }
        chain
    }

    pub fn role_of(&self, entity: &Entity) -> Option<crate::entity_type::EntityRole> {
        self.types.get(&entity.entity_type).map(|found| found.role)
    }
}

/// Runs every check over the snapshot. Findings are ordered by severity, rule, then entity, so the
/// same data always gives the same report.
#[must_use]
pub fn check_snapshot(snapshot: &Snapshot) -> ConsistencyReport {
    let context = Context::new(snapshot);
    let mut findings = Vec::new();
    chronology::run(&context, &mut findings);
    narrative::run(&context, &mut findings);
    canon::run(&context, &mut findings);
    completeness::run(&context, &mut findings);

    let mut seen = BTreeSet::new();
    findings.retain(|finding| seen.insert(finding.fingerprint.clone()));
    findings.sort_by(|left, right| {
        (left.severity, left.rule, left.subject, &left.fingerprint).cmp(&(
            right.severity,
            right.rule,
            right.subject,
            &right.fingerprint,
        ))
    });

    let mut counts = SeverityCounts::default();
    for finding in &findings {
        match finding.severity {
            Severity::Error => counts.errors += 1,
            Severity::Warning => counts.warnings += 1,
            Severity::Info => counts.infos += 1,
        }
    }
    ConsistencyReport {
        findings,
        counts,
        examined: Examined {
            entities: snapshot.entities.len(),
            relationships: snapshot.relationships.len(),
            appearances: snapshot.appearances.len(),
            events: snapshot.events.len(),
        },
    }
}

/// Reads the model and runs every check.
pub fn check_consistency<R: ModelReader>(reader: &R) -> Result<ConsistencyReport, R::Error> {
    Ok(check_snapshot(&Snapshot::load(reader)?))
}

#[cfg(test)]
pub(crate) mod fixtures {
    //! Records made by hand for the rule tests.

    use super::*;
    use crate::appearance::Appearance;
    use crate::canon_state::builtin_canon_states;
    use crate::entity_type::builtin_entity_types;
    use crate::event_detail::{EventDetail, EventParticipant};
    use crate::temporal::{Instant, Temporal};
    use crate::timestamp::Timestamp;

    pub fn key(raw: &str) -> TypeKey {
        TypeKey::new(raw).unwrap_or_else(|error| panic!("{error}"))
    }

    pub fn now() -> Timestamp {
        Timestamp::from_unix_millis(0)
    }

    /// A snapshot that knows the built-in types, canon states and appearance kinds.
    pub fn universe() -> Snapshot {
        Snapshot {
            entity_types: builtin_entity_types(),
            canon_states: builtin_canon_states(),
            appearance_kinds: crate::appearance_kind::builtin_appearance_kinds(),
            ..Snapshot::default()
        }
    }

    pub fn add_entity(snapshot: &mut Snapshot, entity_type: &str, name: &str) -> EntityId {
        let entity = Entity::new(key(entity_type), name, now());
        let id = entity.id;
        snapshot.entities.push(entity);
        id
    }

    pub fn on_day(day: i64) -> Temporal {
        Temporal::At {
            instant: Instant::at_day(day),
        }
    }

    pub fn add_event(
        snapshot: &mut Snapshot,
        name: &str,
        period: Temporal,
        participants: &[EntityId],
    ) -> EntityId {
        let id = add_entity(snapshot, "event", name);
        let mut detail = EventDetail::new(id);
        detail.period = period;
        detail.participants = participants
            .iter()
            .map(|entity| EventParticipant::new(*entity))
            .collect();
        snapshot.events.push(detail);
        id
    }

    pub fn add_appearance(
        snapshot: &mut Snapshot,
        entity: EntityId,
        context: EntityId,
        kind: &str,
    ) -> Appearance {
        let appearance = Appearance::new(entity, context, key(kind), now());
        snapshot.appearances.push(appearance.clone());
        appearance
    }

    pub fn rules_of(report: &ConsistencyReport) -> Vec<Rule> {
        report.findings.iter().map(|finding| finding.rule).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;
    use crate::temporal::Temporal;

    #[test]
    fn a_consistent_universe_has_no_errors() {
        let mut snapshot = universe();
        let ayla = add_entity(&mut snapshot, "character", "Ayla");
        let orin = add_entity(&mut snapshot, "character", "Orin");
        add_event(&mut snapshot, "The crossing", on_day(10), &[ayla, orin]);

        let report = check_snapshot(&snapshot);

        assert_eq!(report.counts.errors, 0);
        assert_eq!(report.examined.entities, 3);
    }

    #[test]
    fn the_same_data_gives_the_same_report() {
        let mut snapshot = universe();
        let first = add_entity(&mut snapshot, "character", "Ayla");
        let second = add_entity(&mut snapshot, "character", "Ayla");
        add_event(
            &mut snapshot,
            "Undated",
            Temporal::Unknown,
            &[first, second],
        );

        assert_eq!(check_snapshot(&snapshot), check_snapshot(&snapshot));
    }
}
