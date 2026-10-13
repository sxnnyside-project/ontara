use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::require_optional_text;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::record::Record;
use crate::temporal::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRarity {
    Mundane,
    Common,
    Uncommon,
    Rare,
    Legendary,
    Unique,
}

impl ItemRarity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mundane => "mundane",
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Legendary => "legendary",
            Self::Unique => "unique",
        }
    }
}

impl fmt::Display for ItemRarity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ItemRarity {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "mundane" => Ok(Self::Mundane),
            "common" => Ok(Self::Common),
            "uncommon" => Ok(Self::Uncommon),
            "rare" => Ok(Self::Rare),
            "legendary" => Ok(Self::Legendary),
            "unique" => Ok(Self::Unique),
            other => Err(DomainError::UnknownValue("item rarity", other.to_owned())),
        }
    }
}

/// What is true of an item as a whole.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemProfile {
    pub entity: EntityId,
    pub material: Option<String>,
    pub rarity: Option<ItemRarity>,
    pub creator: Option<EntityId>,
    pub made: Option<Instant>,
    pub destroyed: Option<Instant>,
}

impl ItemProfile {
    #[must_use]
    pub fn new(entity: EntityId) -> Self {
        Self {
            entity,
            material: None,
            rarity: None,
            creator: None,
            made: None,
            destroyed: None,
        }
    }

    /// Nothing to put in order for this record.
    pub fn normalize(&mut self) {}

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.material.as_deref(), "material")?;
        if let Some(instant) = &self.made {
            instant.validate()?;
        }
        if let Some(instant) = &self.destroyed {
            instant.validate()?;
        }
        Ok(())
    }
}

impl Record for ItemProfile {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.entity
    }
}
