use ontara_core::{
    EntityId, EventCausality, EventCausalityId, EventDetail, TimelineQuery, TypeKey,
};
use tauri::State;

use super::{CausalityInput, EventView, TimelineView};
use crate::workspace::{CommandError, Workspace};

#[tauri::command(async)]
pub fn get_timeline(
    workspace: State<'_, Workspace>,
    query: TimelineQuery,
) -> Result<TimelineView, CommandError> {
    super::chronology(&workspace, &query)
}

#[tauri::command(async)]
pub fn get_event(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<EventView, CommandError> {
    super::event(&workspace, &entity)
}

#[tauri::command(async)]
pub fn save_event_detail(
    workspace: State<'_, Workspace>,
    detail: EventDetail,
) -> Result<EventDetail, CommandError> {
    super::save_event(&workspace, detail)
}

#[tauri::command(async)]
pub fn clear_event_detail(
    workspace: State<'_, Workspace>,
    entity: EntityId,
) -> Result<(), CommandError> {
    super::clear_event(&workspace, &entity)
}

#[tauri::command(async)]
pub fn add_event_causality(
    workspace: State<'_, Workspace>,
    causality: CausalityInput,
) -> Result<EventCausality, CommandError> {
    super::add_causality(&workspace, causality)
}

#[tauri::command(async)]
pub fn remove_event_causality(
    workspace: State<'_, Workspace>,
    id: EventCausalityId,
) -> Result<(), CommandError> {
    super::remove_causality(&workspace, &id)
}

#[tauri::command(async)]
pub fn event_type_keys(workspace: State<'_, Workspace>) -> Result<Vec<TypeKey>, CommandError> {
    super::event_type_keys(&workspace)
}
