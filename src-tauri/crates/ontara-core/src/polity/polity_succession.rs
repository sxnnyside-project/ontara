use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, PolitySuccessionId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Instant;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolitySuccessionKind {
    Renaming,
    Conquest,
    Split,
    Merger,
    Revolution,
    Dissolution,
    Restoration,
}

impl PolitySuccessionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Renaming => "renaming",
            Self::Conquest => "conquest",
            Self::Split => "split",
            Self::Merger => "merger",
            Self::Revolution => "revolution",
            Self::Dissolution => "dissolution",
            Self::Restoration => "restoration",
        }
    }
}

impl fmt::Display for PolitySuccessionKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PolitySuccessionKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "renaming" => Ok(Self::Renaming),
            "conquest" => Ok(Self::Conquest),
            "split" => Ok(Self::Split),
            "merger" => Ok(Self::Merger),
            "revolution" => Ok(Self::Revolution),
            "dissolution" => Ok(Self::Dissolution),
            "restoration" => Ok(Self::Restoration),
            other => Err(DomainError::UnknownValue(
                "polity succession kind",
                other.to_owned(),
            )),
        }
    }
}

/// A polity that came after another: a kingdom that split, a state that was conquered, an empire renamed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolitySuccession {
    pub id: PolitySuccessionId,
    pub predecessor: EntityId,
    pub successor: EntityId,
    pub kind: PolitySuccessionKind,
    pub occurred: Option<Instant>,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl PolitySuccession {
    #[must_use]
    pub fn new(
        predecessor: EntityId,
        successor: EntityId,
        kind: PolitySuccessionKind,
        now: Timestamp,
    ) -> Self {
        Self {
            id: PolitySuccessionId::generate(),
            predecessor,
            successor,
            kind,
            occurred: None,
            note: None,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        if let Some(instant) = &self.occurred {
            instant.validate()?;
        }
        require_optional_text(self.note.as_deref(), "note")?;
        if self.predecessor == self.successor {
            return Err(DomainError::Invalid(
                "the predecessor and the successor must be different".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for PolitySuccession {
    type Id = PolitySuccessionId;

    fn id(&self) -> PolitySuccessionId {
        self.id
    }
}
