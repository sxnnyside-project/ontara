use serde::{Deserialize, Serialize};

use crate::entity::require_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, OfficeId};
use crate::record::Record;
use crate::timestamp::Timestamp;

/// A seat of power a polity or faction has: king, queen, council, high priest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Office {
    pub id: OfficeId,
    pub body: EntityId,
    pub title: String,
    pub description: String,
    pub is_head: bool,
    pub seats: i32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Office {
    #[must_use]
    pub fn new(body: EntityId, title: impl Into<String>, now: Timestamp) -> Self {
        Self {
            id: OfficeId::generate(),
            body,
            title: title.into(),
            description: String::new(),
            is_head: false,
            seats: 1,
            created_at: now,
            updated_at: now,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.title, "title")?;
        if self.seats < 1 {
            return Err(DomainError::Invalid(
                "an office has at least one seat".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for Office {
    type Id = OfficeId;

    fn id(&self) -> OfficeId {
        self.id
    }
}
