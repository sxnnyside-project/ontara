use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::error::DomainError;
use crate::faction::{FactionParent, FactionProfile, Membership};
use crate::identifier::{EntityId, FactionParentId, MembershipId};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_faction_profile(
        &mut self,
        record: FactionProfile,
    ) -> Result<FactionProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("faction"))?;
        if let Some(entity) = &record.headquarters {
            self.require_entity_of(entity, Some("place"))?;
        }
        if self.find::<FactionProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_faction_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<FactionProfile>("faction profile", id)?;
        self.delete::<FactionProfile>(id)
    }

    pub fn add_faction_parent(
        &mut self,
        record: FactionParent,
    ) -> Result<FactionParent, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.child, Some("faction"))?;
        self.require_entity_of(&record.parent, Some("faction"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.faction_parent_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("faction_parent"),
            }
            .into());
        }
        if self.find::<FactionParent>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("faction parent", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_faction_parent(
        &mut self,
        mut record: FactionParent,
        now: Timestamp,
    ) -> Result<FactionParent, Failure<R>> {
        let stored = self.require::<FactionParent>("faction parent", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.child, Some("faction"))?;
        self.require_entity_of(&record.parent, Some("faction"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.faction_parent_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("faction_parent"),
            }
            .into());
        }
        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_faction_parent(&mut self, id: &FactionParentId) -> Result<(), Failure<R>> {
        self.require::<FactionParent>("faction parent", id)?;
        self.remove_record(&RecordRef::new(RecordKind::FactionParent, id.as_uuid()))
    }

    /// Whether adding the record would let the organization lead back to where it started. Only
    /// open-ended records count: one that has ended is history and cannot make a loop today.
    fn faction_parent_would_loop(&self, candidate: &FactionParent) -> Result<bool, Failure<R>> {
        let mut seen = BTreeSet::from([candidate.parent]);
        let mut pending = vec![candidate.parent];
        while let Some(current) = pending.pop() {
            if current == candidate.child {
                return Ok(true);
            }
            for edge in self.query(|repository| repository.faction_parent_by_child(&current))? {
                if edge.id == candidate.id || !edge.period.is_open() {
                    continue;
                }
                if seen.insert(edge.parent) {
                    pending.push(edge.parent);
                }
            }
        }
        Ok(false)
    }

    pub fn add_membership(&mut self, record: Membership) -> Result<Membership, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.faction, Some("faction"))?;
        self.require_entity_of(&record.member, None)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<Membership>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("membership", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_membership(
        &mut self,
        mut record: Membership,
        now: Timestamp,
    ) -> Result<Membership, Failure<R>> {
        let stored = self.require::<Membership>("membership", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.faction, Some("faction"))?;
        self.require_entity_of(&record.member, None)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_membership(&mut self, id: &MembershipId) -> Result<(), Failure<R>> {
        self.require::<Membership>("membership", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Membership, id.as_uuid()))
    }
}
