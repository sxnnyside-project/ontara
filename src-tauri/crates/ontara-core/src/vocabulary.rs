use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::record::Record;
use crate::type_key::TypeKey;

/// The sets of terms an author may extend. Each one is a table of its own in storage, so every
/// use of a term is a reference to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vocabulary {
    TraitKind,
    ContainmentKind,
    FamilyTieKind,
    PartnershipKind,
    OfficeAcquisition,
    ItemAcquisition,
    CitizenshipStatus,
}

impl Vocabulary {
    pub const ALL: [Self; 7] = [
        Self::TraitKind,
        Self::ContainmentKind,
        Self::FamilyTieKind,
        Self::PartnershipKind,
        Self::OfficeAcquisition,
        Self::ItemAcquisition,
        Self::CitizenshipStatus,
    ];

    /// The name of the vocabulary, which is also the table its terms are kept in.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TraitKind => "trait_kind",
            Self::ContainmentKind => "containment_kind",
            Self::FamilyTieKind => "family_tie_kind",
            Self::PartnershipKind => "partnership_kind",
            Self::OfficeAcquisition => "office_acquisition",
            Self::ItemAcquisition => "item_acquisition",
            Self::CitizenshipStatus => "citizenship_status",
        }
    }
}

impl fmt::Display for Vocabulary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Vocabulary {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|vocabulary| vocabulary.as_str() == raw)
            .ok_or_else(|| DomainError::UnknownValue("vocabulary", raw.to_owned()))
    }
}

/// One term of a vocabulary: "fear" among trait kinds, "adoptive" among family ties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyTerm {
    pub vocabulary: Vocabulary,
    pub key: TypeKey,
    pub label: String,
    pub description: String,
    pub sort_order: i32,
    pub builtin: bool,
}

impl VocabularyTerm {
    /// A project-specific term.
    #[must_use]
    pub fn custom(vocabulary: Vocabulary, key: TypeKey, label: impl Into<String>) -> Self {
        Self {
            vocabulary,
            key,
            label: label.into(),
            description: String::new(),
            sort_order: 0,
            builtin: false,
        }
    }
}

impl Record for VocabularyTerm {
    type Id = (Vocabulary, TypeKey);

    fn id(&self) -> Self::Id {
        (self.vocabulary, self.key.clone())
    }
}

use Vocabulary::{
    CitizenshipStatus, ContainmentKind, FamilyTieKind, ItemAcquisition, OfficeAcquisition,
    PartnershipKind, TraitKind,
};

/// (vocabulary, key, label)
#[rustfmt::skip]
const BUILTIN_TERMS: &[(Vocabulary, &str, &str)] = &[
    (TraitKind, "personality", "Personality"), (TraitKind, "value", "Value"),
    (TraitKind, "goal", "Goal"), (TraitKind, "fear", "Fear"), (TraitKind, "quirk", "Quirk"),
    (TraitKind, "habit", "Habit"), (TraitKind, "flaw", "Flaw"), (TraitKind, "secret", "Secret"),
    (TraitKind, "ability", "Ability"), (TraitKind, "skill", "Skill"),
    (TraitKind, "weakness", "Weakness"), (TraitKind, "bond", "Bond"),
    (ContainmentKind, "geographic", "Geographic"), (ContainmentKind, "political", "Political"),
    (ContainmentKind, "administrative", "Administrative"), (ContainmentKind, "religious", "Religious"),
    (FamilyTieKind, "biological", "Biological"), (FamilyTieKind, "adoptive", "Adoptive"),
    (FamilyTieKind, "foster", "Foster"), (FamilyTieKind, "step", "Step"),
    (FamilyTieKind, "guardian", "Guardian"), (FamilyTieKind, "spiritual", "Spiritual"),
    (PartnershipKind, "marriage", "Marriage"), (PartnershipKind, "betrothal", "Betrothal"),
    (PartnershipKind, "partnership", "Partnership"), (PartnershipKind, "sworn_bond", "Sworn bond"),
    (OfficeAcquisition, "inheritance", "Inheritance"), (OfficeAcquisition, "election", "Election"),
    (OfficeAcquisition, "appointment", "Appointment"), (OfficeAcquisition, "conquest", "Conquest"),
    (OfficeAcquisition, "usurpation", "Usurpation"), (OfficeAcquisition, "acclamation", "Acclamation"),
    (OfficeAcquisition, "rotation", "Rotation"),
    (ItemAcquisition, "crafted", "Crafted"), (ItemAcquisition, "inherited", "Inherited"),
    (ItemAcquisition, "gift", "Gift"), (ItemAcquisition, "purchase", "Purchase"),
    (ItemAcquisition, "theft", "Theft"), (ItemAcquisition, "found", "Found"),
    (ItemAcquisition, "conquest", "Conquest"),
    (CitizenshipStatus, "citizen", "Citizen"), (CitizenshipStatus, "subject", "Subject"),
    (CitizenshipStatus, "resident", "Resident"), (CitizenshipStatus, "exile", "Exile"),
    (CitizenshipStatus, "hostage", "Hostage"),
    (TraitKind, "trait", "Trait"), (TraitKind, "belief", "Belief"), (TraitKind, "desire", "Desire"),
    (ContainmentKind, "cultural", "Cultural"), (ContainmentKind, "military", "Military"),
    (FamilyTieKind, "godparent", "Godparent"), (FamilyTieKind, "mentor", "Mentor"),
    (PartnershipKind, "concubinage", "Concubinage"), (PartnershipKind, "courtship", "Courtship"),
    (OfficeAcquisition, "purchase", "Purchase"), (OfficeAcquisition, "divine_right", "Divine right"),
    (ItemAcquisition, "reward", "Reward"),
];

/// The terms every new project starts with.
#[must_use]
pub fn builtin_vocabulary_terms() -> Vec<VocabularyTerm> {
    BUILTIN_TERMS
        .iter()
        .zip(0..)
        .map(|(&(vocabulary, key, label), sort_order)| VocabularyTerm {
            vocabulary,
            key: TypeKey::from_static(key),
            label: label.to_owned(),
            description: String::new(),
            sort_order,
            builtin: true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn terms_are_valid_and_unique_within_their_vocabulary() {
        let mut seen = BTreeSet::new();

        for term in builtin_vocabulary_terms() {
            assert!(TypeKey::new(term.key.as_str()).is_ok());
            assert!(
                seen.insert((term.vocabulary, term.key.clone())),
                "{} repeats",
                term.key
            );
        }
    }

    #[test]
    fn every_vocabulary_has_terms() {
        let terms = builtin_vocabulary_terms();

        for vocabulary in Vocabulary::ALL {
            assert!(
                terms.iter().any(|term| term.vocabulary == vocabulary),
                "{vocabulary} is empty"
            );
        }
    }

    #[test]
    fn vocabulary_names_round_trip() {
        for vocabulary in Vocabulary::ALL {
            assert_eq!(
                vocabulary.as_str().parse::<Vocabulary>().ok(),
                Some(vocabulary)
            );
        }
    }
}
