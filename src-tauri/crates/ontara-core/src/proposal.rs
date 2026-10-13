//! Proposed changes: what an assistant (an AI provider or an MCP client) suggests, kept apart
//! from the universe until an author approves it.
//!
//! Nothing in a proposal touches the model. Approving a change applies it through the model
//! editor like any other edit, so every rule still holds, and what it creates says where it came
//! from and is not canon.

use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::identifier::{EntityId, ProposalId};
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// Most changes one proposal may hold. A longer list is not a proposal a person can review.
pub const MAX_CHANGES: usize = 40;

/// Longest text a change may carry, in characters.
pub const MAX_TEXT: usize = 20_000;

/// Who made a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalSource {
    /// A model of a provider the author configured.
    Ai,
    /// A client that reached the project through the local MCP server.
    Mcp,
}

impl ProposalSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ai => "ai",
            Self::Mcp => "mcp",
        }
    }

    /// Parses the stored name.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "ai" => Ok(Self::Ai),
            "mcp" => Ok(Self::Mcp),
            other => Err(DomainError::UnknownValue(
                "proposal source",
                other.to_owned(),
            )),
        }
    }
}

/// An entity a change refers to: one that exists, or one another change of the same proposal
/// creates, named as that change names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntityTarget {
    Existing { id: EntityId },
    Created { name: String },
}

/// One thing an assistant suggests. The set is closed: an assistant cannot ask for anything else.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ProposedChange {
    /// A new entity, not canon until the author says so.
    CreateEntity {
        entity_type: TypeKey,
        name: String,
        #[serde(default)]
        summary: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        tags: Vec<String>,
    },
    /// New text for an entity's summary or description, or more tags. Never its name, type or
    /// canon state.
    UpdateEntity {
        entity: EntityId,
        #[serde(default)]
        summary: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        add_tags: Vec<String>,
    },
    AddRelationship {
        relationship_type: TypeKey,
        source: EntityTarget,
        target: EntityTarget,
        #[serde(default)]
        qualifier: Option<String>,
    },
    AddNote {
        entity: EntityTarget,
        title: String,
        body: String,
    },
}

impl ProposedChange {
    /// Checks what depends only on the change: that it says something, and not too much.
    pub fn validate(&self) -> Result<(), DomainError> {
        fn text(value: &str, what: &str) -> Result<(), DomainError> {
            if value.chars().count() > MAX_TEXT {
                return Err(DomainError::Invalid(format!(
                    "{what} is longer than {MAX_TEXT} characters"
                )));
            }
            Ok(())
        }
        fn named(value: &str, what: &str) -> Result<(), DomainError> {
            if value.trim().is_empty() {
                return Err(DomainError::Invalid(format!("{what} must not be blank")));
            }
            text(value, what)
        }
        match self {
            Self::CreateEntity {
                name,
                summary,
                description,
                tags,
                ..
            } => {
                named(name, "the name")?;
                text(summary, "the summary")?;
                text(description, "the description")?;
                if tags
                    .iter()
                    .any(|tag| tag.trim().is_empty() || tag.len() > 80)
                {
                    return Err(DomainError::Invalid(
                        "a tag is blank or too long".to_owned(),
                    ));
                }
                Ok(())
            }
            Self::UpdateEntity {
                summary,
                description,
                add_tags,
                ..
            } => {
                if summary.is_none() && description.is_none() && add_tags.is_empty() {
                    return Err(DomainError::Invalid(
                        "the update changes nothing".to_owned(),
                    ));
                }
                text(summary.as_deref().unwrap_or_default(), "the summary")?;
                text(
                    description.as_deref().unwrap_or_default(),
                    "the description",
                )?;
                if add_tags
                    .iter()
                    .any(|tag| tag.trim().is_empty() || tag.len() > 80)
                {
                    return Err(DomainError::Invalid(
                        "a tag is blank or too long".to_owned(),
                    ));
                }
                Ok(())
            }
            Self::AddRelationship { qualifier, .. } => {
                text(qualifier.as_deref().unwrap_or_default(), "the qualifier")
            }
            Self::AddNote { title, body, .. } => {
                named(title, "the title")?;
                text(body, "the body")
            }
        }
    }
}

/// Where a change stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Pending,
    Applied,
    Rejected,
    /// Approved, but the model refused it; the reason is kept.
    Failed,
}

impl ChangeStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Applied => "applied",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }

    /// Parses the stored name.
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        match raw {
            "pending" => Ok(Self::Pending),
            "applied" => Ok(Self::Applied),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            other => Err(DomainError::UnknownValue("change status", other.to_owned())),
        }
    }
}

/// One change of a proposal, with where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalChange {
    /// Position within the proposal; the order changes are applied in.
    pub position: u32,
    pub change: ProposedChange,
    pub status: ChangeStatus,
    /// Why the model refused it, when it did.
    pub error: Option<String>,
}

/// A set of suggested changes with the reason for them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub source: ProposalSource,
    /// Who suggested it: the provider and model, or the MCP client.
    pub author: String,
    pub title: String,
    pub rationale: String,
    pub created_at: Timestamp,
    pub changes: Vec<ProposalChange>,
}

impl Proposal {
    /// A new proposal whose changes are all pending.
    pub fn new(
        source: ProposalSource,
        author: impl Into<String>,
        title: impl Into<String>,
        rationale: impl Into<String>,
        changes: Vec<ProposedChange>,
        now: Timestamp,
    ) -> Result<Self, DomainError> {
        let proposal = Self {
            id: ProposalId::generate(),
            source,
            author: author.into(),
            title: title.into(),
            rationale: rationale.into(),
            created_at: now,
            changes: changes
                .into_iter()
                .zip(0..)
                .map(|(change, position)| ProposalChange {
                    position,
                    change,
                    status: ChangeStatus::Pending,
                    error: None,
                })
                .collect(),
        };
        proposal.validate()?;
        Ok(proposal)
    }

    /// A proposal needs a reason, at least one change and no more than a person can review.
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() {
            return Err(DomainError::Invalid(
                "the proposal needs a title".to_owned(),
            ));
        }
        if self.changes.is_empty() {
            return Err(DomainError::Invalid(
                "the proposal has no changes".to_owned(),
            ));
        }
        if self.changes.len() > MAX_CHANGES {
            return Err(DomainError::Invalid(format!(
                "a proposal holds at most {MAX_CHANGES} changes"
            )));
        }
        if self.rationale.chars().count() > MAX_TEXT {
            return Err(DomainError::Invalid("the rationale is too long".to_owned()));
        }
        for change in &self.changes {
            change.change.validate()?;
        }
        Ok(())
    }

    /// Whether every change has been decided.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        self.changes
            .iter()
            .all(|change| change.status != ChangeStatus::Pending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create(name: &str) -> ProposedChange {
        ProposedChange::CreateEntity {
            entity_type: TypeKey::new("character").unwrap_or_else(|error| panic!("{error}")),
            name: name.to_owned(),
            summary: String::new(),
            description: String::new(),
            tags: Vec::new(),
        }
    }

    #[test]
    fn a_proposal_needs_a_title_and_changes_a_person_can_review() {
        let now = Timestamp::from_unix_millis(1);
        let many = (0..=MAX_CHANGES)
            .map(|n| create(&format!("N{n}")))
            .collect();

        assert!(Proposal::new(ProposalSource::Ai, "m", " ", "", vec![create("A")], now).is_err());
        assert!(Proposal::new(ProposalSource::Ai, "m", "t", "", vec![], now).is_err());
        assert!(Proposal::new(ProposalSource::Ai, "m", "t", "", many, now).is_err());
        assert!(Proposal::new(ProposalSource::Ai, "m", "t", "", vec![create("A")], now).is_ok());
    }

    #[test]
    fn a_change_must_say_something_and_not_too_much() {
        assert!(create(" ").validate().is_err());
        let huge = ProposedChange::AddNote {
            entity: EntityTarget::Created {
                name: "A".to_owned(),
            },
            title: "t".to_owned(),
            body: "x".repeat(MAX_TEXT + 1),
        };
        let nothing = ProposedChange::UpdateEntity {
            entity: EntityId::generate(),
            summary: None,
            description: None,
            add_tags: Vec::new(),
        };

        assert!(huge.validate().is_err());
        assert!(nothing.validate().is_err());
    }

    #[test]
    fn only_the_closed_set_of_operations_can_be_read() {
        let allowed =
            r#"{"op":"add_note","entity":{"kind":"created","name":"A"},"title":"t","body":"b"}"#;
        let forbidden = r#"{"op":"remove_entity","entity":"00000000-0000-0000-0000-000000000000"}"#;
        let smuggled = r#"{"op":"update_entity","entity":"00000000-0000-0000-0000-000000000000","name":"X","canon_state":"canon","summary":"s"}"#;

        assert!(serde_json::from_str::<ProposedChange>(allowed).is_ok());
        assert!(serde_json::from_str::<ProposedChange>(forbidden).is_err());
        // Unknown fields are ignored, never applied: a name or canon state cannot ride along.
        let change: ProposedChange =
            serde_json::from_str(smuggled).unwrap_or_else(|e| panic!("{e}"));
        assert!(matches!(
            change,
            ProposedChange::UpdateEntity {
                summary: Some(_),
                ..
            }
        ));
    }
}
