use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;

/// What is true of a species as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesProfile {
    pub entity: EntityId,
    pub is_sapient: Option<bool>,
    pub lifespan_min_years: Option<i64>,
    pub lifespan_max_years: Option<i64>,
    pub average_height_cm: Option<f64>,
    pub diet: Option<String>,
}

impl SpeciesProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            is_sapient: None,
            lifespan_min_years: None,
            lifespan_max_years: None,
            average_height_cm: None,
            diet: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.diet.as_deref(), "diet")?;
        if self.lifespan_min_years.is_some_and(|value| value < 0) {
            return Err(DomainError::Invalid(
                "a lifespan cannot be negative".to_owned(),
            ));
        }
        if self.lifespan_max_years.is_some_and(|value| value < 0) {
            return Err(DomainError::Invalid(
                "a lifespan cannot be negative".to_owned(),
            ));
        }
        if matches!((self.lifespan_min_years, self.lifespan_max_years), (Some(min), Some(max)) if max < min)
        {
            return Err(DomainError::Invalid(
                "the longest lifespan cannot be shorter than the shortest".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for SpeciesProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
