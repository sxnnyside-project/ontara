use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{CensusId, EntityId, SourceId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Instant;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// The population of a place at a moment, and where the figure comes from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Census {
    pub id: CensusId,
    pub place: EntityId,
    pub taken: Instant,
    pub population: i64,
    pub is_estimate: bool,
    pub source: Option<SourceId>,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Census {
    #[must_use]
    pub fn new(place: EntityId, taken: Instant, population: i64, now: Timestamp) -> Self {
        Self {
            id: CensusId::generate(),
            place,
            taken,
            population,
            is_estimate: false,
            source: None,
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
        self.taken.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        if self.population < 0 {
            return Err(DomainError::Invalid(
                "a population cannot be negative".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for Census {
    type Id = CensusId;

    fn id(&self) -> CensusId {
        self.id
    }
}
