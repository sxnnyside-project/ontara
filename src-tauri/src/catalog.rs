//! The configurable vocabulary of a project: its types, states, levels and terms.

use ontara_core::{
    AppearanceKind, CanonState, CatalogEntry, DiplomaticStance, EditorialLevel, EntityType,
    ModelQueries, ModelReaderExt, PropertyDefinition, PropertyDefinitionId, PropertyGroup,
    ReferenceCount, RelationshipType, RemovalStrategy, VocabularyTerm,
};
use serde::{Deserialize, Serialize};

use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// Everything an author can configure, in the order it is shown.
///
/// Mirrored by `src/features/catalog/api/catalog-api.ts`. The records serialize as the domain
/// defines them.
#[derive(Debug, Serialize)]
pub struct Catalog {
    pub entity_types: Vec<EntityType>,
    pub relationship_types: Vec<RelationshipType>,
    pub canon_states: Vec<CanonState>,
    pub editorial_levels: Vec<EditorialLevel>,
    pub appearance_kinds: Vec<AppearanceKind>,
    pub diplomatic_stances: Vec<DiplomaticStance>,
    pub property_groups: Vec<PropertyGroup>,
    pub property_definitions: Vec<PropertyDefinition>,
    pub vocabulary_terms: Vec<VocabularyTerm>,
}

/// A catalog entry to define or to update, tagged with the catalog it belongs to.
#[derive(Debug, Deserialize)]
#[serde(tag = "catalog", rename_all = "snake_case")]
pub enum CatalogRecord {
    EntityType(EntityType),
    RelationshipType(RelationshipType),
    CanonState(CanonState),
    EditorialLevel(EditorialLevel),
    AppearanceKind(AppearanceKind),
    DiplomaticStance(DiplomaticStance),
    PropertyGroup(PropertyGroup),
    PropertyDefinition(PropertyDefinition),
    VocabularyTerm(VocabularyTerm),
}

pub fn get(workspace: &Workspace) -> Result<Catalog, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        Ok(Catalog {
            entity_types: reader.entity_types()?,
            relationship_types: reader.relationship_types()?,
            canon_states: reader.canon_states()?,
            editorial_levels: reader.editorial_levels()?,
            appearance_kinds: reader.appearance_kinds()?,
            diplomatic_stances: reader.diplomatic_stances()?,
            property_groups: reader.property_groups()?,
            property_definitions: reader.property_definitions()?,
            vocabulary_terms: reader.vocabulary_terms()?,
        })
    })
}

pub fn define(workspace: &Workspace, record: CatalogRecord) -> Result<Catalog, CommandError> {
    workspace.write(|project| {
        project.edit(|editor| match record {
            CatalogRecord::EntityType(value) => editor.define_entity_type(value).map(drop),
            CatalogRecord::RelationshipType(value) => {
                editor.define_relationship_type(value).map(drop)
            }
            CatalogRecord::CanonState(value) => editor.define_canon_state(value).map(drop),
            CatalogRecord::EditorialLevel(value) => editor.define_editorial_level(value).map(drop),
            CatalogRecord::AppearanceKind(value) => editor.define_appearance_kind(value).map(drop),
            CatalogRecord::DiplomaticStance(value) => {
                editor.define_diplomatic_stance(value).map(drop)
            }
            CatalogRecord::PropertyGroup(value) => editor.define_property_group(value).map(drop),
            CatalogRecord::PropertyDefinition(value) => {
                editor.define_property_definition(value).map(drop)
            }
            CatalogRecord::VocabularyTerm(value) => editor.define_vocabulary_term(value).map(drop),
        })?;
        Ok(())
    })?;
    get(workspace)
}

pub fn update(workspace: &Workspace, record: CatalogRecord) -> Result<Catalog, CommandError> {
    workspace.write(|project| {
        project.edit(|editor| match record {
            CatalogRecord::EntityType(value) => editor.update_entity_type(value).map(drop),
            CatalogRecord::RelationshipType(value) => {
                editor.update_relationship_type(value).map(drop)
            }
            CatalogRecord::CanonState(value) => editor.update_canon_state(value).map(drop),
            CatalogRecord::EditorialLevel(value) => editor.update_editorial_level(value).map(drop),
            CatalogRecord::AppearanceKind(value) => editor.update_appearance_kind(value).map(drop),
            CatalogRecord::DiplomaticStance(value) => {
                editor.update_diplomatic_stance(value).map(drop)
            }
            CatalogRecord::PropertyGroup(value) => editor.update_property_group(value).map(drop),
            CatalogRecord::PropertyDefinition(value) => {
                editor.update_property_definition(value).map(drop)
            }
            CatalogRecord::VocabularyTerm(value) => editor.update_vocabulary_term(value).map(drop),
        })?;
        Ok(())
    })?;
    get(workspace)
}

pub fn usage(
    workspace: &Workspace,
    entry: &CatalogEntry,
) -> Result<Vec<ReferenceCount>, CommandError> {
    workspace.read(|project| Ok(project.reader().catalog_usage(entry)?))
}

pub fn remove(workspace: &Workspace, entry: &CatalogEntry) -> Result<Catalog, CommandError> {
    workspace.write(|project| {
        project.edit(|editor| match entry {
            CatalogEntry::PropertyDefinition { id } => editor
                .remove_property_definition(id, RemovalStrategy::RejectIfReferenced)
                .map(drop),
            other => editor.remove_catalog_entry(other),
        })?;
        Ok(())
    })?;
    get(workspace)
}

/// Removes a property together with every value recorded for it.
pub fn remove_property_with_values(
    workspace: &Workspace,
    id: PropertyDefinitionId,
) -> Result<Catalog, CommandError> {
    workspace.write(|project| {
        project.edit(|editor| {
            editor
                .remove_property_definition(&id, RemovalStrategy::RemoveDependents)
                .map(drop)
        })?;
        Ok(())
    })?;
    get(workspace)
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::workspace::fixture::workspace_with_project;

    fn record(json: &str) -> CatalogRecord {
        serde_json::from_str(json).expect("the wire form of a catalog record")
    }

    #[test]
    fn the_catalog_lists_what_every_project_starts_with() {
        let (_directory, workspace) = workspace_with_project();

        let catalog = get(&workspace).expect("reads the catalog");

        assert!(
            catalog
                .entity_types
                .iter()
                .any(|t| t.key.as_str() == "city")
        );
        assert!(
            catalog
                .canon_states
                .iter()
                .any(|s| s.key.as_str() == "disestablished")
        );
        assert!(
            catalog
                .editorial_levels
                .iter()
                .any(|l| l.key.as_str() == "chapter")
        );
        assert!(!catalog.property_definitions.is_empty());
    }

    #[test]
    fn a_custom_type_is_defined_renamed_and_removed_through_the_wire_form() {
        let (_directory, workspace) = workspace_with_project();
        let guild = r#"{"catalog":"entity_type","key":"guild","label":"Guild","description":"",
            "parent":null,"is_abstract":false,"role":"generic","sort_order":0,"builtin":false}"#;

        let defined = define(&workspace, record(guild)).expect("defines");
        assert!(
            defined
                .entity_types
                .iter()
                .any(|t| t.key.as_str() == "guild")
        );

        let renamed = guild.replace(r#""label":"Guild""#, r#""label":"Order""#);
        let updated = update(&workspace, record(&renamed)).expect("renames");
        let label = updated
            .entity_types
            .iter()
            .find(|t| t.key.as_str() == "guild")
            .map(|t| t.label.clone());
        assert_eq!(label.as_deref(), Some("Order"));

        let entry: CatalogEntry =
            serde_json::from_str(r#"{"catalog":"entity_type","key":"guild"}"#).expect("an entry");
        assert_eq!(usage(&workspace, &entry), Ok(Vec::new()));
        let removed = remove(&workspace, &entry).expect("removes");
        assert!(
            removed
                .entity_types
                .iter()
                .all(|t| t.key.as_str() != "guild")
        );
    }

    #[test]
    fn built_in_entries_cannot_be_removed() {
        let (_directory, workspace) = workspace_with_project();
        let entry: CatalogEntry =
            serde_json::from_str(r#"{"catalog":"entity_type","key":"city"}"#).expect("an entry");

        let error = remove(&workspace, &entry).expect_err("city ships with the project");

        assert_eq!(error.code, "built_in");
    }
}
