use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, RelationshipId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// Whether a relationship still holds or has become part of the history.
///
/// A rivalry that ends is not deleted: it becomes historical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    #[default]
    Active,
    Historical,
}

impl Lifecycle {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Historical => "historical",
        }
    }
}

impl fmt::Display for Lifecycle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Lifecycle {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "active" => Ok(Self::Active),
            "historical" => Ok(Self::Historical),
            other => Err(DomainError::UnknownValue(
                "relationship lifecycle",
                other.to_owned(),
            )),
        }
    }
}

/// A first-class, typed connection from a source entity to a target entity.
///
/// For undirected types, source and target are an unordered pair; the order is only storage
/// order. Characteristics of the connection (rank, strength) are property values owned by it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub id: RelationshipId,
    pub relationship_type: TypeKey,
    pub source: EntityId,
    pub target: EntityId,
    /// A short qualifier, such as the role in an affiliation.
    pub qualifier: Option<String>,
    /// Narrative context in which the relationship holds: a chapter, scene or event.
    pub context: Option<EntityId>,
    pub lifecycle: Lifecycle,
    /// When the relationship holds on the in-universe timeline.
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Relationship {
    /// A new established, active relationship with a freshly generated identifier.
    #[must_use]
    pub fn new(
        relationship_type: TypeKey,
        source: EntityId,
        target: EntityId,
        now: Timestamp,
    ) -> Self {
        Self {
            id: RelationshipId::generate(),
            relationship_type,
            source,
            target,
            qualifier: None,
            context: None,
            lifecycle: Lifecycle::default(),
            period: Temporal::Unconstrained,
            note: None,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.qualifier.as_deref(), "qualifier")?;
        require_optional_text(self.note.as_deref(), "note")?;
        self.period.validate()
    }
}

impl Record for Relationship {
    type Id = RelationshipId;

    fn id(&self) -> RelationshipId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::Instant;

    fn relationship() -> Relationship {
        Relationship::new(
            TypeKey::from_static("parent_of"),
            EntityId::generate(),
            EntityId::generate(),
            Timestamp::from_unix_millis(1),
        )
    }

    #[test]
    fn blank_optional_text_is_rejected_but_absent_text_is_fine() {
        let mut value = relationship();
        assert_eq!(value.validate(), Ok(()));

        value.qualifier = Some("  ".to_owned());
        assert_eq!(value.validate(), Err(DomainError::BlankText("qualifier")));
    }

    #[test]
    fn a_relationship_can_be_bounded_in_time_and_become_historical() {
        let mut value = relationship();
        value.period = Temporal::Span {
            start: Some(Instant::at_day(10)),
            end: Some(Instant::at_day(900)),
        };
        value.lifecycle = Lifecycle::Historical;

        assert_eq!(value.validate(), Ok(()));
        assert_eq!(value.period.end_day(), Some(900));
    }
}
