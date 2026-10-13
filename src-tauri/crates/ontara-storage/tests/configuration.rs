//! How an author configures the types and terminology of a project, through a real project.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;

use ontara_core::{
    CatalogEntry, ChoiceOption, Directionality, DomainError, EditError, EditorialLevel, Entity,
    EntityRole, EntityType, ModelReaderExt, PropertyContent, PropertyDataType, PropertyDefinition,
    PropertyOwner, PropertyValue, PropertyValueOwner, RelationshipType, RemovalStrategy, Timestamp,
    TypeKey,
};
use ontara_storage::{Project, StorageError};

type TestResult = Result<(), Box<dyn Error>>;
type EditResult<T> = Result<T, EditError<StorageError>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn project() -> Result<(tempfile::TempDir, Project), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = Project::create(&directory.path().join("Aurea.ontara"), "Aurea", T0)?;
    Ok((directory, project))
}

fn domain<T: std::fmt::Debug>(result: EditResult<T>) -> DomainError {
    match result {
        Err(EditError::Domain(error)) => error,
        other => panic!("expected a domain error, got {other:?}"),
    }
}

fn entity_type(project: &Project, raw: &str) -> Result<EntityType, Box<dyn Error>> {
    project
        .reader()
        .entity_types()?
        .into_iter()
        .find(|candidate| candidate.key == key(raw))
        .ok_or_else(|| format!("entity type {raw} is missing").into())
}

// ---- terminology

#[test]
fn a_built_in_type_can_be_renamed_without_changing_its_key() -> TestResult {
    let (_directory, mut project) = project()?;

    let mut chapter = entity_type(&project, "chapter")?;
    chapter.label = "Episode".to_owned();
    chapter.description = "One broadcast of the serial.".to_owned();
    project.edit(|editor| editor.update_entity_type(chapter))?;

    let renamed = entity_type(&project, "chapter")?;
    assert_eq!(renamed.label, "Episode");
    assert!(renamed.builtin);
    assert_eq!(renamed.role, EntityRole::EditorialUnit);
    Ok(())
}

#[test]
fn what_gives_a_type_its_meaning_cannot_be_edited() -> TestResult {
    let (_directory, mut project) = project()?;

    let mut city = entity_type(&project, "city")?;
    city.parent = Some(key("place"));
    let reparented = project.edit(|editor| editor.update_entity_type(city));
    assert_eq!(domain(reparented), DomainError::Immutable("parent type"));

    let mut city = entity_type(&project, "city")?;
    city.role = EntityRole::Event;
    let rerolled = project.edit(|editor| editor.update_entity_type(city));
    assert_eq!(domain(rerolled), DomainError::Immutable("role"));

    let mut blank = entity_type(&project, "city")?;
    blank.label = "  ".to_owned();
    assert!(matches!(
        domain(project.edit(|editor| editor.update_entity_type(blank))),
        DomainError::BlankText(_)
    ));
    Ok(())
}

#[test]
fn relationship_endpoints_may_widen_but_never_narrow() -> TestResult {
    let (_directory, mut project) = project()?;
    let mut guild_bond =
        RelationshipType::custom(key("guild_bond"), "Guild bond", Directionality::Directed);
    guild_bond.allowed_sources.insert(key("character"));
    project.edit(|editor| editor.define_relationship_type(guild_bond.clone()))?;

    guild_bond.allowed_sources.insert(key("faction"));
    let widened = project.edit(|editor| editor.update_relationship_type(guild_bond.clone()))?;
    assert_eq!(widened.allowed_sources.len(), 2);

    guild_bond.allowed_sources.remove(&key("character"));
    let narrowed = project.edit(|editor| editor.update_relationship_type(guild_bond.clone()));
    assert_eq!(domain(narrowed), DomainError::Immutable("allowed sources"));

    guild_bond.allowed_sources.clear();
    let opened = project.edit(|editor| editor.update_relationship_type(guild_bond.clone()))?;
    assert!(opened.allowed_sources.is_empty());

    guild_bond.allowed_sources.insert(key("faction"));
    let closed = project.edit(|editor| editor.update_relationship_type(guild_bond));
    assert_eq!(domain(closed), DomainError::Immutable("allowed sources"));
    Ok(())
}

#[test]
fn a_choice_property_can_gain_options_but_not_lose_one() -> TestResult {
    let (_directory, mut project) = project()?;
    let mut rank = PropertyDefinition::custom(
        PropertyOwner::EntityType(key("character")),
        key("guild_rank"),
        "Guild rank",
        PropertyDataType::Choice {
            options: vec![ChoiceOption {
                key: key("novice"),
                label: "Novice".to_owned(),
            }],
        },
    );
    project.edit(|editor| editor.define_property_definition(rank.clone()))?;

    rank.data_type = PropertyDataType::Choice {
        options: vec![
            ChoiceOption {
                key: key("novice"),
                label: "Apprentice".to_owned(),
            },
            ChoiceOption {
                key: key("master"),
                label: "Master".to_owned(),
            },
        ],
    };
    let grown = project.edit(|editor| editor.update_property_definition(rank.clone()))?;
    assert!(matches!(
        grown.data_type,
        PropertyDataType::Choice { ref options } if options.len() == 2
    ));

    rank.data_type = PropertyDataType::Choice {
        options: vec![ChoiceOption {
            key: key("master"),
            label: "Master".to_owned(),
        }],
    };
    let shrunk = project.edit(|editor| editor.update_property_definition(rank.clone()));
    assert_eq!(domain(shrunk), DomainError::Immutable("choice options"));

    rank.data_type = PropertyDataType::Text;
    let retyped = project.edit(|editor| editor.update_property_definition(rank));
    assert_eq!(domain(retyped), DomainError::Immutable("data type"));
    Ok(())
}

#[test]
fn an_editorial_level_keeps_its_depth_when_renamed() -> TestResult {
    let (_directory, mut project) = project()?;
    let mut scene = project
        .reader()
        .editorial_levels()?
        .into_iter()
        .find(|level| level.key == key("scene"))
        .expect("scene is built in");

    scene.label = "Beat".to_owned();
    let renamed = project.edit(|editor| editor.update_editorial_level(scene.clone()))?;
    assert_eq!(renamed.label, "Beat");

    scene.depth = 9;
    let moved = project.edit(|editor| editor.update_editorial_level(scene));
    assert_eq!(domain(moved), DomainError::Immutable("depth"));
    Ok(())
}

// ---- retirement

#[test]
fn an_unused_custom_type_can_be_removed_with_its_own_properties() -> TestResult {
    let (_directory, mut project) = project()?;
    project.edit(|editor| {
        editor.define_entity_type(EntityType::custom(
            key("guild"),
            "Guild",
            EntityRole::Generic,
        ))?;
        editor.define_property_definition(PropertyDefinition::custom(
            PropertyOwner::EntityType(key("guild")),
            key("guild_oath"),
            "Oath",
            PropertyDataType::Text,
        ))?;
        Ok(())
    })?;

    project.edit(|editor| {
        editor.remove_catalog_entry(&CatalogEntry::EntityType { key: key("guild") })
    })?;

    assert!(
        project
            .reader()
            .entity_types()?
            .iter()
            .all(|candidate| candidate.key != key("guild"))
    );
    assert!(
        project
            .reader()
            .property_definitions()?
            .iter()
            .all(|definition| definition.key != key("guild_oath"))
    );
    Ok(())
}

#[test]
fn a_type_in_use_cannot_be_removed_and_says_what_uses_it() -> TestResult {
    let (_directory, mut project) = project()?;
    project.edit(|editor| {
        editor.define_entity_type(EntityType::custom(
            key("guild"),
            "Guild",
            EntityRole::Generic,
        ))?;
        editor.add_entity(Entity::new(key("guild"), "Silver Hand", T0))?;
        Ok(())
    })?;

    let entry = CatalogEntry::EntityType { key: key("guild") };
    let refused = project.edit(|editor| editor.remove_catalog_entry(&entry));

    assert!(matches!(
        domain(refused),
        DomainError::StillInUse {
            what: "entity type",
            uses: 1,
            ..
        }
    ));
    let usage = project.edit(|editor| editor.catalog_usage(&entry))?;
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].kind, "entity");
    Ok(())
}

#[test]
fn a_type_with_subtypes_cannot_be_removed() -> TestResult {
    let (_directory, mut project) = project()?;
    project.edit(|editor| {
        editor.define_entity_type(EntityType::custom(
            key("guild"),
            "Guild",
            EntityRole::Generic,
        ))?;
        editor.define_entity_type(
            EntityType::custom(key("thieves_guild"), "Thieves' guild", EntityRole::Generic)
                .specializing(key("guild")),
        )?;
        Ok(())
    })?;

    let refused = project.edit(|editor| {
        editor.remove_catalog_entry(&CatalogEntry::EntityType { key: key("guild") })
    });

    assert!(matches!(domain(refused), DomainError::StillInUse { .. }));
    Ok(())
}

#[test]
fn built_in_entries_are_renamed_never_removed() -> TestResult {
    let (_directory, mut project) = project()?;

    let refused = project
        .edit(|editor| editor.remove_catalog_entry(&CatalogEntry::EntityType { key: key("city") }));

    assert!(matches!(
        domain(refused),
        DomainError::BuiltinCatalogEntry {
            what: "entity type",
            ..
        }
    ));
    Ok(())
}

#[test]
fn removing_a_property_definition_needs_to_decide_about_its_values() -> TestResult {
    let (_directory, mut project) = project()?;
    let (definition, character) = project.edit(|editor| {
        let definition = editor.define_property_definition(PropertyDefinition::custom(
            PropertyOwner::EntityType(key("character")),
            key("callsign"),
            "Callsign",
            PropertyDataType::Text,
        ))?;
        let character = editor.add_entity(Entity::new(key("character"), "Ayla", T0))?;
        editor.add_property_value(PropertyValue::new(
            PropertyValueOwner::Entity(character.id),
            definition.id,
            PropertyContent::Text("Magpie".to_owned()),
            T0,
        ))?;
        Ok((definition, character))
    })?;

    let refused = project.edit(|editor| {
        editor.remove_property_definition(&definition.id, RemovalStrategy::RejectIfReferenced)
    });
    assert!(matches!(
        domain(refused),
        DomainError::StillInUse { uses: 1, .. }
    ));

    let removed_values = project.edit(|editor| {
        editor.remove_property_definition(&definition.id, RemovalStrategy::RemoveDependents)
    })?;
    assert_eq!(removed_values, 1);
    assert!(project.reader().property_values()?.is_empty());
    assert!(project.reader().entity(&character.id)?.is_some());
    Ok(())
}

#[test]
fn a_custom_editorial_level_can_be_removed_once_no_section_sits_in_it() -> TestResult {
    let (_directory, mut project) = project()?;
    project.edit(|editor| {
        editor.define_editorial_level(EditorialLevel::custom(key("arc"), "Arc", 9))
    })?;

    project.edit(|editor| {
        editor.remove_catalog_entry(&CatalogEntry::EditorialLevel { key: key("arc") })
    })?;

    assert!(
        project
            .reader()
            .editorial_levels()?
            .iter()
            .all(|level| level.key != key("arc"))
    );
    Ok(())
}
