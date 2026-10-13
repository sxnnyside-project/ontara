use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::identifier::EntityId;
use crate::record::Record;
use crate::type_key::TypeKey;

/// The place of an editorial entity in the hierarchy of a work.
///
/// The book, chapter or scene is an entity like any other, with its own properties and
/// relationships; the node only says where it sits and in which order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorialNode {
    pub entity: EntityId,
    pub level: TypeKey,
    pub parent: Option<EntityId>,
    /// Order among siblings.
    pub position: i32,
}

impl Record for EditorialNode {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}

/// A section with how deep it sits, listed where a reader meets it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadingStep {
    pub entity: EntityId,
    pub level: TypeKey,
    pub parent: Option<EntityId>,
    pub depth: usize,
    pub has_children: bool,
}

/// Every section in the order a reader goes through the work: each section, then what it
/// contains, siblings by position. Sections whose parent is not among `nodes` count as top-level,
/// and a loop is never followed twice, so the list is always finite.
#[must_use]
pub fn reading_order(nodes: &[EditorialNode]) -> Vec<ReadingStep> {
    let known: BTreeSet<EntityId> = nodes.iter().map(|node| node.entity).collect();
    let mut children: BTreeMap<Option<EntityId>, Vec<&EditorialNode>> = BTreeMap::new();
    for node in nodes {
        let parent = node.parent.filter(|parent| known.contains(parent));
        children.entry(parent).or_default().push(node);
    }
    for siblings in children.values_mut() {
        siblings.sort_by_key(|node| (node.position, node.entity));
    }

    let mut order = Vec::with_capacity(nodes.len());
    let mut visited = BTreeSet::new();
    let mut pending: Vec<(&EditorialNode, usize)> = children
        .get(&None)
        .into_iter()
        .flatten()
        .rev()
        .map(|node| (*node, 0))
        .collect();
    while let Some((node, depth)) = pending.pop() {
        if !visited.insert(node.entity) {
            continue;
        }
        let below = children.get(&Some(node.entity));
        order.push(ReadingStep {
            entity: node.entity,
            level: node.level.clone(),
            parent: node.parent,
            depth,
            has_children: below.is_some_and(|below| !below.is_empty()),
        });
        pending.extend(
            below
                .into_iter()
                .flatten()
                .rev()
                .map(|child| (*child, depth + 1)),
        );
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(
        entity: EntityId,
        level: &str,
        parent: Option<EntityId>,
        position: i32,
    ) -> EditorialNode {
        EditorialNode {
            entity,
            level: TypeKey::new(level).unwrap_or_else(|error| panic!("{error}")),
            parent,
            position,
        }
    }

    #[test]
    fn a_section_is_followed_by_what_it_contains_before_its_next_sibling() {
        let (book, first, second, scene) = (
            EntityId::generate(),
            EntityId::generate(),
            EntityId::generate(),
            EntityId::generate(),
        );
        let nodes = [
            node(second, "chapter", Some(book), 1),
            node(scene, "scene", Some(first), 0),
            node(first, "chapter", Some(book), 0),
            node(book, "volume", None, 0),
        ];

        let order: Vec<_> = reading_order(&nodes)
            .into_iter()
            .map(|step| (step.entity, step.depth, step.has_children))
            .collect();

        assert_eq!(
            order,
            vec![
                (book, 0, true),
                (first, 1, true),
                (scene, 2, false),
                (second, 1, false),
            ]
        );
    }

    #[test]
    fn a_loop_in_the_data_ends_the_listing_instead_of_hanging() {
        let (first, second) = (EntityId::generate(), EntityId::generate());
        let nodes = [
            node(first, "chapter", Some(second), 0),
            node(second, "chapter", Some(first), 0),
        ];

        assert!(reading_order(&nodes).is_empty());
    }
}
