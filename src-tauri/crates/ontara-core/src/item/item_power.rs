use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::{EntityId, ItemPowerId};
use crate::origin::Origin;
use crate::record::Record;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A power or property of an item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemPower {
    pub id: ItemPowerId,
    pub item: EntityId,
    pub description: String,
    pub cost: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl ItemPower {
    #[must_use]
    pub fn new(item: EntityId, description: impl Into<String>, now: Timestamp) -> Self {
        Self {
            id: ItemPowerId::generate(),
            item,
            description: description.into(),
            cost: None,
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
        require_optional_text(self.cost.as_deref(), "cost")?;
        Ok(())
    }
}

impl Record for ItemPower {
    type Id = ItemPowerId;

    fn id(&self) -> ItemPowerId {
        self.id
    }
}
