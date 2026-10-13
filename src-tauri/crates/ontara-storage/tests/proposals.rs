//! Proposals stay apart from the model until approved, and approving is one undoable edit.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;

use ontara_core::{
    ChangeStatus, DomainError, EditError, Entity, ModelReaderExt, Proposal, ProposalSource,
    ProposedChange, Timestamp, TypeKey,
};
use ontara_storage::{AiPolicy, Project};

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn create(name: &str) -> ProposedChange {
    ProposedChange::CreateEntity {
        entity_type: key("character"),
        name: name.to_owned(),
        summary: String::new(),
        description: String::new(),
        tags: Vec::new(),
    }
}

fn project() -> Result<(tempfile::TempDir, Project), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = Project::create(&directory.path().join("Aurea.ontara"), "Aurea", T0)?;
    Ok((directory, project))
}

/// What the adapter does: create what the change says, refuse the name "Refused".
fn apply(
    editor: &mut ontara_core::ModelEditor<'_, ontara_storage::SqliteModel<'_>>,
    change: &ProposedChange,
) -> Result<(), EditError<ontara_storage::StorageError>> {
    if let ProposedChange::CreateEntity {
        name, entity_type, ..
    } = change
    {
        if name == "Refused" {
            // The partial work of a refused change must not survive.
            editor.add_entity(Entity::new(entity_type.clone(), "Half done", T0))?;
            return Err(EditError::Domain(DomainError::Invalid("nope".to_owned())));
        }
        editor.add_entity(Entity::new(entity_type.clone(), name, T0))?;
    }
    Ok(())
}

#[test]
fn a_proposal_changes_nothing_until_it_is_approved() -> TestResult {
    let (_directory, mut project) = project()?;
    let proposal = Proposal::new(
        ProposalSource::Ai,
        "test model",
        "Add two",
        "because",
        vec![create("Ayla"), create("Bren"), create("Refused")],
        T0,
    )?;
    project.add_proposal(&proposal)?;

    assert!(project.reader().entities()?.is_empty());
    assert_eq!(project.proposals()?.len(), 1);

    let decided = project.decide_proposal(&proposal.id, &[0, 2], &[1], apply)?;

    let statuses: Vec<_> = decided.changes.iter().map(|change| change.status).collect();
    assert_eq!(
        statuses,
        [
            ChangeStatus::Applied,
            ChangeStatus::Rejected,
            ChangeStatus::Failed
        ]
    );
    assert!(
        decided.changes[2]
            .error
            .as_deref()
            .is_some_and(|text| text.contains("nope"))
    );
    let names: Vec<String> = project
        .reader()
        .entities()?
        .into_iter()
        .map(|e| e.name)
        .collect();
    assert_eq!(
        names,
        ["Ayla"],
        "the refused change left nothing, not even half"
    );
    Ok(())
}

#[test]
fn approving_is_one_edit_that_undo_takes_back_with_the_statuses() -> TestResult {
    let (_directory, mut project) = project()?;
    let proposal = Proposal::new(
        ProposalSource::Mcp,
        "client",
        "One",
        "",
        vec![create("Ayla")],
        T0,
    )?;
    project.add_proposal(&proposal)?;
    project.set_change_context("proposal.decide", T0);

    project.decide_proposal(&proposal.id, &[0], &[], apply)?;
    assert_eq!(project.reader().entities()?.len(), 1);
    project.undo(T0)?.expect("one edit to undo");

    assert!(project.reader().entities()?.is_empty());
    assert_eq!(
        project.proposals()?[0].changes[0].status,
        ChangeStatus::Pending
    );
    Ok(())
}

#[test]
fn proposals_survive_reopening_and_the_policy_starts_closed() -> TestResult {
    let directory = tempfile::tempdir()?;
    let root = directory.path().join("Aurea.ontara");
    let project = Project::create(&root, "Aurea", T0)?;
    assert_eq!(
        project.ai_policy()?,
        AiPolicy {
            cloud_allowed: false,
            mcp_allowed: false
        }
    );
    project.add_proposal(&Proposal::new(
        ProposalSource::Ai,
        "m",
        "Keep",
        "",
        vec![create("Ayla")],
        T0,
    )?)?;
    project.set_ai_policy(AiPolicy {
        cloud_allowed: true,
        mcp_allowed: false,
    })?;
    project.close()?;

    let reopened = Project::open(&root, ontara_storage::MigrationPolicy::Refuse, T0)?;

    assert_eq!(reopened.proposals()?.len(), 1);
    assert!(reopened.ai_policy()?.cloud_allowed);
    Ok(())
}
