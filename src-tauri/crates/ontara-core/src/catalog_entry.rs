use serde::{Deserialize, Serialize};

use crate::identifier::PropertyDefinitionId;
use crate::type_key::TypeKey;
use crate::vocabulary::Vocabulary;

/// One entry of a catalog the author configures: a type, a state, a level, a term.
///
/// Catalog entries are identified by a key that never changes; the label is what the author
/// sees and may rename freely.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "catalog", rename_all = "snake_case")]
pub enum CatalogEntry {
    CanonState {
        key: TypeKey,
    },
    EntityType {
        key: TypeKey,
    },
    RelationshipType {
        key: TypeKey,
    },
    EditorialLevel {
        key: TypeKey,
    },
    AppearanceKind {
        key: TypeKey,
    },
    PropertyGroup {
        key: TypeKey,
    },
    PropertyDefinition {
        id: PropertyDefinitionId,
    },
    DiplomaticStance {
        key: TypeKey,
    },
    VocabularyTerm {
        vocabulary: Vocabulary,
        key: TypeKey,
    },
}

impl CatalogEntry {
    /// The table the entry is kept in, which is also the catalog's name.
    #[must_use]
    pub const fn table(&self) -> &'static str {
        match self {
            Self::CanonState { .. } => "canon_state",
            Self::EntityType { .. } => "entity_type",
            Self::RelationshipType { .. } => "relationship_type",
            Self::EditorialLevel { .. } => "editorial_level",
            Self::AppearanceKind { .. } => "appearance_kind",
            Self::PropertyGroup { .. } => "property_group",
            Self::PropertyDefinition { .. } => "property_definition",
            Self::DiplomaticStance { .. } => "diplomatic_stance",
            Self::VocabularyTerm { vocabulary, .. } => vocabulary.as_str(),
        }
    }

    /// The column that identifies the entry within its table.
    #[must_use]
    pub const fn key_column(&self) -> &'static str {
        match self {
            Self::PropertyDefinition { .. } => "id",
            _ => "key",
        }
    }

    /// The entry's identity as stored.
    #[must_use]
    pub fn key_text(&self) -> String {
        match self {
            Self::CanonState { key }
            | Self::EntityType { key }
            | Self::RelationshipType { key }
            | Self::EditorialLevel { key }
            | Self::AppearanceKind { key }
            | Self::PropertyGroup { key }
            | Self::DiplomaticStance { key }
            | Self::VocabularyTerm { key, .. } => key.to_string(),
            Self::PropertyDefinition { id } => id.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(raw: &str) -> TypeKey {
        TypeKey::new(raw).unwrap_or_else(|error| panic!("{error}"))
    }

    #[test]
    fn a_vocabulary_term_lives_in_the_table_of_its_vocabulary() {
        let entry = CatalogEntry::VocabularyTerm {
            vocabulary: Vocabulary::TraitKind,
            key: key("fear"),
        };

        assert_eq!(entry.table(), "trait_kind");
        assert_eq!(entry.key_text(), "fear");
        assert_eq!(entry.key_column(), "key");
    }

    #[test]
    fn the_wire_form_names_the_catalog() {
        let entry = CatalogEntry::EntityType { key: key("guild") };

        assert_eq!(
            serde_json::to_string(&entry).ok().as_deref(),
            Some(r#"{"catalog":"entity_type","key":"guild"}"#)
        );
    }
}
