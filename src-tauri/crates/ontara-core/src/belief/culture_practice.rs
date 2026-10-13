use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_text;
use crate::error::DomainError;
use crate::identifier::{CulturePracticeId, EntityId};
use crate::origin::Origin;
use crate::record::Record;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CulturePracticeKind {
    Custom,
    Value,
    Taboo,
    Festival,
    Cuisine,
    Dress,
    Art,
    Law,
    Ritual,
}

impl CulturePracticeKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Value => "value",
            Self::Taboo => "taboo",
            Self::Festival => "festival",
            Self::Cuisine => "cuisine",
            Self::Dress => "dress",
            Self::Art => "art",
            Self::Law => "law",
            Self::Ritual => "ritual",
        }
    }
}

impl fmt::Display for CulturePracticeKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for CulturePracticeKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "custom" => Ok(Self::Custom),
            "value" => Ok(Self::Value),
            "taboo" => Ok(Self::Taboo),
            "festival" => Ok(Self::Festival),
            "cuisine" => Ok(Self::Cuisine),
            "dress" => Ok(Self::Dress),
            "art" => Ok(Self::Art),
            "law" => Ok(Self::Law),
            "ritual" => Ok(Self::Ritual),
            other => Err(DomainError::UnknownValue(
                "culture practice kind",
                other.to_owned(),
            )),
        }
    }
}

/// A custom, value, taboo, festival or other practice of a culture.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CulturePractice {
    pub id: CulturePracticeId,
    pub culture: EntityId,
    pub kind: CulturePracticeKind,
    pub description: String,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl CulturePractice {
    #[must_use]
    pub fn new(
        culture: EntityId,
        kind: CulturePracticeKind,
        description: impl Into<String>,
        now: Timestamp,
    ) -> Self {
        Self {
            id: CulturePracticeId::generate(),
            culture,
            kind,
            description: description.into(),
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.description, "description")?;
        Ok(())
    }
}

impl Record for CulturePractice {
    type Id = CulturePracticeId;

    fn id(&self) -> CulturePracticeId {
        self.id
    }
}
