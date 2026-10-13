use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

const MAX_LENGTH: usize = 64;

/// Stable machine identity of an entity type or relationship type.
///
/// Visible labels may change per project; the key never does, so renaming a label cannot break data.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TypeKey(String);

impl TypeKey {
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let mut characters = raw.chars();
        let valid_start = characters
            .next()
            .is_some_and(|first| first.is_ascii_lowercase());
        let valid_rest =
            characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

        if valid_start && valid_rest && raw.len() <= MAX_LENGTH {
            Ok(Self(raw))
        } else {
            Err(DomainError::InvalidTypeKey(raw))
        }
    }

    /// Builds a key from a compile-time constant. Tests assert every constant is valid.
    pub(crate) fn from_static(key: &'static str) -> Self {
        Self(key.to_owned())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TypeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for TypeKey {
    type Error = DomainError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::new(raw)
    }
}

impl From<TypeKey> for String {
    fn from(key: TypeKey) -> Self {
        key.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_snake_case_keys() {
        assert!(TypeKey::new("character").is_ok());
        assert!(TypeKey::new("hunter_guild2").is_ok());
    }

    #[test]
    fn rejects_keys_that_are_not_stable_machine_names() {
        for raw in [
            "",
            "Character",
            "2fast",
            "has space",
            "dash-ed",
            &"a".repeat(65),
        ] {
            assert!(TypeKey::new(raw).is_err(), "{raw:?} should be rejected");
        }
    }

    #[test]
    fn deserialization_validates() {
        assert!(serde_json::from_str::<TypeKey>("\"Bad Key\"").is_err());
        assert!(serde_json::from_str::<TypeKey>("\"good_key\"").is_ok());
    }
}
