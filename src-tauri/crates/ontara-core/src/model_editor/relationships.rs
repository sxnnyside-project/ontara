use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::error::DomainError;
use crate::identifier::RelationshipId;
use crate::record_ref::{RecordKind, RecordRef};
use crate::relationship::{Lifecycle, Relationship};
use crate::relationship_type::RelationshipType;
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;

impl<R: ModelRepository> ModelEditor<'_, R> {
    fn validate_relationship(&self, relationship: &Relationship) -> Result<(), Failure<R>> {
        relationship.validate()?;
        let relationship_type = self.require_defined::<RelationshipType>(
            "relationship type",
            &relationship.relationship_type,
        )?;
        self.require_defined::<CanonState>("canon state", &relationship.canon_state)?;

        for (side, endpoint, allowed) in [
            (
                "source",
                &relationship.source,
                &relationship_type.allowed_sources,
            ),
            (
                "target",
                &relationship.target,
                &relationship_type.allowed_targets,
            ),
        ] {
            let (entity, _) = self.entity_and_type(endpoint)?;
            let mut fits = allowed.is_empty();
            for candidate in allowed {
                fits = fits || self.is_type(&entity.entity_type, candidate)?;
            }
            if !fits {
                return Err(DomainError::EndpointNotAllowed {
                    relationship_type: relationship_type.key.clone(),
                    side,
                    entity_type: entity.entity_type,
                }
                .into());
            }
        }
        if let Some(context) = &relationship.context {
            self.narrative_context(context)?;
        }
        if relationship_type.acyclic && relationship.lifecycle == Lifecycle::Active {
            // Adding source -> target must not make the source reachable from the target.
            let loops = crate::hierarchy::reaches(
                &*self.repository,
                &relationship.target,
                &relationship.source,
                &relationship.relationship_type,
                Some(&relationship.id),
            )?;
            if loops {
                return Err(DomainError::WouldCreateCycle {
                    relationship_type: relationship.relationship_type.clone(),
                }
                .into());
            }
        }
        Ok(())
    }

    pub fn add_relationship(
        &mut self,
        relationship: Relationship,
    ) -> Result<Relationship, Failure<R>> {
        self.validate_relationship(&relationship)?;
        if self.find::<Relationship>(&relationship.id)?.is_some() {
            return Err(DomainError::duplicate("relationship", relationship.id).into());
        }
        self.insert(&relationship)?;
        Ok(relationship)
    }

    /// Replaces a relationship's editable fields. Identity, origin and creation time are kept.
    pub fn update_relationship(
        &mut self,
        mut relationship: Relationship,
        now: Timestamp,
    ) -> Result<Relationship, Failure<R>> {
        let stored = self.require::<Relationship>("relationship", &relationship.id)?;
        self.validate_relationship(&relationship)?;
        relationship.created_at = stored.created_at;
        relationship.origin = stored.origin;
        relationship.updated_at = now;
        self.update(&relationship)?;
        Ok(relationship)
    }

    /// Removes a relationship together with the property values it owns and its citations.
    pub fn remove_relationship(&mut self, id: &RelationshipId) -> Result<(), Failure<R>> {
        self.require::<Relationship>("relationship", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Relationship, id.as_uuid()))
    }
}
