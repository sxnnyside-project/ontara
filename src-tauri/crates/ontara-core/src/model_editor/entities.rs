use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::canon_state_change::CanonStateChange;
use crate::entity::Entity;
use crate::entity_type::EntityType;
use crate::error::DomainError;
use crate::identifier::{CanonStateChangeId, EntityId};
use crate::origin::Origin;
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::{ModelRepository, ReferenceCount};
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// How to resolve what still depends on an entity that is being removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalStrategy {
    /// Fail if anything else still references the entity.
    RejectIfReferenced,
    /// Remove what depends on the entity, and detach it from what merely mentions it.
    RemoveDependents,
}

impl<R: ModelRepository> ModelEditor<'_, R> {
    fn validate_entity(&self, entity: &Entity) -> Result<(), Failure<R>> {
        entity.validate()?;
        let entity_type = self.require_defined::<EntityType>("entity type", &entity.entity_type)?;
        if entity_type.is_abstract {
            return Err(DomainError::AbstractType(entity_type.key).into());
        }
        self.require_defined::<CanonState>("canon state", &entity.canon_state)?;
        for name in &entity.names {
            if let Some(language) = &name.language {
                self.require_entity_of(language, Some("language"))?;
            }
        }
        Ok(())
    }

    pub fn add_entity(&mut self, entity: Entity) -> Result<Entity, Failure<R>> {
        self.validate_entity(&entity)?;
        if self.find::<Entity>(&entity.id)?.is_some() {
            return Err(DomainError::duplicate("entity", entity.id).into());
        }
        self.insert(&entity)?;
        Ok(entity)
    }

    /// Replaces an entity's editable fields. Identity, type, origin and creation time are kept.
    pub fn update_entity(
        &mut self,
        mut entity: Entity,
        now: Timestamp,
    ) -> Result<Entity, Failure<R>> {
        let stored = self.require::<Entity>("entity", &entity.id)?;
        if stored.entity_type != entity.entity_type {
            return Err(DomainError::Immutable("entity type").into());
        }
        self.validate_entity(&entity)?;
        entity.created_at = stored.created_at;
        entity.origin = stored.origin;
        entity.updated_at = now;
        self.update(&entity)?;
        Ok(entity)
    }

    /// What still points at the entity, counted by the kind of record.
    pub fn references_to(&self, id: &EntityId) -> Result<Vec<ReferenceCount>, Failure<R>> {
        self.query(|repository| repository.references_to(id))
    }

    /// Removes an entity, resolving what depends on it as the strategy says, and returns what
    /// pointed at it. An entity that still has editorial children is never removed: move or
    /// remove them first.
    pub fn remove_entity(
        &mut self,
        id: &EntityId,
        strategy: RemovalStrategy,
    ) -> Result<Vec<ReferenceCount>, Failure<R>> {
        self.require::<Entity>("entity", id)?;
        if !self
            .query(|repository| repository.editorial_children(Some(id)))?
            .is_empty()
        {
            return Err(DomainError::EditorialChildrenRemain(*id).into());
        }
        let references = self.references_to(id)?;
        let total: usize = references.iter().map(|reference| reference.count).sum();
        if strategy == RemovalStrategy::RejectIfReferenced && total > 0 {
            return Err(DomainError::StillReferenced {
                entity: *id,
                references: total,
            }
            .into());
        }
        self.remove_record(&RecordRef::new(RecordKind::Entity, id.as_uuid()))?;
        Ok(references)
    }

    /// Moves a record to another canon state and keeps the reason for the change.
    ///
    /// Returns `None` when the record is already in that state.
    pub fn change_canon_state(
        &mut self,
        target: RecordRef,
        to: TypeKey,
        reason: Option<String>,
        origin: Origin,
        now: Timestamp,
    ) -> Result<Option<CanonStateChange>, Failure<R>> {
        self.require_defined::<CanonState>("canon state", &to)?;
        if reason.as_deref().is_some_and(|text| text.trim().is_empty()) {
            return Err(DomainError::BlankText("reason").into());
        }
        let from = self.swap_canon_state(&target, &to, now)?;
        if from == to {
            return Ok(None);
        }
        let change = CanonStateChange {
            id: CanonStateChangeId::generate(),
            target,
            from,
            to,
            reason,
            origin,
            changed_at: now,
        };
        self.insert(&change)?;
        Ok(Some(change))
    }
}
