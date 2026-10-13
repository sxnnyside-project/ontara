use serde::{Deserialize, Serialize};

use crate::record::Record;
use crate::type_key::TypeKey;

/// How an entity is present in a narrative context. Projects may extend the built-in set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceKind {
    pub key: TypeKey,
    pub label: String,
    pub description: String,
    /// `false` for a recorded, confirmed absence, which is different from "nothing recorded".
    pub is_presence: bool,
    pub sort_order: i32,
    pub builtin: bool,
}

impl AppearanceKind {
    #[must_use]
    pub fn custom(key: TypeKey, label: impl Into<String>, is_presence: bool) -> Self {
        Self {
            key,
            label: label.into(),
            description: String::new(),
            is_presence,
            sort_order: 0,
            builtin: false,
        }
    }
}

impl Record for AppearanceKind {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

const BUILTIN_APPEARANCE_KINDS: [(&str, &str, bool); 6] = [
    ("physical_presence", "Physical presence", true),
    ("dialogue", "Dialogue", true),
    ("indirect_mention", "Indirect mention", true),
    ("flashback", "Flashback", true),
    ("remote_presence", "Remote presence", true),
    ("confirmed_absence", "Confirmed absence", false),
];

/// The appearance kinds every new project starts with.
#[must_use]
pub fn builtin_appearance_kinds() -> Vec<AppearanceKind> {
    BUILTIN_APPEARANCE_KINDS
        .iter()
        .zip(0..)
        .map(|(&(key, label, is_presence), sort_order)| AppearanceKind {
            key: TypeKey::from_static(key),
            label: label.to_owned(),
            description: String::new(),
            is_presence,
            sort_order,
            builtin: true,
        })
        .collect()
}
