use serde::{Deserialize, Serialize};

use crate::record::Record;
use crate::type_key::TypeKey;

/// A level of the editorial hierarchy. Depth `0` is the outermost level.
///
/// Levels are configurable so a project can say "Act" instead of "Part", or drop a level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorialLevel {
    pub key: TypeKey,
    pub label: String,
    pub depth: i32,
    pub builtin: bool,
}

impl EditorialLevel {
    #[must_use]
    pub fn custom(key: TypeKey, label: impl Into<String>, depth: i32) -> Self {
        Self {
            key,
            label: label.into(),
            depth,
            builtin: false,
        }
    }
}

impl Record for EditorialLevel {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

const BUILTIN_EDITORIAL_LEVELS: [(&str, &str); 5] = [
    ("series", "Series"),
    ("volume", "Volume"),
    ("part", "Part"),
    ("chapter", "Chapter"),
    ("scene", "Scene"),
];

/// The editorial levels every new project starts with.
#[must_use]
pub fn builtin_editorial_levels() -> Vec<EditorialLevel> {
    BUILTIN_EDITORIAL_LEVELS
        .iter()
        .zip(0..)
        .map(|(&(key, label), depth)| EditorialLevel {
            key: TypeKey::from_static(key),
            label: label.to_owned(),
            depth,
            builtin: true,
        })
        .collect()
}
