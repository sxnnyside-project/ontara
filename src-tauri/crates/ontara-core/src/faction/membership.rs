use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::{EntityId, MembershipId};
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipEndReason {
    Death,
    Resignation,
    Expulsion,
    Defection,
    Dissolution,
    Ongoing,
}

impl MembershipEndReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Death => "death",
            Self::Resignation => "resignation",
            Self::Expulsion => "expulsion",
            Self::Defection => "defection",
            Self::Dissolution => "dissolution",
            Self::Ongoing => "ongoing",
        }
    }
}

impl fmt::Display for MembershipEndReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for MembershipEndReason {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "death" => Ok(Self::Death),
            "resignation" => Ok(Self::Resignation),
            "expulsion" => Ok(Self::Expulsion),
            "defection" => Ok(Self::Defection),
            "dissolution" => Ok(Self::Dissolution),
            "ongoing" => Ok(Self::Ongoing),
            other => Err(DomainError::UnknownValue(
                "membership end reason",
                other.to_owned(),
            )),
        }
    }
}

/// Someone belonging to a faction, with a role and rank, over a period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Membership {
    pub id: MembershipId,
    pub faction: EntityId,
    pub member: EntityId,
    pub role: Option<String>,
    pub rank: Option<String>,
    pub end_reason: Option<MembershipEndReason>,
    pub period: Temporal,
    pub note: Option<String>,
    pub canon_state: TypeKey,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Membership {
    #[must_use]
    pub fn new(faction: EntityId, member: EntityId, now: Timestamp) -> Self {
        Self {
            id: MembershipId::generate(),
            faction,
            member,
            role: None,
            rank: None,
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
        require_optional_text(self.role.as_deref(), "role")?;
        require_optional_text(self.rank.as_deref(), "rank")?;
        self.period.validate()?;
        require_optional_text(self.note.as_deref(), "note")?;
        if self.faction == self.member {
            return Err(DomainError::Invalid(
                "the faction and the member must be different".to_owned(),
            ));
        }
        Ok(())
    }
}

impl Record for Membership {
    type Id = MembershipId;

    fn id(&self) -> MembershipId {
        self.id
    }
}
