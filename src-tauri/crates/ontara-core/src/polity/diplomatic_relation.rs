use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{DiplomaticRelationId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// A stance between two polities or factions over a period. For a hierarchical stance, `senior` is the senior party.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiplomaticRelation {
    pub id: DiplomaticRelationId,
    pub first: EntityId,
    pub second: EntityId,
    pub stance: TypeKey,
    pub senior: Option<EntityId>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl DiplomaticRelation {
    #[must_use]
    pub fn new(first: EntityId, second: EntityId, stance: TypeKey, now: Timestamp) -> Self {
        Self {
            id: DiplomaticRelationId::generate(),
            first,
            second,
            stance,
            senior: None,
            period: Temporal::Unconstrained,
            note: None,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Puts the pair in storage order, so one pair is one row.
    pub fn normalize(&mut self) {
        if self.first > self.second {
            std::mem::swap(&mut self.first, &mut self.second);
        }
    }

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        if self.first >= self.second {
            return Err(DomainError::Invalid(
                "the two sides must differ and be stored in order".to_owned(),
            ));
        }
        if self
            .senior
            .is_some_and(|senior| senior != self.first && senior != self.second)
        {
            return Err(DomainError::Invalid(
                "the senior party must be one of the two parties".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for DiplomaticRelation {
    type Id = DiplomaticRelationId;

    fn id(&self) -> DiplomaticRelationId {
        self.id
    }
}
