use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, FamilyTieId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A parent-child tie of a given kind. Following ties upward never returns to the start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FamilyTie {
    pub id: FamilyTieId,
    pub parent: EntityId,
    pub child: EntityId,
    pub kind: TypeKey,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl FamilyTie {
    #[must_use]
    pub fn new(parent: EntityId, child: EntityId, kind: TypeKey, now: Timestamp) -> Self {
        Self {
            id: FamilyTieId::generate(),
            parent,
            child,
            kind,
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
        if self.parent == self.child {
            return Err(DomainError::Invalid(
                "the parent and the child must be different".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for FamilyTie {
    type Id = FamilyTieId;

    fn id(&self) -> FamilyTieId {
        self.id
    }
}
