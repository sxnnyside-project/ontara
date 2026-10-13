use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::entity::Entity;
use crate::error::DomainError;
use crate::identifier::PropertyValueId;
use crate::property_definition::{PropertyDataType, PropertyDefinition, PropertyOwner};
use crate::property_value::{PropertyContent, PropertyValue, PropertyValueOwner};
use crate::record_ref::{RecordKind, RecordRef};
use crate::relationship::Relationship;
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;

impl<R: ModelRepository> ModelEditor<'_, R> {
    fn validate_property_value(&self, value: &PropertyValue) -> Result<(), Failure<R>> {
        value.validate()?;
        self.require_defined::<CanonState>("canon state", &value.canon_state)?;
        let definition =
            self.require::<PropertyDefinition>("property definition", &value.definition)?;
        let property = definition.key.to_string();
        let invalid = |reason: String| -> Failure<R> {
            DomainError::InvalidProperty {
                property: property.clone(),
                reason,
            }
            .into()
        };

        match (&value.owner, &definition.owner) {
            (PropertyValueOwner::Entity(id), PropertyOwner::EntityType(expected)) => {
                let entity = self.require::<Entity>("entity", id)?;
                if !self.is_type(&entity.entity_type, expected)? {
                    return Err(invalid(format!(
                        "it describes `{expected}` entities, not `{}`",
                        entity.entity_type
                    )));
                }
            }
            (PropertyValueOwner::Relationship(id), PropertyOwner::RelationshipType(expected)) => {
                let relationship = self.require::<Relationship>("relationship", id)?;
                if &relationship.relationship_type != expected {
                    return Err(invalid(format!(
                        "it describes `{expected}` relationships, not `{}`",
                        relationship.relationship_type
                    )));
                }
            }
            (PropertyValueOwner::Entity(_), PropertyOwner::RelationshipType(_)) => {
                return Err(invalid(
                    "it describes relationships, not entities".to_owned(),
                ));
            }
            (PropertyValueOwner::Relationship(_), PropertyOwner::EntityType(_)) => {
                return Err(invalid(
                    "it describes entities, not relationships".to_owned(),
                ));
            }
        }

        match (&definition.data_type, &value.content) {
            (PropertyDataType::Text, PropertyContent::Text(text)) => {
                if text.trim().is_empty() {
                    return Err(invalid("a text value must not be blank".to_owned()));
                }
            }
            (PropertyDataType::Integer, PropertyContent::Integer(_))
            | (PropertyDataType::Real, PropertyContent::Real(_))
            | (PropertyDataType::Boolean, PropertyContent::Boolean(_))
            | (PropertyDataType::Date, PropertyContent::Date(_)) => {}
            (PropertyDataType::Choice { options }, PropertyContent::Choice(key)) => {
                if !options.iter().any(|option| &option.key == key) {
                    return Err(invalid(format!("`{key}` is not one of its options")));
                }
            }
            (
                PropertyDataType::EntityReference { entity_type },
                PropertyContent::EntityReference(target),
            ) => {
                let referenced = self.require::<Entity>("entity", target)?;
                if let Some(expected) = entity_type
                    && !self.is_type(&referenced.entity_type, expected)?
                {
                    return Err(invalid(format!(
                        "it must reference a `{expected}`, not a `{}`",
                        referenced.entity_type
                    )));
                }
            }
            (expected, actual) => {
                return Err(invalid(format!(
                    "it holds {} values, not {}",
                    expected.name(),
                    actual.type_name()
                )));
            }
        }

        if !definition.multiple {
            let others: Vec<_> = self
                .query(|r| r.property_values_of(&value.owner))?
                .into_iter()
                .filter(|other| other.definition == value.definition && other.id != value.id)
                .collect();
            let all_bounded = value.is_bounded() && others.iter().all(PropertyValue::is_bounded);
            if !others.is_empty() && !all_bounded {
                return Err(invalid(
                    "it holds a single value; give each value a period of validity to record changes over time"
                        .to_owned(),
                ));
            }
        }
        Ok(())
    }

    pub fn add_property_value(
        &mut self,
        value: PropertyValue,
    ) -> Result<PropertyValue, Failure<R>> {
        self.validate_property_value(&value)?;
        if self.find::<PropertyValue>(&value.id)?.is_some() {
            return Err(DomainError::duplicate("property value", value.id).into());
        }
        self.insert(&value)?;
        Ok(value)
    }

    /// Replaces a value's content and validity. Identity, owner, origin and creation time are kept.
    pub fn update_property_value(
        &mut self,
        mut value: PropertyValue,
        now: Timestamp,
    ) -> Result<PropertyValue, Failure<R>> {
        let stored = self.require::<PropertyValue>("property value", &value.id)?;
        if stored.owner != value.owner || stored.definition != value.definition {
            return Err(DomainError::Immutable("owner or definition").into());
        }
        self.validate_property_value(&value)?;
        value.created_at = stored.created_at;
        value.origin = stored.origin;
        value.updated_at = now;
        self.update(&value)?;
        Ok(value)
    }

    pub fn remove_property_value(&mut self, id: &PropertyValueId) -> Result<(), Failure<R>> {
        self.require::<PropertyValue>("property value", id)?;
        self.remove_record(&RecordRef::new(RecordKind::PropertyValue, id.as_uuid()))
    }
}
