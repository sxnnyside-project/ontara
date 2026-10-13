use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;
use crate::temporal::Instant;

/// What is true of a religion as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReligionProfile {
    pub entity: EntityId,
    pub founder: Option<EntityId>,
    pub founded: Option<Instant>,
    pub is_monotheistic: Option<bool>,
}

impl ReligionProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            founder: None,
            founded: None,
            is_monotheistic: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        if let Some(instant) = &self.founded {
            instant.validate()?;
        }
        Ok(())
    }
}

impl Record for ReligionProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
