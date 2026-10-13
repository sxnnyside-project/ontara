use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::appearance_kind::AppearanceKind;
use crate::canon_state::CanonState;
use crate::diplomatic_stance::DiplomaticStance;
use crate::editorial_level::EditorialLevel;
use crate::entity::require_text;
use crate::entity_type::EntityType;
use crate::error::DomainError;
use crate::property_definition::{PropertyDataType, PropertyDefinition, PropertyGroup};
use crate::relationship_type::RelationshipType;
use crate::repository::ModelRepository;
use crate::type_key::TypeKey;
use crate::vocabulary::VocabularyTerm;

/// Fails when a field that gives a catalog entry its meaning was changed.
fn unchanged<T: PartialEq>(stored: &T, given: &T, what: &'static str) -> Result<(), DomainError> {
    if stored == given {
        Ok(())
    } else {
        Err(DomainError::Immutable(what))
    }
}

/// Whether an endpoint rule only grows: an empty rule allows every type, so it can never be
/// narrowed, and a listed rule may add types but not drop any.
fn only_widens(stored: &BTreeSet<TypeKey>, given: &BTreeSet<TypeKey>) -> bool {
    given.is_empty() || (!stored.is_empty() && given.is_superset(stored))
}

/// Terminology and configuration of the catalogs.
///
/// Every operation takes the entry as the author wants it to be. The label, the description and
/// the order are theirs to change at any time, for built-in entries too. What gives an entry its
/// meaning to the rules (a type's parent and role, a relationship type's direction) is fixed
/// once it exists, because records already depend on it.
impl<R: ModelRepository> ModelEditor<'_, R> {
    pub fn update_canon_state(&mut self, state: CanonState) -> Result<CanonState, Failure<R>> {
        require_text(&state.label, "canon state label")?;
        let stored = self.require_defined::<CanonState>("canon state", &state.key)?;
        let state = CanonState {
            builtin: stored.builtin,
            ..state
        };
        self.update(&state)?;
        Ok(state)
    }

    pub fn update_entity_type(
        &mut self,
        entity_type: EntityType,
    ) -> Result<EntityType, Failure<R>> {
        require_text(&entity_type.label, "entity type label")?;
        let stored = self.require_defined::<EntityType>("entity type", &entity_type.key)?;
        unchanged(&stored.parent, &entity_type.parent, "parent type")?;
        unchanged(&stored.role, &entity_type.role, "role")?;
        unchanged(
            &stored.is_abstract,
            &entity_type.is_abstract,
            "abstract flag",
        )?;
        let entity_type = EntityType {
            builtin: stored.builtin,
            ..entity_type
        };
        self.update(&entity_type)?;
        Ok(entity_type)
    }

    /// Endpoint rules may be widened, never narrowed, so existing relationships stay valid.
    pub fn update_relationship_type(
        &mut self,
        relationship_type: RelationshipType,
    ) -> Result<RelationshipType, Failure<R>> {
        relationship_type.validate()?;
        let stored =
            self.require_defined::<RelationshipType>("relationship type", &relationship_type.key)?;
        unchanged(
            &stored.directionality,
            &relationship_type.directionality,
            "directionality",
        )?;
        unchanged(
            &stored.acyclic,
            &relationship_type.acyclic,
            "hierarchy flag",
        )?;
        for (stored_rule, given_rule, side) in [
            (
                &stored.allowed_sources,
                &relationship_type.allowed_sources,
                "allowed sources",
            ),
            (
                &stored.allowed_targets,
                &relationship_type.allowed_targets,
                "allowed targets",
            ),
        ] {
            if !only_widens(stored_rule, given_rule) {
                return Err(DomainError::Immutable(side).into());
            }
            for key in given_rule {
                self.require_defined::<EntityType>("entity type", key)?;
            }
        }
        let relationship_type = RelationshipType {
            builtin: stored.builtin,
            ..relationship_type
        };
        self.update(&relationship_type)?;
        Ok(relationship_type)
    }

    /// Only the label can change: depths order the levels and nodes already sit in them.
    pub fn update_editorial_level(
        &mut self,
        level: EditorialLevel,
    ) -> Result<EditorialLevel, Failure<R>> {
        require_text(&level.label, "editorial level label")?;
        let stored = self.require_defined::<EditorialLevel>("editorial level", &level.key)?;
        unchanged(&stored.depth, &level.depth, "depth")?;
        let level = EditorialLevel {
            builtin: stored.builtin,
            ..level
        };
        self.update(&level)?;
        Ok(level)
    }

    pub fn update_appearance_kind(
        &mut self,
        kind: AppearanceKind,
    ) -> Result<AppearanceKind, Failure<R>> {
        require_text(&kind.label, "appearance kind label")?;
        let stored = self.require_defined::<AppearanceKind>("appearance kind", &kind.key)?;
        unchanged(&stored.is_presence, &kind.is_presence, "presence flag")?;
        let kind = AppearanceKind {
            builtin: stored.builtin,
            ..kind
        };
        self.update(&kind)?;
        Ok(kind)
    }

    pub fn update_diplomatic_stance(
        &mut self,
        stance: DiplomaticStance,
    ) -> Result<DiplomaticStance, Failure<R>> {
        require_text(&stance.label, "stance label")?;
        let stored = self.require_defined::<DiplomaticStance>("diplomatic stance", &stance.key)?;
        unchanged(
            &stored.is_hierarchical,
            &stance.is_hierarchical,
            "hierarchy flag",
        )?;
        let stance = DiplomaticStance {
            builtin: stored.builtin,
            ..stance
        };
        self.update(&stance)?;
        Ok(stance)
    }

    pub fn update_vocabulary_term(
        &mut self,
        term: VocabularyTerm,
    ) -> Result<VocabularyTerm, Failure<R>> {
        require_text(&term.label, "term label")?;
        let stored = self.require_term(term.vocabulary, &term.key)?;
        let term = VocabularyTerm {
            builtin: stored.builtin,
            ..term
        };
        self.update(&term)?;
        Ok(term)
    }

    pub fn update_property_group(
        &mut self,
        group: PropertyGroup,
    ) -> Result<PropertyGroup, Failure<R>> {
        require_text(&group.label, "property group label")?;
        self.require_defined::<PropertyGroup>("property group", &group.key)?;
        self.update(&group)?;
        Ok(group)
    }

    /// The label, description, group and order change freely. A choice may gain options but
    /// never lose one, because values already point at them; nothing else about the shape of
    /// the property changes once values may exist.
    pub fn update_property_definition(
        &mut self,
        definition: PropertyDefinition,
    ) -> Result<PropertyDefinition, Failure<R>> {
        definition.validate()?;
        let stored = self.require::<PropertyDefinition>("property definition", &definition.id)?;
        unchanged(&stored.owner, &definition.owner, "owner")?;
        unchanged(&stored.key, &definition.key, "key")?;
        unchanged(&stored.required, &definition.required, "required flag")?;
        unchanged(&stored.multiple, &definition.multiple, "multiple flag")?;
        match (&stored.data_type, &definition.data_type) {
            (
                PropertyDataType::Choice { options: before },
                PropertyDataType::Choice { options: after },
            ) => {
                if before
                    .iter()
                    .any(|option| after.iter().all(|kept| kept.key != option.key))
                {
                    return Err(DomainError::Immutable("choice options").into());
                }
            }
            (before, after) => unchanged(before, after, "data type")?,
        }
        if let Some(group) = &definition.group {
            self.require_defined::<PropertyGroup>("property group", group)?;
        }
        let definition = PropertyDefinition {
            builtin: stored.builtin,
            ..definition
        };
        self.update(&definition)?;
        Ok(definition)
    }
}
