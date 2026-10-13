//! Proposals of changes and the project's policy for assistants, kept in the database.

use ontara_core::{
    ChangeStatus, Proposal, ProposalChange, ProposalId, ProposalSource, ProposedChange, Timestamp,
};
use rusqlite::{Connection, params};
use serde::Serialize;

use crate::error::StorageError;

/// What assistants may reach in a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AiPolicy {
    /// A provider on the internet may be sent text of the universe.
    pub cloud_allowed: bool,
    /// The local MCP server answers clients for this project.
    pub mcp_allowed: bool,
}

fn corrupt(id: &str, reason: &impl ToString) -> StorageError {
    StorageError::CorruptRecord {
        table: "proposal",
        id: id.to_owned(),
        reason: reason.to_string(),
    }
}

pub(crate) fn insert(connection: &Connection, proposal: &Proposal) -> Result<(), StorageError> {
    connection.execute(
        "INSERT INTO proposal (id, source, author, title, rationale, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            proposal.id.to_string(),
            proposal.source.as_str(),
            proposal.author,
            proposal.title,
            proposal.rationale,
            proposal.created_at.as_unix_millis()
        ],
    )?;
    for change in &proposal.changes {
        let encoded = serde_json::to_string(&change.change)
            .map_err(|error| StorageError::Encoding(error.to_string()))?;
        connection.execute(
            "INSERT INTO proposal_change (proposal_id, position, change, status, error)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                proposal.id.to_string(),
                change.position,
                encoded,
                change.status.as_str(),
                change.error
            ],
        )?;
    }
    Ok(())
}

fn changes_of(connection: &Connection, id: &str) -> Result<Vec<ProposalChange>, StorageError> {
    let mut statement = connection.prepare(
        "SELECT position, change, status, error FROM proposal_change
         WHERE proposal_id = ?1 ORDER BY position",
    )?;
    let rows: Vec<(u32, String, String, Option<String>)> = statement
        .query_map([id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(position, change, status, error)| {
            Ok(ProposalChange {
                position,
                change: serde_json::from_str::<ProposedChange>(&change)
                    .map_err(|reason| corrupt(id, &reason))?,
                status: ChangeStatus::parse(&status).map_err(|reason| corrupt(id, &reason))?,
                error,
            })
        })
        .collect()
}

pub(crate) fn list(connection: &Connection) -> Result<Vec<Proposal>, StorageError> {
    let mut statement = connection.prepare(
        "SELECT id, source, author, title, rationale, created_at FROM proposal
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows: Vec<(String, String, String, String, String, i64)> = statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?
        .collect::<Result<_, _>>()?;
    rows.into_iter()
        .map(|(id, source, author, title, rationale, created_at)| {
            Ok(Proposal {
                id: id
                    .parse::<ProposalId>()
                    .map_err(|reason| corrupt(&id, &reason))?,
                source: ProposalSource::parse(&source).map_err(|reason| corrupt(&id, &reason))?,
                changes: changes_of(connection, &id)?,
                author,
                title,
                rationale,
                created_at: Timestamp::from_unix_millis(created_at),
            })
        })
        .collect()
}

pub(crate) fn get(
    connection: &Connection,
    id: &ProposalId,
) -> Result<Option<Proposal>, StorageError> {
    Ok(list(connection)?
        .into_iter()
        .find(|proposal| &proposal.id == id))
}

pub(crate) fn set_status(
    connection: &Connection,
    id: &ProposalId,
    position: u32,
    status: ChangeStatus,
    error: Option<&str>,
) -> Result<(), StorageError> {
    connection.execute(
        "UPDATE proposal_change SET status = ?3, error = ?4
         WHERE proposal_id = ?1 AND position = ?2",
        params![id.to_string(), position, status.as_str(), error],
    )?;
    Ok(())
}

pub(crate) fn remove(connection: &Connection, id: &ProposalId) -> Result<(), StorageError> {
    connection.execute("DELETE FROM proposal WHERE id = ?1", [id.to_string()])?;
    Ok(())
}

pub(crate) fn policy(connection: &Connection) -> Result<AiPolicy, StorageError> {
    Ok(connection.query_row(
        "SELECT cloud_allowed, mcp_allowed FROM ai_policy WHERE id = 1",
        [],
        |row| {
            Ok(AiPolicy {
                cloud_allowed: row.get::<_, i64>(0)? == 1,
                mcp_allowed: row.get::<_, i64>(1)? == 1,
            })
        },
    )?)
}

pub(crate) fn set_policy(connection: &Connection, policy: AiPolicy) -> Result<(), StorageError> {
    connection.execute(
        "UPDATE ai_policy SET cloud_allowed = ?1, mcp_allowed = ?2 WHERE id = 1",
        params![
            i64::from(policy.cloud_allowed),
            i64::from(policy.mcp_allowed)
        ],
    )?;
    Ok(())
}
