use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::{BeingTraitId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// What makes a being individual: personality, values, fears, quirks, habits, flaws, secrets, abilities, skills and weaknesses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeingTrait {
    pub id: BeingTraitId,
    pub being: EntityId,
    pub kind: TypeKey,
    pub text: String,
    pub intensity: Option<i32>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl BeingTrait {
    #[must_use]
    pub fn new(being: EntityId, kind: TypeKey, text: impl Into<String>, now: Timestamp) -> Self {
        Self {
            id: BeingTraitId::generate(),
            being,
            kind,
            text: text.into(),
            intensity: None,
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
        require_text(&self.text, "text")?;
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        if self
            .intensity
            .is_some_and(|value| !(1..=5).contains(&value))
        {
            return Err(DomainError::Invalid(
                "intensity runs from 1 to 5".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for BeingTrait {
    type Id = BeingTraitId;

    fn id(&self) -> BeingTraitId {
        self.id
    }
}
