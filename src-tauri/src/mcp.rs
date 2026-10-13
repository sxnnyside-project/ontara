//! Local MCP access: lets an assistant running on this computer read a project and suggest
//! changes to it, over standard input and output. There is no network listener.
//!
//! The server can read, and it can add a proposal. It cannot edit the model: whatever a client
//! suggests waits for the author, exactly like what a provider suggests. It answers nothing at
//! all unless the project's policy allows MCP clients, which is checked on every call.

use std::io::{BufRead, Write};
use std::path::Path;

use ontara_core::{EntityFilter, EntityId, ProposalSource, Timestamp};
use ontara_storage::{MigrationPolicy, Project};
use serde_json::{Value, json};

use crate::assistant::generate;
use crate::workspace::{CommandError, Workspace};
use crate::{consistency, entity, search};

const PROTOCOL: &str = "2024-11-05";

/// Most characters one tool result carries.
const MAX_RESULT: usize = 20_000;

/// Most items a list result holds.
const MAX_ITEMS: usize = 50;

const OFF: &str = "Access for MCP clients is switched off for this project. The author can turn it \
                   on in Ontara, under the assistant settings.";

fn tools() -> Value {
    json!([
        {
            "name": "search",
            "description": "Search the universe's text: names, tags, summaries, descriptions, notes.",
            "inputSchema": {"type":"object","properties":{"query":{"type":"string"}},"required":["query"]}
        },
        {
            "name": "list_entities",
            "description": "List entities, optionally of one type key (subtypes included).",
            "inputSchema": {"type":"object","properties":{"entity_type":{"type":"string"}}}
        },
        {
            "name": "get_entity",
            "description": "Read one entity: its text, tags, notes, properties and relationships.",
            "inputSchema": {"type":"object","properties":{"id":{"type":"string"}},"required":["id"]}
        },
        {
            "name": "check_consistency",
            "description": "Run the deterministic consistency checks and list what they found.",
            "inputSchema": {"type":"object","properties":{}}
        },
        {
            "name": "propose_changes",
            "description": "Suggest changes. Nothing is applied: the author reviews each change. \
                            Operations: create_entity, update_entity, add_relationship, add_note.",
            "inputSchema": {"type":"object","properties":{
                "title":{"type":"string"},"rationale":{"type":"string"},
                "changes":{"type":"array","items":{"type":"object"}}
            },"required":["title","changes"]}
        }
    ])
}

fn clip(mut text: String) -> String {
    if text.chars().count() > MAX_RESULT {
        text = text.chars().take(MAX_RESULT).collect();
        text.push('…');
    }
    text
}

// One match over the tools, which is what a table of tools reads like.
#[allow(clippy::too_many_lines)]
fn call(
    workspace: &Workspace,
    client: &str,
    name: &str,
    arguments: &Value,
) -> Result<String, CommandError> {
    let allowed = workspace.read(|project| Ok(project.ai_policy()?.mcp_allowed))?;
    if !allowed {
        return Err(CommandError::new("mcp_off", OFF));
    }
    let text = |key: &str| arguments[key].as_str().unwrap_or_default().to_owned();
    match name {
        "search" => {
            let found = search::find(workspace, &text("query"), false)?;
            Ok(serde_json::to_string_pretty(
                &found
                    .iter()
                    .take(MAX_ITEMS)
                    .map(|hit| {
                        json!({
                            "id": hit.entity.id,
                            "name": hit.entity.name,
                            "type": hit.entity.entity_type,
                            "excerpt": hit.excerpt.replace(['\u{2}', '\u{3}'], ""),
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_default())
        }
        "list_entities" => {
            let filter = EntityFilter {
                entity_type: arguments["entity_type"]
                    .as_str()
                    .and_then(|key| ontara_core::TypeKey::new(key).ok()),
                ..EntityFilter::default()
            };
            let found = entity::list(workspace, &filter)?;
            Ok(serde_json::to_string_pretty(
                &found
                    .iter()
                    .take(MAX_ITEMS)
                    .map(|item| {
                        json!({
                            "id": item.reference.id,
                            "name": item.reference.name,
                            "type": item.reference.entity_type,
                            "summary": item.summary,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_default())
        }
        "get_entity" => {
            let id: EntityId = text("id")
                .parse()
                .map_err(|_| CommandError::new("rejected", "`id` is not an entity identifier."))?;
            let detail = entity::detail(workspace, id)?;
            // An archived entity is hidden from everyday views, and from clients too.
            if detail.entity.archived {
                return Err(CommandError::new(
                    "not_found",
                    format!("entity {id} does not exist"),
                ));
            }
            Ok(serde_json::to_string_pretty(&json!({
                "id": detail.entity.id,
                "name": detail.entity.name,
                "type": detail.entity.entity_type,
                "canon_state": detail.entity.canon_state,
                "summary": detail.entity.summary,
                "description": detail.entity.description,
                "tags": detail.entity.tags,
                "notes": detail.notes.iter().map(|n| json!({"title": n.title, "body": n.body})).collect::<Vec<_>>(),
            }))
            .unwrap_or_default())
        }
        "check_consistency" => {
            let report = consistency::run(workspace)?;
            Ok(serde_json::to_string_pretty(&json!({
                "errors": report.counts.errors,
                "warnings": report.counts.warnings,
                "findings": report.findings.iter().take(MAX_ITEMS).map(|f| json!({
                    "code": f.code,
                    "severity": f.severity,
                    "subject": f.subject.name,
                    "facts": f.facts,
                })).collect::<Vec<_>>(),
            }))
            .unwrap_or_default())
        }
        "propose_changes" => workspace.write(|project| {
            let body = json!({
                "title": arguments["title"],
                "rationale": arguments["rationale"],
                "changes": arguments["changes"],
            })
            .to_string();
            let generated = generate::proposal_from(
                project,
                &body,
                client,
                ProposalSource::Mcp,
                Timestamp::now(),
            )?;
            project.add_proposal(&generated.proposal)?;
            Ok(format!(
                "Proposal recorded with {} change(s); {} left out as unusable. The author must \
                 approve it in Ontara before anything changes.",
                generated.proposal.changes.len(),
                generated.dropped
            ))
        }),
        other => Err(CommandError::new(
            "rejected",
            format!("unknown tool `{other}`"),
        )),
    }
}

fn answer(id: &Value, result: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn failure(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Handles one JSON-RPC message; `None` for a notification, which gets no answer.
fn handle(workspace: &Workspace, client: &mut String, message: &Value) -> Option<Value> {
    let id = message.get("id")?;
    let params = &message["params"];
    Some(match message["method"].as_str().unwrap_or_default() {
        "initialize" => {
            if let Some(name) = params["clientInfo"]["name"].as_str() {
                *client = format!("MCP · {name}");
            }
            answer(
                id,
                &json!({
                    "protocolVersion": PROTOCOL,
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "ontara", "version": env!("CARGO_PKG_VERSION") },
                    "instructions": "Read the author's universe and suggest changes. Suggestions \
                                     are proposals the author reviews; nothing is applied directly."
                }),
            )
        }
        "ping" => answer(id, &json!({})),
        "tools/list" => answer(id, &json!({ "tools": tools() })),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            match call(workspace, client, name, &params["arguments"]) {
                Ok(text) => answer(
                    id,
                    &json!({ "content": [{ "type": "text", "text": clip(text) }], "isError": false }),
                ),
                Err(error) => answer(
                    id,
                    &json!({ "content": [{ "type": "text", "text": error.message }], "isError": true }),
                ),
            }
        }
        other => failure(id, -32601, &format!("method `{other}` is not supported")),
    })
}

/// Serves one client until its input ends.
pub fn serve(project: &Path, input: impl BufRead, mut output: impl Write) -> Result<(), String> {
    let opened = Project::open(project, MigrationPolicy::Refuse, Timestamp::now())
        .map_err(|error| error.to_string())?;
    let workspace = Workspace::default();
    workspace.replace(opened).map_err(|error| error.message)?;
    let mut client = "MCP client".to_owned();

    for line in input.lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(&workspace, &mut client, &message),
            Err(_) => Some(failure(
                &Value::Null,
                -32700,
                "the message is not valid JSON",
            )),
        };
        if let Some(reply) = reply {
            writeln!(output, "{reply}").map_err(|error| error.to_string())?;
            output.flush().map_err(|error| error.to_string())?;
        }
    }
    workspace.close().map_err(|error| error.message)
}

/// How a client is pointed at this project.
///
/// Mirrored by `src/features/assistant/api/assistant-api.ts`.
#[derive(Debug, serde::Serialize)]
pub struct Connection {
    /// The program to run.
    pub command: String,
    pub arguments: Vec<String>,
    /// The program is where it is expected to be.
    pub installed: bool,
    /// A configuration most MCP clients accept, ready to paste.
    pub configuration: String,
}

/// The command a client runs to reach the open project.
pub fn connection(workspace: &Workspace) -> Result<Connection, CommandError> {
    let project = workspace.read(|project| Ok(project.root().to_string_lossy().into_owned()))?;
    let name = if cfg!(windows) {
        "ontara-mcp.exe"
    } else {
        "ontara-mcp"
    };
    let program = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|folder| folder.join(name)));
    let installed = program.as_ref().is_some_and(|path| path.is_file());
    let command = program.map_or_else(
        || name.to_owned(),
        |path| path.to_string_lossy().into_owned(),
    );
    let configuration = serde_json::to_string_pretty(&json!({
        "mcpServers": { "ontara": { "command": command, "args": [project] } }
    }))
    .unwrap_or_default();
    Ok(Connection {
        command,
        arguments: vec![project],
        installed,
        configuration,
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests;
