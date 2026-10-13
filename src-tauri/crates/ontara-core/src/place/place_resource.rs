use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::{EntityId, PlaceResourceId};
use crate::record::Record;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceAbundance {
    Scarce,
    Common,
    Abundant,
    Exhausted,
}

impl ResourceAbundance {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scarce => "scarce",
            Self::Common => "common",
            Self::Abundant => "abundant",
            Self::Exhausted => "exhausted",
        }
    }
}

impl fmt::Display for ResourceAbundance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ResourceAbundance {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "scarce" => Ok(Self::Scarce),
            "common" => Ok(Self::Common),
            "abundant" => Ok(Self::Abundant),
            "exhausted" => Ok(Self::Exhausted),
            other => Err(DomainError::UnknownValue(
                "resource abundance",
                other.to_owned(),
            )),
        }
    }
}

/// A natural resource of a place.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceResource {
    pub id: PlaceResourceId,
    pub place: EntityId,
    pub resource: String,
    pub abundance: Option<ResourceAbundance>,
    pub note: Option<String>,
}

impl PlaceResource {
    #[must_use]
    pub fn new(place: EntityId, resource: impl Into<String>) -> Self {
        Self {
            id: PlaceResourceId::generate(),
            place,
            resource: resource.into(),
            abundance: None,
            note: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.resource, "resource")?;
        require_optional_text(self.note.as_deref(), "note")?;
        Ok(())
    }
}

impl Record for PlaceResource {
    type Id = PlaceResourceId;

    fn id(&self) -> PlaceResourceId {
        self.id
    }
}
