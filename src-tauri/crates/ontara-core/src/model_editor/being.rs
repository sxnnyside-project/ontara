use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::being::{
    BeingAppearance, BeingLanguage, BeingMark, BeingProfile, BeingTrait, DeityDomain, DeityProfile,
    FamilyTie, Partnership, SpeciesHabitat, SpeciesProfile,
};
use crate::canon_state::CanonState;
use crate::error::DomainError;
use crate::identifier::{
    BeingAppearanceId, BeingMarkId, BeingTraitId, DeityDomainId, EntityId, FamilyTieId,
    PartnershipId,
};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;
use crate::vocabulary::Vocabulary;

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_being_profile(&mut self, record: BeingProfile) -> Result<BeingProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("being"))?;
        if let Some(entity) = &record.species {
            self.require_entity_of(entity, Some("species"))?;
        }
        if let Some(entity) = &record.birthplace {
            self.require_entity_of(entity, Some("place"))?;
        }
        if self.find::<BeingProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_being_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<BeingProfile>("being profile", id)?;
        self.delete::<BeingProfile>(id)
    }

    pub fn add_being_appearance(
        &mut self,
        record: BeingAppearance,
    ) -> Result<BeingAppearance, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<BeingAppearance>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("being appearance", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_being_appearance(
        &mut self,
        mut record: BeingAppearance,
        now: Timestamp,
    ) -> Result<BeingAppearance, Failure<R>> {
        let stored = self.require::<BeingAppearance>("being appearance", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_being_appearance(&mut self, id: &BeingAppearanceId) -> Result<(), Failure<R>> {
        self.require::<BeingAppearance>("being appearance", id)?;
        self.remove_record(&RecordRef::new(RecordKind::BeingAppearance, id.as_uuid()))
    }

    pub fn add_being_mark(&mut self, record: BeingMark) -> Result<BeingMark, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<BeingMark>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("being mark", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_being_mark(
        &mut self,
        mut record: BeingMark,
        now: Timestamp,
    ) -> Result<BeingMark, Failure<R>> {
        let stored = self.require::<BeingMark>("being mark", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_being_mark(&mut self, id: &BeingMarkId) -> Result<(), Failure<R>> {
        self.require::<BeingMark>("being mark", id)?;
        self.remove_record(&RecordRef::new(RecordKind::BeingMark, id.as_uuid()))
    }

    pub fn add_being_trait(&mut self, record: BeingTrait) -> Result<BeingTrait, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_term(Vocabulary::TraitKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<BeingTrait>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("being trait", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_being_trait(
        &mut self,
        mut record: BeingTrait,
        now: Timestamp,
    ) -> Result<BeingTrait, Failure<R>> {
        let stored = self.require::<BeingTrait>("being trait", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_term(Vocabulary::TraitKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_being_trait(&mut self, id: &BeingTraitId) -> Result<(), Failure<R>> {
        self.require::<BeingTrait>("being trait", id)?;
        self.remove_record(&RecordRef::new(RecordKind::BeingTrait, id.as_uuid()))
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_being_language(
        &mut self,
        record: BeingLanguage,
    ) -> Result<BeingLanguage, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_entity_of(&record.language, Some("language"))?;
        if self
            .find::<BeingLanguage>(&(record.being, record.language))?
            .is_some()
        {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_being_language(&mut self, id: &(EntityId, EntityId)) -> Result<(), Failure<R>> {
        self.require_key::<BeingLanguage>("being language", id)?;
        self.delete::<BeingLanguage>(id)
    }

    pub fn add_family_tie(&mut self, record: FamilyTie) -> Result<FamilyTie, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.parent, Some("being"))?;
        self.require_entity_of(&record.child, Some("being"))?;
        self.require_term(Vocabulary::FamilyTieKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.family_tie_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("family_tie"),
            }
            .into());
        }
        if self.find::<FamilyTie>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("family tie", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_family_tie(
        &mut self,
        mut record: FamilyTie,
        now: Timestamp,
    ) -> Result<FamilyTie, Failure<R>> {
        let stored = self.require::<FamilyTie>("family tie", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.parent, Some("being"))?;
        self.require_entity_of(&record.child, Some("being"))?;
        self.require_term(Vocabulary::FamilyTieKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.family_tie_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("family_tie"),
            }
            .into());
        }
        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_family_tie(&mut self, id: &FamilyTieId) -> Result<(), Failure<R>> {
        self.require::<FamilyTie>("family tie", id)?;
        self.remove_record(&RecordRef::new(RecordKind::FamilyTie, id.as_uuid()))
    }

    /// Whether adding the record would let the family lead back to where it started. Only
    /// open-ended records count: one that has ended is history and cannot make a loop today.
    fn family_tie_would_loop(&self, candidate: &FamilyTie) -> Result<bool, Failure<R>> {
        let mut seen = BTreeSet::from([candidate.child]);
        let mut pending = vec![candidate.child];
        while let Some(current) = pending.pop() {
            if current == candidate.parent {
                return Ok(true);
            }
            for edge in self.query(|repository| repository.family_tie_by_parent(&current))? {
                if edge.id == candidate.id || !edge.period.is_open() {
                    continue;
                }
                if seen.insert(edge.child) {
                    pending.push(edge.child);
                }
            }
        }
        Ok(false)
    }

    pub fn add_partnership(&mut self, mut record: Partnership) -> Result<Partnership, Failure<R>> {
        record.normalize();
        record.validate()?;
        self.require_entity_of(&record.first, Some("being"))?;
        self.require_entity_of(&record.second, Some("being"))?;
        self.require_term(Vocabulary::PartnershipKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<Partnership>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("partnership", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_partnership(
        &mut self,
        mut record: Partnership,
        now: Timestamp,
    ) -> Result<Partnership, Failure<R>> {
        let stored = self.require::<Partnership>("partnership", &record.id)?;
        record.normalize();
        record.validate()?;
        self.require_entity_of(&record.first, Some("being"))?;
        self.require_entity_of(&record.second, Some("being"))?;
        self.require_term(Vocabulary::PartnershipKind, &record.kind)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_partnership(&mut self, id: &PartnershipId) -> Result<(), Failure<R>> {
        self.require::<Partnership>("partnership", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Partnership, id.as_uuid()))
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_deity_profile(&mut self, record: DeityProfile) -> Result<DeityProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("deity"))?;
        if self.find::<DeityProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_deity_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<DeityProfile>("deity profile", id)?;
        self.delete::<DeityProfile>(id)
    }

    pub fn add_deity_domain(&mut self, record: DeityDomain) -> Result<DeityDomain, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.deity, Some("deity"))?;

        if self.find::<DeityDomain>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("deity domain", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_deity_domain(&mut self, record: DeityDomain) -> Result<DeityDomain, Failure<R>> {
        let _stored = self.require::<DeityDomain>("deity domain", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.deity, Some("deity"))?;

        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_deity_domain(&mut self, id: &DeityDomainId) -> Result<(), Failure<R>> {
        self.require::<DeityDomain>("deity domain", id)?;
        self.remove_record(&RecordRef::new(RecordKind::DeityDomain, id.as_uuid()))
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_species_profile(
        &mut self,
        record: SpeciesProfile,
    ) -> Result<SpeciesProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("species"))?;
        if self.find::<SpeciesProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_species_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<SpeciesProfile>("species profile", id)?;
        self.delete::<SpeciesProfile>(id)
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_species_habitat(
        &mut self,
        record: SpeciesHabitat,
    ) -> Result<SpeciesHabitat, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.species, Some("species"))?;
        self.require_entity_of(&record.place, Some("place"))?;
        if self
            .find::<SpeciesHabitat>(&(record.species, record.place))?
            .is_some()
        {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_species_habitat(&mut self, id: &(EntityId, EntityId)) -> Result<(), Failure<R>> {
        self.require_key::<SpeciesHabitat>("species habitat", id)?;
        self.delete::<SpeciesHabitat>(id)
    }
}
