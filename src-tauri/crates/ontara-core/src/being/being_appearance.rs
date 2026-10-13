use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{BeingAppearanceId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// How a being looks over a period. One row with no period is the usual look; more rows record aging, a scar, a curse or a disguise.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BeingAppearance {
    pub id: BeingAppearanceId,
    pub being: EntityId,
    pub label: Option<String>,
    pub hair_color: Option<String>,
    pub hair_style: Option<String>,
    pub eye_color: Option<String>,
    pub skin_tone: Option<String>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub build: Option<String>,
    pub voice: Option<String>,
    pub clothing_style: Option<String>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl BeingAppearance {
    #[must_use]
    pub fn new(being: EntityId, now: Timestamp) -> Self {
        Self {
            id: BeingAppearanceId::generate(),
            being,
            label: None,
            hair_color: None,
            hair_style: None,
            eye_color: None,
            skin_tone: None,
            height_cm: None,
            weight_kg: None,
            build: None,
            voice: None,
            clothing_style: None,
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
        require_optional_text(self.label.as_deref(), "label")?;
        require_optional_text(self.hair_color.as_deref(), "hair color")?;
        require_optional_text(self.hair_style.as_deref(), "hair style")?;
        require_optional_text(self.eye_color.as_deref(), "eye color")?;
        require_optional_text(self.skin_tone.as_deref(), "skin tone")?;
        require_optional_text(self.build.as_deref(), "build")?;
        require_optional_text(self.voice.as_deref(), "voice")?;
        require_optional_text(self.clothing_style.as_deref(), "clothing style")?;
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        if self.height_cm.is_some_and(|value| value <= 0.0) {
            return Err(DomainError::Invalid("a height must be positive".to_owned()));
        }
        if self.weight_kg.is_some_and(|value| value <= 0.0) {
            return Err(DomainError::Invalid("a weight must be positive".to_owned()));
        }
        Ok(())
    }
}

impl Record for BeingAppearance {
    type Id = BeingAppearanceId;

    fn id(&self) -> BeingAppearanceId {
        self.id
    }
}
