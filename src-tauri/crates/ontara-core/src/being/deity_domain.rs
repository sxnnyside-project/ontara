use serde::{Deserialize, Serialize};

use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::{DeityDomainId, EntityId};
use crate::record::Record;

/// A domain a deity presides over.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeityDomain {
    pub id: DeityDomainId,
    pub deity: EntityId,
    pub domain: String,
    pub note: Option<String>,
}

impl DeityDomain {
    #[must_use]
    pub fn new(deity: EntityId, domain: impl Into<String>) -> Self {
        Self {
            id: DeityDomainId::generate(),
            deity,
            domain: domain.into(),
            note: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.domain, "domain")?;
        require_optional_text(self.note.as_deref(), "note")?;
        Ok(())
    }
}

impl Record for DeityDomain {
    type Id = DeityDomainId;

    fn id(&self) -> DeityDomainId {
        self.id
    }
}
