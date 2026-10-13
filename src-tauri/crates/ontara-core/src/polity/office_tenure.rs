use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, OfficeId, OfficeTenureId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenureEndReason {
    Death,
    Abdication,
    Deposition,
    Resignation,
    TermEnded,
    Exile,
    Ongoing,
}

impl TenureEndReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Death => "death",
            Self::Abdication => "abdication",
            Self::Deposition => "deposition",
            Self::Resignation => "resignation",
            Self::TermEnded => "term_ended",
            Self::Exile => "exile",
            Self::Ongoing => "ongoing",
        }
    }
}

impl fmt::Display for TenureEndReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for TenureEndReason {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "death" => Ok(Self::Death),
            "abdication" => Ok(Self::Abdication),
            "deposition" => Ok(Self::Deposition),
            "resignation" => Ok(Self::Resignation),
            "term_ended" => Ok(Self::TermEnded),
            "exile" => Ok(Self::Exile),
            "ongoing" => Ok(Self::Ongoing),
            other => Err(DomainError::UnknownValue(
                "tenure end reason",
                other.to_owned(),
            )),
        }
    }
}

/// Who held an office, when, and how they came to hold it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfficeTenure {
    pub id: OfficeTenureId,
    pub office: OfficeId,
    pub holder: EntityId,
    pub acquisition: Option<TypeKey>,
    pub end_reason: Option<TenureEndReason>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl OfficeTenure {
    #[must_use]
    pub fn new(office: OfficeId, holder: EntityId, now: Timestamp) -> Self {
        Self {
            id: OfficeTenureId::generate(),
            office,
            holder,
            acquisition: None,
            end_reason: None,
            period: Temporal::Unconstrained,
            note: None,
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
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        Ok(())
    }
}

impl Record for OfficeTenure {
    type Id = OfficeTenureId;

    fn id(&self) -> OfficeTenureId {
        self.id
    }
}
