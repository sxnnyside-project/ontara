use serde::{Deserialize, Serialize};

use crate::record::Record;
use crate::type_key::TypeKey;

/// A canon state an author can assign to a record.
///
/// States are data, not code: every project starts with the built-in set and may add its own.
/// A contradiction between records that do not count as canon is not automatically an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonState {
    pub key: TypeKey,
    pub label: String,
    pub description: String,
    /// Whether records in this state are part of the canon the consistency checks protect.
    pub counts_as_canon: bool,
    pub sort_order: i32,
    pub builtin: bool,
}

impl CanonState {
    /// A project-specific state.
    #[must_use]
    pub fn custom(
        key: TypeKey,
        label: impl Into<String>,
        description: impl Into<String>,
        counts_as_canon: bool,
    ) -> Self {
        Self {
            key,
            label: label.into(),
            description: description.into(),
            counts_as_canon,
            sort_order: 0,
            builtin: false,
        }
    }
}

impl Record for CanonState {
    type Id = TypeKey;

    fn id(&self) -> TypeKey {
        self.key.clone()
    }
}

/// The state new records start in.
#[must_use]
pub fn default_canon_state() -> TypeKey {
    TypeKey::from_static("established")
}

const BUILTIN_CANON_STATES: [(&str, &str, &str, bool); 6] = [
    (
        "established",
        "Established",
        "Settled canon of the universe.",
        true,
    ),
    (
        "provisional",
        "Provisional",
        "Intended as canon but not yet settled.",
        false,
    ),
    (
        "disputed",
        "Disputed",
        "Conflicting accounts; deliberately unresolved.",
        false,
    ),
    (
        "deprecated",
        "Deprecated",
        "Superseded by newer canon; kept for history.",
        false,
    ),
    (
        "rejected",
        "Rejected",
        "Considered and ruled out; never canon.",
        false,
    ),
    (
        "disestablished",
        "Disestablished",
        "Was canon and has been retracted.",
        false,
    ),
];

/// The canon states every new project starts with.
#[must_use]
pub fn builtin_canon_states() -> Vec<CanonState> {
    BUILTIN_CANON_STATES
        .iter()
        .zip(0..)
        .map(
            |(&(key, label, description, counts_as_canon), sort_order)| CanonState {
                key: TypeKey::from_static(key),
                label: label.to_owned(),
                description: description.to_owned(),
                counts_as_canon,
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
    fn builtin_states_include_disestablished_and_rejected_separately() {
        let keys: Vec<_> = builtin_canon_states()
            .into_iter()
            .map(|state| state.key.to_string())
            .collect();

        assert!(keys.contains(&"disestablished".to_owned()));
        assert!(keys.contains(&"rejected".to_owned()));
        assert!(keys.contains(&"deprecated".to_owned()));
    }

    #[test]
    fn only_established_counts_as_canon_by_default() {
        let canon: Vec<_> = builtin_canon_states()
            .into_iter()
            .filter(|state| state.counts_as_canon)
            .map(|state| state.key.to_string())
            .collect();

        assert_eq!(canon, ["established"]);
    }

    #[test]
    fn the_default_state_is_defined() {
        let default = default_canon_state();

        assert!(
            builtin_canon_states()
                .iter()
                .any(|state| state.key == default)
        );
    }
}
