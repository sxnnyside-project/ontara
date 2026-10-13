//! When a being was born and when they died, the dates the consistency checks lean on.

use ontara_core::{BeingProfile, EntityId, Instant, Reader};
use serde::Serialize;

use crate::entity::{resolve, type_labels};
use crate::workspace::{CommandError, Workspace};

// Tauri hands commands owned, deserialized arguments and `State`, so they cannot be borrowed.
#[allow(clippy::needless_pass_by_value)]
pub mod commands;

/// The birth and death recorded for a being.
///
/// Mirrored by `src/features/entity/api/life-api.ts`.
#[derive(Debug, Serialize)]
pub struct Life {
    pub birth: Option<Instant>,
    pub death: Option<Instant>,
}

pub fn get(workspace: &Workspace, entity: &EntityId) -> Result<Life, CommandError> {
    workspace.read(|project| {
        let reader = project.reader();
        resolve(&reader, &type_labels(&reader)?, entity)?;
        let profile = Reader::<BeingProfile>::get(&reader, entity)?;
        Ok(Life {
            birth: profile.as_ref().and_then(|profile| profile.birth.clone()),
            death: profile.and_then(|profile| profile.death),
        })
    })
}

/// Records birth and death, keeping whatever else the being's profile holds.
pub fn save(
    workspace: &Workspace,
    entity: EntityId,
    birth: Option<Instant>,
    death: Option<Instant>,
) -> Result<Life, CommandError> {
    workspace.write(|project| {
        let mut profile = Reader::<BeingProfile>::get(&project.reader(), &entity)?
            .unwrap_or_else(|| BeingProfile::new(entity));
        profile.birth = birth;
        profile.death = death;
        let stored = project.edit(|editor| editor.set_being_profile(profile))?;
        Ok(Life {
            birth: stored.birth,
            death: stored.death,
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

    fn entity(workspace: &Workspace, entity_type: &str) -> EntityId {
        create(
            workspace,
            NewEntity {
                entity_type: TypeKey::new(entity_type).expect("a valid key"),
                name: "Ayla".to_owned(),
                summary: String::new(),
                description: String::new(),
                tags: std::collections::BTreeSet::new(),
                canon_state: None,
            },
        )
        .expect("creates")
        .id
    }

    #[test]
    fn birth_and_death_are_recorded_and_read_back() {
        let (_directory, workspace) = workspace_with_project();
        let ayla = entity(&workspace, "character");

        let before = get(&workspace, &ayla).expect("reads");
        save(&workspace, ayla, Some(Instant::at_day(100)), None).expect("saves");
        let after = get(&workspace, &ayla).expect("reads");

        assert!(before.birth.is_none() && before.death.is_none());
        assert_eq!(after.birth, Some(Instant::at_day(100)));
        assert!(after.death.is_none());
    }

    #[test]
    fn only_beings_have_a_life() {
        let (_directory, workspace) = workspace_with_project();
        let city = entity(&workspace, "city");

        let error =
            save(&workspace, city, Some(Instant::at_day(1)), None).expect_err("not a being");

        assert_eq!(error.code, "rejected");
    }
}
