use serde::{Deserialize, Serialize};

use crate::record::Record;
use crate::type_key::TypeKey;

/// A stance one polity or faction can hold towards another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomaticStance {
    pub key: TypeKey,
    pub label: String,
    pub description: String,
    /// The stance is between unequals, so one side is the senior party (vassalage).
    pub is_hierarchical: bool,
    pub sort_order: i32,
    pub builtin: bool,
}

impl DiplomaticStance {
    /// A project-specific stance.
    #[must_use]
    pub fn custom(key: TypeKey, label: impl Into<String>, is_hierarchical: bool) -> Self {
        Self {
            key,
            label: label.into(),
            description: String::new(),
            is_hierarchical,
            sort_order: 0,
            builtin: false,
        }
    }
}

impl Record for DiplomaticStance {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

/// (key, label, hierarchical)
const BUILTIN_STANCES: [(&str, &str, bool); 8] = [
    ("alliance", "Alliance", false),
    ("war", "War", false),
    ("truce", "Truce", false),
    ("vassalage", "Vassalage", true),
    ("protectorate", "Protectorate", true),
    ("trade", "Trade partnership", false),
    ("rivalry", "Rivalry", false),
    ("neutral", "Neutrality", false),
];

/// The stances every new project starts with.
#[must_use]
pub fn builtin_diplomatic_stances() -> Vec<DiplomaticStance> {
    BUILTIN_STANCES
        .iter()
        .zip(0..)
        .map(
            |(&(key, label, is_hierarchical), sort_order)| DiplomaticStance {
                key: TypeKey::from_static(key),
                label: label.to_owned(),
                description: String::new(),
                is_hierarchical,
                sort_order,
                builtin: true,
            },
        )
        .collect()
}
