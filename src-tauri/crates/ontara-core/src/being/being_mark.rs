use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::{BeingMarkId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Instant;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A scar, tattoo, birthmark or other distinguishing mark.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeingMark {
    pub id: BeingMarkId,
    pub being: EntityId,
    pub description: String,
    pub body_location: Option<String>,
    pub acquired: Option<Instant>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl BeingMark {
    #[must_use]
    pub fn new(being: EntityId, description: impl Into<String>, now: Timestamp) -> Self {
        Self {
            id: BeingMarkId::generate(),
            being,
            description: description.into(),
            body_location: None,
            acquired: None,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.description, "description")?;
        require_optional_text(self.body_location.as_deref(), "body location")?;
        if let Some(instant) = &self.acquired {
            instant.validate()?;
        }
        Ok(())
    }
}

impl Record for BeingMark {
    type Id = BeingMarkId;

    fn id(&self) -> BeingMarkId {
        self.id
    }
}
