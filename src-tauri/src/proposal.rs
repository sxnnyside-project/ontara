//! Reviewing what assistants suggested: listing proposals, approving or rejecting each change,
//! and the project's policy for assistants.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use ontara_core::{
    ChangeStatus, DomainError, EditError, Entity, EntityId, EntityTarget, ModelReaderExt, Note,
    Origin, Proposal, ProposalId, ProposalSource, ProposedChange, Relationship, Timestamp, TypeKey,
};
use ontara_storage::{AiPolicy, StorageError};
use serde::Serialize;

use crate::entity::{EntityRef, reference, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// What an entity says now, so an update can be read as a difference.
///
/// Mirrored by `src/features/assistant/api/proposal-api.ts`.
#[derive(Debug, Serialize)]
pub struct EntityBefore {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ChangeView {
    pub position: u32,
    pub change: ProposedChange,
    pub status: ChangeStatus,
    pub error: Option<String>,
    /// For an update: the entity as it is now.
    pub before: Option<EntityBefore>,
}

#[derive(Debug, Serialize)]
pub struct ProposalView {
    pub id: ProposalId,
    pub source: ProposalSource,
    pub author: String,
    pub title: String,
    pub rationale: String,
    pub created_at: Timestamp,
    pub changes: Vec<ChangeView>,
    /// The existing entities the changes mention, by identifier.
    pub entities: BTreeMap<String, EntityRef>,
    /// Every change has been decided.
    pub settled: bool,
}

fn mentioned_ids(change: &ProposedChange) -> Vec<EntityId> {
    let target = |target: &EntityTarget| match target {
        EntityTarget::Existing { id } => Some(*id),
        EntityTarget::Created { .. } => None,
    };
    match change {
        ProposedChange::CreateEntity { .. } => Vec::new(),
        ProposedChange::UpdateEntity { entity, .. } => vec![*entity],
        ProposedChange::AddRelationship {
            source, target: to, ..
        } => [target(source), target(to)].into_iter().flatten().collect(),
        ProposedChange::AddNote { entity, .. } => target(entity).into_iter().collect(),
    }
}

fn view(
    project: &ontara_storage::Project,
    proposal: Proposal,
) -> Result<ProposalView, CommandError> {
    let reader = project.reader();
    let labels = type_labels(&reader)?;
    let mut entities = BTreeMap::new();
    let mut changes = Vec::new();
    for change in &proposal.changes {
        for id in mentioned_ids(&change.change) {
            if let Some(entity) = reader.entity(&id)? {
                entities.insert(id.to_string(), reference(&entity, &labels));
            }
        }
        let before = match &change.change {
            ProposedChange::UpdateEntity { entity, .. } => {
                reader.entity(entity)?.map(|entity| EntityBefore {
                    name: entity.name,
                    summary: entity.summary,
                    description: entity.description,
                    tags: entity.tags.into_iter().collect(),
                })
            }
            _ => None,
        };
        changes.push(ChangeView {
            position: change.position,
            change: change.change.clone(),
            status: change.status,
            error: change.error.clone(),
            before,
        });
    }
    Ok(ProposalView {
        settled: proposal.is_settled(),
        id: proposal.id,
        source: proposal.source,
        author: proposal.author,
        title: proposal.title,
        rationale: proposal.rationale,
        created_at: proposal.created_at,
        changes,
        entities,
    })
}

pub fn list(workspace: &Workspace) -> Result<Vec<ProposalView>, CommandError> {
    workspace.read(|project| {
        project
            .proposals()?
            .into_iter()
            .map(|proposal| view(project, proposal))
            .collect()
    })
}

type Editor<'a, 'b> = ontara_core::ModelEditor<'a, ontara_storage::SqliteModel<'b>>;

fn unmet(name: &str) -> EditError<StorageError> {
    EditError::Domain(DomainError::Invalid(format!(
        "it needs \"{name}\", which was not created"
    )))
}

fn resolve(
    target: &EntityTarget,
    created: &HashMap<String, EntityId>,
) -> Result<EntityId, EditError<StorageError>> {
    match target {
        EntityTarget::Existing { id } => Ok(*id),
        EntityTarget::Created { name } => created.get(name).copied().ok_or_else(|| unmet(name)),
    }
}

/// Applies one approved change through the editor, with the origin of whoever suggested it, and
/// never as canon.
struct Applier {
    origin: Origin,
    now: Timestamp,
    canon: TypeKey,
    created: HashMap<String, EntityId>,
    current: HashMap<EntityId, Entity>,
}

impl Applier {
    fn apply(
        &mut self,
        editor: &mut Editor<'_, '_>,
        change: &ProposedChange,
    ) -> Result<(), EditError<StorageError>> {
        match change {
            ProposedChange::CreateEntity {
                entity_type,
                name,
                summary,
                description,
                tags,
            } => {
                let mut entity = Entity::new(entity_type.clone(), name.trim(), self.now);
                summary.clone_into(&mut entity.summary);
                description.clone_into(&mut entity.description);
                entity.tags = tags
                    .iter()
                    .map(|tag| tag.trim().to_owned())
                    .collect::<BTreeSet<_>>();
                entity.origin = self.origin;
                entity.canon_state = self.canon.clone();
                let added = editor.add_entity(entity)?;
                self.created.insert(name.clone(), added.id);
            }
            ProposedChange::UpdateEntity {
                entity,
                summary,
                description,
                add_tags,
            } => {
                let mut updated = self.current.get(entity).cloned().ok_or_else(|| {
                    EditError::Domain(DomainError::NotFound {
                        what: "entity",
                        id: entity.to_string(),
                    })
                })?;
                if let Some(summary) = summary {
                    summary.clone_into(&mut updated.summary);
                }
                if let Some(description) = description {
                    description.clone_into(&mut updated.description);
                }
                updated
                    .tags
                    .extend(add_tags.iter().map(|tag| tag.trim().to_owned()));
                let stored = editor.update_entity(updated, self.now)?;
                self.current.insert(*entity, stored);
            }
            ProposedChange::AddRelationship {
                relationship_type,
                source,
                target,
                qualifier,
            } => {
                let mut relationship = Relationship::new(
                    relationship_type.clone(),
                    resolve(source, &self.created)?,
                    resolve(target, &self.created)?,
                    self.now,
                );
                relationship.qualifier.clone_from(qualifier);
                relationship.origin = self.origin;
                relationship.canon_state = self.canon.clone();
                editor.add_relationship(relationship)?;
            }
            ProposedChange::AddNote {
                entity,
                title,
                body,
            } => {
                editor.add_note(Note::new(
                    resolve(entity, &self.created)?,
                    title,
                    body,
                    self.now,
                ))?;
            }
        }
        Ok(())
    }
}

/// Approves some changes of a proposal and rejects others, in one undoable edit.
pub fn decide(
    workspace: &Workspace,
    id: ProposalId,
    approve: &[u32],
    reject: &[u32],
) -> Result<ProposalView, CommandError> {
    workspace.record("proposal.decide", |project| {
        let proposal = project
            .proposals()?
            .into_iter()
            .find(|proposal| proposal.id == id)
            .ok_or_else(|| {
                CommandError::new("not_found", format!("proposal {id} does not exist"))
            })?;
        let origin = match proposal.source {
            ProposalSource::Ai => Origin::Ai,
            ProposalSource::Mcp => Origin::Mcp,
        };
        let reader = project.reader();
        let mut current = HashMap::new();
        for change in &proposal.changes {
            if let ProposedChange::UpdateEntity { entity, .. } = &change.change
                && let Some(found) = reader.entity(entity)?
            {
                current.insert(*entity, found);
            }
        }
        drop(reader);
        let mut applier = Applier {
            origin,
            now: Timestamp::now(),
            canon: TypeKey::new("provisional")?,
            created: HashMap::new(),
            current,
        };
        project.decide_proposal(&id, approve, reject, |editor, change| {
            applier.apply(editor, change)
        })?;
        let decided = project
            .proposals()?
            .into_iter()
            .find(|proposal| proposal.id == id)
            .ok_or_else(|| CommandError::new("not_found", "the proposal vanished"))?;
        view(project, decided)
    })
}

/// Removes a proposal the author is done with. Changes already applied stay applied.
pub fn discard(workspace: &Workspace, id: ProposalId) -> Result<(), CommandError> {
    workspace.read(|project| Ok(project.discard_proposal(&id)?))
}

/// Mirrors `AiPolicy` for the interface.
#[derive(Debug, Serialize)]
pub struct PolicyView {
    pub cloud_allowed: bool,
    pub mcp_allowed: bool,
}

pub fn policy(workspace: &Workspace) -> Result<PolicyView, CommandError> {
    workspace.read(|project| {
        let policy = project.ai_policy()?;
        Ok(PolicyView {
            cloud_allowed: policy.cloud_allowed,
            mcp_allowed: policy.mcp_allowed,
        })
    })
}

pub fn set_policy(
    workspace: &Workspace,
    cloud_allowed: bool,
    mcp_allowed: bool,
) -> Result<PolicyView, CommandError> {
    workspace.read(|project| {
        project.set_ai_policy(AiPolicy {
            cloud_allowed,
            mcp_allowed,
        })?;
        Ok(())
    })?;
    policy(workspace)
}
