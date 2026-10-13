use ontara_core::{CatalogEntry, PropertyDefinitionId, ReferenceCount};
use tauri::State;

use super::{Catalog, CatalogRecord};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_catalog(workspace: State<'_, Workspace>) -> Result<Catalog, CommandError> {
    super::get(&workspace)
}

#[tauri::command(async)]
pub fn define_catalog_entry(
    workspace: State<'_, Workspace>,
    record: CatalogRecord,
) -> Result<Catalog, CommandError> {
    super::define(&workspace, record)
}

#[tauri::command(async)]
pub fn update_catalog_entry(
    workspace: State<'_, Workspace>,
    record: CatalogRecord,
) -> Result<Catalog, CommandError> {
    super::update(&workspace, record)
}

#[tauri::command(async)]
pub fn catalog_entry_usage(
    workspace: State<'_, Workspace>,
    entry: CatalogEntry,
) -> Result<Vec<ReferenceCount>, CommandError> {
    super::usage(&workspace, &entry)
}

#[tauri::command(async)]
pub fn remove_catalog_entry(
    workspace: State<'_, Workspace>,
    entry: CatalogEntry,
) -> Result<Catalog, CommandError> {
    super::remove(&workspace, &entry)
}

#[tauri::command(async)]
pub fn remove_property_with_values(
    workspace: State<'_, Workspace>,
    id: PropertyDefinitionId,
) -> Result<Catalog, CommandError> {
    super::remove_property_with_values(&workspace, id)
}
