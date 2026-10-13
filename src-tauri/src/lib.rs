//! Tauri adapter: registers commands and bridges the frontend to `ontara-core`.
//! Domain rules do not live here.

mod app_info;
mod asset;
mod catalog;
mod consistency;
mod editorial;
mod entity;
mod graph;
mod life;
mod matrix;
mod navigation;
mod project;
mod relationship;
mod timeline;
mod workspace;

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
        .invoke_handler(tauri::generate_handler![
            app_info::commands::get_app_info,
            project::commands::create_project,
            project::commands::open_project,
            project::commands::close_project,
            project::commands::current_project,
            project::commands::rename_project,
            project::commands::update_universe,
            project::commands::create_project_backup,
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
            asset::commands::import_asset,
            asset::commands::list_assets,
            asset::commands::list_attachments,
            asset::commands::attach_asset,
            asset::commands::detach_asset,
            asset::commands::read_asset,
            navigation::commands::get_connections,
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
            life::commands::get_life,
            life::commands::save_life,
            consistency::commands::check_consistency,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the Ontara application");
}
