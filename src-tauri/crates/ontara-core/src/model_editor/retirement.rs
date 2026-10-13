use super::{Failure, ModelEditor, RemovalStrategy};
use crate::appearance_kind::AppearanceKind;
use crate::canon_state::CanonState;
use crate::catalog_entry::CatalogEntry;
use crate::diplomatic_stance::DiplomaticStance;
use crate::editorial_level::EditorialLevel;
use crate::entity_type::EntityType;
use crate::error::DomainError;
use crate::identifier::PropertyDefinitionId;
use crate::property_definition::{PropertyDefinition, PropertyGroup, PropertyOwner};
use crate::relationship_type::RelationshipType;
use crate::repository::{ModelRepository, ReferenceCount};

/// Retiring what a project defined but no longer uses.
impl<R: ModelRepository> ModelEditor<'_, R> {
    /// What still uses a catalog entry, counted by the kind of record.
    pub fn catalog_usage(&self, entry: &CatalogEntry) -> Result<Vec<ReferenceCount>, Failure<R>> {
        self.query(|repository| repository.catalog_usage(entry))
    }

    /// The name used in messages, and whether the entry ships with every project.
    fn catalog_identity(&self, entry: &CatalogEntry) -> Result<(&'static str, bool), Failure<R>> {
        Ok(match entry {
            CatalogEntry::CanonState { key } => (
                "canon state",
                self.require_defined::<CanonState>("canon state", key)?
                    .builtin,
            ),
            CatalogEntry::EntityType { key } => (
                "entity type",
                self.require_defined::<EntityType>("entity type", key)?
                    .builtin,
            ),
            CatalogEntry::RelationshipType { key } => (
                "relationship type",
                self.require_defined::<RelationshipType>("relationship type", key)?
                    .builtin,
            ),
            CatalogEntry::EditorialLevel { key } => (
                "editorial level",
                self.require_defined::<EditorialLevel>("editorial level", key)?
                    .builtin,
            ),
            CatalogEntry::AppearanceKind { key } => (
                "appearance kind",
                self.require_defined::<AppearanceKind>("appearance kind", key)?
                    .builtin,
            ),
            CatalogEntry::PropertyGroup { key } => {
                self.require_defined::<PropertyGroup>("property group", key)?;
                // Groups only organize the display; every one of them can go.
                ("property group", false)
            }
            CatalogEntry::PropertyDefinition { id } => (
                "property",
                self.require::<PropertyDefinition>("property definition", id)?
                    .builtin,
            ),
            CatalogEntry::DiplomaticStance { key } => (
                "diplomatic stance",
                self.require_defined::<DiplomaticStance>("diplomatic stance", key)?
                    .builtin,
            ),
            CatalogEntry::VocabularyTerm { vocabulary, key } => (
                vocabulary.as_str(),
                self.require_term(*vocabulary, key)?.builtin,
            ),
        })
    }

    /// Removes a catalog entry the project defined and nothing uses.
    ///
    /// Built-in entries stay (rename them instead). A type takes its own property definitions
    /// with it, since no entity can carry a value of them.
    pub fn remove_catalog_entry(&mut self, entry: &CatalogEntry) -> Result<(), Failure<R>> {
        let (what, builtin) = self.catalog_identity(entry)?;
        if builtin {
            return Err(DomainError::BuiltinCatalogEntry {
                what,
                key: entry.key_text(),
            }
            .into());
        }
        let uses: usize = self
            .catalog_usage(entry)?
            .iter()
            .map(|usage| usage.count)
            .sum();
        if uses > 0 {
            return Err(DomainError::StillInUse {
                what,
                key: entry.key_text(),
                uses,
            }
            .into());
        }

        let owner = match entry {
            CatalogEntry::EntityType { key } => Some(PropertyOwner::EntityType(key.clone())),
            CatalogEntry::RelationshipType { key } => {
                Some(PropertyOwner::RelationshipType(key.clone()))
            }
            _ => None,
        };
        if let Some(owner) = owner {
            for definition in self.list::<PropertyDefinition>()? {
                if definition.owner == owner {
                    self.remove_property_definition_unchecked(&definition.id)?;
                }
            }
        }
        self.catalog_removal(entry)
    }

    /// Removes a property definition. With `RemoveDependents` the values recorded for it go too;
    /// otherwise the removal fails while any value remains.
    pub fn remove_property_definition(
        &mut self,
        id: &PropertyDefinitionId,
        strategy: RemovalStrategy,
    ) -> Result<usize, Failure<R>> {
        let entry = CatalogEntry::PropertyDefinition { id: *id };
        let (what, builtin) = self.catalog_identity(&entry)?;
        if builtin {
            return Err(DomainError::BuiltinCatalogEntry {
                what,
                key: entry.key_text(),
            }
            .into());
        }
        let uses: usize = self
            .catalog_usage(&entry)?
            .iter()
            .map(|usage| usage.count)
            .sum();
        if uses > 0 && strategy == RemovalStrategy::RejectIfReferenced {
            return Err(DomainError::StillInUse {
                what,
                key: entry.key_text(),
                uses,
            }
            .into());
        }
        self.remove_property_definition_unchecked(id)?;
        Ok(uses)
    }

    fn remove_property_definition_unchecked(
        &mut self,
        id: &PropertyDefinitionId,
    ) -> Result<(), Failure<R>> {
        self.catalog_removal(&CatalogEntry::PropertyDefinition { id: *id })
    }

    fn catalog_removal(&mut self, entry: &CatalogEntry) -> Result<(), Failure<R>> {
        use crate::repository::RecordRemoval;
        RecordRemoval::remove_catalog_entry(self.repository, entry)
            .map_err(super::EditError::Repository)
    }
}
