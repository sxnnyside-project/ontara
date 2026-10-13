use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_identifier {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Generates a new time-ordered identifier.
            #[must_use]
            pub fn generate() -> Self {
                Self(Uuid::now_v7())
            }

            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, formatter)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(raw: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(raw).map(Self)
            }
        }
    };
}

define_identifier!(
    /// Stable identity of an entity. Names are never identifiers.
    EntityId
);
define_identifier!(
    /// Stable identity of a relationship.
    RelationshipId
);
define_identifier!(
    /// Stable identity of a property definition (a trait, characteristic or attribute of a type).
    PropertyDefinitionId
);
define_identifier!(
    /// Stable identity of one recorded property value.
    PropertyValueId
);
define_identifier!(
    /// Stable identity of an appearance.
    AppearanceId
);
define_identifier!(
    /// Stable identity of an asset attachment.
    AttachmentId
);
define_identifier!(
    /// Stable identity of a source.
    SourceId
);
define_identifier!(
    /// Stable identity of a citation of a source.
    SourceReferenceId
);
define_identifier!(
    /// Stable identity of a note.
    NoteId
);
define_identifier!(
    /// Stable identity of a calendar.
    CalendarId
);
define_identifier!(
    /// Stable identity of one canon state change.
    CanonStateChangeId
);
define_identifier!(
    /// Stable identity of a proposal of changes made by an assistant.
    ProposalId
);

// <generated ids>
define_identifier!(
    /// Stable identity of a being appearance record.
    BeingAppearanceId
);
define_identifier!(
    /// Stable identity of a being mark record.
    BeingMarkId
);
define_identifier!(
    /// Stable identity of a being trait record.
    BeingTraitId
);
define_identifier!(
    /// Stable identity of a family tie record.
    FamilyTieId
);
define_identifier!(
    /// Stable identity of a partnership record.
    PartnershipId
);
define_identifier!(
    /// Stable identity of a deity domain record.
    DeityDomainId
);
define_identifier!(
    /// Stable identity of a place containment record.
    PlaceContainmentId
);
define_identifier!(
    /// Stable identity of a place border record.
    PlaceBorderId
);
define_identifier!(
    /// Stable identity of a place resource record.
    PlaceResourceId
);
define_identifier!(
    /// Stable identity of a census record.
    CensusId
);
define_identifier!(
    /// Stable identity of a polity symbol record.
    PolitySymbolId
);
define_identifier!(
    /// Stable identity of a polity succession record.
    PolitySuccessionId
);
define_identifier!(
    /// Stable identity of a office record.
    OfficeId
);
define_identifier!(
    /// Stable identity of a office tenure record.
    OfficeTenureId
);
define_identifier!(
    /// Stable identity of a citizenship record.
    CitizenshipId
);
define_identifier!(
    /// Stable identity of a diplomatic relation record.
    DiplomaticRelationId
);
define_identifier!(
    /// Stable identity of a faction parent record.
    FactionParentId
);
define_identifier!(
    /// Stable identity of a membership record.
    MembershipId
);
define_identifier!(
    /// Stable identity of a culture practice record.
    CulturePracticeId
);
define_identifier!(
    /// Stable identity of a religion tenet record.
    ReligionTenetId
);
define_identifier!(
    /// Stable identity of a adherence record.
    AdherenceId
);
define_identifier!(
    /// Stable identity of a item power record.
    ItemPowerId
);
define_identifier!(
    /// Stable identity of a item ownership record.
    ItemOwnershipId
);
define_identifier!(
    /// Stable identity of a event causality record.
    EventCausalityId
);
// </generated ids>

impl PropertyDefinitionId {
    /// Deterministic identity for a built-in definition, identical in every project.
    #[must_use]
    pub fn builtin(owner_key: &str, key: &str) -> Self {
        let name = format!("ontara/property/{owner_key}/{key}");
        Self(Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes()))
    }
}

impl CalendarId {
    /// Deterministic identity of the calendar every project starts with.
    #[must_use]
    pub fn builtin_default() -> Self {
        Self(Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            b"ontara/calendar/default",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_round_trip_through_text() {
        let id = EntityId::generate();

        assert_eq!(id.to_string().parse::<EntityId>().ok(), Some(id));
    }

    #[test]
    fn generated_identifiers_are_unique() {
        assert_ne!(EntityId::generate(), EntityId::generate());
    }

    #[test]
    fn malformed_text_is_rejected() {
        assert!("not-an-identifier".parse::<RelationshipId>().is_err());
    }

    #[test]
    fn builtin_identities_are_stable_and_distinct() {
        assert_eq!(
            PropertyDefinitionId::builtin("character", "birth"),
            PropertyDefinitionId::builtin("character", "birth")
        );
        assert_ne!(
            PropertyDefinitionId::builtin("character", "birth"),
            PropertyDefinitionId::builtin("character", "death")
        );
    }
}
