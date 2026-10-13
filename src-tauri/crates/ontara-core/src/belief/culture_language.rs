use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

/// A language spoken by a culture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CultureLanguage {
    pub culture: EntityId,
    pub language: EntityId,
}

impl CultureLanguage {
    #[must_use]
    pub fn new(culture: EntityId, language: EntityId) -> Self {
        Self { culture, language }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

impl Record for CultureLanguage {
    type Id = (EntityId, EntityId);

    fn id(&self) -> (EntityId, EntityId) {
        (self.culture, self.language)
    }
}
