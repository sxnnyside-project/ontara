use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;
use crate::temporal::Temporal;

/// How far an event reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventScale {
    Personal,
    Local,
    Regional,
    National,
    World,
}

impl EventScale {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Personal => "personal",
            Self::Local => "local",
            Self::Regional => "regional",
            Self::National => "national",
            Self::World => "world",
        }
    }
}

impl fmt::Display for EventScale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EventScale {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "personal" => Ok(Self::Personal),
            "local" => Ok(Self::Local),
            "regional" => Ok(Self::Regional),
            "national" => Ok(Self::National),
            "world" => Ok(Self::World),
            other => Err(DomainError::UnknownValue("event scale", other.to_owned())),
        }
    }
}

/// An entity taking part in an event, in a named role and, if the event has sides, on one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventParticipant {
    pub entity: EntityId,
    pub role: String,
    pub side: Option<String>,
}

impl EventParticipant {
    /// A participant with the plain role "participant".
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            role: "participant".to_owned(),
            side: None,
        }
    }
}

/// What makes an entity of an event type an event: when, where, who and with what consequences.
///
/// Chronology lives here, in in-universe time, and is independent of where the event is
/// presented in the editorial structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventDetail {
    pub entity: EntityId,
    pub period: Temporal,
    pub location: Option<EntityId>,
    pub scale: Option<EventScale>,
    pub consequences: String,
    pub participants: Vec<EventParticipant>,
}

impl EventDetail {
    /// An event with an unknown date and nothing else recorded yet.
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            period: Temporal::Unknown,
            location: None,
            scale: None,
            consequences: String::new(),
            participants: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        self.period.validate()?;
        let mut seen = BTreeSet::new();
        for participant in &self.participants {
            require_text(&participant.role, "participant role")?;
            require_optional_text(participant.side.as_deref(), "participant side")?;
            if !seen.insert((participant.entity, participant.role.as_str())) {
                return Err(DomainError::duplicate(
                    "event participant",
                    participant.entity,
                ));
            }
        }
        Ok(())
    }
}

impl Record for EventDetail {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
