use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, ReligionTenetId};
use crate::origin::Origin;
use crate::record::Record;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenetKind {
    Tenet,
    Practice,
    Taboo,
    Festival,
    Sacrament,
    Prophecy,
}

impl TenetKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tenet => "tenet",
            Self::Practice => "practice",
            Self::Taboo => "taboo",
            Self::Festival => "festival",
            Self::Sacrament => "sacrament",
            Self::Prophecy => "prophecy",
        }
    }
}

impl fmt::Display for TenetKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for TenetKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "tenet" => Ok(Self::Tenet),
            "practice" => Ok(Self::Practice),
            "taboo" => Ok(Self::Taboo),
            "festival" => Ok(Self::Festival),
            "sacrament" => Ok(Self::Sacrament),
            "prophecy" => Ok(Self::Prophecy),
            other => Err(DomainError::UnknownValue("tenet kind", other.to_owned())),
        }
    }
}

/// A tenet, practice, taboo, festival, sacrament or prophecy of a religion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReligionTenet {
    pub id: ReligionTenetId,
    pub religion: EntityId,
    pub kind: TenetKind,
    pub description: String,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl ReligionTenet {
    #[must_use]
    pub fn new(
        religion: EntityId,
        kind: TenetKind,
        description: impl Into<String>,
        now: Timestamp,
    ) -> Self {
        Self {
            id: ReligionTenetId::generate(),
            religion,
            kind,
            description: description.into(),
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
        require_text(&self.description, "description")?;
        Ok(())
    }
}

impl Record for ReligionTenet {
    type Id = ReligionTenetId;

    fn id(&self) -> ReligionTenetId {
        self.id
    }
}
