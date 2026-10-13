use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;
use crate::temporal::Instant;

/// What is true of a faction or organization as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionProfile {
    pub entity: EntityId,
    pub ideology: Option<String>,
    pub size_estimate: Option<i64>,
    pub headquarters: Option<EntityId>,
    pub founded: Option<Instant>,
    pub dissolved: Option<Instant>,
}

impl FactionProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            ideology: None,
            size_estimate: None,
            headquarters: None,
            founded: None,
            dissolved: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.ideology.as_deref(), "ideology")?;
        if let Some(instant) = &self.founded {
            instant.validate()?;
        }
        if let Some(instant) = &self.dissolved {
            instant.validate()?;
        }
        if self.size_estimate.is_some_and(|value| value < 0) {
            return Err(DomainError::Invalid("a size cannot be negative".to_owned()));
        }
        Ok(())
    }
}

impl Record for FactionProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
