//! Creating, opening and closing the project the author works in.

use std::path::Path;

use ontara_core::{ModelReaderExt, Timestamp, UniverseProfile};
use ontara_storage::{MigrationPolicy, Project};
use serde::Serialize;

use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// The open project, as the frontend sees it.
///
/// Mirrored by `src/features/project/api/project-api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    /// Absolute path of the project folder.
    pub path: String,
    pub created_at: Timestamp,
    pub universe: UniverseProfile,
}

fn summarize(project: &Project) -> Result<ProjectSummary, CommandError> {
    let manifest = project.manifest();
    Ok(ProjectSummary {
        id: manifest.project_id.to_string(),
        name: manifest.name.clone(),
        path: project.root().to_string_lossy().into_owned(),
        created_at: manifest.created_at,
        universe: project.reader().universe_profile()?,
    })
}

pub fn create(
    workspace: &Workspace,
    path: &str,
    name: &str,
) -> Result<ProjectSummary, CommandError> {
    let project = Project::create(Path::new(path), name, Timestamp::now())?;
    let summary = summarize(&project)?;
    workspace.replace(project)?;
    Ok(summary)
}

pub fn open(
    workspace: &Workspace,
    path: &str,
    allow_migration: bool,
) -> Result<ProjectSummary, CommandError> {
    let policy = if allow_migration {
        MigrationPolicy::Apply
    } else {
        MigrationPolicy::Refuse
    };
    let project = Project::open(Path::new(path), policy, Timestamp::now())?;
    let summary = summarize(&project)?;
    workspace.replace(project)?;
    Ok(summary)
}

pub fn current(workspace: &Workspace) -> Result<Option<ProjectSummary>, CommandError> {
    match workspace.read(summarize) {
        Ok(summary) => Ok(Some(summary)),
        Err(error) if error.code == "no_project" => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn rename(workspace: &Workspace, name: &str) -> Result<ProjectSummary, CommandError> {
    workspace.write(|project| {
        project.rename(name)?;
        summarize(project)
    })
}

pub fn update_universe(
    workspace: &Workspace,
    summary: String,
    description: String,
) -> Result<ProjectSummary, CommandError> {
    workspace.write(|project| {
        project.edit(|editor| {
            editor.update_universe_profile(
                UniverseProfile {
                    summary,
                    description,
                    updated_at: None,
                },
                Timestamp::now(),
            )
        })?;
        summarize(project)
    })
}

pub fn create_backup(workspace: &Workspace) -> Result<String, CommandError> {
    workspace.read(|project| {
        let path = project.create_backup(Timestamp::now())?;
        Ok(path.to_string_lossy().into_owned())
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn nothing_is_open_until_a_project_is_created_or_opened() {
        let workspace = Workspace::default();

        assert_eq!(current(&workspace), Ok(None));
        assert_eq!(
            create_backup(&workspace).map_err(|error| error.code),
            Err("no_project")
        );
    }

    #[test]
    fn a_project_can_be_created_closed_and_opened_again() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let path = directory.path().join("Aurea.ontara");
        let path = path.to_string_lossy();
        let workspace = Workspace::default();

        let created = create(&workspace, &path, "Aurea").expect("the project is created");
        workspace.close().expect("the project closes");
        let reopened = open(&workspace, &path, false).expect("the project opens");

        assert_eq!(created.id, reopened.id);
        assert_eq!(reopened.name, "Aurea");
        assert_eq!(
            current(&workspace).map(|found| found.map(|p| p.name)),
            Ok(Some("Aurea".to_owned()))
        );
    }

    #[test]
    fn a_folder_that_is_not_a_project_says_so() {
        let directory = tempfile::tempdir().expect("a temporary folder");
        let workspace = Workspace::default();

        let error = open(&workspace, &directory.path().to_string_lossy(), false)
            .expect_err("an empty folder is not a project");

        assert_eq!(error.code, "not_a_project");
    }

    #[test]
    fn the_universe_description_and_the_name_can_change() {
        let (_directory, workspace) = workspace_with_project();

        let renamed = rename(&workspace, "Aurea Prime").expect("renames");
        let described = update_universe(&workspace, "A sunken empire".into(), String::new())
            .expect("describes");

        assert_eq!(renamed.name, "Aurea Prime");
        assert_eq!(described.universe.summary, "A sunken empire");
    }
}
