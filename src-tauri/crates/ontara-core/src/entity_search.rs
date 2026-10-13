use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::entity::Entity;
use crate::entity_type::{EntityRole, EntityType};
use crate::hierarchy::HierarchyError;
use crate::repository::{ModelReader, Reader};
use crate::type_key::TypeKey;

/// Longest chain of types followed when working out which entities a type covers.
const MAX_TYPE_DEPTH: usize = 32;

/// What an author is looking for when they browse the entities of a universe.
///
/// Every field narrows the result; an empty filter matches every entity that is not archived.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EntityFilter {
    /// This type or any of its subtypes: filtering by `place` includes cities.
    pub entity_type: Option<TypeKey>,
    pub role: Option<EntityRole>,
    pub canon_state: Option<TypeKey>,
    pub tag: Option<String>,
    /// Matched, ignoring case, against the name, the alternative names, the summary and the tags.
    pub text: Option<String>,
    pub include_archived: bool,
}

impl EntityFilter {
    fn matches_text(entity: &Entity, needle: &str) -> bool {
        let contains = |haystack: &str| haystack.to_lowercase().contains(needle);
        contains(&entity.name)
            || contains(&entity.summary)
            || entity.sort_name.as_deref().is_some_and(contains)
            || entity.names.iter().any(|name| contains(&name.name))
            || entity.tags.iter().any(|tag| contains(tag))
    }
}

/// The entities that match the filter, in the order an author expects to read them: by sort name,
/// falling back to the name.
pub fn find_entities<R: ModelReader>(
    reader: &R,
    filter: &EntityFilter,
) -> Result<Vec<Entity>, HierarchyError<R::Error>> {
    let covered = covered_types(reader, filter)?;
    let needle = filter
        .text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_lowercase);

    let mut found: Vec<Entity> = Reader::<Entity>::list(reader)
        .map_err(HierarchyError::Repository)?
        .into_iter()
        .filter(|entity| filter.include_archived || !entity.archived)
        .filter(|entity| covered.contains(&entity.entity_type))
        .filter(|entity| {
            filter
                .canon_state
                .as_ref()
                .is_none_or(|state| &entity.canon_state == state)
        })
        .filter(|entity| {
            filter
                .tag
                .as_ref()
                .is_none_or(|tag| entity.tags.contains(tag))
        })
        .filter(|entity| {
            needle
                .as_deref()
                .is_none_or(|text| EntityFilter::matches_text(entity, text))
        })
        .collect();

    found.sort_by_cached_key(|entity| {
        (
            entity
                .sort_name
                .as_deref()
                .unwrap_or(&entity.name)
                .to_lowercase(),
            entity.id,
        )
    });
    Ok(found)
}

/// The keys of every type whose entities the filter's type and role accept.
fn covered_types<R: ModelReader>(
    reader: &R,
    filter: &EntityFilter,
) -> Result<BTreeSet<TypeKey>, HierarchyError<R::Error>> {
    let types: BTreeMap<TypeKey, EntityType> = Reader::<EntityType>::list(reader)
        .map_err(HierarchyError::Repository)?
        .into_iter()
        .map(|entity_type| (entity_type.key.clone(), entity_type))
        .collect();

    let covered = types
        .values()
        .filter(|entity_type| filter.role.is_none_or(|role| entity_type.role == role))
        .filter(|entity_type| {
            let Some(wanted) = &filter.entity_type else {
                return true;
            };
            let mut current = Some(&entity_type.key);
            for _ in 0..MAX_TYPE_DEPTH {
                let Some(key) = current else {
                    return false;
                };
                if key == wanted {
                    return true;
                }
                current = types.get(key).and_then(|found| found.parent.as_ref());
            }
            false
        })
        .map(|entity_type| entity_type.key.clone())
        .collect();
    Ok(covered)
}
