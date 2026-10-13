//! Undo and redo: what the author did, and taking it back.

use ontara_core::Timestamp;
use ontara_storage::{ChangeSummary, History};
use serde::Serialize;

use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// The edits that can be undone or redone, and the one that just was.
///
/// Mirrored by `src/features/history/api/history-api.ts`.
#[derive(Debug, Serialize)]
pub struct HistoryState {
    /// Newest first.
    pub undo: Vec<ChangeSummary>,
    /// The next redo first.
    pub redo: Vec<ChangeSummary>,
    /// What an undo or redo just reversed or reapplied; absent when only reading.
    pub applied: Option<ChangeSummary>,
}

fn state(history: History, applied: Option<ChangeSummary>) -> HistoryState {
    HistoryState {
        undo: history.undo,
        redo: history.redo,
        applied,
    }
}

pub fn get(workspace: &Workspace) -> Result<HistoryState, CommandError> {
    workspace.read(|project| Ok(state(project.history()?, None)))
}

pub fn undo(workspace: &Workspace) -> Result<HistoryState, CommandError> {
    workspace.write(|project| {
        let applied = project.undo(Timestamp::now())?;
        Ok(state(project.history()?, applied))
    })
}

pub fn redo(workspace: &Workspace) -> Result<HistoryState, CommandError> {
    workspace.write(|project| {
        let applied = project.redo(Timestamp::now())?;
        Ok(state(project.history()?, applied))
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn an_edit_is_listed_by_its_name_and_can_be_undone_and_redone() {
        let (_directory, workspace) = workspace_with_project();
        crate::project::update_universe(&workspace, "A sunken empire".into(), String::new())
            .expect("edits");

        let before = get(&workspace).expect("reads");
        let undone = undo(&workspace).expect("undoes");
        let universe = crate::project::current(&workspace)
            .expect("reads")
            .expect("open")
            .universe;
        let redone = redo(&workspace).expect("redoes");

        assert_eq!(before.undo[0].label, "project.update_universe");
        assert_eq!(
            undone.applied.as_ref().map(|change| change.label.as_str()),
            Some("project.update_universe")
        );
        assert_eq!(universe.summary, "");
        assert!(undone.undo.is_empty() && undone.redo.len() == 1);
        assert!(redone.redo.is_empty() && redone.undo.len() == 1);
    }

    #[test]
    fn undoing_with_nothing_to_undo_changes_nothing() {
        let (_directory, workspace) = workspace_with_project();

        let state = undo(&workspace).expect("is not an error");

        assert!(state.applied.is_none());
    }
}
