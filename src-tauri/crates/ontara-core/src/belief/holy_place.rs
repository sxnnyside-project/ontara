use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

/// A place holy to a religion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HolyPlace {
    pub religion: EntityId,
    pub place: EntityId,
    pub significance: Option<String>,
}

impl HolyPlace {
    #[must_use]
    pub fn new(religion: EntityId, place: EntityId) -> Self {
        Self {
            religion,
            place,
            significance: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.significance.as_deref(), "significance")?;
        Ok(())
    }
}

impl Record for HolyPlace {
    type Id = (EntityId, EntityId);

    fn id(&self) -> (EntityId, EntityId) {
        (self.religion, self.place)
    }
}
