//! Typed relationships between entities, with their own context and properties.

use std::collections::BTreeSet;

use ontara_core::{
    EntityFilter, EntityId, Lifecycle, ModelQueries, ModelReaderExt, Origin, PropertyDefinition,
    PropertyOwner, PropertyValue, PropertyValueOwner, Reader, Relationship, RelationshipId,
    Temporal, Timestamp, TypeKey, find_entities, is_entity_type,
};
use serde::{Deserialize, Serialize};

use crate::entity::{EntityRef, EntitySummary, found, resolve, summary_of, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// A relationship with both ends resolved, ready to show.
///
/// Mirrored by `src/features/relationship/api/relationship-api.ts`.
#[derive(Debug, Serialize)]
pub struct RelationshipView {
    pub relationship: Relationship,
    pub source: EntityRef,
    pub target: EntityRef,
    pub context: Option<EntityRef>,
}

/// A relationship with the properties its type defines and the values recorded for them.
#[derive(Debug, Serialize)]
pub struct RelationshipDetail {
    #[serde(flatten)]
    pub view: RelationshipView,
    pub definitions: Vec<PropertyDefinition>,
    pub values: Vec<PropertyValue>,
}

#[derive(Debug, Deserialize)]
pub struct RelationshipInput {
    /// Present when updating a relationship, absent when adding one.
    pub id: Option<RelationshipId>,
    pub relationship_type: TypeKey,
    pub source: EntityId,
    pub target: EntityId,
    pub qualifier: Option<String>,
    pub context: Option<EntityId>,
    #[serde(default)]
    pub lifecycle: Lifecycle,
    #[serde(default)]
    pub period: Temporal,
    pub note: Option<String>,
}

/// A way the entity can take part in a relationship.
#[derive(Debug, Serialize)]
pub struct RelationshipOption {
    pub relationship_type: TypeKey,
    /// The entity is the source of the relationship; otherwise it is the target.
    pub outgoing: bool,
}

/// Which end of a relationship.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Source,
    Target,
}

fn view(
    reader: &impl ontara_core::ModelReader<Error = ontara_storage::StorageError>,
    relationship: Relationship,
) -> Result<RelationshipView, CommandError> {
    let labels = type_labels(reader)?;
    Ok(RelationshipView {
        source: resolve(reader, &labels, &relationship.source)?,
        target: resolve(reader, &labels, &relationship.target)?,
        context: relationship
            .context
            .as_ref()
            .map(|context| resolve(reader, &labels, context))
            .transpose()?,
        relationship,
    })
}

pub fn list_for(
    workspace: &Workspace,
    entity: &EntityId,
) -> Result<Vec<RelationshipView>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        reader
            .relationships_of(entity)?
            .into_iter()
            .map(|relationship| view(&reader, relationship))
            .collect()
    })
}

pub fn detail(
    workspace: &Workspace,
    id: &RelationshipId,
) -> Result<RelationshipDetail, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let relationship = found(reader.relationship(id)?, "relationship", id)?;
        let owner = PropertyOwner::RelationshipType(relationship.relationship_type.clone());
        Ok(RelationshipDetail {
            definitions: reader.property_definitions_of(&owner)?,
            values: reader.property_values_of(&PropertyValueOwner::Relationship(*id))?,
            view: view(&reader, relationship)?,
        })
    })
}

/// The relationship types the entity may take part in, and in which role. A type with no
/// endpoint rule on a side accepts every entity there.
pub fn options_for(
    workspace: &Workspace,
    entity: &EntityId,
) -> Result<Vec<RelationshipOption>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let entity = found(reader.entity(entity)?, "entity", entity)?;
        let mut options = Vec::new();
        for relationship_type in reader.relationship_types()? {
            for (outgoing, allowed) in [
                (true, &relationship_type.allowed_sources),
                (false, &relationship_type.allowed_targets),
            ] {
                let mut fits = allowed.is_empty();
                for candidate in allowed {
                    fits = fits || is_entity_type(&reader, &entity.entity_type, candidate)?;
                }
                if fits {
                    options.push(RelationshipOption {
                        relationship_type: relationship_type.key.clone(),
                        outgoing,
                    });
                }
            }
        }
        Ok(options)
    })
}

/// The entities that may sit at one end of a relationship of the given type.
pub fn candidates(
    workspace: &Workspace,
    relationship_type: &TypeKey,
    side: Side,
    text: Option<&str>,
) -> Result<Vec<EntitySummary>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let stored = reader
            .relationship_types()?
            .into_iter()
            .find(|candidate| &candidate.key == relationship_type);
        let stored = found(stored, "relationship type", relationship_type)?;
        let allowed = match side {
            Side::Source => stored.allowed_sources,
            Side::Target => stored.allowed_targets,
        };
        let labels = type_labels(&reader)?;
        let text = text.map(str::to_owned);
        let filters: Vec<EntityFilter> = if allowed.is_empty() {
            vec![EntityFilter {
                text: text.clone(),
                ..EntityFilter::default()
            }]
        } else {
            allowed
                .into_iter()
                .map(|entity_type| EntityFilter {
                    entity_type: Some(entity_type),
                    text: text.clone(),
                    ..EntityFilter::default()
                })
                .collect()
        };
        let mut seen = BTreeSet::new();
        let mut summaries = Vec::new();
        for filter in &filters {
            for entity in find_entities(&reader, filter)? {
                if seen.insert(entity.id.to_string()) {
                    summaries.push(summary_of(&entity, &labels));
                }
            }
        }
        summaries.sort_by_key(|summary| summary.reference.name.to_lowercase());
        Ok(summaries)
    })
}

pub fn save(workspace: &Workspace, input: RelationshipInput) -> Result<Relationship, CommandError> {
    workspace.record("relationship.save", |project| {
        let now = Timestamp::now();
        let stored = match input.id {
            Some(id) => Some(found(
                Reader::<Relationship>::get(&project.reader(), &id)?,
                "relationship",
                id,
            )?),
            None => None,
        };
        let is_update = stored.is_some();
        let mut relationship = stored.unwrap_or_else(|| {
            Relationship::new(
                input.relationship_type.clone(),
                input.source,
                input.target,
                now,
            )
        });
        relationship.relationship_type = input.relationship_type;
        relationship.source = input.source;
        relationship.target = input.target;
        relationship.qualifier = input.qualifier;
        relationship.context = input.context;
        relationship.lifecycle = input.lifecycle;
        relationship.period = input.period;
        relationship.note = input.note;
        relationship.origin = Origin::User;
        Ok(project.edit(|editor| {
            if is_update {
                editor.update_relationship(relationship, now)
            } else {
                editor.add_relationship(relationship)
            }
        })?)
    })
}

pub fn remove(workspace: &Workspace, id: &RelationshipId) -> Result<(), CommandError> {
    workspace.record("relationship.remove", |project| {
        Ok(project.edit(|editor| editor.remove_relationship(id))?)
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    fn entity(workspace: &Workspace, entity_type: &str, name: &str) -> EntityId {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new(entity_type).expect("a valid key"),
                name: name.to_owned(),
                summary: String::new(),
                description: String::new(),
                tags: std::collections::BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates")
        .id
    }

    fn input(relationship_type: &str, source: EntityId, target: EntityId) -> RelationshipInput {
        RelationshipInput {
            id: None,
            relationship_type: TypeKey::new(relationship_type).expect("a valid key"),
            source,
            target,
            qualifier: None,
            context: None,
            lifecycle: Lifecycle::Active,
            period: Temporal::Unconstrained,
            note: None,
        }
    }

    #[test]
    fn a_relationship_is_listed_from_both_of_its_ends() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let guild = entity(&workspace, "faction", "Silver Hand");

        save(&workspace, input("affiliated_with", ayla, guild)).expect("relates");

        let from_ayla = list_for(&workspace, &ayla).expect("lists");
        let from_guild = list_for(&workspace, &guild).expect("lists");
        assert_eq!(from_ayla.len(), 1);
        assert_eq!(from_guild.len(), 1);
        assert_eq!(from_ayla[0].source.name, "Ayla");
        assert_eq!(from_ayla[0].target.type_label, "Faction or organization");
    }

    #[test]
    fn a_type_rejects_endpoints_it_does_not_allow() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let city = entity(&workspace, "city", "Ashmere");

        let error =
            save(&workspace, input("spouse_of", ayla, city)).expect_err("a city is no spouse");

        assert_eq!(error.code, "rejected");
        assert!(error.message.contains("spouse_of"));
    }

    #[test]
    fn a_relationship_can_be_updated_and_carries_its_type_definitions() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let orin = entity(&workspace, "character", "Orin");
        let created = save(&workspace, input("mentor_of", ayla, orin)).expect("relates");

        let mut change = input("mentor_of", ayla, orin);
        change.id = Some(created.id);
        change.qualifier = Some("in swordsmanship".to_owned());
        change.lifecycle = Lifecycle::Historical;
        save(&workspace, change).expect("updates");

        let shown = detail(&workspace, &created.id).expect("opens");
        assert_eq!(
            shown.view.relationship.qualifier.as_deref(),
            Some("in swordsmanship")
        );
        assert_eq!(shown.view.relationship.lifecycle, Lifecycle::Historical);
    }

    #[test]
    fn the_options_and_the_candidates_follow_the_endpoint_rules() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let orin = entity(&workspace, "character", "Orin");
        let ashmere = entity(&workspace, "city", "Ashmere");

        let options = options_for(&workspace, &ayla).expect("lists the options");
        let spouse = TypeKey::new("spouse_of").expect("a valid key");
        let born_in = TypeKey::new("born_in").expect("a valid key");
        assert!(
            options
                .iter()
                .any(|o| o.relationship_type == spouse && o.outgoing)
        );
        assert!(
            options
                .iter()
                .any(|o| o.relationship_type == born_in && o.outgoing)
        );
        let for_the_city = options_for(&workspace, &ashmere).expect("lists the options");
        assert!(
            !for_the_city.iter().any(|o| o.relationship_type == spouse),
            "a city cannot be anyone's spouse"
        );

        let spouses = candidates(&workspace, &spouse, Side::Target, None).expect("lists");
        assert_eq!(
            spouses
                .iter()
                .map(|c| c.reference.name.as_str())
                .collect::<Vec<_>>(),
            ["Ayla", "Orin"]
        );
        let places = candidates(&workspace, &born_in, Side::Target, None).expect("lists");
        assert_eq!(places.len(), 1);
        assert_eq!(places[0].reference.name, "Ashmere");
        let found_by_text =
            candidates(&workspace, &spouse, Side::Target, Some("ori")).expect("lists");
        assert_eq!(found_by_text.len(), 1);
        assert_eq!(found_by_text[0].reference.id, orin);
    }
}
