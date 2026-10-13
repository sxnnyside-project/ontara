use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;
use crate::temporal::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceStatus {
    Thriving,
    Declining,
    Abandoned,
    Ruined,
    Destroyed,
    Lost,
    Hidden,
}

impl PlaceStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Thriving => "thriving",
            Self::Declining => "declining",
            Self::Abandoned => "abandoned",
            Self::Ruined => "ruined",
            Self::Destroyed => "destroyed",
            Self::Lost => "lost",
            Self::Hidden => "hidden",
        }
    }
}

impl fmt::Display for PlaceStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PlaceStatus {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "thriving" => Ok(Self::Thriving),
            "declining" => Ok(Self::Declining),
            "abandoned" => Ok(Self::Abandoned),
            "ruined" => Ok(Self::Ruined),
            "destroyed" => Ok(Self::Destroyed),
            "lost" => Ok(Self::Lost),
            "hidden" => Ok(Self::Hidden),
            other => Err(DomainError::UnknownValue("place status", other.to_owned())),
        }
    }
}

/// What is true of a place as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceProfile {
    pub entity: EntityId,
    pub climate: Option<String>,
    pub terrain: Option<String>,
    pub area_km2: Option<f64>,
    pub elevation_m: Option<f64>,
    pub status: Option<PlaceStatus>,
    pub founded: Option<Instant>,
    pub dissolved: Option<Instant>,
}

impl PlaceProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            climate: None,
            terrain: None,
            area_km2: None,
            elevation_m: None,
            status: None,
            founded: None,
            dissolved: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.climate.as_deref(), "climate")?;
        require_optional_text(self.terrain.as_deref(), "terrain")?;
        if let Some(instant) = &self.founded {
            instant.validate()?;
        }
        if let Some(instant) = &self.dissolved {
            instant.validate()?;
        }
        if self.area_km2.is_some_and(|value| value < 0.0) {
            return Err(DomainError::Invalid(
                "an area cannot be negative".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for PlaceProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
