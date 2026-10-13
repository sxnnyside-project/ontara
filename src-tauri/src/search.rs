//! Searching the text of the whole universe.

use ontara_core::{ModelQueries, ModelReaderExt, TextHit};
use serde::Serialize;

use crate::entity::{EntityRef, reference, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// Most results one search returns; the best come first.
pub const MAX_RESULTS: usize = 30;

/// An entity whose text matched, and the passage that matched.
///
/// Mirrored by `src/features/search/api/search-api.ts`.
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub entity: EntityRef,
    /// The passage, with each matching word between U+0002 and U+0003.
    pub excerpt: String,
}

pub fn find(
    workspace: &Workspace,
    query: &str,
    include_archived: bool,
) -> Result<Vec<SearchResult>, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        let labels = type_labels(&reader)?;
        let mut results = Vec::new();
        for TextHit {
            entity, excerpt, ..
        } in reader.search_text(query, include_archived, MAX_RESULTS)?
        {
            if let Some(found) = reader.entity(&entity)? {
                results.push(SearchResult {
                    entity: reference(&found, &labels),
                    excerpt,
                });
            }
        }
        Ok(results)
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixtures fail the test loudly.
mod tests {
    use std::collections::BTreeSet;

    use ontara_core::TypeKey;

    use super::*;
    use crate::entity::{NewEntity, create};
    use crate::workspace::fixture::workspace_with_project;

    fn entity(workspace: &Workspace, name: &str, summary: &str) {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new("character").expect("a valid key"),
                name: name.to_owned(),
                summary: summary.to_owned(),
                description: String::new(),
                tags: BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates");
    }

    #[test]
    fn a_word_finds_names_and_prose_and_marks_what_matched() {
        let (_directory, workspace) = workspace_with_project();
        entity(&workspace, "Ayla", "Keeper of the drowned lighthouse");
        entity(&workspace, "Bren", "A courier");

        let by_name = find(&workspace, "ayl", false).expect("searches");
        let by_prose = find(&workspace, "drowned lighthouse", false).expect("searches");

        assert_eq!(by_name.len(), 1);
        assert_eq!(by_name[0].entity.name, "Ayla");
        assert_eq!(by_prose.len(), 1);
        assert!(by_prose[0].excerpt.contains('\u{2}'));
        assert!(find(&workspace, "   ", false).expect("searches").is_empty());
    }

    #[test]
    fn accents_do_not_get_in_the_way() {
        let (_directory, workspace) = workspace_with_project();
        entity(&workspace, "Élowen", "Guardiana del faro");

        assert_eq!(
            find(&workspace, "elowen", false).expect("searches").len(),
            1
        );
        assert_eq!(
            find(&workspace, "GUARDIANA", false)
                .expect("searches")
                .len(),
            1
        );
    }
}
