//! The dossier of an entity: the records that describe what it is, beyond its name and text.
//!
//! A being has a profile, traits, marks, family ties and partnerships; a place has borders,
//! resources and censuses; and so on. Each kind of record is a *section*. The sections that
//! apply to an entity follow from its type, and every record travels as JSON so one pair of
//! commands serves them all. The domain still validates every write: this module only routes.

use std::collections::{BTreeMap, BTreeSet};

use ontara_core::{
    EditError, EntityId, EntityRole, ModelEditor, ModelReaderExt, Timestamp, default_canon_state,
    entity_type_chain,
};
use ontara_storage::{Project, SqliteModel, StorageError};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::entity::{EntityRef, reference, type_labels};
use crate::workspace::{CommandError, Workspace};

mod sections;

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// How the records of a section relate to the entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    /// At most one record for the entity, keyed by it.
    Profile,
    /// Any number of records, each with its own identifier.
    Items,
    /// Any number of records keyed by the pair they link.
    Links,
}

type Editor<'a, 'b> = ModelEditor<'a, SqliteModel<'b>>;
type Failure = EditError<StorageError>;

/// One kind of record, and how to list, save and remove it.
pub(crate) struct Section {
    pub key: &'static str,
    pub shape: Shape,
    /// The fields of the record that hold an entity, with the kinds of entity each may hold; the
    /// section applies to an entity that is one of those kinds, and lists records that hold it.
    /// An empty list means any kind.
    pub owners: &'static [(&'static str, &'static [&'static str])],
    pub list: fn(&Project) -> Result<Vec<Value>, CommandError>,
    pub exists: fn(&Project, &Value) -> Result<bool, CommandError>,
    pub save: for<'a, 'b> fn(&mut Editor<'a, 'b>, Value, bool, Timestamp) -> Result<Value, Failure>,
    pub remove: for<'a, 'b> fn(&mut Editor<'a, 'b>, Value) -> Result<(), Failure>,
}

/// The records of one section that concern the entity.
///
/// Mirrored by `src/features/dossier/api/dossier-api.ts`.
#[derive(Debug, Serialize)]
pub struct SectionRecords {
    pub key: &'static str,
    pub shape: Shape,
    pub records: Vec<Value>,
}

/// Everything recorded about an entity, by section, with the entities the records mention.
#[derive(Debug, Serialize)]
pub struct Dossier {
    pub sections: Vec<SectionRecords>,
    pub entities: BTreeMap<String, EntityRef>,
}

pub(crate) fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, Failure> {
    serde_json::from_value(value).map_err(|error| {
        EditError::Domain(ontara_core::DomainError::Invalid(format!(
            "the record is not valid: {error}"
        )))
    })
}

pub(crate) fn encode<T: Serialize>(record: &T) -> Result<Value, Failure> {
    serde_json::to_value(record).map_err(|error| {
        EditError::Domain(ontara_core::DomainError::Invalid(format!(
            "the record cannot be written: {error}"
        )))
    })
}

/// Every section, in the order the interface shows them.
pub(crate) fn sections() -> &'static [Section] {
    sections::ALL
}

fn section(key: &str) -> Result<&'static Section, CommandError> {
    sections::ALL
        .iter()
        .find(|section| section.key == key)
        .ok_or_else(|| CommandError::new("rejected", format!("`{key}` is not a dossier section")))
}

/// Fills in what a draft leaves out: a new identifier, the clock and where the claim comes from.
fn complete(mut value: Value, now: Timestamp) -> Value {
    if let Value::Object(fields) = &mut value {
        let missing =
            |fields: &Map<String, Value>, name: &str| fields.get(name).is_none_or(Value::is_null);
        if missing(fields, "id") {
            fields.insert(
                "id".to_owned(),
                Value::String(EntityId::generate().to_string()),
            );
        }
        let at = Value::from(now.as_unix_millis());
        for name in ["created_at", "updated_at"] {
            if missing(fields, name) {
                fields.insert(name.to_owned(), at.clone());
            }
        }
        if missing(fields, "origin") {
            fields.insert("origin".to_owned(), Value::String("user".to_owned()));
        }
        if missing(fields, "canon_state") {
            fields.insert(
                "canon_state".to_owned(),
                Value::String(default_canon_state().to_string()),
            );
        }
    }
    value
}

fn holds(record: &Value, owners: &[(&str, &[&str])], entity: &str) -> bool {
    owners
        .iter()
        .any(|(field, _)| record.get(field).and_then(Value::as_str) == Some(entity))
}

/// Every string of a record that could be the identifier of an entity.
fn mentioned(record: &Value, into: &mut BTreeSet<String>) {
    match record {
        Value::String(text) if text.parse::<EntityId>().is_ok() => {
            into.insert(text.clone());
        }
        Value::Array(items) => {
            for item in items {
                mentioned(item, into);
            }
        }
        Value::Object(fields) => {
            for item in fields.values() {
                mentioned(item, into);
            }
        }
        _ => {}
    }
}

pub fn get(workspace: &Workspace, entity: EntityId) -> Result<Dossier, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let found = reader.entity(&entity)?.ok_or_else(|| {
            CommandError::new("not_found", format!("entity {entity} does not exist"))
        })?;
        let kinds: BTreeSet<String> = entity_type_chain(&reader, &found.entity_type)?
            .into_iter()
            .map(|entity_type| entity_type.key.to_string())
            .collect();
        let roles_allow_it = reader
            .entity_types()?
            .into_iter()
            .find(|entity_type| entity_type.key == found.entity_type)
            .is_some_and(|entity_type| entity_type.role == EntityRole::Generic);
        let id = entity.to_string();

        let mut sections = Vec::new();
        let mut mentions = BTreeSet::new();
        if roles_allow_it {
            // A tenure belongs to an office, and so is part of the body that holds the office.
            let offices: BTreeSet<String> = (section("office")?.list)(project)?
                .into_iter()
                .filter(|office| office.get("body").and_then(Value::as_str) == Some(&id))
                .filter_map(|office| office.get("id")?.as_str().map(str::to_owned))
                .collect();
            for section in sections::ALL {
                let tenures = section.key == "office_tenure";
                let applies = (tenures && !offices.is_empty())
                    || section.owners.iter().any(|(_, classes)| {
                        classes.is_empty() || classes.iter().any(|class| kinds.contains(*class))
                    });
                if !applies {
                    continue;
                }
                let mut records = (section.list)(project)?;
                records.retain(|record| {
                    holds(record, section.owners, &id)
                        || (tenures
                            && record
                                .get("office")
                                .and_then(Value::as_str)
                                .is_some_and(|office| offices.contains(office)))
                });
                for record in &records {
                    mentioned(record, &mut mentions);
                }
                sections.push(SectionRecords {
                    key: section.key,
                    shape: section.shape,
                    records,
                });
            }
        }

        let labels = type_labels(&reader)?;
        let mut entities = BTreeMap::new();
        for mention in mentions {
            if let Ok(identifier) = mention.parse::<EntityId>()
                && let Some(entity) = reader.entity(&identifier)?
            {
                entities.insert(mention, reference(&entity, &labels));
            }
        }
        Ok(Dossier { sections, entities })
    })
}

/// Every record of one section, whoever it concerns, with the entities they mention. The
/// choices of a field that points at a record rather than an entity, such as an office, come
/// from here.
pub fn all(workspace: &Workspace, key: &str) -> Result<Dossier, CommandError> {
    let section = section(key)?;
    workspace.read(|project| {
        let records = (section.list)(project)?;
        let mut mentions = BTreeSet::new();
        for record in &records {
            mentioned(record, &mut mentions);
        }
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let mut entities = BTreeMap::new();
        for mention in mentions {
            if let Ok(identifier) = mention.parse::<EntityId>()
                && let Some(entity) = reader.entity(&identifier)?
            {
                entities.insert(mention, reference(&entity, &labels));
            }
        }
        Ok(Dossier {
            sections: vec![SectionRecords {
                key: section.key,
                shape: section.shape,
                records,
            }],
            entities,
        })
    })
}

pub fn save(workspace: &Workspace, key: &str, record: Value) -> Result<Value, CommandError> {
    let section = section(key)?;
    let now = Timestamp::now();
    workspace.record("dossier.save", |project| {
        let record = complete(record, now);
        let exists = (section.exists)(project, &record)?;
        Ok(project.edit(|editor| (section.save)(editor, record, exists, now))?)
    })
}

pub fn remove(workspace: &Workspace, key: &str, record: Value) -> Result<(), CommandError> {
    let section = section(key)?;
    workspace.record("dossier.remove", |project| {
        Ok(project.edit(|editor| (section.remove)(editor, record))?)
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use std::collections::BTreeSet;

    use ontara_core::TypeKey;
    use serde_json::json;

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
                tags: BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates")
        .id
    }

    fn keys(dossier: &Dossier) -> Vec<&'static str> {
        dossier.sections.iter().map(|section| section.key).collect()
    }

    #[test]
    fn what_applies_follows_the_type_of_the_entity() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let aurea = entity(&workspace, "city", "Aurea");

        let being = get(&workspace, ayla).expect("reads");
        let place = get(&workspace, aurea).expect("reads");

        assert!(keys(&being).contains(&"being_profile"));
        assert!(keys(&being).contains(&"being_trait"));
        assert!(!keys(&being).contains(&"place_border"));
        assert!(keys(&place).contains(&"place_border"));
        assert!(!keys(&place).contains(&"being_trait"));
    }

    #[test]
    fn a_draft_is_completed_saved_listed_edited_and_removed() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let draft = json!({
            "being": ayla.to_string(),
            "kind": "fear",
            "text": "Deep water",
            "intensity": 4,
            "period": { "kind": "unconstrained" },
            "note": null
        });

        let saved = save(&workspace, "being_trait", draft).expect("saves");
        let listed = get(&workspace, ayla).expect("reads");
        let traits = listed
            .sections
            .iter()
            .find(|section| section.key == "being_trait")
            .expect("the section");
        let mut edited = saved.clone();
        edited["text"] = json!("Open sea");
        save(&workspace, "being_trait", edited).expect("edits");
        let after = get(&workspace, ayla).expect("reads");
        remove(&workspace, "being_trait", saved).expect("removes");
        let gone = get(&workspace, ayla).expect("reads");

        assert_eq!(traits.records.len(), 1);
        let count = |dossier: &Dossier| {
            dossier
                .sections
                .iter()
                .find(|section| section.key == "being_trait")
                .map(|section| section.records.len())
        };
        assert_eq!(count(&after), Some(1));
        assert_eq!(count(&gone), Some(0));
    }

    #[test]
    fn the_domain_still_rejects_what_does_not_make_sense() {
        let (_directory, workspace) = workspace_with_project();
        let aurea = entity(&workspace, "city", "Aurea");
        let draft = json!({
            "being": aurea.to_string(),
            "kind": "fear",
            "text": "Deep water",
            "period": { "kind": "unconstrained" }
        });

        let error = save(&workspace, "being_trait", draft).expect_err("a place has no traits");

        assert_eq!(error.code, "rejected");
    }

    #[test]
    fn an_unknown_section_is_refused() {
        let (_directory, workspace) = workspace_with_project();

        let error = save(&workspace, "made_up", json!({})).expect_err("unknown");

        assert_eq!(error.code, "rejected");
    }

    #[test]
    fn a_pair_keyed_record_is_set_and_removed_by_its_pair() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character", "Ayla");
        let tongue = entity(&workspace, "language", "Old Aurean");
        let record = json!({
            "being": ayla.to_string(),
            "language": tongue.to_string(),
            "proficiency": "fluent",
            "note": null
        });

        save(&workspace, "being_language", record.clone()).expect("links");
        let linked = get(&workspace, ayla).expect("reads");
        remove(&workspace, "being_language", record).expect("unlinks");
        let unlinked = get(&workspace, ayla).expect("reads");

        let count = |dossier: &Dossier| {
            dossier
                .sections
                .iter()
                .find(|section| section.key == "being_language")
                .map(|section| section.records.len())
        };
        assert_eq!(count(&linked), Some(1));
        assert_eq!(count(&unlinked), Some(0));
        assert!(linked.entities.contains_key(&tongue.to_string()));
    }
}
