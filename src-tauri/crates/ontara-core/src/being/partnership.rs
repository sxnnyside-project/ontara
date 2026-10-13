use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, PartnershipId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartnershipEndReason {
    Death,
    Divorce,
    Annulment,
    Separation,
    Broken,
    Ongoing,
}

impl PartnershipEndReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Death => "death",
            Self::Divorce => "divorce",
            Self::Annulment => "annulment",
            Self::Separation => "separation",
            Self::Broken => "broken",
            Self::Ongoing => "ongoing",
        }
    }
}

impl fmt::Display for PartnershipEndReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PartnershipEndReason {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "death" => Ok(Self::Death),
            "divorce" => Ok(Self::Divorce),
            "annulment" => Ok(Self::Annulment),
            "separation" => Ok(Self::Separation),
            "broken" => Ok(Self::Broken),
            "ongoing" => Ok(Self::Ongoing),
            other => Err(DomainError::UnknownValue(
                "partnership end reason",
                other.to_owned(),
            )),
        }
    }
}

/// A marriage, betrothal or other partnership between two beings, over a period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Partnership {
    pub id: PartnershipId,
    pub first: EntityId,
    pub second: EntityId,
    pub kind: TypeKey,
    pub end_reason: Option<PartnershipEndReason>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Partnership {
    #[must_use]
    pub fn new(first: EntityId, second: EntityId, kind: TypeKey, now: Timestamp) -> Self {
        Self {
            id: PartnershipId::generate(),
            first,
            second,
            kind,
            end_reason: None,
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
        Ok(())
    }
}

impl Record for Partnership {
    type Id = PartnershipId;

    fn id(&self) -> PartnershipId {
        self.id
    }
}
