//! The project the application has open, and how failures reach the frontend.

use std::sync::{Mutex, MutexGuard, PoisonError};

use ontara_core::{DomainError, EditError, HierarchyError};
use ontara_storage::{Project, StorageError};
use serde::Serialize;

/// Why a command failed, in a form the frontend can branch on.
///
/// Mirrored by `src/shared/backend/command-error.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    /// Stable machine name of the failure; `message` is for people.
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn no_project() -> Self {
        Self::new("no_project", "Open or create a project first.")
    }
}

impl From<DomainError> for CommandError {
    fn from(error: DomainError) -> Self {
        let code = match error {
            DomainError::StillReferenced { .. } => "still_referenced",
            DomainError::StillInUse { .. } => "still_in_use",
            DomainError::EditorialChildrenRemain(_) => "editorial_children_remain",
            DomainError::BuiltinCatalogEntry { .. } => "built_in",
            DomainError::NotFound { .. } => "not_found",
            _ => "rejected",
        };
        Self::new(code, error.to_string())
    }
}

impl From<StorageError> for CommandError {
    fn from(error: StorageError) -> Self {
        let code = match error {
            StorageError::MigrationRequired { .. } => "migration_required",
            StorageError::NotAProject(_) => "not_a_project",
            StorageError::LocationNotEmpty(_) => "location_not_empty",
            StorageError::ProjectAlreadyExists(_) => "project_exists",
            _ => "storage",
        };
        Self::new(code, error.to_string())
    }
}

impl From<EditError<StorageError>> for CommandError {
    fn from(error: EditError<StorageError>) -> Self {
        match error {
            EditError::Domain(domain) => domain.into(),
            EditError::Repository(storage) => storage.into(),
        }
    }
}

impl From<HierarchyError<StorageError>> for CommandError {
    fn from(error: HierarchyError<StorageError>) -> Self {
        match error {
            HierarchyError::Domain(domain) => domain.into(),
            HierarchyError::Repository(storage) => storage.into(),
        }
    }
}

/// The application's one open project, if any. Registered as Tauri state.
#[derive(Default)]
pub struct Workspace {
    project: Mutex<Option<Project>>,
}

impl Workspace {
    fn lock(&self) -> MutexGuard<'_, Option<Project>> {
        // A panic inside a command must not lock the author out of their project.
        self.project.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs `operation` against the open project without changing it.
    pub fn read<T>(
        &self,
        operation: impl FnOnce(&Project) -> Result<T, CommandError>,
    ) -> Result<T, CommandError> {
        let guard = self.lock();
        let project = guard.as_ref().ok_or_else(CommandError::no_project)?;
        operation(project)
    }

    /// Runs `operation` against the open project, which it may edit.
    pub fn write<T>(
        &self,
        operation: impl FnOnce(&mut Project) -> Result<T, CommandError>,
    ) -> Result<T, CommandError> {
        let mut guard = self.lock();
        let project = guard.as_mut().ok_or_else(CommandError::no_project)?;
        operation(project)
    }

    /// Makes `project` the open one, closing the previous project first.
    pub fn replace(&self, project: Project) -> Result<(), CommandError> {
        let previous = self.lock().replace(project);
        previous.map_or(Ok(()), |previous| Ok(previous.close()?))
    }

    /// Closes the open project, if any.
    pub fn close(&self) -> Result<(), CommandError> {
        let previous = self.lock().take();
        previous.map_or(Ok(()), |previous| Ok(previous.close()?))
    }
}

/// A workspace with a freshly created project, for the tests of the features.
#[cfg(test)]
#[allow(clippy::expect_used)] // A fixture that cannot be built should fail the test loudly.
pub(crate) mod fixture {
    use ontara_core::Timestamp;
    use ontara_storage::Project;

    use super::Workspace;

    pub(crate) fn workspace_with_project() -> (tempfile::TempDir, Workspace) {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let project = Project::create(
            &directory.path().join("Aurea.ontara"),
            "Aurea",
            Timestamp::from_unix_millis(1_000),
        )
        .expect("a new project");
        let workspace = Workspace::default();
        workspace.replace(project).expect("the project opens");
        (directory, workspace)
    }
}
