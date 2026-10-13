//! Running the consistency checks and presenting what they found.

use std::collections::BTreeMap;

use ontara_core::{EntityId, Examined, Rule, Severity, SeverityCounts, check_consistency};
use serde::Serialize;

use crate::entity::{EntityRef, resolve, resolve_all, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// One finding with the entities it names resolved.
///
/// Mirrored by `src/features/consistency/api/consistency-api.ts`.
#[derive(Debug, Serialize)]
pub struct FindingView {
    pub fingerprint: String,
    pub rule: Rule,
    pub code: &'static str,
    pub severity: Severity,
    /// The entity to open to deal with the finding.
    pub subject: EntityRef,
    pub related: Vec<EntityRef>,
    pub facts: BTreeMap<&'static str, String>,
}

#[derive(Debug, Serialize)]
pub struct ReportView {
    pub findings: Vec<FindingView>,
    pub counts: SeverityCounts,
    pub examined: Examined,
}

pub fn run(workspace: &Workspace) -> Result<ReportView, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let report = check_consistency(&reader)?;
        let findings = report
            .findings
            .into_iter()
            .map(|finding| {
                let related: &[EntityId] = &finding.related;
                Ok(FindingView {
                    subject: resolve(&reader, &labels, &finding.subject)?,
                    related: resolve_all(&reader, &labels, related)?,
                    fingerprint: finding.fingerprint,
                    rule: finding.rule,
                    code: finding.code,
                    severity: finding.severity,
                    facts: finding.facts,
                })
            })
            .collect::<Result<Vec<_>, CommandError>>()?;
        Ok(ReportView {
            findings,
            counts: report.counts,
            examined: report.examined,
        })
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use ontara_core::TypeKey;

    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    #[test]
    fn findings_name_the_entities_involved() {
        let (_directory, workspace) = workspace_with_project();
        for _ in 0..2 {
            create(
                &workspace,
                NewEntity {
                    entity_type: TypeKey::new("character").expect("a valid key"),
                    name: "Ayla".to_owned(),
                    summary: String::new(),
                    description: String::new(),
                    tags: std::collections::BTreeSet::new(),
                    canon_state: None,
                },
            )
            .expect("creates");
        }

        let report = run(&workspace).expect("checks");

        let duplicate = report
            .findings
            .iter()
            .find(|finding| finding.rule == Rule::DuplicateName)
            .expect("the duplicate is found");
        assert_eq!(duplicate.subject.name, "Ayla");
        assert_eq!(duplicate.related.len(), 1);
        assert_eq!(report.examined.entities, 2);
    }
}
