//! Taking the universe out of Ontara: a readable bible in Markdown and a complete copy in JSON.
//!
//! Both are written into the project's `exports/` folder, so what leaves the application is
//! always a file the author can find next to the project.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;

use ontara_core::{
    CanonState, EditorialNode, Entity, EntityId, EntityType, ModelReaderExt, Note, PropertyContent,
    PropertyDefinition, PropertyValue, Relationship, Timestamp, reading_order,
};
use ontara_storage::{FORMAT_VERSION, Project, StorageError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::dossier;
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// A document an author reads: entities by type, with their text, properties and links.
    Markdown,
    /// Every record, as the application stores it.
    Json,
}

impl ExportFormat {
    const fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Json => "json",
        }
    }
}

/// How a relationship type reads in each direction.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RelationshipWords {
    pub label: String,
    pub inverse: Option<String>,
}

/// The words the interface uses, so a Markdown export reads in the author's language and with
/// the names the author gave to types and properties.
///
/// Mirrored by `src/features/project/api/export-api.ts`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ExportWords {
    pub contents: String,
    pub summary: String,
    pub description: String,
    pub tags: String,
    pub aliases: String,
    pub properties: String,
    pub relationships: String,
    pub notes: String,
    pub structure: String,
    pub canon: String,
    pub types: BTreeMap<String, String>,
    pub relationship_types: BTreeMap<String, RelationshipWords>,
    pub canon_states: BTreeMap<String, String>,
    pub property_names: BTreeMap<String, String>,
}

/// The file an export wrote.
///
/// Mirrored by `src/features/project/api/export-api.ts`.
#[derive(Debug, Serialize)]
pub struct ExportedFile {
    pub name: String,
    /// Absolute path.
    pub path: String,
    pub size_bytes: u64,
    pub entities: usize,
}

/// Everything a Markdown export reads from the project.
struct Source {
    project_name: String,
    premise: String,
    description: String,
    entities: Vec<Entity>,
    entity_types: Vec<EntityType>,
    relationships: Vec<Relationship>,
    notes: Vec<Note>,
    values: Vec<PropertyValue>,
    definitions: Vec<PropertyDefinition>,
    canon_states: Vec<CanonState>,
    nodes: Vec<EditorialNode>,
}

fn source(project: &Project) -> Result<Source, StorageError> {
    let reader = project.reader();
    let universe = reader.universe_profile()?;
    Ok(Source {
        project_name: project.manifest().name.clone(),
        premise: universe.summary,
        description: universe.description,
        entities: reader.entities()?,
        entity_types: reader.entity_types()?,
        relationships: reader.relationships()?,
        notes: reader.notes()?,
        values: reader.property_values()?,
        definitions: reader.property_definitions()?,
        canon_states: reader.canon_states()?,
        nodes: reader.editorial_nodes()?,
    })
}

fn quote(text: &str) -> String {
    text.lines()
        .map(|line| format!("> {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn value_text(content: &PropertyContent, names: &BTreeMap<EntityId, String>) -> String {
    match content {
        PropertyContent::Text(text) => text.clone(),
        PropertyContent::Integer(number) => number.to_string(),
        PropertyContent::Real(number) => number.to_string(),
        PropertyContent::Boolean(flag) => flag.to_string(),
        PropertyContent::Date(instant) => instant
            .label
            .clone()
            .or_else(|| instant.day.map(|day| format!("D+{day}")))
            .unwrap_or_default(),
        PropertyContent::EntityReference(id) => names.get(id).cloned().unwrap_or_default(),
        PropertyContent::Choice(key) => key.to_string(),
    }
}

/// The bible: the universe's premise, every entity grouped by type, then the structure of the
/// work in reading order. Archived entities are left out.
fn markdown(source: &Source, words: &ExportWords) -> String {
    let type_label = |key: &str, stored: &str| -> String {
        words
            .types
            .get(key)
            .cloned()
            .unwrap_or_else(|| stored.to_owned())
    };
    let canon_label = |key: &str| -> String {
        words.canon_states.get(key).cloned().unwrap_or_else(|| {
            source
                .canon_states
                .iter()
                .find(|state| state.key.as_str() == key)
                .map_or_else(|| key.to_owned(), |state| state.label.clone())
        })
    };
    let names: BTreeMap<EntityId, String> = source
        .entities
        .iter()
        .map(|entity| (entity.id, entity.name.clone()))
        .collect();

    let mut out = String::new();
    let _ = write!(out, "# {}\n\n", source.project_name);
    if !source.premise.is_empty() {
        let _ = write!(out, "{}\n\n", quote(&source.premise));
    }
    if !source.description.is_empty() {
        let _ = write!(out, "{}\n\n", source.description);
    }

    let mut types: Vec<&EntityType> = source.entity_types.iter().collect();
    types.sort_by_key(|entity_type| (entity_type.sort_order, entity_type.key.clone()));
    let present: Vec<(&EntityType, Vec<&Entity>)> = types
        .into_iter()
        .map(|entity_type| {
            let mut members: Vec<&Entity> = source
                .entities
                .iter()
                .filter(|entity| !entity.archived && entity.entity_type == entity_type.key)
                .collect();
            members.sort_by_cached_key(|entity| {
                entity
                    .sort_name
                    .as_deref()
                    .unwrap_or(&entity.name)
                    .to_lowercase()
            });
            (entity_type, members)
        })
        .filter(|(_, members)| !members.is_empty())
        .collect();

    if !present.is_empty() {
        let _ = write!(out, "## {}\n\n", words.contents);
        for (entity_type, members) in &present {
            let _ = writeln!(
                out,
                "- {} ({})",
                type_label(entity_type.key.as_str(), &entity_type.label),
                members.len()
            );
        }
        out.push('\n');
    }

    for (entity_type, members) in &present {
        let _ = write!(
            out,
            "## {}\n\n",
            type_label(entity_type.key.as_str(), &entity_type.label)
        );
        for entity in members {
            entity_section(&mut out, entity, source, words, &names, &canon_label);
        }
    }

    let order = reading_order(&source.nodes);
    if !order.is_empty() {
        let _ = write!(out, "## {}\n\n", words.structure);
        for step in order {
            let name = names.get(&step.entity).map_or("…", String::as_str);
            let _ = writeln!(out, "{}- {name}", "  ".repeat(step.depth));
        }
        out.push('\n');
    }
    out
}

fn entity_section(
    out: &mut String,
    entity: &Entity,
    source: &Source,
    words: &ExportWords,
    names: &BTreeMap<EntityId, String>,
    canon_label: &dyn Fn(&str) -> String,
) {
    let _ = write!(out, "### {}\n\n", entity.name);
    let _ = write!(
        out,
        "*{}: {}*\n\n",
        words.canon,
        canon_label(entity.canon_state.as_str())
    );
    if !entity.summary.is_empty() {
        let _ = write!(out, "{}\n\n", quote(&entity.summary));
    }
    if !entity.description.is_empty() {
        let _ = write!(out, "{}\n\n", entity.description);
    }
    if !entity.names.is_empty() {
        let aliases: Vec<&str> = entity.names.iter().map(|name| name.name.as_str()).collect();
        let _ = write!(out, "**{}:** {}\n\n", words.aliases, aliases.join(", "));
    }
    if !entity.tags.is_empty() {
        let tags: Vec<&str> = entity.tags.iter().map(String::as_str).collect();
        let _ = write!(out, "**{}:** {}\n\n", words.tags, tags.join(", "));
    }

    let mut properties: Vec<String> = Vec::new();
    for value in source.values.iter().filter(|value| {
        matches!(&value.owner, ontara_core::PropertyValueOwner::Entity(id) if *id == entity.id)
    }) {
        let Some(definition) = source
            .definitions
            .iter()
            .find(|definition| definition.id == value.definition)
        else {
            continue;
        };
        let label = words
            .property_names
            .get(&definition.id.to_string())
            .cloned()
            .unwrap_or_else(|| definition.label.clone());
        let text = value_text(&value.content, names);
        if !text.is_empty() {
            properties.push(format!("- {label}: {text}"));
        }
    }
    if !properties.is_empty() {
        let _ = write!(
            out,
            "**{}**\n\n{}\n\n",
            words.properties,
            properties.join("\n")
        );
    }

    let mut links: Vec<String> = Vec::new();
    for relationship in &source.relationships {
        let (other, outgoing) = if relationship.source == entity.id {
            (relationship.target, true)
        } else if relationship.target == entity.id {
            (relationship.source, false)
        } else {
            continue;
        };
        let Some(other_name) = names.get(&other) else {
            continue;
        };
        let key = relationship.relationship_type.as_str();
        let phrase = words.relationship_types.get(key).map_or_else(
            || key.replace('_', " "),
            |words| {
                if outgoing {
                    words.label.clone()
                } else {
                    words.inverse.clone().unwrap_or_else(|| words.label.clone())
                }
            },
        );
        links.push(format!("- {phrase}: {other_name}"));
    }
    if !links.is_empty() {
        let _ = write!(
            out,
            "**{}**\n\n{}\n\n",
            words.relationships,
            links.join("\n")
        );
    }

    let notes: Vec<&Note> = source
        .notes
        .iter()
        .filter(|note| note.entity == entity.id)
        .collect();
    if !notes.is_empty() {
        let _ = write!(out, "**{}**\n\n", words.notes);
        for note in notes {
            let _ = write!(out, "#### {}\n\n{}\n\n", note.title, note.body);
        }
    }
}

fn json_document(project: &Project, now: Timestamp) -> Result<Value, CommandError> {
    let reader = project.reader();
    let mut records: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for section in dossier::sections() {
        let listed = (section.list)(project)?;
        if !listed.is_empty() {
            records.insert(section.key, listed);
        }
    }
    let to_json = |error: serde_json::Error| CommandError::new("storage", error.to_string());
    Ok(json!({
        "format": "ontara-export",
        "format_version": FORMAT_VERSION,
        "exported_at": now.as_unix_millis(),
        "project": {
            "id": project.manifest().project_id.to_string(),
            "name": project.manifest().name,
        },
        "universe": serde_json::to_value(reader.universe_profile()?).map_err(to_json)?,
        "catalog": serde_json::to_value(crate::catalog::snapshot(project)?).map_err(to_json)?,
        "entities": serde_json::to_value(reader.entities()?).map_err(to_json)?,
        "relationships": serde_json::to_value(reader.relationships()?).map_err(to_json)?,
        "property_values": serde_json::to_value(reader.property_values()?).map_err(to_json)?,
        "notes": serde_json::to_value(reader.notes()?).map_err(to_json)?,
        "editorial_nodes": serde_json::to_value(reader.editorial_nodes()?).map_err(to_json)?,
        "appearances": serde_json::to_value(reader.appearances()?).map_err(to_json)?,
        "events": serde_json::to_value(reader.events()?).map_err(to_json)?,
        "dossier": records,
    }))
}

/// `20261013-142530`, in UTC, from milliseconds since the Unix epoch.
fn stamp(millis: i64) -> String {
    let seconds = millis.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    let in_day = seconds.rem_euclid(86_400);
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        in_day / 3600,
        in_day % 3600 / 60,
        in_day % 60
    )
}

/// A file name made of the project's name: lowercase letters and digits joined by hyphens.
fn slug(name: &str) -> String {
    let joined: String = name
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_lowercase().collect::<String>()
            } else {
                "-".to_owned()
            }
        })
        .collect();
    let parts: Vec<&str> = joined.split('-').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        "universe".to_owned()
    } else {
        parts.join("-")
    }
}

pub fn run(
    workspace: &Workspace,
    format: ExportFormat,
    words: &ExportWords,
) -> Result<ExportedFile, CommandError> {
    let now = Timestamp::now();
    workspace.read(|project| {
        let (content, entities) = match format {
            ExportFormat::Markdown => {
                let source = source(project)?;
                let count = source
                    .entities
                    .iter()
                    .filter(|entity| !entity.archived)
                    .count();
                (markdown(&source, words), count)
            }
            ExportFormat::Json => {
                let document = json_document(project, now)?;
                let count = document["entities"].as_array().map_or(0, Vec::len);
                let text = serde_json::to_string_pretty(&document)
                    .map_err(|error| CommandError::new("storage", error.to_string()))?;
                (text, count)
            }
        };
        let directory = project.layout().exports_directory();
        fs::create_dir_all(&directory)
            .map_err(|error| CommandError::new("storage", error.to_string()))?;
        let name = format!(
            "{}-{}.{}",
            slug(&project.manifest().name),
            stamp(now.as_unix_millis()),
            format.extension()
        );
        let path = directory.join(&name);
        fs::write(&path, &content).map_err(|error| {
            CommandError::new("storage", format!("{}: {error}", path.display()))
        })?;
        Ok(ExportedFile {
            name,
            path: path.to_string_lossy().into_owned(),
            size_bytes: content.len() as u64,
            entities,
        })
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use std::collections::BTreeSet;

    use ontara_core::TypeKey;

    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    fn entity(workspace: &Workspace, entity_type: &str, name: &str, summary: &str) {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new(entity_type).expect("a valid key"),
                name: name.to_owned(),
                summary: summary.to_owned(),
                description: String::new(),
                tags: BTreeSet::from(["keeper".to_owned()]),
                canon_state: None,
            },
        )
        .expect("creates");
    }

    fn words() -> ExportWords {
        ExportWords {
            contents: "Contents".into(),
            summary: "Summary".into(),
            tags: "Tags".into(),
            aliases: "Also known as".into(),
            properties: "Properties".into(),
            relationships: "Relationships".into(),
            notes: "Notes".into(),
            structure: "Structure".into(),
            canon: "Canon".into(),
            ..ExportWords::default()
        }
    }

    #[test]
    fn the_bible_lists_entities_by_type_with_their_text() {
        let (_directory, workspace) = workspace_with_project();
        entity(&workspace, "character", "Ayla", "Keeper of the lighthouse");
        entity(&workspace, "city", "Aurea", "A sunken capital");

        let file = run(&workspace, ExportFormat::Markdown, &words()).expect("exports");
        let text = fs::read_to_string(&file.path).expect("reads");

        assert!(text.starts_with("# Aurea\n"));
        assert!(text.contains("### Ayla"));
        assert!(text.contains("> Keeper of the lighthouse"));
        assert!(text.contains("**Tags:** keeper"));
        assert_eq!(file.entities, 2);
        assert!(
            file.name
                .rsplit_once('.')
                .is_some_and(|(_, extension)| extension == "md")
        );
    }

    #[test]
    fn the_json_export_carries_every_kind_of_record() {
        let (_directory, workspace) = workspace_with_project();
        entity(&workspace, "character", "Ayla", "");

        let file = run(&workspace, ExportFormat::Json, &words()).expect("exports");
        let document: Value =
            serde_json::from_str(&fs::read_to_string(&file.path).expect("reads")).expect("json");

        assert_eq!(document["format"], "ontara-export");
        assert_eq!(document["entities"].as_array().map(Vec::len), Some(1));
        assert!(document["catalog"]["entity_types"].is_array());
        assert!(document.get("dossier").is_some());
    }

    #[test]
    fn names_and_times_make_safe_file_names() {
        assert_eq!(slug("Aurea: la ciudad hundida!"), "aurea-la-ciudad-hundida");
        assert_eq!(slug("???"), "universe");
        assert_eq!(stamp(0), "19700101-000000");
        assert_eq!(stamp(1_791_919_446_000), "20261013-192406");
    }
}
