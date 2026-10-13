use super::{Failure, ModelEditor};
use crate::belief::{
    Adherence, CultureLanguage, CulturePractice, HolyPlace, LanguageProfile, ReligionDeity,
    ReligionProfile, ReligionTenet,
};
use crate::canon_state::CanonState;
use crate::error::DomainError;
use crate::identifier::{AdherenceId, CulturePracticeId, EntityId, ReligionTenetId};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;

impl<R: ModelRepository> ModelEditor<'_, R> {
    pub fn add_culture_practice(
        &mut self,
        record: CulturePractice,
    ) -> Result<CulturePractice, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.culture, Some("culture"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<CulturePractice>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("culture practice", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_culture_practice(
        &mut self,
        mut record: CulturePractice,
        now: Timestamp,
    ) -> Result<CulturePractice, Failure<R>> {
        let stored = self.require::<CulturePractice>("culture practice", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.culture, Some("culture"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_culture_practice(&mut self, id: &CulturePracticeId) -> Result<(), Failure<R>> {
        self.require::<CulturePractice>("culture practice", id)?;
        self.remove_record(&RecordRef::new(RecordKind::CulturePractice, id.as_uuid()))
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_culture_language(
        &mut self,
        record: CultureLanguage,
    ) -> Result<CultureLanguage, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.culture, Some("culture"))?;
        self.require_entity_of(&record.language, Some("language"))?;
        if self
            .find::<CultureLanguage>(&(record.culture, record.language))?
            .is_some()
        {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_culture_language(&mut self, id: &(EntityId, EntityId)) -> Result<(), Failure<R>> {
        self.require_key::<CultureLanguage>("culture language", id)?;
        self.delete::<CultureLanguage>(id)
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_language_profile(
        &mut self,
        record: LanguageProfile,
    ) -> Result<LanguageProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("language"))?;
        if let Some(entity) = &record.parent_language {
            self.require_entity_of(entity, Some("language"))?;
        }
        if self.find::<LanguageProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_language_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<LanguageProfile>("language profile", id)?;
        self.delete::<LanguageProfile>(id)
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_religion_profile(
        &mut self,
        record: ReligionProfile,
    ) -> Result<ReligionProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("religion"))?;
        if let Some(entity) = &record.founder {
            self.require_entity_of(entity, Some("being"))?;
        }
        if self.find::<ReligionProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_religion_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<ReligionProfile>("religion profile", id)?;
        self.delete::<ReligionProfile>(id)
    }

    pub fn add_religion_tenet(
        &mut self,
        record: ReligionTenet,
    ) -> Result<ReligionTenet, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.religion, Some("religion"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<ReligionTenet>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("religion tenet", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_religion_tenet(
        &mut self,
        mut record: ReligionTenet,
        now: Timestamp,
    ) -> Result<ReligionTenet, Failure<R>> {
        let stored = self.require::<ReligionTenet>("religion tenet", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.religion, Some("religion"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_religion_tenet(&mut self, id: &ReligionTenetId) -> Result<(), Failure<R>> {
        self.require::<ReligionTenet>("religion tenet", id)?;
        self.remove_record(&RecordRef::new(RecordKind::ReligionTenet, id.as_uuid()))
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_religion_deity(
        &mut self,
        record: ReligionDeity,
    ) -> Result<ReligionDeity, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.religion, Some("religion"))?;
        self.require_entity_of(&record.deity, Some("deity"))?;
        if self
            .find::<ReligionDeity>(&(record.religion, record.deity))?
            .is_some()
        {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_religion_deity(&mut self, id: &(EntityId, EntityId)) -> Result<(), Failure<R>> {
        self.require_key::<ReligionDeity>("religion deity", id)?;
        self.delete::<ReligionDeity>(id)
    }

    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_holy_place(&mut self, record: HolyPlace) -> Result<HolyPlace, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.religion, Some("religion"))?;
        self.require_entity_of(&record.place, Some("place"))?;
        if self
            .find::<HolyPlace>(&(record.religion, record.place))?
            .is_some()
        {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_holy_place(&mut self, id: &(EntityId, EntityId)) -> Result<(), Failure<R>> {
        self.require_key::<HolyPlace>("holy place", id)?;
        self.delete::<HolyPlace>(id)
    }

    pub fn add_adherence(&mut self, record: Adherence) -> Result<Adherence, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.adherent, None)?;
        self.require_entity_of(&record.religion, Some("religion"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<Adherence>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("adherence", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_adherence(
        &mut self,
        mut record: Adherence,
        now: Timestamp,
    ) -> Result<Adherence, Failure<R>> {
        let stored = self.require::<Adherence>("adherence", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.adherent, None)?;
        self.require_entity_of(&record.religion, Some("religion"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_adherence(&mut self, id: &AdherenceId) -> Result<(), Failure<R>> {
        self.require::<Adherence>("adherence", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Adherence, id.as_uuid()))
    }
}
