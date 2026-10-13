use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{CitizenshipId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A being's standing in a polity, over a period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Citizenship {
    pub id: CitizenshipId,
    pub being: EntityId,
    pub polity: EntityId,
    pub status: TypeKey,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Citizenship {
    #[must_use]
    pub fn new(being: EntityId, polity: EntityId, status: TypeKey, now: Timestamp) -> Self {
        Self {
            id: CitizenshipId::generate(),
            being,
            polity,
            status,
            period: Temporal::Unconstrained,
            note: None,
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
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        Ok(())
    }
}

impl Record for Citizenship {
    type Id = CitizenshipId;

    fn id(&self) -> CitizenshipId {
        self.id
    }
}
