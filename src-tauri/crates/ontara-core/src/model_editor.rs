// <generated modules>
mod being;
mod belief;
mod event;
mod faction;
mod item;
mod place;
mod polity;
// </generated modules>
mod catalog;
mod entities;
mod narrative;
mod properties;
mod record_kinds;
mod relationships;
mod retirement;
mod supporting;
mod terminology;

use std::fmt::Display;

use thiserror::Error;

use crate::entity::Entity;
use crate::entity_type::{EntityRole, EntityType};
use crate::error::DomainError;
use crate::hierarchy::{self, HierarchyError};
use crate::identifier::EntityId;
use crate::record::Record;
use crate::record_ref::RecordRef;
use crate::repository::{ModelRepository, Reader, RecordRemoval, Repository, Writer};
use crate::type_key::TypeKey;
use crate::vocabulary::{Vocabulary, VocabularyTerm};

pub use entities::RemovalStrategy;
pub use narrative::EditorialImpact;

/// An edit failed: either a domain rule rejected it, or the repository could not complete it.
#[derive(Debug, Error)]
pub enum EditError<E: std::error::Error + 'static> {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("the repository failed: {0}")]
    Repository(#[source] E),
}

impl<E: std::error::Error + 'static> From<HierarchyError<E>> for EditError<E> {
    fn from(error: HierarchyError<E>) -> Self {
        match error {
            HierarchyError::Domain(domain) => Self::Domain(domain),
            HierarchyError::Repository(repository) => Self::Repository(repository),
        }
    }
}

pub(crate) type Failure<R> = EditError<<R as Repository>::Error>;

/// Applies edits to a universe while enforcing every domain rule.
///
/// This is the only sanctioned way to write records. Run it inside one repository transaction so
/// a failed multi-record edit leaves nothing partially applied.
pub struct ModelEditor<'repository, R: ModelRepository> {
    repository: &'repository mut R,
}

impl<'repository, R: ModelRepository> ModelEditor<'repository, R> {
    pub fn new(repository: &'repository mut R) -> Self {
        Self { repository }
    }

    pub(crate) fn find<T: Record>(&self, id: &T::Id) -> Result<Option<T>, Failure<R>>
    where
        R: Reader<T>,
    {
        <R as Reader<T>>::get(self.repository, id).map_err(EditError::Repository)
    }

    pub(crate) fn require<T: Record>(&self, what: &'static str, id: &T::Id) -> Result<T, Failure<R>>
    where
        R: Reader<T>,
        T::Id: Display,
    {
        self.find::<T>(id)?
            .ok_or_else(|| DomainError::not_found(what, id).into())
    }

    pub(crate) fn require_defined<T: Record>(
        &self,
        what: &'static str,
        key: &T::Id,
    ) -> Result<T, Failure<R>>
    where
        R: Reader<T>,
        T::Id: Display,
    {
        self.find::<T>(key)?
            .ok_or_else(|| DomainError::not_defined(what, key).into())
    }

    pub(crate) fn list<T: Record>(&self) -> Result<Vec<T>, Failure<R>>
    where
        R: Reader<T>,
    {
        <R as Reader<T>>::list(self.repository).map_err(EditError::Repository)
    }

    pub(crate) fn insert<T: Record>(&mut self, record: &T) -> Result<(), Failure<R>>
    where
        R: Writer<T>,
    {
        <R as Writer<T>>::insert(self.repository, record).map_err(EditError::Repository)
    }

    pub(crate) fn update<T: Record>(&mut self, record: &T) -> Result<(), Failure<R>>
    where
        R: Writer<T>,
    {
        <R as Writer<T>>::update(self.repository, record).map_err(EditError::Repository)
    }

    pub(crate) fn delete<T: Record>(&mut self, id: &T::Id) -> Result<(), Failure<R>>
    where
        R: Writer<T>,
    {
        <R as Writer<T>>::delete(self.repository, id).map_err(EditError::Repository)
    }

    pub(crate) fn query<T>(
        &self,
        ask: impl FnOnce(&R) -> Result<T, <R as Repository>::Error>,
    ) -> Result<T, Failure<R>> {
        ask(self.repository).map_err(EditError::Repository)
    }

    /// An entity that exists and, when `expected` is given, is of that type or a subtype of it.
    pub(crate) fn require_entity_of(
        &self,
        id: &EntityId,
        expected: Option<&'static str>,
    ) -> Result<Entity, Failure<R>> {
        let entity = self.require::<Entity>("entity", id)?;
        if let Some(expected) = expected {
            let expected = TypeKey::from_static(expected);
            if !self.is_type(&entity.entity_type, &expected)? {
                return Err(DomainError::WrongEntityType {
                    entity: entity.id,
                    actual: entity.entity_type,
                    expected,
                }
                .into());
            }
        }
        Ok(entity)
    }

    /// An entity whose type has the given role, wherever it sits in the type hierarchy.
    pub(crate) fn require_entity_with_role(
        &self,
        id: &EntityId,
        role: EntityRole,
        required: &'static str,
    ) -> Result<Entity, Failure<R>> {
        self.entity_with_role(id, role, required)
    }

    /// A term that exists in the vocabulary.
    pub(crate) fn require_term(
        &self,
        vocabulary: Vocabulary,
        key: &TypeKey,
    ) -> Result<VocabularyTerm, Failure<R>> {
        self.find::<VocabularyTerm>(&(vocabulary, key.clone()))?
            .ok_or_else(|| DomainError::not_defined(vocabulary.as_str(), key).into())
    }

    /// A record that is keyed by an entity or by a pair of entities.
    pub(crate) fn require_key<T: Record>(
        &self,
        what: &'static str,
        id: &T::Id,
    ) -> Result<T, Failure<R>>
    where
        R: Reader<T>,
    {
        self.find::<T>(id)?
            .ok_or_else(|| DomainError::not_found(what, format!("{id:?}")).into())
    }

    /// Removes a record together with everything that cannot exist without it.
    pub(crate) fn remove_record(&mut self, target: &RecordRef) -> Result<(), Failure<R>> {
        RecordRemoval::remove_record(self.repository, target).map_err(EditError::Repository)
    }

    /// The type and its ancestors, nearest first.
    pub(crate) fn type_chain(&self, key: &TypeKey) -> Result<Vec<EntityType>, Failure<R>> {
        Ok(hierarchy::entity_type_chain(&*self.repository, key)?)
    }

    /// Whether `actual` is `expected` or one of its subtypes.
    pub(crate) fn is_type(&self, actual: &TypeKey, expected: &TypeKey) -> Result<bool, Failure<R>> {
        Ok(hierarchy::is_entity_type(
            &*self.repository,
            actual,
            expected,
        )?)
    }

    /// An entity together with its type.
    pub(crate) fn entity_and_type(
        &self,
        id: &EntityId,
    ) -> Result<(Entity, EntityType), Failure<R>> {
        let entity = self.require::<Entity>("entity", id)?;
        let entity_type = self.require_defined::<EntityType>("entity type", &entity.entity_type)?;
        Ok((entity, entity_type))
    }

    /// An entity whose type has the given role.
    pub(crate) fn entity_with_role(
        &self,
        id: &EntityId,
        role: EntityRole,
        required: &'static str,
    ) -> Result<Entity, Failure<R>> {
        let (entity, entity_type) = self.entity_and_type(id)?;
        if entity_type.role == role {
            Ok(entity)
        } else {
            Err(DomainError::RoleMismatch {
                entity: entity.id,
                actual: entity.entity_type,
                required,
            }
            .into())
        }
    }

    /// An entity that can serve as a narrative context: an editorial unit or an event.
    pub(crate) fn narrative_context(&self, id: &EntityId) -> Result<Entity, Failure<R>> {
        let (entity, entity_type) = self.entity_and_type(id)?;
        match entity_type.role {
            EntityRole::EditorialUnit | EntityRole::Event => Ok(entity),
            EntityRole::Generic | EntityRole::Asset => Err(DomainError::RoleMismatch {
                entity: entity.id,
                actual: entity.entity_type,
                required: "editorial unit or event",
            }
            .into()),
        }
    }
}
