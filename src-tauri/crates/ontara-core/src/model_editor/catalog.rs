use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::appearance_kind::AppearanceKind;
use crate::calendar::Calendar;
use crate::canon_state::CanonState;
use crate::diplomatic_stance::DiplomaticStance;
use crate::editorial_level::EditorialLevel;
use crate::entity::require_text;
use crate::entity_type::{EntityRole, EntityType};
use crate::error::DomainError;
use crate::property_definition::{
    PropertyDataType, PropertyDefinition, PropertyGroup, PropertyOwner,
};
use crate::record::Record;
use crate::relationship_type::RelationshipType;
use crate::repository::{ModelRepository, Writer};
use crate::type_key::TypeKey;
use crate::vocabulary::VocabularyTerm;

impl<R: ModelRepository> ModelEditor<'_, R> {
    fn define_keyed<T>(&mut self, what: &'static str, record: T) -> Result<T, Failure<R>>
    where
        T: Record<Id = TypeKey>,
        R: Writer<T>,
    {
        if self.find::<T>(&record.id())?.is_some() {
            return Err(DomainError::already_defined(what, record.id()).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    /// Defines a project-specific canon state.
    pub fn define_canon_state(&mut self, mut state: CanonState) -> Result<CanonState, Failure<R>> {
        require_text(&state.label, "canon state label")?;
        state.builtin = false;
        self.define_keyed("canon state", state)
    }

    /// Defines a project-specific entity type.
    pub fn define_entity_type(
        &mut self,
        mut entity_type: EntityType,
    ) -> Result<EntityType, Failure<R>> {
        require_text(&entity_type.label, "entity type label")?;
        entity_type.builtin = false;
        if let Some(parent_key) = &entity_type.parent {
            let parent = self.require_defined::<EntityType>("entity type", parent_key)?;
            if parent.role != EntityRole::Generic && parent.role != entity_type.role {
                return Err(DomainError::InvalidTypeHierarchy(format!(
                    "`{}` cannot specialize `{}`: they have different roles",
                    entity_type.key, parent.key
                ))
                .into());
            }
        }
        self.define_keyed("entity type", entity_type)
    }

    /// Defines a project-specific relationship type. Endpoint rules must name defined types.
    pub fn define_relationship_type(
        &mut self,
        mut relationship_type: RelationshipType,
    ) -> Result<RelationshipType, Failure<R>> {
        relationship_type.validate()?;
        relationship_type.builtin = false;
        for key in relationship_type
            .allowed_sources
            .iter()
            .chain(&relationship_type.allowed_targets)
        {
            self.require_defined::<EntityType>("entity type", key)?;
        }
        self.define_keyed("relationship type", relationship_type)
    }

    /// Adds a term to one of the vocabularies an author may extend.
    pub fn define_vocabulary_term(
        &mut self,
        mut term: VocabularyTerm,
    ) -> Result<VocabularyTerm, Failure<R>> {
        require_text(&term.label, "term label")?;
        term.builtin = false;
        if self.find::<VocabularyTerm>(&term.id())?.is_some() {
            return Err(DomainError::already_defined(term.vocabulary.as_str(), &term.key).into());
        }
        self.insert(&term)?;
        Ok(term)
    }

    /// Defines a project-specific diplomatic stance.
    pub fn define_diplomatic_stance(
        &mut self,
        mut stance: DiplomaticStance,
    ) -> Result<DiplomaticStance, Failure<R>> {
        require_text(&stance.label, "stance label")?;
        stance.builtin = false;
        self.define_keyed("diplomatic stance", stance)
    }

    /// Defines a project-specific appearance kind.
    pub fn define_appearance_kind(
        &mut self,
        mut kind: AppearanceKind,
    ) -> Result<AppearanceKind, Failure<R>> {
        require_text(&kind.label, "appearance kind label")?;
        kind.builtin = false;
        self.define_keyed("appearance kind", kind)
    }

    /// Defines a level of the editorial hierarchy. Depths are unique.
    pub fn define_editorial_level(
        &mut self,
        mut level: EditorialLevel,
    ) -> Result<EditorialLevel, Failure<R>> {
        require_text(&level.label, "editorial level label")?;
        level.builtin = false;
        if self
            .list::<EditorialLevel>()?
            .iter()
            .any(|existing| existing.depth == level.depth)
        {
            return Err(DomainError::already_defined("editorial depth", level.depth).into());
        }
        self.define_keyed("editorial level", level)
    }

    /// Defines a section that organizes properties for display.
    pub fn define_property_group(
        &mut self,
        group: PropertyGroup,
    ) -> Result<PropertyGroup, Failure<R>> {
        require_text(&group.label, "property group label")?;
        if self.find::<PropertyGroup>(&group.key)?.is_some() {
            return Err(DomainError::already_defined("property group", &group.key).into());
        }
        self.insert(&group)?;
        Ok(group)
    }

    /// Defines a trait, characteristic or attribute of an entity type or relationship type.
    pub fn define_property_definition(
        &mut self,
        mut definition: PropertyDefinition,
    ) -> Result<PropertyDefinition, Failure<R>> {
        definition.validate()?;
        definition.builtin = false;
        if let Some(group) = &definition.group {
            self.require_defined::<PropertyGroup>("property group", group)?;
        }
        let related_owners: BTreeSet<TypeKey> = match &definition.owner {
            PropertyOwner::EntityType(key) => {
                self.require_defined::<EntityType>("entity type", key)?;
                // A key must stay unambiguous for every type that inherits or specializes it.
                let mut related: BTreeSet<TypeKey> =
                    self.type_chain(key)?.into_iter().map(|t| t.key).collect();
                for candidate in self.list::<EntityType>()? {
                    if self
                        .type_chain(&candidate.key)?
                        .iter()
                        .any(|t| &t.key == key)
                    {
                        related.insert(candidate.key);
                    }
                }
                related
            }
            PropertyOwner::RelationshipType(key) => {
                self.require_defined::<RelationshipType>("relationship type", key)?;
                BTreeSet::new()
            }
        };
        if let PropertyDataType::EntityReference {
            entity_type: Some(key),
        } = &definition.data_type
        {
            self.require_defined::<EntityType>("entity type", key)?;
        }
        let clash = self
            .list::<PropertyDefinition>()?
            .into_iter()
            .any(|existing| {
                existing.key == definition.key
                    && match (&existing.owner, &definition.owner) {
                        (PropertyOwner::EntityType(owner), PropertyOwner::EntityType(_)) => {
                            related_owners.contains(owner)
                        }
                        (existing_owner, new_owner) => existing_owner == new_owner,
                    }
            });
        if clash {
            return Err(DomainError::already_defined("property", &definition.key).into());
        }
        self.insert(&definition)?;
        Ok(definition)
    }

    /// Defines a calendar. Making it the default demotes the previous default.
    pub fn define_calendar(&mut self, calendar: Calendar) -> Result<Calendar, Failure<R>> {
        calendar.validate()?;
        if self.find::<Calendar>(&calendar.id)?.is_some() {
            return Err(DomainError::duplicate("calendar", calendar.id).into());
        }
        if calendar.is_default {
            for mut existing in self.list::<Calendar>()? {
                if existing.is_default {
                    existing.is_default = false;
                    self.update(&existing)?;
                }
            }
        }
        self.insert(&calendar)?;
        Ok(calendar)
    }
}
