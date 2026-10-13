//! Entities: browsing, creating, describing and removing the elements of a universe.

use std::collections::{BTreeMap, BTreeSet};

use ontara_core::{
    CanonStateChange, Entity, EntityFilter, EntityId, EntityName, EntityType, Instant,
    ModelQueries, ModelReader, ModelReaderExt, Note, NoteId, Origin, PropertyContent,
    PropertyDefinition, PropertyDefinitionId, PropertyValue, PropertyValueId, PropertyValueOwner,
    Reader, RecordKind, RecordRef, ReferenceCount, RemovalStrategy, Timestamp, TypeKey,
    applicable_property_definitions, find_entities,
};
use serde::{Deserialize, Serialize};

use ontara_storage::StorageError;

use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// A pointer to an entity that is enough to show it and to navigate to it.
///
/// Mirrored by `src/features/entity/api/entity-api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntityRef {
    pub id: EntityId,
    pub name: String,
    pub entity_type: TypeKey,
    pub type_label: String,
    pub canon_state: TypeKey,
    pub archived: bool,
}

/// One row of an entity list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntitySummary {
    #[serde(flatten)]
    pub reference: EntityRef,
    pub sort_name: Option<String>,
    pub summary: String,
    pub tags: BTreeSet<String>,
    pub updated_at: Timestamp,
}

/// Everything shown when one entity is open.
#[derive(Debug, Serialize)]
pub struct EntityDetail {
    pub entity: Entity,
    pub type_label: String,
    /// The entity's type followed by its ancestors, nearest first.
    pub type_chain: Vec<TypeKey>,
    /// The properties the entity's type, and the types it inherits from, define.
    pub definitions: Vec<PropertyDefinition>,
    pub values: Vec<PropertyValue>,
    pub notes: Vec<Note>,
    pub canon_history: Vec<CanonStateChange>,
}

#[derive(Debug, Deserialize)]
pub struct NewEntity {
    pub entity_type: TypeKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: BTreeSet<String>,
    pub canon_state: Option<TypeKey>,
}

/// The fields of an entity an author edits directly. Type and canon state are not among them:
/// the type is fixed, and the canon state changes through [`change_canon_state`], which keeps
/// the reason.
#[derive(Debug, Deserialize)]
pub struct EntityEdit {
    pub id: EntityId,
    pub name: String,
    pub sort_name: Option<String>,
    pub summary: String,
    pub description: String,
    pub tags: BTreeSet<String>,
    pub names: Vec<EntityName>,
    pub archived: bool,
}

#[derive(Debug, Deserialize)]
pub struct PropertyValueInput {
    /// Present when updating a value, absent when adding one.
    pub id: Option<PropertyValueId>,
    pub owner: PropertyValueOwner,
    pub definition: PropertyDefinitionId,
    pub content: PropertyContent,
    pub valid_from: Option<Instant>,
    pub valid_until: Option<Instant>,
    pub note: Option<String>,
    #[serde(default)]
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct NoteInput {
    pub id: Option<NoteId>,
    pub entity: EntityId,
    pub title: String,
    pub body: String,
}

/// Names the type of every entity type, so lists can show it without a second round trip.
pub(crate) fn type_labels(
    reader: &impl ModelReader<Error = StorageError>,
) -> Result<BTreeMap<TypeKey, String>, StorageError> {
    Ok(reader
        .entity_types()?
        .into_iter()
        .map(|entity_type: EntityType| (entity_type.key, entity_type.label))
        .collect())
}

pub(crate) fn reference(entity: &Entity, labels: &BTreeMap<TypeKey, String>) -> EntityRef {
    EntityRef {
        id: entity.id,
        name: entity.name.clone(),
        entity_type: entity.entity_type.clone(),
        type_label: labels
            .get(&entity.entity_type)
            .cloned()
            .unwrap_or_else(|| entity.entity_type.to_string()),
        canon_state: entity.canon_state.clone(),
        archived: entity.archived,
    }
}

/// Resolves an entity identifier that a stored record points at.
pub(crate) fn resolve(
    reader: &impl ModelReader<Error = StorageError>,
    labels: &BTreeMap<TypeKey, String>,
    id: &EntityId,
) -> Result<EntityRef, CommandError> {
    reader
        .entity(id)?
        .map(|entity| reference(&entity, labels))
        .ok_or_else(|| CommandError::new("not_found", format!("entity {id} does not exist")))
}

/// Resolves several entity identifiers, in the order given.
pub(crate) fn resolve_all<'a>(
    reader: &impl ModelReader<Error = StorageError>,
    labels: &BTreeMap<TypeKey, String>,
    ids: impl IntoIterator<Item = &'a EntityId>,
) -> Result<Vec<EntityRef>, CommandError> {
    ids.into_iter()
        .map(|id| resolve(reader, labels, id))
        .collect()
}

pub(crate) fn summary_of(entity: &Entity, labels: &BTreeMap<TypeKey, String>) -> EntitySummary {
    EntitySummary {
        reference: reference(entity, labels),
        sort_name: entity.sort_name.clone(),
        summary: entity.summary.clone(),
        tags: entity.tags.clone(),
        updated_at: entity.updated_at,
    }
}

pub fn list(
    workspace: &Workspace,
    filter: &EntityFilter,
) -> Result<Vec<EntitySummary>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        Ok(find_entities(&reader, filter)?
            .iter()
            .map(|entity| summary_of(entity, &labels))
            .collect())
    })
}

pub fn detail(workspace: &Workspace, id: EntityId) -> Result<EntityDetail, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let entity = reader
            .entity(&id)?
            .ok_or_else(|| CommandError::new("not_found", format!("entity {id} does not exist")))?;
        let type_chain = ontara_core::entity_type_chain(&reader, &entity.entity_type)?
            .into_iter()
            .map(|entity_type| entity_type.key)
            .collect();
        Ok(EntityDetail {
            type_label: labels
                .get(&entity.entity_type)
                .cloned()
                .unwrap_or_else(|| entity.entity_type.to_string()),
            type_chain,
            definitions: applicable_property_definitions(&reader, &entity.entity_type)?,
            values: reader.property_values_of(&PropertyValueOwner::Entity(id))?,
            notes: reader.notes_of(&id)?,
            canon_history: reader
                .canon_history(&RecordRef::new(RecordKind::Entity, id.as_uuid()))?,
            entity,
        })
    })
}

pub fn create(workspace: &Workspace, input: NewEntity) -> Result<Entity, CommandError> {
    workspace.record("entity.create", |project| {
        let now = Timestamp::now();
        let mut entity = Entity::new(input.entity_type, input.name, now);
        entity.summary = input.summary;
        entity.description = input.description;
        entity.tags = input.tags;
        if let Some(state) = input.canon_state {
            entity.canon_state = state;
        }
        Ok(project.edit(|editor| editor.add_entity(entity))?)
    })
}

/// The record, or a `not_found` failure naming what was looked for.
pub(crate) fn found<T>(
    record: Option<T>,
    what: &str,
    id: impl std::fmt::Display,
) -> Result<T, CommandError> {
    record.ok_or_else(|| CommandError::new("not_found", format!("{what} {id} does not exist")))
}

pub fn update(workspace: &Workspace, edit: EntityEdit) -> Result<Entity, CommandError> {
    workspace.record("entity.update", |project| {
        let mut entity = found(project.reader().entity(&edit.id)?, "entity", edit.id)?;
        entity.name = edit.name;
        entity.sort_name = edit.sort_name;
        entity.summary = edit.summary;
        entity.description = edit.description;
        entity.tags = edit.tags;
        entity.names = edit.names;
        entity.archived = edit.archived;
        Ok(project.edit(|editor| editor.update_entity(entity, Timestamp::now()))?)
    })
}

pub fn references(
    workspace: &Workspace,
    id: &EntityId,
) -> Result<Vec<ReferenceCount>, CommandError> {
    workspace.read(|project| Ok(project.reader().references_to(id)?))
}

pub fn remove(
    workspace: &Workspace,
    id: &EntityId,
    remove_dependents: bool,
) -> Result<Vec<ReferenceCount>, CommandError> {
    let strategy = if remove_dependents {
        RemovalStrategy::RemoveDependents
    } else {
        RemovalStrategy::RejectIfReferenced
    };
    workspace.record("entity.remove", |project| {
        Ok(project.edit(|editor| editor.remove_entity(id, strategy))?)
    })
}

pub fn change_canon_state(
    workspace: &Workspace,
    target: RecordRef,
    to: TypeKey,
    reason: Option<String>,
) -> Result<Option<CanonStateChange>, CommandError> {
    workspace.record("entity.change_canon_state", |project| {
        Ok(project.edit(|editor| {
            editor.change_canon_state(target, to, reason, Origin::User, Timestamp::now())
        })?)
    })
}

pub fn save_property_value(
    workspace: &Workspace,
    input: PropertyValueInput,
) -> Result<PropertyValue, CommandError> {
    workspace.record("entity.save_property_value", |project| {
        let now = Timestamp::now();
        let stored = match input.id {
            Some(id) => Some(found(
                Reader::<PropertyValue>::get(&project.reader(), &id)?,
                "property value",
                id,
            )?),
            None => None,
        };
        let is_update = stored.is_some();
        let mut value = stored.unwrap_or_else(|| {
            PropertyValue::new(input.owner, input.definition, input.content.clone(), now)
        });
        value.content = input.content;
        value.valid_from = input.valid_from;
        value.valid_until = input.valid_until;
        value.note = input.note;
        value.position = input.position;
        Ok(project.edit(|editor| {
            if is_update {
                editor.update_property_value(value, now)
            } else {
                editor.add_property_value(value)
            }
        })?)
    })
}

pub fn remove_property_value(
    workspace: &Workspace,
    id: &PropertyValueId,
) -> Result<(), CommandError> {
    workspace.record("entity.remove_property_value", |project| {
        Ok(project.edit(|editor| editor.remove_property_value(id))?)
    })
}

pub fn save_note(workspace: &Workspace, input: NoteInput) -> Result<Note, CommandError> {
    workspace.record("entity.save_note", |project| {
        let now = Timestamp::now();
        if let Some(id) = input.id {
            let mut note = found(Reader::<Note>::get(&project.reader(), &id)?, "note", id)?;
            note.title = input.title;
            note.body = input.body;
            Ok(project.edit(|editor| editor.update_note(note, now))?)
        } else {
            let note = Note::new(input.entity, input.title, input.body, now);
            Ok(project.edit(|editor| editor.add_note(note))?)
        }
    })
}

pub fn remove_note(workspace: &Workspace, id: &NoteId) -> Result<(), CommandError> {
    workspace.record("entity.remove_note", |project| {
        Ok(project.edit(|editor| editor.remove_note(id))?)
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use ontara_core::PropertyOwner;

    use super::*;
    use crate::workspace::fixture::workspace_with_project;

    fn new_entity(entity_type: &str, name: &str) -> NewEntity {
        NewEntity {
            entity_type: TypeKey::new(entity_type).expect("a valid key"),
            name: name.to_owned(),
            summary: String::new(),
            description: String::new(),
            tags: BTreeSet::new(),
            canon_state: None,
        }
    }

    #[test]
    fn nothing_works_without_an_open_project() {
        let workspace = Workspace::default();

        let error = list(&workspace, &EntityFilter::default()).expect_err("no project");

        assert_eq!(error.code, "no_project");
    }

    #[test]
    fn an_entity_is_created_listed_described_and_edited() {
        let (_directory, workspace) = workspace_with_project();
        let city = create(&workspace, new_entity("city", "Ashmere")).expect("creates");
        create(&workspace, new_entity("character", "Ayla")).expect("creates");

        let places = list(
            &workspace,
            &EntityFilter {
                entity_type: Some(TypeKey::new("place").expect("a valid key")),
                ..EntityFilter::default()
            },
        )
        .expect("lists");
        assert_eq!(places.len(), 1);
        assert_eq!(places[0].reference.name, "Ashmere");
        assert_eq!(places[0].reference.type_label, "City");

        let shown = detail(&workspace, city.id).expect("opens");
        assert_eq!(shown.type_chain.first().map(TypeKey::as_str), Some("city"));
        assert!(shown.type_chain.iter().any(|key| key.as_str() == "place"));
        assert!(
            shown
                .definitions
                .iter()
                .any(|d| d.key.as_str() == "population")
        );

        let edited = update(
            &workspace,
            EntityEdit {
                id: city.id,
                name: "Ashmere-on-Sable".to_owned(),
                sort_name: None,
                summary: "A river port".to_owned(),
                description: String::new(),
                tags: BTreeSet::from(["port".to_owned()]),
                names: Vec::new(),
                archived: false,
            },
        )
        .expect("edits");
        assert_eq!(edited.name, "Ashmere-on-Sable");
        assert_eq!(edited.entity_type.as_str(), "city");
    }

    #[test]
    fn a_trait_is_recorded_against_the_definition_of_the_entity_type() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = create(&workspace, new_entity("character", "Ayla")).expect("creates");
        let shown = detail(&workspace, ayla.id).expect("opens");
        let hair = shown
            .definitions
            .iter()
            .find(|definition| definition.key.as_str() == "hair_color")
            .expect("characters have hair color")
            .clone();
        assert_eq!(
            hair.owner,
            PropertyOwner::EntityType(TypeKey::new("being").expect("a valid key"))
        );

        save_property_value(
            &workspace,
            PropertyValueInput {
                id: None,
                owner: PropertyValueOwner::Entity(ayla.id),
                definition: hair.id,
                content: PropertyContent::Text("Copper".to_owned()),
                valid_from: None,
                valid_until: None,
                note: None,
                position: 0,
            },
        )
        .expect("records the trait");

        let reopened = detail(&workspace, ayla.id).expect("opens");
        assert_eq!(reopened.values.len(), 1);
        assert_eq!(
            reopened.values[0].content,
            PropertyContent::Text("Copper".to_owned())
        );
    }

    #[test]
    fn changing_canon_state_keeps_the_reason() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = create(&workspace, new_entity("character", "Ayla")).expect("creates");

        let change = change_canon_state(
            &workspace,
            RecordRef::new(RecordKind::Entity, ayla.id.as_uuid()),
            TypeKey::new("disputed").expect("a valid key"),
            Some("Chapter 3 contradicts chapter 1".to_owned()),
        )
        .expect("changes");

        assert!(change.is_some());
        let shown = detail(&workspace, ayla.id).expect("opens");
        assert_eq!(shown.entity.canon_state.as_str(), "disputed");
        assert_eq!(shown.canon_history.len(), 1);
        assert_eq!(
            shown.canon_history[0].reason.as_deref(),
            Some("Chapter 3 contradicts chapter 1")
        );
    }

    #[test]
    fn a_referenced_entity_is_not_removed_until_the_author_chooses() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = create(&workspace, new_entity("character", "Ayla")).expect("creates");
        let orin = create(&workspace, new_entity("character", "Orin")).expect("creates");
        crate::relationship::save(
            &workspace,
            crate::relationship::RelationshipInput {
                id: None,
                relationship_type: TypeKey::new("mentor_of").expect("a valid key"),
                source: ayla.id,
                target: orin.id,
                qualifier: None,
                context: None,
                lifecycle: ontara_core::Lifecycle::Active,
                period: ontara_core::Temporal::Unconstrained,
                note: None,
            },
        )
        .expect("relates");

        let refused = remove(&workspace, &orin.id, false).expect_err("it is referenced");
        assert_eq!(refused.code, "still_referenced");
        let counts = references(&workspace, &orin.id).expect("counts");
        assert_eq!(counts.iter().map(|count| count.count).sum::<usize>(), 1);

        remove(&workspace, &orin.id, true).expect("removes with its dependents");
        let remaining = list(&workspace, &EntityFilter::default()).expect("lists");
        assert_eq!(remaining.len(), 1);
    }
}
