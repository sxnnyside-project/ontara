//! Tauri adapter: registers commands and bridges the frontend to `ontara-core`.
//! Domain rules do not live here.

use tauri::Manager;

mod app_info;
mod asset;
mod assistant;
mod backup;
mod catalog;
mod consistency;
mod dossier;
mod editorial;
mod entity;
mod example;
mod export;
mod graph;
mod history;
mod matrix;
mod mcp;
mod navigation;
mod project;
mod proposal;
mod relationship;
mod search;
mod timeline;
mod workflow;
mod workspace;

/// Serves the local MCP server for the project folder in `args[0]`, over standard input and
/// output. Returns the process exit code.
#[must_use]
pub fn run_mcp(args: &[String]) -> std::process::ExitCode {
    let Some(project) = args.first() else {
        eprintln!("usage: ontara-mcp <project folder>");
        return std::process::ExitCode::from(2);
    };
    let stdin = std::io::stdin();
    match mcp::serve(
        std::path::Path::new(project),
        stdin.lock(),
        std::io::stdout(),
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ontara-mcp: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Starts the desktop application.
///
/// # Panics
///
/// Panics if the Tauri runtime fails to start, which is unrecoverable at this point.
#[allow(clippy::expect_used)] // Startup failure has no caller to report to.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(workspace::Workspace::default())
        .setup(|app| {
            // Provider settings live in the application's own configuration folder, apart from
            // every project.
            let directory = app.path().app_config_dir()?;
            app.manage(assistant::Assistant::new(&directory));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info::commands::get_app_info,
            project::commands::create_project,
            project::commands::open_project,
            project::commands::close_project,
            project::commands::current_project,
            project::commands::rename_project,
            project::commands::update_universe,
            project::commands::create_project_backup,
            project::commands::universe_counts,
            backup::commands::list_backups,
            backup::commands::restore_project_backup,
            catalog::commands::get_catalog,
            catalog::commands::define_catalog_entry,
            catalog::commands::update_catalog_entry,
            catalog::commands::catalog_entry_usage,
            catalog::commands::remove_catalog_entry,
            catalog::commands::remove_property_with_values,
            entity::commands::list_entities,
            entity::commands::get_entity,
            entity::commands::create_entity,
            entity::commands::update_entity,
            entity::commands::entity_references,
            entity::commands::remove_entity,
            entity::commands::change_canon_state,
            entity::commands::save_property_value,
            entity::commands::remove_property_value,
            entity::commands::save_note,
            entity::commands::remove_note,
            relationship::commands::list_relationships,
            relationship::commands::get_relationship,
            relationship::commands::save_relationship,
            relationship::commands::remove_relationship,
            relationship::commands::relationship_options,
            relationship::commands::relationship_candidates,
            editorial::commands::get_editorial_tree,
            editorial::commands::add_section,
            editorial::commands::reorder_section,
            editorial::commands::section_impact,
            editorial::commands::remove_section,
            assistant::commands::list_providers,
            assistant::commands::save_provider,
            assistant::commands::remove_provider,
            assistant::commands::test_provider,
            assistant::commands::preview_task,
            assistant::commands::run_task,
            assistant::commands::mcp_connection,
            proposal::commands::list_proposals,
            proposal::commands::decide_proposal,
            proposal::commands::discard_proposal,
            proposal::commands::get_ai_policy,
            proposal::commands::set_ai_policy,
            asset::commands::import_asset,
            asset::commands::list_assets,
            asset::commands::list_attachments,
            asset::commands::attach_asset,
            asset::commands::detach_asset,
            asset::commands::read_asset,
            navigation::commands::get_connections,
            example::commands::load_example_universe,
            export::commands::export_universe,
            graph::commands::explore_graph,
            matrix::commands::get_appearance_matrix,
            matrix::commands::save_appearance,
            matrix::commands::set_appearance_kind,
            matrix::commands::remove_appearance,
            timeline::commands::get_timeline,
            timeline::commands::get_event,
            timeline::commands::save_event_detail,
            timeline::commands::clear_event_detail,
            timeline::commands::add_event_causality,
            timeline::commands::remove_event_causality,
            timeline::commands::event_type_keys,
            consistency::commands::check_consistency,
            dossier::commands::get_dossier,
            dossier::commands::get_dossier_section,
            dossier::commands::save_dossier_record,
            dossier::commands::remove_dossier_record,
            history::commands::get_history,
            history::commands::undo,
            history::commands::redo,
            search::commands::search_entities,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the Ontara application");
}
