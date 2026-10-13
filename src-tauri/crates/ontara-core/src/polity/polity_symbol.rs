use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::require_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, PolitySymbolId};
use crate::record::Record;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolitySymbolKind {
    Flag,
    CoatOfArms,
    Anthem,
    Colors,
    Seal,
    Emblem,
}

impl PolitySymbolKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Flag => "flag",
            Self::CoatOfArms => "coat_of_arms",
            Self::Anthem => "anthem",
            Self::Colors => "colors",
            Self::Seal => "seal",
            Self::Emblem => "emblem",
        }
    }
}

impl fmt::Display for PolitySymbolKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PolitySymbolKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "flag" => Ok(Self::Flag),
            "coat_of_arms" => Ok(Self::CoatOfArms),
            "anthem" => Ok(Self::Anthem),
            "colors" => Ok(Self::Colors),
            "seal" => Ok(Self::Seal),
            "emblem" => Ok(Self::Emblem),
            other => Err(DomainError::UnknownValue(
                "polity symbol kind",
                other.to_owned(),
            )),
        }
    }
}

/// A flag, coat of arms, anthem or other symbol of a polity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolitySymbol {
    pub id: PolitySymbolId,
    pub polity: EntityId,
    pub kind: PolitySymbolKind,
    pub description: String,
}

impl PolitySymbol {
    #[must_use]
    pub fn new(polity: EntityId, kind: PolitySymbolKind, description: impl Into<String>) -> Self {
        Self {
            id: PolitySymbolId::generate(),
            polity,
            kind,
            description: description.into(),
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

impl Record for PolitySymbol {
    type Id = PolitySymbolId;

    fn id(&self) -> PolitySymbolId {
        self.id
    }
}
