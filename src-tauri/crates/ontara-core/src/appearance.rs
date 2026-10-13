use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{AppearanceId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// The presence of an entity in a narrative context.
///
/// An appearance is a record, not a flag: it has a kind, an order, a viewpoint and its own canon
/// state. Absence in the matrix is derived: no record means unknown, a record of a non-presence
/// kind means confirmed absence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Appearance {
    pub id: AppearanceId,
    pub entity: EntityId,
    /// The chapter, scene or event the entity appears in.
    pub context: EntityId,
    pub kind: TypeKey,
    /// Narrative order among the appearances in the same context.
    pub position: i32,
    /// The entity through whose perspective the appearance is told.
    pub viewpoint: Option<EntityId>,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Appearance {
    #[must_use]
    pub fn new(entity: EntityId, context: EntityId, kind: TypeKey, now: Timestamp) -> Self {
        Self {
            id: AppearanceId::generate(),
            entity,
            context,
            kind,
            position: 0,
            viewpoint: None,
            note: None,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.note.as_deref(), "note")
    }
}

impl Record for Appearance {
    type Id = AppearanceId;

    fn id(&self) -> AppearanceId {
        self.id
    }
}
