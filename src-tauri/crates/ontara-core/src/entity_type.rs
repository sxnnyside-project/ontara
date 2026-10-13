use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::record::Record;
use crate::type_key::TypeKey;

/// The structural job an entity type does, so rules can find the right records without
/// depending on names. Projects choose a role when they define a type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityRole {
    #[default]
    Generic,
    /// Records of this type are events: they can carry a time, a location and participants.
    Event,
    /// Records of this type are placed in the editorial hierarchy: books, chapters, scenes.
    EditorialUnit,
    /// Records of this type describe a file in the project's assets.
    Asset,
}

impl EntityRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::Event => "event",
            Self::EditorialUnit => "editorial_unit",
            Self::Asset => "asset",
        }
    }
}

impl fmt::Display for EntityRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for EntityRole {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "generic" => Ok(Self::Generic),
            "event" => Ok(Self::Event),
            "editorial_unit" => Ok(Self::EditorialUnit),
            "asset" => Ok(Self::Asset),
            other => Err(DomainError::UnknownValue("entity role", other.to_owned())),
        }
    }
}

/// The kind of an entity. Built-in types ship with every project; projects may define more.
///
/// Types form a hierarchy: a type inherits the properties of its ancestors and satisfies any
/// rule written for them, so a `city` is a `settlement` is a `place`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityType {
    pub key: TypeKey,
    /// Visible name. Customizable per project without changing `key`.
    pub label: String,
    pub description: String,
    /// The type this one specializes, if any.
    pub parent: Option<TypeKey>,
    /// An abstract type only groups its subtypes: no entity can be created with it directly.
    pub is_abstract: bool,
    pub role: EntityRole,
    pub sort_order: i32,
    pub builtin: bool,
}

impl EntityType {
    /// A project-specific type with no parent.
    #[must_use]
    pub fn custom(key: TypeKey, label: impl Into<String>, role: EntityRole) -> Self {
        Self {
            key,
            label: label.into(),
            description: String::new(),
            parent: None,
            is_abstract: false,
            role,
            sort_order: 0,
            builtin: false,
        }
    }

    /// The same type, specializing `parent`.
    #[must_use]
    pub fn specializing(mut self, parent: TypeKey) -> Self {
        self.parent = Some(parent);
        self
    }
}

impl Record for EntityType {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

use EntityRole::{Asset, EditorialUnit, Event, Generic};

/// (key, label, parent, abstract, role). Parents are listed before their children.
#[rustfmt::skip]
const BUILTIN_ENTITY_TYPES: [(&str, &str, Option<&str>, bool, EntityRole); 32] = [
    ("being", "Being", None, true, Generic),
    ("character", "Character", Some("being"), false, Generic),
    ("deity", "Deity", Some("being"), false, Generic),
    ("species", "Creature or species", None, false, Generic),
    ("faction", "Faction or organization", None, false, Generic),
    ("culture", "Culture or people", None, false, Generic),
    ("language", "Language", None, false, Generic),
    ("religion", "Religion or belief", None, false, Generic),
    ("item", "Item or artifact", None, false, Generic),
    ("place", "Place", None, true, Generic),
    ("world", "World or plane", Some("place"), false, Generic),
    ("continent", "Continent", Some("place"), false, Generic),
    ("region", "Region", Some("place"), false, Generic),
    ("polity", "Polity", Some("place"), true, Generic),
    ("empire", "Empire", Some("polity"), false, Generic),
    ("realm", "Kingdom or realm", Some("polity"), false, Generic),
    ("country", "Country or nation", Some("polity"), false, Generic),
    ("province", "Province or state", Some("polity"), false, Generic),
    ("settlement", "Settlement", Some("place"), true, Generic),
    ("city", "City", Some("settlement"), false, Generic),
    ("town", "Town", Some("settlement"), false, Generic),
    ("village", "Village", Some("settlement"), false, Generic),
    ("district", "District or quarter", Some("settlement"), false, Generic),
    ("stronghold", "Stronghold or fortress", Some("place"), false, Generic),
    ("location", "Location", Some("place"), false, Generic),
    ("event", "Event", None, false, Event),
    ("series", "Series", None, false, EditorialUnit),
    ("volume", "Volume or book", None, false, EditorialUnit),
    ("part", "Part", None, false, EditorialUnit),
    ("chapter", "Chapter", None, false, EditorialUnit),
    ("scene", "Scene", None, false, EditorialUnit),
    ("asset", "Asset or reference", None, false, Asset),
];

/// The entity types every new project starts with.
#[must_use]
pub fn builtin_entity_types() -> Vec<EntityType> {
    BUILTIN_ENTITY_TYPES
        .iter()
        .zip(0..)
        .map(
            |(&(key, label, parent, is_abstract, role), sort_order)| EntityType {
                key: TypeKey::from_static(key),
                label: label.to_owned(),
                description: String::new(),
                parent: parent.map(TypeKey::from_static),
                is_abstract,
                role,
                sort_order,
                builtin: true,
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_keys_are_valid_and_unique() {
        let types = builtin_entity_types();

        for entity_type in &types {
            assert!(TypeKey::new(entity_type.key.as_str()).is_ok());
            assert!(entity_type.builtin);
        }
        let mut keys: Vec<_> = types.iter().map(|t| t.key.clone()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), types.len());
    }

    #[test]
    fn every_parent_is_defined_before_its_children() {
        let types = builtin_entity_types();

        for (position, entity_type) in types.iter().enumerate() {
            if let Some(parent) = &entity_type.parent {
                let parent_position = types.iter().position(|t| &t.key == parent);
                assert!(
                    parent_position.is_some_and(|found| found < position),
                    "{} must come after {parent}",
                    entity_type.key
                );
            }
        }
    }

    #[test]
    fn places_and_beings_form_hierarchies() {
        let types = builtin_entity_types();
        let parent_of = |key: &str| {
            types
                .iter()
                .find(|t| t.key.as_str() == key)
                .and_then(|t| t.parent.clone())
        };

        assert_eq!(
            parent_of("city").map(|k| k.to_string()),
            Some("settlement".to_owned())
        );
        assert_eq!(
            parent_of("settlement").map(|k| k.to_string()),
            Some("place".to_owned())
        );
        assert_eq!(
            parent_of("realm").map(|k| k.to_string()),
            Some("polity".to_owned())
        );
        assert_eq!(
            parent_of("character").map(|k| k.to_string()),
            Some("being".to_owned())
        );
    }

    #[test]
    fn structural_types_carry_their_role() {
        let role_of = |key: &str| {
            builtin_entity_types()
                .into_iter()
                .find(|t| t.key.as_str() == key)
                .map(|t| t.role)
        };

        assert_eq!(role_of("event"), Some(EntityRole::Event));
        assert_eq!(role_of("chapter"), Some(EntityRole::EditorialUnit));
        assert_eq!(role_of("asset"), Some(EntityRole::Asset));
        assert_eq!(role_of("city"), Some(EntityRole::Generic));
    }
}
