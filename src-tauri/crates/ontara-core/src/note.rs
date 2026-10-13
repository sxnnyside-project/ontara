use serde::{Deserialize, Serialize};

use crate::entity::require_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, NoteId};
use crate::record::Record;
use crate::timestamp::Timestamp;

/// A supporting document attached to an entity, written in Markdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: NoteId,
    pub entity: EntityId,
    pub title: String,
    pub body: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Note {
    #[must_use]
    pub fn new(
        entity: EntityId,
        title: impl Into<String>,
        body: impl Into<String>,
        now: Timestamp,
    ) -> Self {
        Self {
            id: NoteId::generate(),
            entity,
            title: title.into(),
            body: body.into(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.title, "note title")
    }
}

impl Record for Note {
    type Id = NoteId;

    fn id(&self) -> NoteId {
        self.id
    }
}
