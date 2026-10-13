use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifier::PropertyDefinitionId;
use crate::record::Record;
use crate::type_key::TypeKey;

/// Which kind of record a property definition describes.
///
/// An entity type's definitions apply to every entity of that type and of its subtypes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "key", rename_all = "snake_case")]
pub enum PropertyOwner {
    EntityType(TypeKey),
    RelationshipType(TypeKey),
}

/// A named choice of a choice property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption {
    pub key: TypeKey,
    pub label: String,
}

/// The kind of value a property holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PropertyDataType {
    Text,
    Integer,
    Real,
    Boolean,
    /// A date or span on the in-universe timeline.
    Date,
    /// A reference to another entity, optionally restricted to one entity type and its subtypes.
    EntityReference {
        entity_type: Option<TypeKey>,
    },
    Choice {
        options: Vec<ChoiceOption>,
    },
}

impl PropertyDataType {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Real => "real",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::EntityReference { .. } => "entity reference",
            Self::Choice { .. } => "choice",
        }
    }
}

/// A section that organizes properties for display, such as "Appearance" or "Personality".
///
/// Groups are shared by the whole project, so the same section can serve many types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyGroup {
    pub key: TypeKey,
    pub label: String,
    pub sort_order: i32,
}

impl Record for PropertyGroup {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

/// A trait, characteristic or attribute that records of a type can carry.
///
/// Definitions are data, so an author can describe what makes a character or place individual
/// without any change to the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub id: PropertyDefinitionId,
    pub owner: PropertyOwner,
    /// Stable within its owner and its owner's ancestors and descendants.
    pub key: TypeKey,
    pub label: String,
    pub description: String,
    /// Section this property is shown under.
    pub group: Option<TypeKey>,
    pub data_type: PropertyDataType,
    /// Records of the owner type are expected to have a value.
    pub required: bool,
    /// More than one value may be recorded at the same time.
    pub multiple: bool,
    pub sort_order: i32,
    pub builtin: bool,
}

impl PropertyDefinition {
    /// A project-specific definition.
    #[must_use]
    pub fn custom(
        owner: PropertyOwner,
        key: TypeKey,
        label: impl Into<String>,
        data_type: PropertyDataType,
    ) -> Self {
        Self {
            id: PropertyDefinitionId::generate(),
            owner,
            key,
            label: label.into(),
            description: String::new(),
            group: None,
            data_type,
            required: false,
            multiple: false,
            sort_order: 0,
            builtin: false,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.label.trim().is_empty() {
            return Err(DomainError::BlankText("property label"));
        }
        if let PropertyDataType::Choice { options } = &self.data_type {
            let invalid = |reason: &str| DomainError::InvalidProperty {
                property: self.key.to_string(),
                reason: reason.to_owned(),
            };
            if options.is_empty() {
                return Err(invalid("a choice needs at least one option"));
            }
            if options.iter().any(|option| option.label.trim().is_empty()) {
                return Err(invalid("every option needs a label"));
            }
            let keys: BTreeSet<_> = options.iter().map(|option| &option.key).collect();
            if keys.len() != options.len() {
                return Err(invalid("option keys must be unique"));
            }
        }
        Ok(())
    }
}

impl Record for PropertyDefinition {
    type Id = PropertyDefinitionId;

    fn id(&self) -> PropertyDefinitionId {
        self.id
    }
}

/// The sections and traits that ship with a new project.
pub struct BuiltinPropertySet {
    pub groups: Vec<PropertyGroup>,
    pub definitions: Vec<PropertyDefinition>,
}

/// (key, label)
#[rustfmt::skip]
const BUILTIN_GROUPS: [(&str, &str); 12] = [
    ("identity", "Identity"),
    ("appearance", "Appearance"),
    ("personality", "Personality"),
    ("abilities", "Abilities"),
    ("background", "Background"),
    ("biology", "Biology"),
    ("geography", "Geography"),
    ("history", "History"),
    ("governance", "Governance"),
    ("demography", "Demography"),
    ("beliefs", "Beliefs and customs"),
    ("organization", "Organization"),
];

#[derive(Clone, Copy)]
enum Shape {
    Text,
    Integer,
    Date,
    /// An entity reference restricted to a type and its subtypes.
    Reference(&'static str),
}

use Shape::{Date, Integer, Reference, Text};

/// (owner entity type, group, key, label, shape, multiple)
///
/// Properties are declared on the most general type they make sense for, so every subtype
/// inherits them: hair color is a trait of a being, population of a settlement.
#[rustfmt::skip]
const BUILTIN_DEFINITIONS: [(&str, &str, &str, &str, Shape, bool); 85] = [
    // Every being: who they are, how they look, what they are like, what they can do.
    ("being", "identity", "titles", "Titles and honorifics", Text, true),
    ("being", "identity", "pronouns", "Pronouns", Text, false),
    ("being", "identity", "gender", "Gender", Text, false),
    ("being", "identity", "age", "Age", Text, false),
    ("being", "identity", "birth", "Birth", Date, false),
    ("being", "identity", "death", "Death", Date, false),
    ("being", "identity", "occupation", "Occupation", Text, true),
    ("being", "identity", "nationality", "Nationality", Reference("polity"), true),
    ("being", "identity", "culture", "Culture", Reference("culture"), true),
    ("being", "identity", "religion", "Religion", Reference("religion"), true),
    ("being", "identity", "languages", "Languages", Reference("language"), true),
    ("being", "appearance", "hair_color", "Hair color", Text, false),
    ("being", "appearance", "hair_style", "Hair style", Text, false),
    ("being", "appearance", "eye_color", "Eye color", Text, false),
    ("being", "appearance", "skin_tone", "Skin tone", Text, false),
    ("being", "appearance", "height", "Height", Text, false),
    ("being", "appearance", "build", "Build", Text, false),
    ("being", "appearance", "distinguishing_marks", "Distinguishing marks", Text, true),
    ("being", "appearance", "clothing_style", "Clothing style", Text, false),
    ("being", "appearance", "voice", "Voice", Text, false),
    ("being", "personality", "personality_traits", "Personality traits", Text, true),
    ("being", "personality", "temperament", "Temperament", Text, false),
    ("being", "personality", "values", "Values", Text, true),
    ("being", "personality", "fears", "Fears", Text, true),
    ("being", "personality", "quirks", "Quirks and mannerisms", Text, true),
    ("being", "personality", "habits", "Habits", Text, true),
    ("being", "personality", "speech_style", "Speech style", Text, false),
    ("being", "personality", "motivation", "Motivation", Text, false),
    ("being", "personality", "flaws", "Flaws", Text, true),
    ("being", "personality", "secrets", "Secrets", Text, true),
    ("being", "abilities", "abilities", "Abilities", Text, true),
    ("being", "abilities", "skills", "Skills", Text, true),
    ("being", "abilities", "weaknesses", "Weaknesses", Text, true),
    ("being", "background", "birthplace", "Birthplace", Reference("place"), false),
    ("being", "background", "residence", "Residence", Reference("place"), false),
    ("being", "background", "upbringing", "Upbringing", Text, false),
    ("character", "identity", "species", "Species", Reference("species"), false),
    ("deity", "beliefs", "portfolio", "Portfolio", Text, true),
    ("deity", "beliefs", "symbols", "Symbols", Text, true),
    ("deity", "beliefs", "worship", "Manner of worship", Text, false),
    ("species", "biology", "lifespan", "Lifespan", Text, false),
    ("species", "biology", "average_height", "Average height", Text, false),
    ("species", "biology", "diet", "Diet", Text, false),
    ("species", "biology", "habitat", "Habitat", Reference("place"), true),
    ("species", "biology", "traits", "Characteristic traits", Text, true),
    // Every place.
    ("place", "geography", "climate", "Climate", Text, false),
    ("place", "geography", "terrain", "Terrain", Text, false),
    ("place", "geography", "landmarks", "Landmarks", Text, true),
    ("place", "geography", "resources", "Resources", Text, true),
    ("place", "history", "founded", "Founded", Date, false),
    ("place", "history", "dissolved", "Dissolved", Date, false),
    // Kingdoms, countries, empires and provinces.
    ("polity", "governance", "government_type", "Government type", Text, false),
    ("polity", "governance", "ruler", "Ruler", Reference("character"), false),
    ("polity", "governance", "heir", "Heir", Reference("character"), false),
    ("polity", "governance", "capital", "Capital", Reference("settlement"), false),
    ("polity", "governance", "currency", "Currency", Text, false),
    ("polity", "governance", "official_languages", "Official languages", Reference("language"), true),
    ("polity", "governance", "state_religion", "State religion", Reference("religion"), false),
    ("polity", "governance", "motto", "Motto", Text, false),
    ("polity", "governance", "symbols", "Symbols", Text, true),
    ("polity", "demography", "population", "Population", Integer, false),
    // Cities, towns, villages and districts.
    ("settlement", "demography", "population", "Population", Integer, false),
    ("settlement", "demography", "notable_features", "Notable features", Text, true),
    ("settlement", "governance", "governed_by", "Governed by", Reference("being"), false),
    // Organizations, cultures, languages, religions and items.
    ("faction", "organization", "founded", "Founded", Date, false),
    ("faction", "organization", "ideology", "Ideology", Text, true),
    ("faction", "organization", "leader", "Leader", Reference("character"), false),
    ("faction", "organization", "headquarters", "Headquarters", Reference("place"), false),
    ("faction", "organization", "size", "Size", Text, false),
    ("faction", "organization", "symbols", "Symbols", Text, true),
    ("culture", "beliefs", "customs", "Customs", Text, true),
    ("culture", "beliefs", "values", "Values", Text, true),
    ("culture", "beliefs", "cuisine", "Cuisine", Text, true),
    ("culture", "beliefs", "dress", "Dress", Text, true),
    ("language", "identity", "speakers", "Speakers", Text, false),
    ("language", "identity", "writing_system", "Writing system", Text, false),
    ("language", "identity", "language_family", "Language family", Text, false),
    ("religion", "beliefs", "tenets", "Tenets", Text, true),
    ("religion", "beliefs", "practices", "Practices", Text, true),
    ("religion", "beliefs", "holy_places", "Holy places", Reference("place"), true),
    ("religion", "beliefs", "deities", "Deities", Reference("deity"), true),
    ("item", "abilities", "material", "Material", Text, false),
    ("item", "abilities", "powers", "Powers", Text, true),
    ("item", "background", "creator", "Creator", Reference("being"), false),
    ("item", "background", "created", "Created", Date, false),
];

/// Starting sections and traits for the built-in entity types.
///
/// They are a starting point: a project can add, relabel or regroup them.
#[must_use]
pub fn builtin_property_set() -> BuiltinPropertySet {
    BuiltinPropertySet {
        groups: BUILTIN_GROUPS
            .iter()
            .zip(1..)
            .map(|(&(key, label), sort_order)| PropertyGroup {
                key: TypeKey::from_static(key),
                label: label.to_owned(),
                sort_order,
            })
            .collect(),
        definitions: BUILTIN_DEFINITIONS
            .iter()
            .zip(1..)
            .map(
                |(&(owner, group, key, label, shape, multiple), sort_order)| PropertyDefinition {
                    id: PropertyDefinitionId::builtin(owner, key),
                    owner: PropertyOwner::EntityType(TypeKey::from_static(owner)),
                    key: TypeKey::from_static(key),
                    label: label.to_owned(),
                    description: String::new(),
                    group: Some(TypeKey::from_static(group)),
                    data_type: match shape {
                        Shape::Text => PropertyDataType::Text,
                        Shape::Integer => PropertyDataType::Integer,
                        Shape::Date => PropertyDataType::Date,
                        Shape::Reference(entity_type) => PropertyDataType::EntityReference {
                            entity_type: Some(TypeKey::from_static(entity_type)),
                        },
                    },
                    required: false,
                    multiple,
                    sort_order,
                    builtin: true,
                },
            )
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_type::builtin_entity_types;

    #[test]
    fn builtin_definitions_are_valid_grouped_and_have_unique_ids() {
        let set = builtin_property_set();

        let mut ids: Vec<_> = set.definitions.iter().map(|d| d.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), set.definitions.len());
        for definition in &set.definitions {
            assert_eq!(definition.validate(), Ok(()));
            let group = definition.group.as_ref();
            assert!(
                set.groups.iter().any(|g| Some(&g.key) == group),
                "{} has no group",
                definition.key
            );
        }
    }

    #[test]
    fn owners_and_references_name_defined_entity_types() {
        let types: BTreeSet<_> = builtin_entity_types().into_iter().map(|t| t.key).collect();

        for definition in builtin_property_set().definitions {
            let PropertyOwner::EntityType(owner) = &definition.owner else {
                panic!("built-in definitions describe entity types");
            };
            assert!(
                types.contains(owner),
                "{} has undefined owner {owner}",
                definition.key
            );
            if let PropertyDataType::EntityReference {
                entity_type: Some(target),
            } = &definition.data_type
            {
                assert!(
                    types.contains(target),
                    "{} references undefined {target}",
                    definition.key
                );
            }
        }
    }

    #[test]
    fn a_character_inherits_hair_and_eye_color_from_being() {
        let set = builtin_property_set();
        let owner_of = |key: &str| {
            set.definitions
                .iter()
                .find(|d| d.key.as_str() == key)
                .map(|d| d.owner.clone())
        };

        let being = PropertyOwner::EntityType(TypeKey::from_static("being"));
        assert_eq!(owner_of("hair_color"), Some(being.clone()));
        assert_eq!(owner_of("eye_color"), Some(being));
    }

    #[test]
    fn a_key_is_never_declared_twice_on_the_same_owner() {
        let set = builtin_property_set();
        let mut seen = BTreeSet::new();

        for definition in &set.definitions {
            let identity = (format!("{:?}", definition.owner), definition.key.clone());
            assert!(
                seen.insert(identity),
                "{} is declared twice",
                definition.key
            );
        }
    }

    #[test]
    fn choices_need_unique_labelled_options() {
        let choice = |options: Vec<ChoiceOption>| {
            PropertyDefinition::custom(
                PropertyOwner::EntityType(TypeKey::from_static("character")),
                TypeKey::from_static("allegiance"),
                "Allegiance",
                PropertyDataType::Choice { options },
            )
        };
        let option = |key: &'static str| ChoiceOption {
            key: TypeKey::from_static(key),
            label: key.to_owned(),
        };

        assert!(
            choice(vec![option("light"), option("dark")])
                .validate()
                .is_ok()
        );
        assert!(choice(Vec::new()).validate().is_err());
        assert!(
            choice(vec![option("light"), option("light")])
                .validate()
                .is_err()
        );
    }
}
