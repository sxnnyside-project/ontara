use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::record::Record;
use crate::type_key::TypeKey;

/// Whether a relationship has a meaningful direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Directionality {
    Directed,
    Undirected,
}

impl Directionality {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Directed => "directed",
            Self::Undirected => "undirected",
        }
    }
}

impl fmt::Display for Directionality {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Directionality {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "directed" => Ok(Self::Directed),
            "undirected" => Ok(Self::Undirected),
            other => Err(DomainError::UnknownValue(
                "directionality",
                other.to_owned(),
            )),
        }
    }
}

/// The kind of a relationship between two entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipType {
    pub key: TypeKey,
    /// Reads from source to target: "Is the parent of".
    pub label: String,
    /// Reads from target to source: "Is a child of". Only directed types have one.
    pub inverse_label: Option<String>,
    pub description: String,
    pub directionality: Directionality,
    /// A hierarchical type: following it from entity to entity can never lead back, so it
    /// describes containment, succession or descent rather than a loose connection.
    pub acyclic: bool,
    /// Entity types allowed as the source, subtypes included. Empty means any type.
    pub allowed_sources: BTreeSet<TypeKey>,
    /// Entity types allowed as the target, subtypes included. Empty means any type.
    pub allowed_targets: BTreeSet<TypeKey>,
    pub sort_order: i32,
    pub builtin: bool,
}

impl RelationshipType {
    /// A project-specific type that connects any entities.
    #[must_use]
    pub fn custom(key: TypeKey, label: impl Into<String>, directionality: Directionality) -> Self {
        Self {
            key,
            label: label.into(),
            inverse_label: None,
            description: String::new(),
            directionality,
            acyclic: false,
            allowed_sources: BTreeSet::new(),
            allowed_targets: BTreeSet::new(),
            sort_order: 0,
            builtin: false,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.label.trim().is_empty() {
            return Err(DomainError::BlankText("relationship type label"));
        }
        if self
            .inverse_label
            .as_deref()
            .is_some_and(|label| label.trim().is_empty())
        {
            return Err(DomainError::BlankText("inverse label"));
        }
        if self.directionality == Directionality::Undirected {
            if self.inverse_label.is_some() {
                return Err(DomainError::UnknownValue(
                    "undirected relationship type",
                    "an undirected type has no inverse label".to_owned(),
                ));
            }
            if self.acyclic {
                return Err(DomainError::UnknownValue(
                    "undirected relationship type",
                    "an undirected type cannot be hierarchical".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

impl Record for RelationshipType {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

use Directionality::{Directed, Undirected};

/// (key, label, inverse label, directionality, hierarchical, allowed sources, allowed targets)
type BuiltinRelationship = (
    &'static str,
    &'static str,
    Option<&'static str>,
    Directionality,
    bool,
    &'static [&'static str],
    &'static [&'static str],
);

#[rustfmt::skip]
const BUILTIN_RELATIONSHIP_TYPES: [BuiltinRelationship; 26] = [
    ("parent_of", "Is the parent of", Some("Is a child of"), Directed, true, &["being"], &["being"]),
    ("sibling_of", "Is a sibling of", None, Undirected, false, &["being"], &["being"]),
    ("spouse_of", "Is the spouse of", None, Undirected, false, &["being"], &["being"]),
    ("mentor_of", "Mentors", Some("Is mentored by"), Directed, false, &["being"], &["being"]),
    ("related_to", "Is related to", None, Undirected, false, &[], &[]),
    ("affiliated_with", "Is affiliated with", Some("Has as a member"), Directed, false, &["being"], &["faction"]),
    ("rules_over", "Rules over", Some("Is ruled by"), Directed, false, &["being", "faction"], &["place", "faction"]),
    ("located_within", "Is located within", Some("Contains"), Directed, true, &[], &["place"]),
    ("capital_of", "Is the capital of", Some("Has as its capital"), Directed, false, &["settlement"], &["polity"]),
    ("borders", "Borders", None, Undirected, false, &["place"], &["place"]),
    ("vassal_of", "Is a vassal of", Some("Is the suzerain of"), Directed, true, &["polity", "faction"], &["polity", "faction"]),
    ("successor_of", "Is the successor of", Some("Was succeeded by"), Directed, true, &["polity", "faction"], &["polity", "faction"]),
    ("allied_with", "Is allied with", None, Undirected, false, &["polity", "faction"], &["polity", "faction"]),
    ("at_war_with", "Is at war with", None, Undirected, false, &["polity", "faction"], &["polity", "faction"]),
    ("born_in", "Was born in", Some("Is the birthplace of"), Directed, false, &["being"], &["place"]),
    ("resides_in", "Resides in", Some("Is the home of"), Directed, false, &["being", "faction"], &["place"]),
    ("owns", "Owns", Some("Is owned by"), Directed, false, &["being", "faction"], &["item"]),
    ("worships", "Worships", Some("Is worshipped by"), Directed, false, &["being", "culture"], &["deity"]),
    ("follows", "Follows", Some("Is followed by"), Directed, false, &["being", "culture", "polity"], &["religion"]),
    ("speaks", "Speaks", Some("Is spoken by"), Directed, false, &["being", "culture", "polity"], &["language"]),
    ("participated_in", "Participated in", Some("Had as a participant"), Directed, false, &["being", "faction"], &["event"]),
    ("caused", "Caused", Some("Was caused by"), Directed, false, &["being", "faction", "event"], &["event"]),
    ("influenced", "Influenced", Some("Was influenced by"), Directed, false, &[], &[]),
    ("reincarnation_of", "Is the reincarnation of", Some("Was reincarnated as"), Directed, true, &["being"], &["being"]),
    ("set_in", "Is set in", Some("Is the setting of"), Directed, false, &["series", "volume", "part", "chapter", "scene"], &["place"]),
    ("depicts", "Depicts", Some("Is depicted in"), Directed, false, &["series", "volume", "part", "chapter", "scene"], &["event"]),
];

/// The relationship types every new project starts with.
///
/// "Appears in" is deliberately absent: an appearance is its own record, not a relationship.
#[must_use]
pub fn builtin_relationship_types() -> Vec<RelationshipType> {
    BUILTIN_RELATIONSHIP_TYPES
        .iter()
        .zip(0..)
        .map(
            |(&(key, label, inverse, directionality, acyclic, sources, targets), sort_order)| {
                RelationshipType {
                    key: TypeKey::from_static(key),
                    label: label.to_owned(),
                    inverse_label: inverse.map(str::to_owned),
                    description: String::new(),
                    directionality,
                    acyclic,
                    allowed_sources: sources.iter().map(|k| TypeKey::from_static(k)).collect(),
                    allowed_targets: targets.iter().map(|k| TypeKey::from_static(k)).collect(),
                    sort_order,
                    builtin: true,
                }
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_type::builtin_entity_types;

    #[test]
    fn builtin_types_are_valid_and_unique() {
        let types = builtin_relationship_types();

        for relationship_type in &types {
            assert!(TypeKey::new(relationship_type.key.as_str()).is_ok());
            assert_eq!(relationship_type.validate(), Ok(()));
        }
        let mut keys: Vec<_> = types.iter().map(|t| t.key.clone()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), types.len());
    }

    #[test]
    fn endpoint_rules_only_name_defined_entity_types() {
        let entity_types: BTreeSet<_> = builtin_entity_types().into_iter().map(|t| t.key).collect();

        for relationship_type in builtin_relationship_types() {
            for key in relationship_type
                .allowed_sources
                .iter()
                .chain(&relationship_type.allowed_targets)
            {
                assert!(
                    entity_types.contains(key),
                    "{} names undefined type {key}",
                    relationship_type.key
                );
            }
        }
    }

    #[test]
    fn containment_and_descent_are_hierarchical() {
        let hierarchical: Vec<_> = builtin_relationship_types()
            .into_iter()
            .filter(|t| t.acyclic)
            .map(|t| t.key.to_string())
            .collect();

        for expected in ["located_within", "parent_of", "vassal_of", "successor_of"] {
            assert!(
                hierarchical.iter().any(|key| key == expected),
                "{expected} must be hierarchical"
            );
        }
        assert!(!hierarchical.iter().any(|key| key == "borders"));
    }

    #[test]
    fn directed_types_read_both_ways_and_undirected_ones_do_not() {
        let types = builtin_relationship_types();
        let find = |key: &str| types.iter().find(|t| t.key.as_str() == key);

        assert_eq!(
            find("located_within").and_then(|t| t.inverse_label.as_deref()),
            Some("Contains")
        );
        assert_eq!(
            find("borders").map(|t| t.directionality),
            Some(Directionality::Undirected)
        );
    }

    #[test]
    fn an_undirected_type_cannot_be_hierarchical_or_have_an_inverse_label() {
        let mut relationship_type = RelationshipType::custom(
            TypeKey::from_static("rival_of"),
            "Is a rival of",
            Directionality::Undirected,
        );
        relationship_type.inverse_label = Some("Has as a rival".to_owned());
        assert!(relationship_type.validate().is_err());

        relationship_type.inverse_label = None;
        relationship_type.acyclic = true;
        assert!(relationship_type.validate().is_err());
    }
}
