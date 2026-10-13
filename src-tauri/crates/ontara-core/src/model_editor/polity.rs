use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::diplomatic_stance::DiplomaticStance;
use crate::error::DomainError;
use crate::identifier::{
    CitizenshipId, DiplomaticRelationId, EntityId, OfficeId, OfficeTenureId, PolitySuccessionId,
    PolitySymbolId,
};
use crate::polity::{
    Citizenship, DiplomaticRelation, Office, OfficeTenure, PolityLanguage, PolityProfile,
    PolitySuccession, PolitySymbol,
};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;
use crate::vocabulary::Vocabulary;

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_polity_profile(
        &mut self,
        record: PolityProfile,
    ) -> Result<PolityProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("polity"))?;
        if let Some(entity) = &record.capital {
            self.require_entity_of(entity, Some("settlement"))?;
        }
        if let Some(entity) = &record.state_religion {
            self.require_entity_of(entity, Some("religion"))?;
        }
        if self.find::<PolityProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_polity_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<PolityProfile>("polity profile", id)?;
        self.delete::<PolityProfile>(id)
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_polity_language(
        &mut self,
        record: PolityLanguage,
    ) -> Result<PolityLanguage, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.polity, Some("polity"))?;
        self.require_entity_of(&record.language, Some("language"))?;
        if self
            .find::<PolityLanguage>(&(record.polity, record.language))?
            .is_some()
        {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_polity_language(&mut self, id: &(EntityId, EntityId)) -> Result<(), Failure<R>> {
        self.require_key::<PolityLanguage>("polity language", id)?;
        self.delete::<PolityLanguage>(id)
    }

    pub fn add_polity_symbol(&mut self, record: PolitySymbol) -> Result<PolitySymbol, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.polity, Some("polity"))?;

        if self.find::<PolitySymbol>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("polity symbol", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_polity_symbol(
        &mut self,
        record: PolitySymbol,
    ) -> Result<PolitySymbol, Failure<R>> {
        let _stored = self.require::<PolitySymbol>("polity symbol", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.polity, Some("polity"))?;

        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_polity_symbol(&mut self, id: &PolitySymbolId) -> Result<(), Failure<R>> {
        self.require::<PolitySymbol>("polity symbol", id)?;
        self.remove_record(&RecordRef::new(RecordKind::PolitySymbol, id.as_uuid()))
    }

    pub fn add_polity_succession(
        &mut self,
        record: PolitySuccession,
    ) -> Result<PolitySuccession, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.predecessor, Some("polity"))?;
        self.require_entity_of(&record.successor, Some("polity"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.polity_succession_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("polity_succession"),
            }
            .into());
        }
        if self.find::<PolitySuccession>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("polity succession", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_polity_succession(
        &mut self,
        mut record: PolitySuccession,
        now: Timestamp,
    ) -> Result<PolitySuccession, Failure<R>> {
        let stored = self.require::<PolitySuccession>("polity succession", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.predecessor, Some("polity"))?;
        self.require_entity_of(&record.successor, Some("polity"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.polity_succession_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("polity_succession"),
            }
            .into());
        }
        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_polity_succession(&mut self, id: &PolitySuccessionId) -> Result<(), Failure<R>> {
        self.require::<PolitySuccession>("polity succession", id)?;
        self.remove_record(&RecordRef::new(RecordKind::PolitySuccession, id.as_uuid()))
    }

    /// Whether adding the record would let the succession lead back to where it started. Only
    /// open-ended records count: one that has ended is history and cannot make a loop today.
    fn polity_succession_would_loop(
        &self,
        candidate: &PolitySuccession,
    ) -> Result<bool, Failure<R>> {
        let mut seen = BTreeSet::from([candidate.successor]);
        let mut pending = vec![candidate.successor];
        while let Some(current) = pending.pop() {
            if current == candidate.predecessor {
                return Ok(true);
            }
            for edge in
                self.query(|repository| repository.polity_succession_by_predecessor(&current))?
            {
                if edge.id == candidate.id {
                    continue;
                }
                if seen.insert(edge.successor) {
                    pending.push(edge.successor);
                }
            }
        }
        Ok(false)
    }

    pub fn add_office(&mut self, record: Office) -> Result<Office, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.body, None)?;

        if self.find::<Office>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("office", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_office(
        &mut self,
        mut record: Office,
        now: Timestamp,
    ) -> Result<Office, Failure<R>> {
        let stored = self.require::<Office>("office", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.body, None)?;

        record.created_at = stored.created_at;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_office(&mut self, id: &OfficeId) -> Result<(), Failure<R>> {
        self.require::<Office>("office", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Office, id.as_uuid()))
    }

    pub fn add_office_tenure(&mut self, record: OfficeTenure) -> Result<OfficeTenure, Failure<R>> {
        record.validate()?;
        self.require::<Office>("office", &record.office)?;
        self.require_entity_of(&record.holder, Some("being"))?;
        if let Some(term) = &record.acquisition {
            self.require_term(Vocabulary::OfficeAcquisition, term)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<OfficeTenure>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("office tenure", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_office_tenure(
        &mut self,
        mut record: OfficeTenure,
        now: Timestamp,
    ) -> Result<OfficeTenure, Failure<R>> {
        let stored = self.require::<OfficeTenure>("office tenure", &record.id)?;
        record.validate()?;
        self.require::<Office>("office", &record.office)?;
        self.require_entity_of(&record.holder, Some("being"))?;
        if let Some(term) = &record.acquisition {
            self.require_term(Vocabulary::OfficeAcquisition, term)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_office_tenure(&mut self, id: &OfficeTenureId) -> Result<(), Failure<R>> {
        self.require::<OfficeTenure>("office tenure", id)?;
        self.remove_record(&RecordRef::new(RecordKind::OfficeTenure, id.as_uuid()))
    }

    pub fn add_citizenship(&mut self, record: Citizenship) -> Result<Citizenship, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_entity_of(&record.polity, Some("polity"))?;
        self.require_term(Vocabulary::CitizenshipStatus, &record.status)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<Citizenship>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("citizenship", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_citizenship(
        &mut self,
        mut record: Citizenship,
        now: Timestamp,
    ) -> Result<Citizenship, Failure<R>> {
        let stored = self.require::<Citizenship>("citizenship", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.being, Some("being"))?;
        self.require_entity_of(&record.polity, Some("polity"))?;
        self.require_term(Vocabulary::CitizenshipStatus, &record.status)?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_citizenship(&mut self, id: &CitizenshipId) -> Result<(), Failure<R>> {
        self.require::<Citizenship>("citizenship", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Citizenship, id.as_uuid()))
    }

    pub fn add_diplomatic_relation(
        &mut self,
        mut record: DiplomaticRelation,
    ) -> Result<DiplomaticRelation, Failure<R>> {
        record.normalize();
        record.validate()?;
        self.require_entity_of(&record.first, None)?;
        self.require_entity_of(&record.second, None)?;
        self.require_defined::<DiplomaticStance>("diplomatic stance", &record.stance)?;
        if let Some(entity) = &record.senior {
            self.require_entity_of(entity, None)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<DiplomaticRelation>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("diplomatic relation", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_diplomatic_relation(
        &mut self,
        mut record: DiplomaticRelation,
        now: Timestamp,
    ) -> Result<DiplomaticRelation, Failure<R>> {
        let stored = self.require::<DiplomaticRelation>("diplomatic relation", &record.id)?;
        record.normalize();
        record.validate()?;
        self.require_entity_of(&record.first, None)?;
        self.require_entity_of(&record.second, None)?;
        self.require_defined::<DiplomaticStance>("diplomatic stance", &record.stance)?;
        if let Some(entity) = &record.senior {
            self.require_entity_of(entity, None)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_diplomatic_relation(
        &mut self,
        id: &DiplomaticRelationId,
    ) -> Result<(), Failure<R>> {
        self.require::<DiplomaticRelation>("diplomatic relation", id)?;
        self.remove_record(&RecordRef::new(
            RecordKind::DiplomaticRelation,
            id.as_uuid(),
        ))
    }
}
