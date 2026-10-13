use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;
use crate::temporal::Instant;

/// What is true of a being as a whole: who they are and where they come from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeingProfile {
    pub entity: EntityId,
    pub gender: Option<String>,
    pub pronouns: Option<String>,
    pub species: Option<EntityId>,
    pub birth: Option<Instant>,
    pub death: Option<Instant>,
    pub birthplace: Option<EntityId>,
    pub upbringing: Option<String>,
    pub occupation: Option<String>,
}

impl BeingProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            gender: None,
            pronouns: None,
            species: None,
            birth: None,
            death: None,
            birthplace: None,
            upbringing: None,
            occupation: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.gender.as_deref(), "gender")?;
        require_optional_text(self.pronouns.as_deref(), "pronouns")?;
        if let Some(instant) = &self.birth {
            instant.validate()?;
        }
        if let Some(instant) = &self.death {
            instant.validate()?;
        }
        require_optional_text(self.upbringing.as_deref(), "upbringing")?;
        require_optional_text(self.occupation.as_deref(), "occupation")?;
        Ok(())
    }
}

impl Record for BeingProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
