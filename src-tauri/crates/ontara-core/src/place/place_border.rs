use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, PlaceBorderId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderKind {
    Land,
    River,
    Sea,
    Mountain,
    Wall,
    Magical,
    Disputed,
}

impl BorderKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Land => "land",
            Self::River => "river",
            Self::Sea => "sea",
            Self::Mountain => "mountain",
            Self::Wall => "wall",
            Self::Magical => "magical",
            Self::Disputed => "disputed",
        }
    }
}

impl fmt::Display for BorderKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for BorderKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "land" => Ok(Self::Land),
            "river" => Ok(Self::River),
            "sea" => Ok(Self::Sea),
            "mountain" => Ok(Self::Mountain),
            "wall" => Ok(Self::Wall),
            "magical" => Ok(Self::Magical),
            "disputed" => Ok(Self::Disputed),
            other => Err(DomainError::UnknownValue("border kind", other.to_owned())),
        }
    }
}

/// A border between two places, over a period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceBorder {
    pub id: PlaceBorderId,
    pub first: EntityId,
    pub second: EntityId,
    pub kind: BorderKind,
    pub length_km: Option<f64>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl PlaceBorder {
    #[must_use]
    pub fn new(first: EntityId, second: EntityId, kind: BorderKind, now: Timestamp) -> Self {
        Self {
            id: PlaceBorderId::generate(),
            first,
            second,
            kind,
            length_km: None,
            period: Temporal::Unconstrained,
            note: None,
            canon_state: default_canon_state(),
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Puts the pair in storage order, so one pair is one row.
    pub fn normalize(&mut self) {
        if self.first > self.second {
            std::mem::swap(&mut self.first, &mut self.second);
        }
    }

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        if self.first >= self.second {
            return Err(DomainError::Invalid(
                "the two sides must differ and be stored in order".to_owned(),
            ));
        }
        if self.length_km.is_some_and(|value| value < 0.0) {
            return Err(DomainError::Invalid(
                "a length cannot be negative".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for PlaceBorder {
    type Id = PlaceBorderId;

    fn id(&self) -> PlaceBorderId {
        self.id
    }
}
