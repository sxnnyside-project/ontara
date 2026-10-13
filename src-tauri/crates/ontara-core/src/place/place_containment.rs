use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, PlaceContainmentId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A place sitting inside another, of a given kind and over a period. Following one kind upward never returns to the start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceContainment {
    pub id: PlaceContainmentId,
    pub child: EntityId,
    pub parent: EntityId,
    pub kind: TypeKey,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl PlaceContainment {
    #[must_use]
    pub fn new(child: EntityId, parent: EntityId, kind: TypeKey, now: Timestamp) -> Self {
        Self {
            id: PlaceContainmentId::generate(),
            child,
            parent,
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
        if self.child == self.parent {
            return Err(DomainError::Invalid(
                "the child and the parent must be different".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for PlaceContainment {
    type Id = PlaceContainmentId;

    fn id(&self) -> PlaceContainmentId {
        self.id
    }
}
