use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, ItemOwnershipId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// Who owned an item, when, and how they got it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemOwnership {
    pub id: ItemOwnershipId,
    pub item: EntityId,
    pub owner: EntityId,
    pub acquisition: Option<TypeKey>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl ItemOwnership {
    #[must_use]
    pub fn new(item: EntityId, owner: EntityId, now: Timestamp) -> Self {
        Self {
            id: ItemOwnershipId::generate(),
            item,
            owner,
            acquisition: None,
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

impl Record for ItemOwnership {
    type Id = ItemOwnershipId;

    fn id(&self) -> ItemOwnershipId {
        self.id
    }
}
