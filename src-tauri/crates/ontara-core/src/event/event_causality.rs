use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, EventCausalityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalityKind {
    Direct,
    Contributing,
    Enabling,
    Prevented,
    Triggered,
}

impl CausalityKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Contributing => "contributing",
            Self::Enabling => "enabling",
            Self::Prevented => "prevented",
            Self::Triggered => "triggered",
        }
    }
}

impl fmt::Display for CausalityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CausalityKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "direct" => Ok(Self::Direct),
            "contributing" => Ok(Self::Contributing),
            "enabling" => Ok(Self::Enabling),
            "prevented" => Ok(Self::Prevented),
            "triggered" => Ok(Self::Triggered),
            other => Err(DomainError::UnknownValue(
                "causality kind",
                other.to_owned(),
            )),
        }
    }
}

/// One event leading to another. Following causes backwards never returns to the start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventCausality {
    pub id: EventCausalityId,
    pub cause: EntityId,
    pub effect: EntityId,
    pub kind: CausalityKind,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl EventCausality {
    #[must_use]
    pub fn new(cause: EntityId, effect: EntityId, kind: CausalityKind, now: Timestamp) -> Self {
        Self {
            id: EventCausalityId::generate(),
            cause,
            effect,
            kind,
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
        require_optional_text(self.note.as_deref(), "note")?;
        if self.cause == self.effect {
            return Err(DomainError::Invalid(
                "the cause and the effect must be different".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for EventCausality {
    type Id = EventCausalityId;

    fn id(&self) -> EventCausalityId {
        self.id
    }
}
