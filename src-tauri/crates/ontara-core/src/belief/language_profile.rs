use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LanguageStatus {
    Living,
    Dead,
    Ancient,
    Ritual,
    Constructed,
}

impl LanguageStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Living => "living",
            Self::Dead => "dead",
            Self::Ancient => "ancient",
            Self::Ritual => "ritual",
            Self::Constructed => "constructed",
        }
    }
}

impl fmt::Display for LanguageStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for LanguageStatus {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "living" => Ok(Self::Living),
            "dead" => Ok(Self::Dead),
            "ancient" => Ok(Self::Ancient),
            "ritual" => Ok(Self::Ritual),
            "constructed" => Ok(Self::Constructed),
            other => Err(DomainError::UnknownValue(
                "language status",
                other.to_owned(),
            )),
        }
    }
}

/// What is true of a language as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageProfile {
    pub entity: EntityId,
    pub parent_language: Option<EntityId>,
    pub writing_system: Option<String>,
    pub speakers_estimate: Option<i64>,
    pub status: Option<LanguageStatus>,
}

impl LanguageProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            parent_language: None,
            writing_system: None,
            speakers_estimate: None,
            status: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.writing_system.as_deref(), "writing system")?;
        if self.speakers_estimate.is_some_and(|value| value < 0) {
            return Err(DomainError::Invalid(
                "a speaker count cannot be negative".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for LanguageProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
