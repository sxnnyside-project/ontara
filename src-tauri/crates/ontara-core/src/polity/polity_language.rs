use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolityLanguageStatus {
    Official,
    Regional,
    Trade,
    Liturgical,
}

impl PolityLanguageStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Regional => "regional",
            Self::Trade => "trade",
            Self::Liturgical => "liturgical",
        }
    }
}

impl fmt::Display for PolityLanguageStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PolityLanguageStatus {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "official" => Ok(Self::Official),
            "regional" => Ok(Self::Regional),
            "trade" => Ok(Self::Trade),
            "liturgical" => Ok(Self::Liturgical),
            other => Err(DomainError::UnknownValue(
                "polity language status",
                other.to_owned(),
            )),
        }
    }
}

/// A language used in a polity and its status there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolityLanguage {
    pub polity: EntityId,
    pub language: EntityId,
    pub status: PolityLanguageStatus,
}

impl PolityLanguage {
    #[must_use]
    pub fn new(polity: EntityId, language: EntityId, status: PolityLanguageStatus) -> Self {
        Self {
            polity,
            language,
            status,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

impl Record for PolityLanguage {
    type Id = (EntityId, EntityId);

    fn id(&self) -> (EntityId, EntityId) {
        (self.polity, self.language)
    }
}
