use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::error::DomainError;
use crate::identifier::{EntityId, PropertyDefinitionId, PropertyValueId, RelationshipId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Instant;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// The record a property value belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum PropertyValueOwner {
    Entity(EntityId),
    Relationship(RelationshipId),
}

/// The content of a property value. Its variant must match the definition's data type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum PropertyContent {
    Text(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Date(Instant),
    EntityReference(EntityId),
    Choice(TypeKey),
}

impl PropertyContent {
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Text(_) => "text",
            Self::Integer(_) => "integer",
            Self::Real(_) => "real",
            Self::Boolean(_) => "boolean",
            Self::Date(_) => "date",
            Self::EntityReference(_) => "entity reference",
            Self::Choice(_) => "choice",
        }
    }
}

/// One recorded value of a property: a trait of a character, a rank in an affiliation.
///
/// A value can have its own canon state, a period of validity (hair that was white only after
/// the war), and its own sources.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyValue {
    pub id: PropertyValueId,
    pub owner: PropertyValueOwner,
    pub definition: PropertyDefinitionId,
    pub content: PropertyContent,
    pub valid_from: Option<Instant>,
    pub valid_until: Option<Instant>,
    pub note: Option<String>,
    /// Order among the values of the same property.
    pub position: i32,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl PropertyValue {
    #[must_use]
    pub fn new(
        owner: PropertyValueOwner,
        definition: PropertyDefinitionId,
        content: PropertyContent,
        now: Timestamp,
    ) -> Self {
        Self {
            id: PropertyValueId::generate(),
            owner,
            definition,
            content,
            valid_from: None,
            valid_until: None,
            note: None,
            position: 0,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// The entity this value points at, if its content is an entity reference.
    #[must_use]
    pub fn referenced_entity(&self) -> Option<EntityId> {
        match self.content {
            PropertyContent::EntityReference(entity) => Some(entity),
            _ => None,
        }
    }

    /// Whether the value is limited to a period of the timeline.
    #[must_use]
    pub fn is_bounded(&self) -> bool {
        self.valid_from.is_some() || self.valid_until.is_some()
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if let PropertyContent::Date(instant) = &self.content {
            instant.validate()?;
        }
        if let PropertyContent::Real(number) = self.content
            && !number.is_finite()
        {
            return Err(DomainError::InvalidTime(
                "a real value must be finite".to_owned(),
            ));
        }
        for instant in [&self.valid_from, &self.valid_until].into_iter().flatten() {
            instant.validate()?;
        }
        if let (Some(Some(from)), Some(Some(until))) = (
            self.valid_from.as_ref().map(|instant| instant.day),
            self.valid_until.as_ref().map(|instant| instant.day),
        ) && from > until
        {
            return Err(DomainError::InvalidTime(
                "a value cannot stop being valid before it starts".to_owned(),
            ));
        }
        if self
            .note
            .as_deref()
            .is_some_and(|note| note.trim().is_empty())
        {
            return Err(DomainError::BlankText("note"));
        }
        Ok(())
    }
}

impl Record for PropertyValue {
    type Id = PropertyValueId;

    fn id(&self) -> PropertyValueId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(content: PropertyContent) -> PropertyValue {
        PropertyValue::new(
            PropertyValueOwner::Entity(EntityId::generate()),
            PropertyDefinitionId::builtin("character", "occupation"),
            content,
            Timestamp::from_unix_millis(1),
        )
    }

    #[test]
    fn validity_cannot_end_before_it_starts() {
        let mut recorded = value(PropertyContent::Text("Smith".to_owned()));
        recorded.valid_from = Some(Instant::at_day(50));
        recorded.valid_until = Some(Instant::at_day(10));

        assert!(recorded.validate().is_err());

        recorded.valid_until = Some(Instant::at_day(60));
        assert_eq!(recorded.validate(), Ok(()));
        assert!(recorded.is_bounded());
    }

    #[test]
    fn entity_references_expose_their_target() {
        let target = EntityId::generate();

        assert_eq!(
            value(PropertyContent::EntityReference(target)).referenced_entity(),
            Some(target)
        );
        assert_eq!(
            value(PropertyContent::Boolean(true)).referenced_entity(),
            None
        );
    }

    #[test]
    fn non_finite_numbers_are_rejected() {
        assert!(value(PropertyContent::Real(f64::NAN)).validate().is_err());
    }
}
