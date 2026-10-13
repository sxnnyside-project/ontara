use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{AdherenceId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Devotion {
    Nominal,
    Observant,
    Devout,
    Zealous,
    Clergy,
}

impl Devotion {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Observant => "observant",
            Self::Devout => "devout",
            Self::Zealous => "zealous",
            Self::Clergy => "clergy",
        }
    }
}

impl fmt::Display for Devotion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Devotion {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "nominal" => Ok(Self::Nominal),
            "observant" => Ok(Self::Observant),
            "devout" => Ok(Self::Devout),
            "zealous" => Ok(Self::Zealous),
            "clergy" => Ok(Self::Clergy),
            other => Err(DomainError::UnknownValue("devotion", other.to_owned())),
        }
    }
}

/// A being, culture or polity following a religion, over a period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Adherence {
    pub id: AdherenceId,
    pub adherent: EntityId,
    pub religion: EntityId,
    pub devotion: Option<Devotion>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Adherence {
    #[must_use]
    pub fn new(adherent: EntityId, religion: EntityId, now: Timestamp) -> Self {
        Self {
            id: AdherenceId::generate(),
            adherent,
            religion,
            devotion: None,
            period: Temporal::Unconstrained,
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
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        Ok(())
    }
}

impl Record for Adherence {
    type Id = AdherenceId;

    fn id(&self) -> AdherenceId {
        self.id
    }
}
