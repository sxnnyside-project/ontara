use super::{Failure, ModelEditor};
use crate::error::DomainError;
use crate::identifier::{
    AdherenceId, AppearanceId, BeingAppearanceId, BeingMarkId, BeingTraitId, CensusId,
    CitizenshipId, CulturePracticeId, DeityDomainId, DiplomaticRelationId, EntityId,
    EventCausalityId, FactionParentId, FamilyTieId, ItemOwnershipId, ItemPowerId, MembershipId,
    OfficeId, OfficeTenureId, PartnershipId, PlaceBorderId, PlaceContainmentId, PlaceResourceId,
    PolitySuccessionId, PolitySymbolId, PropertyValueId, RelationshipId, ReligionTenetId,
};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;
use crate::{
    Adherence, Appearance, BeingAppearance, BeingMark, BeingTrait, Census, Citizenship,
    CulturePractice, DeityDomain, DiplomaticRelation, Entity, EventCausality, FactionParent,
    FamilyTie, ItemOwnership, ItemPower, Membership, Office, OfficeTenure, Partnership,
    PlaceBorder, PlaceContainment, PlaceResource, PolitySuccession, PolitySymbol, PropertyValue,
    Relationship, ReligionTenet,
};

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Moves a record of any kind to another canon state and returns the state it had.
    #[allow(clippy::too_many_lines)] // One arm per kind of record; the arms are uniform.
    pub(super) fn swap_canon_state(
        &mut self,
        target: &RecordRef,
        to: &TypeKey,
        now: Timestamp,
    ) -> Result<TypeKey, Failure<R>> {
        Ok(match target.kind {
            RecordKind::Entity => {
                let id = EntityId::from_uuid(target.id);
                let mut record = self.require::<Entity>("entity", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Relationship => {
                let id = RelationshipId::from_uuid(target.id);
                let mut record = self.require::<Relationship>("relationship", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::PropertyValue => {
                let id = PropertyValueId::from_uuid(target.id);
                let mut record = self.require::<PropertyValue>("property value", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Appearance => {
                let id = AppearanceId::from_uuid(target.id);
                let mut record = self.require::<Appearance>("appearance", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::BeingAppearance => {
                let id = BeingAppearanceId::from_uuid(target.id);
                let mut record = self.require::<BeingAppearance>("being appearance", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::BeingMark => {
                let id = BeingMarkId::from_uuid(target.id);
                let mut record = self.require::<BeingMark>("being mark", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::BeingTrait => {
                let id = BeingTraitId::from_uuid(target.id);
                let mut record = self.require::<BeingTrait>("being trait", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::FamilyTie => {
                let id = FamilyTieId::from_uuid(target.id);
                let mut record = self.require::<FamilyTie>("family tie", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Partnership => {
                let id = PartnershipId::from_uuid(target.id);
                let mut record = self.require::<Partnership>("partnership", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::DeityDomain => {
                return Err(
                    DomainError::Invalid("a deity domain has no canon state".to_owned()).into(),
                );
            }
            RecordKind::PlaceContainment => {
                let id = PlaceContainmentId::from_uuid(target.id);
                let mut record = self.require::<PlaceContainment>("place containment", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::PlaceBorder => {
                let id = PlaceBorderId::from_uuid(target.id);
                let mut record = self.require::<PlaceBorder>("place border", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::PlaceResource => {
                return Err(
                    DomainError::Invalid("a place resource has no canon state".to_owned()).into(),
                );
            }
            RecordKind::Census => {
                let id = CensusId::from_uuid(target.id);
                let mut record = self.require::<Census>("census", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::PolitySymbol => {
                return Err(
                    DomainError::Invalid("a polity symbol has no canon state".to_owned()).into(),
                );
            }
            RecordKind::PolitySuccession => {
                let id = PolitySuccessionId::from_uuid(target.id);
                let mut record = self.require::<PolitySuccession>("polity succession", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Office => {
                return Err(DomainError::Invalid("a office has no canon state".to_owned()).into());
            }
            RecordKind::OfficeTenure => {
                let id = OfficeTenureId::from_uuid(target.id);
                let mut record = self.require::<OfficeTenure>("office tenure", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Citizenship => {
                let id = CitizenshipId::from_uuid(target.id);
                let mut record = self.require::<Citizenship>("citizenship", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::DiplomaticRelation => {
                let id = DiplomaticRelationId::from_uuid(target.id);
                let mut record = self.require::<DiplomaticRelation>("diplomatic relation", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::FactionParent => {
                let id = FactionParentId::from_uuid(target.id);
                let mut record = self.require::<FactionParent>("faction parent", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Membership => {
                let id = MembershipId::from_uuid(target.id);
                let mut record = self.require::<Membership>("membership", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::CulturePractice => {
                let id = CulturePracticeId::from_uuid(target.id);
                let mut record = self.require::<CulturePractice>("culture practice", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::ReligionTenet => {
                let id = ReligionTenetId::from_uuid(target.id);
                let mut record = self.require::<ReligionTenet>("religion tenet", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::Adherence => {
                let id = AdherenceId::from_uuid(target.id);
                let mut record = self.require::<Adherence>("adherence", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::ItemPower => {
                let id = ItemPowerId::from_uuid(target.id);
                let mut record = self.require::<ItemPower>("item power", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::ItemOwnership => {
                let id = ItemOwnershipId::from_uuid(target.id);
                let mut record = self.require::<ItemOwnership>("item ownership", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
            RecordKind::EventCausality => {
                let id = EventCausalityId::from_uuid(target.id);
                let mut record = self.require::<EventCausality>("event causality", &id)?;
                let from = std::mem::replace(&mut record.canon_state, to.clone());
                record.updated_at = now;
                if &from != to {
                    self.update(&record)?;
                }
                from
            }
        })
    }

    /// Requires that the record a citation points at exists.
    #[allow(clippy::too_many_lines)] // One arm per kind of record; the arms are uniform.
    pub(super) fn require_record(&self, target: &RecordRef) -> Result<(), Failure<R>> {
        match target.kind {
            RecordKind::Entity => {
                self.require::<Entity>("entity", &EntityId::from_uuid(target.id))?;
            }
            RecordKind::Relationship => {
                self.require::<Relationship>(
                    "relationship",
                    &RelationshipId::from_uuid(target.id),
                )?;
            }
            RecordKind::PropertyValue => {
                self.require::<PropertyValue>(
                    "property value",
                    &PropertyValueId::from_uuid(target.id),
                )?;
            }
            RecordKind::Appearance => {
                self.require::<Appearance>("appearance", &AppearanceId::from_uuid(target.id))?;
            }
            RecordKind::BeingAppearance => {
                self.require::<BeingAppearance>(
                    "being appearance",
                    &BeingAppearanceId::from_uuid(target.id),
                )?;
            }
            RecordKind::BeingMark => {
                self.require::<BeingMark>("being mark", &BeingMarkId::from_uuid(target.id))?;
            }
            RecordKind::BeingTrait => {
                self.require::<BeingTrait>("being trait", &BeingTraitId::from_uuid(target.id))?;
            }
            RecordKind::FamilyTie => {
                self.require::<FamilyTie>("family tie", &FamilyTieId::from_uuid(target.id))?;
            }
            RecordKind::Partnership => {
                self.require::<Partnership>("partnership", &PartnershipId::from_uuid(target.id))?;
            }
            RecordKind::DeityDomain => {
                self.require::<DeityDomain>("deity domain", &DeityDomainId::from_uuid(target.id))?;
            }
            RecordKind::PlaceContainment => {
                self.require::<PlaceContainment>(
                    "place containment",
                    &PlaceContainmentId::from_uuid(target.id),
                )?;
            }
            RecordKind::PlaceBorder => {
                self.require::<PlaceBorder>("place border", &PlaceBorderId::from_uuid(target.id))?;
            }
            RecordKind::PlaceResource => {
                self.require::<PlaceResource>(
                    "place resource",
                    &PlaceResourceId::from_uuid(target.id),
                )?;
            }
            RecordKind::Census => {
                self.require::<Census>("census", &CensusId::from_uuid(target.id))?;
            }
            RecordKind::PolitySymbol => {
                self.require::<PolitySymbol>(
                    "polity symbol",
                    &PolitySymbolId::from_uuid(target.id),
                )?;
            }
            RecordKind::PolitySuccession => {
                self.require::<PolitySuccession>(
                    "polity succession",
                    &PolitySuccessionId::from_uuid(target.id),
                )?;
            }
            RecordKind::Office => {
                self.require::<Office>("office", &OfficeId::from_uuid(target.id))?;
            }
            RecordKind::OfficeTenure => {
                self.require::<OfficeTenure>(
                    "office tenure",
                    &OfficeTenureId::from_uuid(target.id),
                )?;
            }
            RecordKind::Citizenship => {
                self.require::<Citizenship>("citizenship", &CitizenshipId::from_uuid(target.id))?;
            }
            RecordKind::DiplomaticRelation => {
                self.require::<DiplomaticRelation>(
                    "diplomatic relation",
                    &DiplomaticRelationId::from_uuid(target.id),
                )?;
            }
            RecordKind::FactionParent => {
                self.require::<FactionParent>(
                    "faction parent",
                    &FactionParentId::from_uuid(target.id),
                )?;
            }
            RecordKind::Membership => {
                self.require::<Membership>("membership", &MembershipId::from_uuid(target.id))?;
            }
            RecordKind::CulturePractice => {
                self.require::<CulturePractice>(
                    "culture practice",
                    &CulturePracticeId::from_uuid(target.id),
                )?;
            }
            RecordKind::ReligionTenet => {
                self.require::<ReligionTenet>(
                    "religion tenet",
                    &ReligionTenetId::from_uuid(target.id),
                )?;
            }
            RecordKind::Adherence => {
                self.require::<Adherence>("adherence", &AdherenceId::from_uuid(target.id))?;
            }
            RecordKind::ItemPower => {
                self.require::<ItemPower>("item power", &ItemPowerId::from_uuid(target.id))?;
            }
            RecordKind::ItemOwnership => {
                self.require::<ItemOwnership>(
                    "item ownership",
                    &ItemOwnershipId::from_uuid(target.id),
                )?;
            }
            RecordKind::EventCausality => {
                self.require::<EventCausality>(
                    "event causality",
                    &EventCausalityId::from_uuid(target.id),
                )?;
            }
        }
        Ok(())
    }
}
