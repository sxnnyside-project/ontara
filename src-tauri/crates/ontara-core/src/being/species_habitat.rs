use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

/// A place where a species lives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesHabitat {
    pub species: EntityId,
    pub place: EntityId,
    pub note: Option<String>,
}

impl SpeciesHabitat {
    #[must_use]
    pub fn new(species: EntityId, place: EntityId) -> Self {
        Self {
            species,
            place,
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

impl Record for SpeciesHabitat {
    type Id = (EntityId, EntityId);

    fn id(&self) -> (EntityId, EntityId) {
        (self.species, self.place)
    }
}
