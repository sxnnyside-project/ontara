use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::appearance::Appearance;
use crate::appearance_kind::AppearanceKind;
use crate::editorial_node::{EditorialNode, reading_order};
use crate::entity::Entity;
use crate::entity_search::{EntityFilter, find_entities};
use crate::event_detail::EventDetail;
use crate::hierarchy::HierarchyError;
use crate::identifier::{AppearanceId, EntityId};
use crate::repository::{ModelReader, Reader};
use crate::timeline::{TimelineQuery, TimelineSource, build_timeline};
use crate::type_key::TypeKey;

/// What the columns of the matrix are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum MatrixColumns {
    /// Sections of the work in reading order: the scenes of a chapter, the chapters of a book.
    Structure {
        /// Only what is inside this section; the whole work if absent.
        #[serde(default)]
        section: Option<EntityId>,
        /// Only sections of this level; the innermost sections (those with nothing inside) if
        /// absent.
        #[serde(default)]
        level: Option<TypeKey>,
    },
    /// Events in in-universe order, undated ones last.
    Events,
}

/// Which entities (rows) against which contexts (columns).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatrixQuery {
    #[serde(default)]
    pub rows: EntityFilter,
    pub columns: MatrixColumns,
    /// Keep rows with no appearance in any column, which are the entities nobody has placed yet.
    #[serde(default)]
    pub include_empty_rows: bool,
}

impl MatrixQuery {
    /// The innermost sections of the whole work against every entity.
    #[must_use]
    pub fn whole_work() -> Self {
        Self {
            rows: EntityFilter::default(),
            columns: MatrixColumns::Structure {
                section: None,
                level: None,
            },
            include_empty_rows: false,
        }
    }
}

/// One column: a section or an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatrixColumn {
    pub entity: EntityId,
    /// The editorial level; absent for events.
    pub level: Option<TypeKey>,
    pub depth: usize,
    pub parent: Option<EntityId>,
}

/// One recorded appearance of a row entity in a column context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatrixCell {
    pub entity: EntityId,
    pub context: EntityId,
    pub appearance: AppearanceId,
    pub kind: TypeKey,
    /// `false` for a confirmed absence, which is not the same as nothing recorded.
    pub is_presence: bool,
    pub viewpoint: Option<EntityId>,
    pub canon_state: TypeKey,
    pub position: i32,
}

/// Who is present where: rows by name, columns in reading or chronological order, and only the
/// cells somebody recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppearanceMatrix {
    pub columns: Vec<MatrixColumn>,
    pub rows: Vec<EntityId>,
    pub cells: Vec<MatrixCell>,
}

fn structure_columns(
    nodes: &[EditorialNode],
    section: Option<EntityId>,
    level: Option<&TypeKey>,
) -> Vec<MatrixColumn> {
    let order = reading_order(nodes);
    let inside: Vec<_> = match section {
        None => order.iter().collect(),
        Some(section) => {
            let Some(start) = order.iter().position(|step| step.entity == section) else {
                return Vec::new();
            };
            let base = order[start].depth;
            order[start + 1..]
                .iter()
                .take_while(|step| step.depth > base)
                .collect()
        }
    };
    inside
        .into_iter()
        .filter(|step| match level {
            Some(level) => &step.level == level,
            None => !step.has_children,
        })
        .map(|step| MatrixColumn {
            entity: step.entity,
            level: Some(step.level.clone()),
            depth: step.depth,
            parent: step.parent,
        })
        .collect()
}

fn event_columns<R: ModelReader>(reader: &R) -> Result<Vec<MatrixColumn>, R::Error> {
    let entities = Reader::<Entity>::list(reader)?;
    let events = Reader::<EventDetail>::list(reader)?;
    let timeline = build_timeline(
        &TimelineSource {
            entities: &entities,
            events: &events,
            appearances: &[],
            nodes: &[],
            causalities: &[],
        },
        &TimelineQuery::default(),
    );
    Ok(timeline
        .dated
        .iter()
        .chain(&timeline.undated)
        .map(|entry| MatrixColumn {
            entity: entry.entity,
            level: None,
            depth: 0,
            parent: None,
        })
        .collect())
}

/// Builds the matrix of appearances for the rows and columns asked for.
pub fn appearance_matrix<R: ModelReader>(
    reader: &R,
    query: &MatrixQuery,
) -> Result<AppearanceMatrix, HierarchyError<R::Error>> {
    let repository = HierarchyError::Repository;
    let columns = match &query.columns {
        MatrixColumns::Structure { section, level } => {
            let nodes = Reader::<EditorialNode>::list(reader).map_err(repository)?;
            structure_columns(&nodes, *section, level.as_ref())
        }
        MatrixColumns::Events => event_columns(reader).map_err(repository)?,
    };

    let kinds: BTreeMap<TypeKey, bool> = Reader::<AppearanceKind>::list(reader)
        .map_err(repository)?
        .into_iter()
        .map(|kind| (kind.key, kind.is_presence))
        .collect();
    let column_set: BTreeSet<EntityId> = columns.iter().map(|column| column.entity).collect();
    let candidates = find_entities(reader, &query.rows)?;
    let candidate_ids: BTreeSet<EntityId> = candidates.iter().map(|entity| entity.id).collect();

    let mut cells: Vec<MatrixCell> = Reader::<Appearance>::list(reader)
        .map_err(repository)?
        .into_iter()
        .filter(|appearance| {
            column_set.contains(&appearance.context) && candidate_ids.contains(&appearance.entity)
        })
        .map(|appearance| MatrixCell {
            entity: appearance.entity,
            context: appearance.context,
            appearance: appearance.id,
            is_presence: kinds.get(&appearance.kind).copied().unwrap_or(true),
            kind: appearance.kind,
            viewpoint: appearance.viewpoint,
            canon_state: appearance.canon_state,
            position: appearance.position,
        })
        .collect();
    cells.sort_by_key(|cell| (cell.context, cell.position, cell.appearance));

    let placed: BTreeSet<EntityId> = cells.iter().map(|cell| cell.entity).collect();
    let rows = candidates
        .into_iter()
        .filter(|entity| query.include_empty_rows || placed.contains(&entity.id))
        .map(|entity| entity.id)
        .collect();

    Ok(AppearanceMatrix {
        columns,
        rows,
        cells,
    })
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
    fn columns_default_to_the_innermost_sections_in_reading_order() {
        let (book, one, two, scene_a, scene_b) = (
            EntityId::generate(),
            EntityId::generate(),
            EntityId::generate(),
            EntityId::generate(),
            EntityId::generate(),
        );
        let nodes = [
            node(book, "volume", None, 0),
            node(one, "chapter", Some(book), 0),
            node(two, "chapter", Some(book), 1),
            node(scene_a, "scene", Some(one), 0),
            node(scene_b, "scene", Some(one), 1),
        ];
        let entities = |columns: Vec<MatrixColumn>| -> Vec<EntityId> {
            columns.into_iter().map(|column| column.entity).collect()
        };

        let whole = entities(structure_columns(&nodes, None, None));
        let chapters = entities(structure_columns(
            &nodes,
            None,
            Some(&TypeKey::from_static("chapter")),
        ));
        let inside_one = entities(structure_columns(&nodes, Some(one), None));

        assert_eq!(whole, vec![scene_a, scene_b, two]);
        assert_eq!(chapters, vec![one, two]);
        assert_eq!(inside_one, vec![scene_a, scene_b]);
    }
}
