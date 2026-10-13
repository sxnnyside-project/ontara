use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::error::DomainError;
use crate::identifier::{EntityId, FactionParentId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A faction that is part of a larger one, over a period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactionParent {
    pub id: FactionParentId,
    pub child: EntityId,
    pub parent: EntityId,
    pub period: Temporal,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl FactionParent {
    #[must_use]
    pub fn new(child: EntityId, parent: EntityId, now: Timestamp) -> Self {
        Self {
            id: FactionParentId::generate(),
            child,
            parent,
            period: Temporal::Unconstrained,
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
        if self.child == self.parent {
            return Err(DomainError::Invalid(
                "the child and the parent must be different".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for FactionParent {
    type Id = FactionParentId;

    fn id(&self) -> FactionParentId {
        self.id
    }
}
