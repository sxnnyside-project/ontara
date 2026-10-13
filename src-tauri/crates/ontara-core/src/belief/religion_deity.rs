use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

/// A deity a religion reveres, and in what role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReligionDeity {
    pub religion: EntityId,
    pub deity: EntityId,
    pub role: Option<String>,
}

impl ReligionDeity {
    #[must_use]
    pub fn new(religion: EntityId, deity: EntityId) -> Self {
        Self {
            religion,
            deity,
            role: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.role.as_deref(), "role")?;
        Ok(())
    }
}

impl Record for ReligionDeity {
    type Id = (EntityId, EntityId);

    fn id(&self) -> (EntityId, EntityId) {
        (self.religion, self.deity)
    }
}
