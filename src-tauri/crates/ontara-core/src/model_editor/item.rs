use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::error::DomainError;
use crate::identifier::{EntityId, ItemOwnershipId, ItemPowerId};
use crate::item::{ItemOwnership, ItemPower, ItemProfile};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;
use crate::vocabulary::Vocabulary;

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Creates the record, or replaces it if one already exists for the same key.
    pub fn set_item_profile(&mut self, record: ItemProfile) -> Result<ItemProfile, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.entity, Some("item"))?;
        if let Some(entity) = &record.creator {
            self.require_entity_of(entity, Some("being"))?;
        }
        if self.find::<ItemProfile>(&record.entity)?.is_some() {
            self.update(&record)?;
        } else {
            self.insert(&record)?;
        }
        Ok(record)
    }

    pub fn remove_item_profile(&mut self, id: &EntityId) -> Result<(), Failure<R>> {
        self.require_key::<ItemProfile>("item profile", id)?;
        self.delete::<ItemProfile>(id)
    }

    pub fn add_item_power(&mut self, record: ItemPower) -> Result<ItemPower, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.item, Some("item"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<ItemPower>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("item power", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_item_power(
        &mut self,
        mut record: ItemPower,
        now: Timestamp,
    ) -> Result<ItemPower, Failure<R>> {
        let stored = self.require::<ItemPower>("item power", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.item, Some("item"))?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_item_power(&mut self, id: &ItemPowerId) -> Result<(), Failure<R>> {
        self.require::<ItemPower>("item power", id)?;
        self.remove_record(&RecordRef::new(RecordKind::ItemPower, id.as_uuid()))
    }

    pub fn add_item_ownership(
        &mut self,
        record: ItemOwnership,
    ) -> Result<ItemOwnership, Failure<R>> {
        record.validate()?;
        self.require_entity_of(&record.item, Some("item"))?;
        self.require_entity_of(&record.owner, None)?;
        if let Some(term) = &record.acquisition {
            self.require_term(Vocabulary::ItemAcquisition, term)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        if self.find::<ItemOwnership>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("item ownership", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_item_ownership(
        &mut self,
        mut record: ItemOwnership,
        now: Timestamp,
    ) -> Result<ItemOwnership, Failure<R>> {
        let stored = self.require::<ItemOwnership>("item ownership", &record.id)?;
        record.validate()?;
        self.require_entity_of(&record.item, Some("item"))?;
        self.require_entity_of(&record.owner, None)?;
        if let Some(term) = &record.acquisition {
            self.require_term(Vocabulary::ItemAcquisition, term)?;
        }
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;

        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_item_ownership(&mut self, id: &ItemOwnershipId) -> Result<(), Failure<R>> {
        self.require::<ItemOwnership>("item ownership", id)?;
        self.remove_record(&RecordRef::new(RecordKind::ItemOwnership, id.as_uuid()))
    }
}
