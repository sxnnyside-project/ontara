use serde_json::json;

use super::*;
use crate::entity::{NewEntity, create};
use crate::workspace::fixture::workspace_with_project;

fn rpc(workspace: &Workspace, method: &str, params: &Value) -> Value {
    let mut client = "test".to_owned();
    handle(
        workspace,
        &mut client,
        &json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }),
    )
    .expect("an answer")
}

fn tool(workspace: &Workspace, name: &str, arguments: &Value) -> (bool, String) {
    let reply = rpc(
        workspace,
        "tools/call",
        &json!({ "name": name, "arguments": arguments }),
    );
    (
        reply["result"]["isError"].as_bool().unwrap_or(true),
        reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    )
}

fn with_ayla() -> (tempfile::TempDir, Workspace, EntityId) {
    let (directory, workspace) = workspace_with_project();
    let ayla = create(
        &workspace,
        NewEntity {
            entity_type: ontara_core::TypeKey::new("character").expect("key"),
            name: "Ayla".to_owned(),
            summary: "Keeper of the lighthouse".to_owned(),
            description: String::new(),
            tags: std::collections::BTreeSet::new(),
            canon_state: None,
        },
    )
    .expect("creates")
    .id;
    (directory, workspace, ayla)
}

#[test]
fn a_project_answers_no_one_until_the_author_turns_access_on() {
    let (_directory, workspace, _ayla) = with_ayla();

    let (is_error, text) = tool(&workspace, "search", &json!({ "query": "ayla" }));

    assert!(is_error);
    assert!(text.contains("switched off"));
    let listed = rpc(&workspace, "tools/list", &json!({}));
    assert_eq!(listed["result"]["tools"].as_array().map(Vec::len), Some(5));
}

#[test]
fn with_access_on_a_client_reads_and_can_only_propose() {
    let (_directory, workspace, ayla) = with_ayla();
    workspace
        .read(|project| {
            project.set_ai_policy(ontara_storage::AiPolicy {
                cloud_allowed: false,
                mcp_allowed: true,
            })?;
            Ok(())
        })
        .expect("policy");

    let (error, found) = tool(&workspace, "search", &json!({ "query": "lighthouse" }));
    let (_, read) = tool(&workspace, "get_entity", &json!({ "id": ayla.to_string() }));
    let (proposed_error, message) = tool(
        &workspace,
        "propose_changes",
        &json!({
            "title": "Better",
            "rationale": "thin",
            "changes": [{ "op": "update_entity", "entity": ayla, "summary": "Keeper of the drowned lighthouse" }]
        }),
    );
    let (refused, _) = tool(
        &workspace,
        "remove_entity",
        &json!({ "id": ayla.to_string() }),
    );

    assert!(!error && found.contains("Ayla"));
    assert!(read.contains("Keeper of the lighthouse"));
    assert!(!proposed_error, "{message}");
    assert!(refused, "there is no tool that removes anything");
    // Proposing changed nothing; it waits for approval and says where it came from.
    let summary = crate::entity::detail(&workspace, ayla)
        .expect("reads")
        .entity
        .summary;
    assert_eq!(summary, "Keeper of the lighthouse");
    let proposals = crate::proposal::list(&workspace).expect("lists");
    assert_eq!(proposals.len(), 1);
    assert!(matches!(
        proposals[0].source,
        ontara_core::ProposalSource::Mcp
    ));
}

#[test]
fn bad_messages_get_an_error_not_a_crash() {
    let (_directory, workspace, _ayla) = with_ayla();
    let mut client = String::new();

    let unknown = handle(
        &workspace,
        &mut client,
        &json!({ "jsonrpc": "2.0", "id": 7, "method": "resources/list" }),
    )
    .expect("an answer");
    let notification = handle(
        &workspace,
        &mut client,
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    );

    assert_eq!(unknown["error"]["code"], -32601);
    assert!(notification.is_none());
}

#[test]
fn a_whole_session_over_text_lines() {
    let (_directory, workspace) = workspace_with_project();
    let path = workspace
        .read(|project| Ok(project.root().to_path_buf()))
        .expect("path");
    workspace.close().expect("closes");
    let input = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"tester"}}}"#,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        "not json",
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
    ]
    .join("\n");
    let mut output = Vec::new();

    serve(&path, input.as_bytes(), &mut output).expect("serves");

    let lines: Vec<Value> = String::from_utf8(output)
        .expect("utf8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("json"))
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["result"]["serverInfo"]["name"], "ontara");
    assert_eq!(lines[1]["error"]["code"], -32700);
    assert_eq!(lines[2]["id"], 2);
}

#[test]
fn the_connection_points_at_the_open_project() {
    let (_directory, workspace, _ayla) = with_ayla();

    let connection = connection(&workspace).expect("describes");

    assert!(connection.command.contains("ontara-mcp"));
    assert_eq!(connection.arguments.len(), 1);
    assert!(connection.configuration.contains("mcpServers"));
}

#[test]
fn archived_entities_and_oversized_answers_stay_back() {
    let (_directory, workspace, ayla) = with_ayla();
    workspace
        .read(|project| {
            project.set_ai_policy(ontara_storage::AiPolicy {
                cloud_allowed: false,
                mcp_allowed: true,
            })?;
            Ok(())
        })
        .expect("policy");
    let mut edit = crate::entity::detail(&workspace, ayla)
        .expect("reads")
        .entity;
    edit.description = "long ".repeat(10_000);
    crate::entity::update(
        &workspace,
        crate::entity::EntityEdit {
            id: ayla,
            name: edit.name.clone(),
            sort_name: None,
            summary: edit.summary.clone(),
            description: edit.description,
            tags: std::collections::BTreeSet::new(),
            names: Vec::new(),
            archived: false,
        },
    )
    .expect("updates");

    let (_, long) = tool(&workspace, "get_entity", &json!({ "id": ayla.to_string() }));
    assert!(long.chars().count() <= MAX_RESULT + 1);

    crate::entity::update(
        &workspace,
        crate::entity::EntityEdit {
            id: ayla,
            name: "Ayla".to_owned(),
            sort_name: None,
            summary: "Keeper".to_owned(),
            description: String::new(),
            tags: std::collections::BTreeSet::new(),
            names: Vec::new(),
            archived: true,
        },
    )
    .expect("archives");
    let (hidden, _) = tool(&workspace, "get_entity", &json!({ "id": ayla.to_string() }));
    let (_, found) = tool(&workspace, "search", &json!({ "query": "Keeper" }));

    assert!(hidden);
    assert!(!found.contains("Ayla"));
}
