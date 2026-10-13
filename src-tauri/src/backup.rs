//! Copies of the project the author can go back to: listing them and restoring one.

use ontara_core::Timestamp;
use ontara_storage::{BackupFile, MigrationPolicy, Project, restore_backup};
use serde::Serialize;

use crate::project::{ProjectSummary, summarize};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// How many automatic backups are kept; older ones are deleted as new ones are made.
pub const KEEP_AUTOMATIC: usize = 7;

/// How long an automatic backup stays current before the next open takes another.
const AUTOMATIC_INTERVAL_MS: i64 = 24 * 60 * 60 * 1000;

/// One backup of the project.
///
/// Mirrored by `src/features/project/api/backup-api.ts`.
#[derive(Debug, Serialize)]
pub struct BackupSummary {
    pub name: String,
    /// What made it: `backup` (the author), `auto`, `pre-migration-v1` or `pre-restore`.
    pub kind: String,
    pub created_at: i64,
    pub size_bytes: u64,
}

impl From<BackupFile> for BackupSummary {
    fn from(file: BackupFile) -> Self {
        Self {
            name: file.name,
            kind: file.label,
            created_at: file.created_at,
            size_bytes: file.size_bytes,
        }
    }
}

pub fn list(workspace: &Workspace) -> Result<Vec<BackupSummary>, CommandError> {
    workspace.read(|project| {
        Ok(project
            .backups()?
            .into_iter()
            .map(BackupSummary::from)
            .collect())
    })
}

/// Takes the daily automatic backup when the latest one is older than a day, and keeps only
/// the most recent few. A failure to back up never stops the project from opening.
pub(crate) fn keep_automatic(project: &Project, now: Timestamp) {
    let Ok(existing) = project.backups() else {
        return;
    };
    let due = existing
        .iter()
        .filter(|file| file.label == "auto")
        .map(|file| file.created_at)
        .max()
        .is_none_or(|latest| now.as_unix_millis() - latest >= AUTOMATIC_INTERVAL_MS);
    if due && project.create_backup_labeled("auto", now).is_ok() {
        let _ = project.prune_backups("auto", KEEP_AUTOMATIC);
    }
}

/// Replaces the project with one of its backups. What the project was before is kept as its
/// own backup, so a restore can itself be undone.
pub fn restore(workspace: &Workspace, name: &str) -> Result<ProjectSummary, CommandError> {
    let (root, chosen) = workspace.read(|project| {
        let chosen = project
            .backups()?
            .into_iter()
            .find(|file| file.name == name)
            .ok_or_else(|| {
                CommandError::new(
                    "not_found",
                    format!("`{name}` is not a backup of this project"),
                )
            })?;
        Ok((project.root().to_path_buf(), chosen.path))
    })?;

    workspace.close()?;
    let restored = restore_backup(&root, &chosen, Timestamp::now());
    // Whether or not the restore worked, the author gets a project back.
    let reopened = Project::open(&root, MigrationPolicy::Apply, Timestamp::now())?;
    let summary = summarize(&reopened)?;
    workspace.replace(reopened)?;
    restored?;
    Ok(summary)
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use super::*;
    use crate::project::update_universe;
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn a_backup_restores_what_the_project_was_then() {
        let (_directory, workspace) = workspace_with_project();
        update_universe(&workspace, "Before".into(), String::new()).expect("describes");
        workspace
            .read(|project| Ok(project.create_backup(Timestamp::now())?))
            .expect("backs up");
        update_universe(&workspace, "After".into(), String::new()).expect("describes");
        let name = list(&workspace).expect("lists")[0].name.clone();

        let summary = restore(&workspace, &name).expect("restores");

        assert_eq!(summary.universe.summary, "Before");
        let kinds: Vec<String> = list(&workspace)
            .expect("lists")
            .into_iter()
            .map(|backup| backup.kind)
            .collect();
        assert!(kinds.iter().any(|kind| kind == "pre-restore"));
    }

    #[test]
    fn only_a_backup_of_this_project_can_be_restored() {
        let (_directory, workspace) = workspace_with_project();

        let error = restore(&workspace, "../../elsewhere.sqlite").expect_err("not a backup");

        assert_eq!(error.code, "not_found");
    }

    #[test]
    fn the_automatic_backup_is_taken_once_a_day() {
        let (_directory, workspace) = workspace_with_project();
        let count = |workspace: &Workspace| {
            list(workspace)
                .expect("lists")
                .iter()
                .filter(|backup| backup.kind == "auto")
                .count()
        };

        workspace
            .read(|project| {
                keep_automatic(project, Timestamp::from_unix_millis(1_000_000));
                keep_automatic(project, Timestamp::from_unix_millis(1_000_500));
                Ok(())
            })
            .expect("reads");
        let same_day = count(&workspace);
        workspace
            .read(|project| {
                keep_automatic(
                    project,
                    Timestamp::from_unix_millis(1_000_000 + AUTOMATIC_INTERVAL_MS),
                );
                Ok(())
            })
            .expect("reads");

        assert_eq!(same_day, 1);
        assert_eq!(count(&workspace), 2);
    }
}
