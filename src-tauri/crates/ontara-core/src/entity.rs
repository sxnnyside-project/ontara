use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::canon_state::default_canon_state;
use crate::error::DomainError;
use crate::identifier::EntityId;
use crate::origin::Origin;
use crate::record::Record;
use crate::temporal::Temporal;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// What kind of name this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NameKind {
    #[default]
    Alias,
    Epithet,
    BirthName,
    FormerName,
    Title,
    Translation,
    Nickname,
}

impl NameKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alias => "alias",
            Self::Epithet => "epithet",
            Self::BirthName => "birth_name",
            Self::FormerName => "former_name",
            Self::Title => "title",
            Self::Translation => "translation",
            Self::Nickname => "nickname",
        }
    }
}

impl fmt::Display for NameKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for NameKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "alias" => Ok(Self::Alias),
            "epithet" => Ok(Self::Epithet),
            "birth_name" => Ok(Self::BirthName),
            "former_name" => Ok(Self::FormerName),
            "title" => Ok(Self::Title),
            "translation" => Ok(Self::Translation),
            "nickname" => Ok(Self::Nickname),
            other => Err(DomainError::UnknownValue("name kind", other.to_owned())),
        }
    }
}

/// A name an entity has had or is known by, and when and in what language it applies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityName {
    pub name: String,
    pub kind: NameKind,
    /// The language entity the name belongs to, for translations and in-universe languages.
    pub language: Option<EntityId>,
    /// The name to show for the entity when several apply.
    pub is_preferred: bool,
    pub period: Temporal,
    pub note: Option<String>,
}

impl EntityName {
    #[must_use]
    pub fn new(name: impl Into<String>, kind: NameKind) -> Self {
        Self {
            name: name.into(),
            kind,
            language: None,
            is_preferred: false,
            period: Temporal::Unconstrained,
            note: None,
        }
    }
}

/// An identified element of a universe: a character, a location, a chapter and so on.
///
/// What makes the entity individual (traits, characteristics, abilities) lives in its property
/// values; its connections live in relationships; where it shows up lives in appearances.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: TypeKey,
    pub name: String,
    /// Name used to order lists ("Wanderer, The").
    pub sort_name: Option<String>,
    pub names: Vec<EntityName>,
    /// One line, for lists and previews.
    pub summary: String,
    pub description: String,
    pub tags: BTreeSet<String>,
    pub canon_state: TypeKey,
    /// An archived entity is hidden from everyday views but keeps every reference.
    pub archived: bool,
    pub origin: Origin,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Entity {
    /// A new established entity with a freshly generated identifier.
    #[must_use]
    pub fn new(entity_type: TypeKey, name: impl Into<String>, now: Timestamp) -> Self {
        Self {
            id: EntityId::generate(),
            entity_type,
            name: name.into(),
            sort_name: None,
            names: Vec::new(),
            summary: String::new(),
            description: String::new(),
            tags: BTreeSet::new(),
            canon_state: default_canon_state(),
            archived: false,
            origin: Origin::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Checks the rules that depend only on this record.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.name, "name")?;
        require_optional_text(self.sort_name.as_deref(), "sort name")?;
        for name in &self.names {
            require_text(&name.name, "alternative name")?;
            require_optional_text(name.note.as_deref(), "name note")?;
            name.period.validate()?;
        }
        for tag in &self.tags {
            require_text(tag, "tag")?;
        }
        Ok(())
    }
}

impl Record for Entity {
    type Id = EntityId;

    fn id(&self) -> EntityId {
        self.id
    }
}

pub(crate) fn require_text(value: &str, what: &'static str) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        Err(DomainError::BlankText(what))
    } else {
        Ok(())
    }
}

pub(crate) fn require_optional_text(
    value: Option<&str>,
    what: &'static str,
) -> Result<(), DomainError> {
    value.map_or(Ok(()), |text| require_text(text, what))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn character(name: &str) -> Entity {
        Entity::new(
            TypeKey::from_static("character"),
            name,
            Timestamp::from_unix_millis(1),
        )
    }

    #[test]
    fn new_entities_are_established_user_records_with_matching_timestamps() {
        let entity = character("Ayla");

        assert_eq!(entity.canon_state, default_canon_state());
        assert_eq!(entity.origin, Origin::User);
        assert!(!entity.archived);
        assert_eq!(entity.created_at, entity.updated_at);
    }

    #[test]
    fn blank_names_and_tags_are_rejected() {
        assert_eq!(
            character("   ").validate(),
            Err(DomainError::BlankText("name"))
        );

        let mut entity = character("Ayla");
        entity.names.push(EntityName::new(" ", NameKind::Epithet));
        assert_eq!(
            entity.validate(),
            Err(DomainError::BlankText("alternative name"))
        );

        entity.names.clear();
        entity.tags.insert(String::new());
        assert_eq!(entity.validate(), Err(DomainError::BlankText("tag")));
    }

    #[test]
    fn names_can_carry_a_kind_a_language_and_a_period() {
        let mut entity = character("Ayla");
        let mut former = EntityName::new("Ayla of the Reeds", NameKind::FormerName);
        former.language = Some(EntityId::generate());
        former.period = Temporal::Span {
            start: None,
            end: Some(crate::temporal::Instant::at_day(100)),
        };
        entity.names.push(former);

        assert_eq!(entity.validate(), Ok(()));
    }

    #[test]
    fn duplicate_names_are_allowed_because_identity_is_the_id() {
        assert_ne!(character("Ayla").id, character("Ayla").id);
    }
}
