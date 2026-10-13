use std::collections::{BTreeSet, VecDeque};

use thiserror::Error;

use crate::entity_type::EntityType;
use crate::error::DomainError;
use crate::identifier::{EntityId, RelationshipId};
use crate::property_definition::{PropertyDefinition, PropertyOwner};
use crate::relationship::Lifecycle;
use crate::repository::{ModelReader, Reader};
use crate::type_key::TypeKey;

/// Longest chain of entity types followed before the hierarchy is declared broken.
const MAX_TYPE_DEPTH: usize = 32;

/// A hierarchy question failed: the data is inconsistent, or the repository failed.
#[derive(Debug, Error)]
pub enum HierarchyError<E: std::error::Error + 'static> {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("the repository failed: {0}")]
    Repository(#[source] E),
}

/// The type itself followed by each ancestor, nearest first.
pub fn entity_type_chain<R: ModelReader>(
    reader: &R,
    key: &TypeKey,
) -> Result<Vec<EntityType>, HierarchyError<R::Error>> {
    let mut chain: Vec<EntityType> = Vec::new();
    let mut next = Some(key.clone());
    while let Some(current) = next {
        if chain.len() >= MAX_TYPE_DEPTH || chain.iter().any(|seen| seen.key == current) {
            return Err(DomainError::InvalidTypeHierarchy(format!(
                "`{key}` is part of a cycle or is nested too deeply"
            ))
            .into());
        }
        let found = Reader::<EntityType>::get(reader, &current)
            .map_err(HierarchyError::Repository)?
            .ok_or_else(|| DomainError::not_defined("entity type", &current))?;
        let parent = found.parent.clone();
        chain.push(found);
        next = parent;
    }
    Ok(chain)
}

/// Whether `actual` is `expected` or one of its subtypes.
pub fn is_entity_type<R: ModelReader>(
    reader: &R,
    actual: &TypeKey,
    expected: &TypeKey,
) -> Result<bool, HierarchyError<R::Error>> {
    Ok(entity_type_chain(reader, actual)?
        .iter()
        .any(|entity_type| &entity_type.key == expected))
}

/// Every property definition that applies to entities of a type, ancestors first.
pub fn applicable_property_definitions<R: ModelReader>(
    reader: &R,
    key: &TypeKey,
) -> Result<Vec<PropertyDefinition>, HierarchyError<R::Error>> {
    let mut definitions = Vec::new();
    for entity_type in entity_type_chain(reader, key)?.iter().rev() {
        let owner = PropertyOwner::EntityType(entity_type.key.clone());
        let mut declared = reader
            .property_definitions_of(&owner)
            .map_err(HierarchyError::Repository)?;
        declared.sort_by_key(|definition| definition.sort_order);
        definitions.append(&mut declared);
    }
    Ok(definitions)
}

/// What an entity is directly inside of, following one hierarchical relationship type, then
/// what those are inside of, and so on. Nearest first. Historical relationships are ignored.
pub fn containers_of<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    relationship_type: &TypeKey,
) -> Result<Vec<EntityId>, HierarchyError<R::Error>> {
    walk(reader, entity, relationship_type, Direction::Outwards, None)
}

/// What is directly inside an entity, and what is inside those. Nearest first.
pub fn contents_of<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    relationship_type: &TypeKey,
) -> Result<Vec<EntityId>, HierarchyError<R::Error>> {
    walk(reader, entity, relationship_type, Direction::Inwards, None)
}

/// Whether `to` can be reached from `from` along active relationships of one type.
///
/// `ignoring` lets an edit be checked as if the relationship being replaced were not there.
pub fn reaches<R: ModelReader>(
    reader: &R,
    from: &EntityId,
    to: &EntityId,
    relationship_type: &TypeKey,
    ignoring: Option<&RelationshipId>,
) -> Result<bool, HierarchyError<R::Error>> {
    Ok(from == to
        || walk(
            reader,
            from,
            relationship_type,
            Direction::Outwards,
            ignoring,
        )?
        .contains(to))
}

#[derive(Clone, Copy)]
enum Direction {
    /// From source to target: what an entity is inside of.
    Outwards,
    /// From target to source: what is inside an entity.
    Inwards,
}

fn walk<R: ModelReader>(
    reader: &R,
    start: &EntityId,
    relationship_type: &TypeKey,
    direction: Direction,
    ignoring: Option<&RelationshipId>,
) -> Result<Vec<EntityId>, HierarchyError<R::Error>> {
    let mut visited: BTreeSet<EntityId> = BTreeSet::from([*start]);
    let mut order = Vec::new();
    let mut queue = VecDeque::from([*start]);
    while let Some(current) = queue.pop_front() {
        let relationships = reader
            .relationships_of(&current)
            .map_err(HierarchyError::Repository)?;
        for relationship in relationships {
            if &relationship.relationship_type != relationship_type
                || relationship.lifecycle == Lifecycle::Historical
                || ignoring == Some(&relationship.id)
            {
                continue;
            }
            let neighbour = match direction {
                Direction::Outwards if relationship.source == current => relationship.target,
                Direction::Inwards if relationship.target == current => relationship.source,
                Direction::Outwards | Direction::Inwards => continue,
            };
            if visited.insert(neighbour) {
                order.push(neighbour);
                queue.push_back(neighbour);
            }
        }
    }
    Ok(order)
}

/// The places a place sits inside through one kind of containment, nearest first. Containment
/// that has ended is history and is ignored.
pub fn places_containing<R: ModelReader>(
    reader: &R,
    place: &EntityId,
    kind: &TypeKey,
) -> Result<Vec<EntityId>, HierarchyError<R::Error>> {
    walk_places(reader, place, kind, true)
}

/// The places inside a place, and the places inside those, nearest first.
pub fn places_within<R: ModelReader>(
    reader: &R,
    place: &EntityId,
    kind: &TypeKey,
) -> Result<Vec<EntityId>, HierarchyError<R::Error>> {
    walk_places(reader, place, kind, false)
}

fn walk_places<R: ModelReader>(
    reader: &R,
    start: &EntityId,
    kind: &TypeKey,
    upwards: bool,
) -> Result<Vec<EntityId>, HierarchyError<R::Error>> {
    let mut visited: BTreeSet<EntityId> = BTreeSet::from([*start]);
    let mut order = Vec::new();
    let mut queue = VecDeque::from([*start]);
    while let Some(current) = queue.pop_front() {
        let edges = if upwards {
            reader.place_containment_by_child(&current)
        } else {
            reader.place_containment_by_parent(&current)
        }
        .map_err(HierarchyError::Repository)?;
        for edge in edges {
            if &edge.kind != kind || !edge.period.is_open() {
                continue;
            }
            let neighbour = if upwards { edge.parent } else { edge.child };
            if visited.insert(neighbour) {
                order.push(neighbour);
                queue.push_back(neighbour);
            }
        }
    }
    Ok(order)
}
