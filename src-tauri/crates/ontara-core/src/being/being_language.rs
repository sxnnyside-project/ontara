use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Proficiency {
    Native,
    Fluent,
    Conversational,
    Basic,
    ReadsOnly,
}

impl Proficiency {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Fluent => "fluent",
            Self::Conversational => "conversational",
            Self::Basic => "basic",
            Self::ReadsOnly => "reads_only",
        }
    }
}

impl fmt::Display for Proficiency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Proficiency {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "native" => Ok(Self::Native),
            "fluent" => Ok(Self::Fluent),
            "conversational" => Ok(Self::Conversational),
            "basic" => Ok(Self::Basic),
            "reads_only" => Ok(Self::ReadsOnly),
            other => Err(DomainError::UnknownValue("proficiency", other.to_owned())),
        }
    }
}

/// A language a being speaks and how well.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeingLanguage {
    pub being: EntityId,
    pub language: EntityId,
    pub proficiency: Proficiency,
    pub note: Option<String>,
}

impl BeingLanguage {
    #[must_use]
    pub fn new(being: EntityId, language: EntityId, proficiency: Proficiency) -> Self {
        Self {
            being,
            language,
            proficiency,
            note: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.note.as_deref(), "note")?;
        Ok(())
    }
}

impl Record for BeingLanguage {
    type Id = (EntityId, EntityId);

    fn id(&self) -> (EntityId, EntityId) {
        (self.being, self.language)
    }
}
