use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

/// What is true of a deity as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeityProfile {
    pub entity: EntityId,
    pub rank: Option<String>,
    pub alignment: Option<String>,
}

impl DeityProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            rank: None,
            alignment: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.rank.as_deref(), "rank")?;
        require_optional_text(self.alignment.as_deref(), "alignment")?;
        Ok(())
    }
}

impl Record for DeityProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
