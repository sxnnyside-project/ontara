use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::error::DomainError;
use crate::identifier::{CensusId, EntityId, PlaceBorderId, PlaceContainmentId, PlaceResourceId};
use crate::place::{Census, PlaceBorder, PlaceContainment, PlaceProfile, PlaceResource};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::source::Source;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;
use crate::vocabulary::Vocabulary;

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_place_profile(&mut self, record: PlaceProfile) -> Result<PlaceProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("place"))?;
        if self.find::<PlaceProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_place_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<PlaceProfile>("place profile", id)?;
        self.delete::<PlaceProfile>(id)
    }

    pub fn add_place_containment(
        &mut self,
        record: PlaceContainment,
    ) -> Result<PlaceContainment, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.child, Some("place"))?;
        self.require_entity_of(&record.parent, Some("place"))?;
        self.require_term(Vocabulary::ContainmentKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.place_containment_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("place_containment"),
            }
            .into());
        }
        if self.find::<PlaceContainment>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("place containment", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_place_containment(
        &mut self,
        mut record: PlaceContainment,
        now: Timestamp,
    ) -> Result<PlaceContainment, Failure<R>> {
        let stored = self.require::<PlaceContainment>("place containment", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.child, Some("place"))?;
        self.require_entity_of(&record.parent, Some("place"))?;
        self.require_term(Vocabulary::ContainmentKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.place_containment_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("place_containment"),
            }
            .into());
        }
        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_place_containment(&mut self, id: &PlaceContainmentId) -> Result<(), Failure<R>> {
        self.require::<PlaceContainment>("place containment", id)?;
        self.remove_record(&RecordRef::new(RecordKind::PlaceContainment, id.as_uuid()))
    }

    /// Whether adding the record would let the containment lead back to where it started. Only
    /// open-ended records count: one that has ended is history and cannot make a loop today.
    fn place_containment_would_loop(
        &self,
        candidate: &PlaceContainment,
    ) -> Result<bool, Failure<R>> {
        let mut seen = BTreeSet::from([candidate.parent]);
        let mut pending = vec![candidate.parent];
        while let Some(current) = pending.pop() {
            if current == candidate.child {
                return Ok(true);
            }
            for edge in self.query(|repository| repository.place_containment_by_child(&current))? {
                if edge.id == candidate.id || !edge.period.is_open() {
                    continue;
                }
                if edge.kind != candidate.kind {
                    continue;
                }
                if seen.insert(edge.parent) {
                    pending.push(edge.parent);
                }
            }
        }
        Ok(false)
    }

    pub fn add_place_border(&mut self, mut record: PlaceBorder) -> Result<PlaceBorder, Failure<R>> {
        record.normalize();
        record.validate()?;
        self.require_entity_of(&record.first, Some("place"))?;
        self.require_entity_of(&record.second, Some("place"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<PlaceBorder>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("place border", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_place_border(
        &mut self,
        mut record: PlaceBorder,
        now: Timestamp,
    ) -> Result<PlaceBorder, Failure<R>> {
        let stored = self.require::<PlaceBorder>("place border", &record.id)?;
        record.normalize();
        record.validate()?;
        self.require_entity_of(&record.first, Some("place"))?;
        self.require_entity_of(&record.second, Some("place"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_place_border(&mut self, id: &PlaceBorderId) -> Result<(), Failure<R>> {
        self.require::<PlaceBorder>("place border", id)?;
        self.remove_record(&RecordRef::new(RecordKind::PlaceBorder, id.as_uuid()))
    }

    pub fn add_place_resource(
        &mut self,
        record: PlaceResource,
    ) -> Result<PlaceResource, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.place, Some("place"))?;

        if self.find::<PlaceResource>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("place resource", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_place_resource(
        &mut self,
        record: PlaceResource,
    ) -> Result<PlaceResource, Failure<R>> {
        let _stored = self.require::<PlaceResource>("place resource", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.place, Some("place"))?;

        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_place_resource(&mut self, id: &PlaceResourceId) -> Result<(), Failure<R>> {
        self.require::<PlaceResource>("place resource", id)?;
        self.remove_record(&RecordRef::new(RecordKind::PlaceResource, id.as_uuid()))
    }

    pub fn add_census(&mut self, record: Census) -> Result<Census, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.place, Some("place"))?;
        if let Some(source) = &record.source {
            self.require::<Source>("source", source)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<Census>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("census", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_census(
        &mut self,
        mut record: Census,
        now: Timestamp,
    ) -> Result<Census, Failure<R>> {
        let stored = self.require::<Census>("census", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.place, Some("place"))?;
        if let Some(source) = &record.source {
            self.require::<Source>("source", source)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_census(&mut self, id: &CensusId) -> Result<(), Failure<R>> {
        self.require::<Census>("census", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Census, id.as_uuid()))
    }
}
