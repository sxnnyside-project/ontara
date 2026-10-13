use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessionRule {
    Hereditary,
    Elective,
    Appointed,
    Rotating,
    Conquest,
    Mixed,
}

impl SuccessionRule {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hereditary => "hereditary",
            Self::Elective => "elective",
            Self::Appointed => "appointed",
            Self::Rotating => "rotating",
            Self::Conquest => "conquest",
            Self::Mixed => "mixed",
        }
    }
}

impl fmt::Display for SuccessionRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for SuccessionRule {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "hereditary" => Ok(Self::Hereditary),
            "elective" => Ok(Self::Elective),
            "appointed" => Ok(Self::Appointed),
            "rotating" => Ok(Self::Rotating),
            "conquest" => Ok(Self::Conquest),
            "mixed" => Ok(Self::Mixed),
            other => Err(DomainError::UnknownValue(
                "succession rule",
                other.to_owned(),
            )),
        }
    }
}

/// What is true of a kingdom, country, empire or province as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolityProfile {
    pub entity: EntityId,
    pub government_type: Option<String>,
    pub succession_rule: Option<SuccessionRule>,
    pub currency: Option<String>,
    pub motto: Option<String>,
    pub capital: Option<EntityId>,
    pub state_religion: Option<EntityId>,
}

impl PolityProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            government_type: None,
            succession_rule: None,
            currency: None,
            motto: None,
            capital: None,
            state_religion: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.government_type.as_deref(), "government type")?;
        require_optional_text(self.currency.as_deref(), "currency")?;
        require_optional_text(self.motto.as_deref(), "motto")?;
        Ok(())
    }
}

impl Record for PolityProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
