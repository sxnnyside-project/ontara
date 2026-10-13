use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Who or what produced a record, so generated content is always identifiable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    #[default]
    User,
    Ai,
    Mcp,
    Import,
}

impl Origin {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Ai => "ai",
            Self::Mcp => "mcp",
            Self::Import => "import",
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Origin {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "user" => Ok(Self::User),
            "ai" => Ok(Self::Ai),
            "mcp" => Ok(Self::Mcp),
            "import" => Ok(Self::Import),
            other => Err(DomainError::UnknownValue("origin", other.to_owned())),
        }
    }
}
