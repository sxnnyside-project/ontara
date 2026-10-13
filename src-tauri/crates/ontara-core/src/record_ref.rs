use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Every kind of record that has its own identity, and so can be cited, removed or have its
/// canon state changed. The name is the table the record is kept in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Entity,
    Relationship,
    PropertyValue,
    Appearance,
    BeingAppearance,
    BeingMark,
    BeingTrait,
    FamilyTie,
    Partnership,
    DeityDomain,
    PlaceContainment,
    PlaceBorder,
    PlaceResource,
    Census,
    PolitySymbol,
    PolitySuccession,
    Office,
    OfficeTenure,
    Citizenship,
    DiplomaticRelation,
    FactionParent,
    Membership,
    CulturePractice,
    ReligionTenet,
    Adherence,
    ItemPower,
    ItemOwnership,
    EventCausality,
}

impl RecordKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Relationship => "relationship",
            Self::PropertyValue => "property_value",
            Self::Appearance => "appearance",
            Self::BeingAppearance => "being_appearance",
            Self::BeingMark => "being_mark",
            Self::BeingTrait => "being_trait",
            Self::FamilyTie => "family_tie",
            Self::Partnership => "partnership",
            Self::DeityDomain => "deity_domain",
            Self::PlaceContainment => "place_containment",
            Self::PlaceBorder => "place_border",
            Self::PlaceResource => "place_resource",
            Self::Census => "census",
            Self::PolitySymbol => "polity_symbol",
            Self::PolitySuccession => "polity_succession",
            Self::Office => "office",
            Self::OfficeTenure => "office_tenure",
            Self::Citizenship => "citizenship",
            Self::DiplomaticRelation => "diplomatic_relation",
            Self::FactionParent => "faction_parent",
            Self::Membership => "membership",
            Self::CulturePractice => "culture_practice",
            Self::ReligionTenet => "religion_tenet",
            Self::Adherence => "adherence",
            Self::ItemPower => "item_power",
            Self::ItemOwnership => "item_ownership",
            Self::EventCausality => "event_causality",
        }
    }
}

impl fmt::Display for RecordKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RecordKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "entity" => Ok(Self::Entity),
            "relationship" => Ok(Self::Relationship),
            "property_value" => Ok(Self::PropertyValue),
            "appearance" => Ok(Self::Appearance),
            "being_appearance" => Ok(Self::BeingAppearance),
            "being_mark" => Ok(Self::BeingMark),
            "being_trait" => Ok(Self::BeingTrait),
            "family_tie" => Ok(Self::FamilyTie),
            "partnership" => Ok(Self::Partnership),
            "deity_domain" => Ok(Self::DeityDomain),
            "place_containment" => Ok(Self::PlaceContainment),
            "place_border" => Ok(Self::PlaceBorder),
            "place_resource" => Ok(Self::PlaceResource),
            "census" => Ok(Self::Census),
            "polity_symbol" => Ok(Self::PolitySymbol),
            "polity_succession" => Ok(Self::PolitySuccession),
            "office" => Ok(Self::Office),
            "office_tenure" => Ok(Self::OfficeTenure),
            "citizenship" => Ok(Self::Citizenship),
            "diplomatic_relation" => Ok(Self::DiplomaticRelation),
            "faction_parent" => Ok(Self::FactionParent),
            "membership" => Ok(Self::Membership),
            "culture_practice" => Ok(Self::CulturePractice),
            "religion_tenet" => Ok(Self::ReligionTenet),
            "adherence" => Ok(Self::Adherence),
            "item_power" => Ok(Self::ItemPower),
            "item_ownership" => Ok(Self::ItemOwnership),
            "event_causality" => Ok(Self::EventCausality),
            other => Err(DomainError::UnknownValue("record kind", other.to_owned())),
        }
    }
}

/// A reference to one record of any kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordRef {
    pub kind: RecordKind,
    pub id: uuid::Uuid,
}

impl RecordRef {
    #[must_use]
    pub const fn new(kind: RecordKind, id: uuid::Uuid) -> Self {
        Self { kind, id }
    }
}
