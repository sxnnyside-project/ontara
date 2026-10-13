use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::entity::{require_optional_text, require_text};
use crate::error::DomainError;
use crate::identifier::{SourceId, SourceReferenceId};
use crate::record::Record;
use crate::record_ref::RecordRef;
use crate::timestamp::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// A passage of the manuscript itself.
    #[default]
    Manuscript,
    /// Planning material, drafts or notes.
    Document,
    /// Something outside the project: a book, a site, an interview.
    External,
    /// A play session or an editorial review.
    Session,
}

impl SourceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manuscript => "manuscript",
            Self::Document => "document",
            Self::External => "external",
            Self::Session => "session",
        }
    }
}

impl fmt::Display for SourceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for SourceKind {
    type Err = DomainError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "manuscript" => Ok(Self::Manuscript),
            "document" => Ok(Self::Document),
            "external" => Ok(Self::External),
            "session" => Ok(Self::Session),
            other => Err(DomainError::UnknownValue("source kind", other.to_owned())),
        }
    }
}

/// Where a piece of lore comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub kind: SourceKind,
    pub title: String,
    /// A path inside the project, a URL or a bibliographic locator.
    pub locator: Option<String>,
    pub note: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Source {
    #[must_use]
    pub fn new(kind: SourceKind, title: impl Into<String>, now: Timestamp) -> Self {
        Self {
            id: SourceId::generate(),
            kind,
            title: title.into(),
            locator: None,
            note: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_text(&self.title, "source title")?;
        require_optional_text(self.locator.as_deref(), "locator")?;
        require_optional_text(self.note.as_deref(), "note")
    }
}

impl Record for Source {
    type Id = SourceId;

    fn id(&self) -> SourceId {
        self.id
    }
}

/// A citation: a source cited as support for one record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    pub id: SourceReferenceId,
    pub source: SourceId,
    pub target: RecordRef,
    /// Where in the source: a page, a chapter, a timestamp.
    pub locator: Option<String>,
    pub quote: Option<String>,
    pub created_at: Timestamp,
}

impl SourceReference {
    #[must_use]
    pub fn new(source: SourceId, target: RecordRef, now: Timestamp) -> Self {
        Self {
            id: SourceReferenceId::generate(),
            source,
            target,
            locator: None,
            quote: None,
            created_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        require_optional_text(self.locator.as_deref(), "locator")?;
        require_optional_text(self.quote.as_deref(), "quote")
    }
}

impl Record for SourceReference {
    type Id = SourceReferenceId;

    fn id(&self) -> SourceReferenceId {
        self.id
    }
}
