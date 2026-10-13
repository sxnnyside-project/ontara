use serde::Serialize;

use crate::appearance::Appearance;
use crate::appearance_kind::AppearanceKind;
use crate::asset::{AssetFile, Attachment};
use crate::being::{
    BeingAppearance, BeingLanguage, BeingMark, BeingProfile, BeingTrait, DeityDomain, DeityProfile,
    FamilyTie, Partnership, SpeciesHabitat, SpeciesProfile,
};
use crate::belief::{
    Adherence, CultureLanguage, CulturePractice, HolyPlace, LanguageProfile, ReligionDeity,
    ReligionProfile, ReligionTenet,
};
use crate::calendar::Calendar;
use crate::canon_state::CanonState;
use crate::canon_state_change::CanonStateChange;
use crate::catalog_entry::CatalogEntry;
use crate::diplomatic_stance::DiplomaticStance;
use crate::editorial_level::EditorialLevel;
use crate::editorial_node::EditorialNode;
use crate::entity::Entity;
use crate::entity_type::EntityType;
use crate::event::EventCausality;
use crate::event_detail::EventDetail;
use crate::faction::{FactionParent, FactionProfile, Membership};
use crate::hierarchy::{self, HierarchyError};
use crate::identifier::{EntityId, RelationshipId};
use crate::item::{ItemOwnership, ItemPower, ItemProfile};
use crate::note::Note;
use crate::place::{Census, PlaceBorder, PlaceContainment, PlaceProfile, PlaceResource};
use crate::polity::{
    Citizenship, DiplomaticRelation, Office, OfficeTenure, PolityLanguage, PolityProfile,
    PolitySuccession, PolitySymbol,
};
use crate::property_definition::{PropertyDefinition, PropertyGroup, PropertyOwner};
use crate::property_value::{PropertyValue, PropertyValueOwner};
use crate::record::Record;
use crate::record_ref::RecordRef;
use crate::relationship::Relationship;
use crate::relationship_type::RelationshipType;
use crate::source::{Source, SourceReference};
use crate::type_key::TypeKey;
use crate::universe_profile::UniverseProfile;
use crate::vocabulary::VocabularyTerm;

/// A storage back end for the domain. Implemented by persistence adapters.
pub trait Repository {
    type Error: std::error::Error + 'static;
}

/// Read access to the records of one type.
pub trait Reader<R: Record>: Repository {
    fn get(&self, id: &R::Id) -> Result<Option<R>, Self::Error>;
    fn list(&self) -> Result<Vec<R>, Self::Error>;
}

/// Raw write access to the records of one type.
///
/// Callers must go through [`ModelEditor`](crate::model_editor::ModelEditor), which enforces the
/// domain rules; these operations only store what they are given.
pub trait Writer<R: Record>: Reader<R> {
    fn insert(&mut self, record: &R) -> Result<(), Self::Error>;
    fn update(&mut self, record: &R) -> Result<(), Self::Error>;
    fn delete(&mut self, id: &R::Id) -> Result<(), Self::Error>;
}

/// How many records of one kind point at an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceCount {
    /// The table the referencing records are kept in.
    pub kind: String,
    pub count: usize,
}

/// Removal of a record together with everything that cannot exist without it.
pub trait RecordRemoval: Repository {
    /// Removes the record, the records that belong to it, the records that connect it to others,
    /// and the citations of all of them. Optional references to it are cleared.
    fn remove_record(&mut self, target: &RecordRef) -> Result<(), Self::Error>;

    /// Removes a catalog entry together with the records that belong to it: the property values
    /// of a property definition. Callers check first that nothing else uses the entry.
    fn remove_catalog_entry(&mut self, entry: &CatalogEntry) -> Result<(), Self::Error>;
}

/// Questions the rules need to ask that are not "get one" or "list all".
pub trait ModelQueries: Repository {
    /// Relationships where the entity is the source or the target.
    fn relationships_of(&self, entity: &EntityId) -> Result<Vec<Relationship>, Self::Error>;
    /// Relationships whose narrative context is the entity.
    fn relationships_in_context(&self, entity: &EntityId)
    -> Result<Vec<Relationship>, Self::Error>;
    fn property_definitions_of(
        &self,
        owner: &PropertyOwner,
    ) -> Result<Vec<PropertyDefinition>, Self::Error>;
    fn property_values_of(
        &self,
        owner: &PropertyValueOwner,
    ) -> Result<Vec<PropertyValue>, Self::Error>;
    /// Values, of any owner, whose content points at the entity.
    fn property_values_referencing(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PropertyValue>, Self::Error>;
    /// Appearances of the entity itself.
    fn appearances_of(&self, entity: &EntityId) -> Result<Vec<Appearance>, Self::Error>;
    /// Appearances that take place in the entity (a chapter, a scene or an event).
    fn appearances_in(&self, context: &EntityId) -> Result<Vec<Appearance>, Self::Error>;
    fn appearances_with_viewpoint(&self, entity: &EntityId)
    -> Result<Vec<Appearance>, Self::Error>;
    /// Events where the entity is a participant or the location.
    fn events_involving(&self, entity: &EntityId) -> Result<Vec<EventDetail>, Self::Error>;
    /// Children of a node, or the roots when `parent` is `None`, in order.
    fn editorial_children(
        &self,
        parent: Option<&EntityId>,
    ) -> Result<Vec<EditorialNode>, Self::Error>;
    /// Attachments where the entity is the owner or the asset.
    fn attachments_of(&self, entity: &EntityId) -> Result<Vec<Attachment>, Self::Error>;
    fn notes_of(&self, entity: &EntityId) -> Result<Vec<Note>, Self::Error>;
    fn citations_of(&self, target: &RecordRef) -> Result<Vec<SourceReference>, Self::Error>;
    fn citations_from(&self, source: &Source) -> Result<Vec<SourceReference>, Self::Error>;
    fn canon_history(&self, target: &RecordRef) -> Result<Vec<CanonStateChange>, Self::Error>;
    /// Records, of any kind, that still point at the entity, counted by kind. What belongs to
    /// the entity itself (its names, its traits, its own profile) is not a reference.
    fn references_to(&self, entity: &EntityId) -> Result<Vec<ReferenceCount>, Self::Error>;
    /// What still uses a catalog entry, counted by the kind of record. What the entry owns (a
    /// relationship type's endpoint rules, an entity type's own property definitions) is not
    /// a use.
    fn catalog_usage(&self, entry: &CatalogEntry) -> Result<Vec<ReferenceCount>, Self::Error>;
    fn being_appearance_by_being(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<BeingAppearance>, Self::Error>;
    fn being_mark_by_being(&self, entity: &EntityId) -> Result<Vec<BeingMark>, Self::Error>;
    fn being_trait_by_being(&self, entity: &EntityId) -> Result<Vec<BeingTrait>, Self::Error>;
    fn family_tie_by_parent(&self, entity: &EntityId) -> Result<Vec<FamilyTie>, Self::Error>;
    fn family_tie_by_child(&self, entity: &EntityId) -> Result<Vec<FamilyTie>, Self::Error>;
    fn partnership_by_first(&self, entity: &EntityId) -> Result<Vec<Partnership>, Self::Error>;
    fn deity_domain_by_deity(&self, entity: &EntityId) -> Result<Vec<DeityDomain>, Self::Error>;
    fn place_containment_by_child(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PlaceContainment>, Self::Error>;
    fn place_containment_by_parent(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PlaceContainment>, Self::Error>;
    fn place_border_by_first(&self, entity: &EntityId) -> Result<Vec<PlaceBorder>, Self::Error>;
    fn place_resource_by_place(&self, entity: &EntityId)
    -> Result<Vec<PlaceResource>, Self::Error>;
    fn census_by_place(&self, entity: &EntityId) -> Result<Vec<Census>, Self::Error>;
    fn polity_symbol_by_polity(&self, entity: &EntityId) -> Result<Vec<PolitySymbol>, Self::Error>;
    fn polity_succession_by_predecessor(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PolitySuccession>, Self::Error>;
    fn polity_succession_by_successor(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PolitySuccession>, Self::Error>;
    fn office_by_body(&self, entity: &EntityId) -> Result<Vec<Office>, Self::Error>;
    fn office_tenure_by_holder(&self, entity: &EntityId) -> Result<Vec<OfficeTenure>, Self::Error>;
    fn citizenship_by_being(&self, entity: &EntityId) -> Result<Vec<Citizenship>, Self::Error>;
    fn diplomatic_relation_by_first(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<DiplomaticRelation>, Self::Error>;
    fn faction_parent_by_child(&self, entity: &EntityId)
    -> Result<Vec<FactionParent>, Self::Error>;
    fn faction_parent_by_parent(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<FactionParent>, Self::Error>;
    fn membership_by_faction(&self, entity: &EntityId) -> Result<Vec<Membership>, Self::Error>;
    fn culture_practice_by_culture(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<CulturePractice>, Self::Error>;
    fn religion_tenet_by_religion(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<ReligionTenet>, Self::Error>;
    fn adherence_by_adherent(&self, entity: &EntityId) -> Result<Vec<Adherence>, Self::Error>;
    fn item_power_by_item(&self, entity: &EntityId) -> Result<Vec<ItemPower>, Self::Error>;
    fn item_ownership_by_item(&self, entity: &EntityId) -> Result<Vec<ItemOwnership>, Self::Error>;
    fn event_causality_by_cause(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<EventCausality>, Self::Error>;
    fn event_causality_by_effect(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<EventCausality>, Self::Error>;
}

/// Everything the domain needs to read a universe.
pub trait ModelReader:
    ModelQueries
    + Reader<CanonState>
    + Reader<EntityType>
    + Reader<RelationshipType>
    + Reader<PropertyGroup>
    + Reader<PropertyDefinition>
    + Reader<AppearanceKind>
    + Reader<EditorialLevel>
    + Reader<Calendar>
    + Reader<Entity>
    + Reader<Relationship>
    + Reader<PropertyValue>
    + Reader<Appearance>
    + Reader<EventDetail>
    + Reader<EditorialNode>
    + Reader<AssetFile>
    + Reader<Attachment>
    + Reader<Source>
    + Reader<SourceReference>
    + Reader<Note>
    + Reader<CanonStateChange>
    + Reader<UniverseProfile>
    + Reader<VocabularyTerm>
    + Reader<DiplomaticStance>
    + Reader<BeingProfile>
    + Reader<BeingAppearance>
    + Reader<BeingMark>
    + Reader<BeingTrait>
    + Reader<BeingLanguage>
    + Reader<FamilyTie>
    + Reader<Partnership>
    + Reader<DeityProfile>
    + Reader<DeityDomain>
    + Reader<SpeciesProfile>
    + Reader<SpeciesHabitat>
    + Reader<PlaceProfile>
    + Reader<PlaceContainment>
    + Reader<PlaceBorder>
    + Reader<PlaceResource>
    + Reader<Census>
    + Reader<PolityProfile>
    + Reader<PolityLanguage>
    + Reader<PolitySymbol>
    + Reader<PolitySuccession>
    + Reader<Office>
    + Reader<OfficeTenure>
    + Reader<Citizenship>
    + Reader<DiplomaticRelation>
    + Reader<FactionProfile>
    + Reader<FactionParent>
    + Reader<Membership>
    + Reader<CulturePractice>
    + Reader<CultureLanguage>
    + Reader<LanguageProfile>
    + Reader<ReligionProfile>
    + Reader<ReligionTenet>
    + Reader<ReligionDeity>
    + Reader<HolyPlace>
    + Reader<Adherence>
    + Reader<ItemProfile>
    + Reader<ItemPower>
    + Reader<ItemOwnership>
    + Reader<EventCausality>
{
}

impl<T> ModelReader for T where
    T: ModelQueries
        + Reader<CanonState>
        + Reader<EntityType>
        + Reader<RelationshipType>
        + Reader<PropertyGroup>
        + Reader<PropertyDefinition>
        + Reader<AppearanceKind>
        + Reader<EditorialLevel>
        + Reader<Calendar>
        + Reader<Entity>
        + Reader<Relationship>
        + Reader<PropertyValue>
        + Reader<Appearance>
        + Reader<EventDetail>
        + Reader<EditorialNode>
        + Reader<AssetFile>
        + Reader<Attachment>
        + Reader<Source>
        + Reader<SourceReference>
        + Reader<Note>
        + Reader<CanonStateChange>
        + Reader<UniverseProfile>
        + Reader<VocabularyTerm>
        + Reader<DiplomaticStance>
        + Reader<BeingProfile>
        + Reader<BeingAppearance>
        + Reader<BeingMark>
        + Reader<BeingTrait>
        + Reader<BeingLanguage>
        + Reader<FamilyTie>
        + Reader<Partnership>
        + Reader<DeityProfile>
        + Reader<DeityDomain>
        + Reader<SpeciesProfile>
        + Reader<SpeciesHabitat>
        + Reader<PlaceProfile>
        + Reader<PlaceContainment>
        + Reader<PlaceBorder>
        + Reader<PlaceResource>
        + Reader<Census>
        + Reader<PolityProfile>
        + Reader<PolityLanguage>
        + Reader<PolitySymbol>
        + Reader<PolitySuccession>
        + Reader<Office>
        + Reader<OfficeTenure>
        + Reader<Citizenship>
        + Reader<DiplomaticRelation>
        + Reader<FactionProfile>
        + Reader<FactionParent>
        + Reader<Membership>
        + Reader<CulturePractice>
        + Reader<CultureLanguage>
        + Reader<LanguageProfile>
        + Reader<ReligionProfile>
        + Reader<ReligionTenet>
        + Reader<ReligionDeity>
        + Reader<HolyPlace>
        + Reader<Adherence>
        + Reader<ItemProfile>
        + Reader<ItemPower>
        + Reader<ItemOwnership>
        + Reader<EventCausality>
{
}

/// Everything the domain needs to write a universe.
pub trait ModelRepository:
    ModelReader
    + RecordRemoval
    + Writer<CanonState>
    + Writer<EntityType>
    + Writer<RelationshipType>
    + Writer<PropertyGroup>
    + Writer<PropertyDefinition>
    + Writer<AppearanceKind>
    + Writer<EditorialLevel>
    + Writer<Calendar>
    + Writer<Entity>
    + Writer<Relationship>
    + Writer<PropertyValue>
    + Writer<Appearance>
    + Writer<EventDetail>
    + Writer<EditorialNode>
    + Writer<AssetFile>
    + Writer<Attachment>
    + Writer<Source>
    + Writer<SourceReference>
    + Writer<Note>
    + Writer<CanonStateChange>
    + Writer<UniverseProfile>
    + Writer<VocabularyTerm>
    + Writer<DiplomaticStance>
    + Writer<BeingProfile>
    + Writer<BeingAppearance>
    + Writer<BeingMark>
    + Writer<BeingTrait>
    + Writer<BeingLanguage>
    + Writer<FamilyTie>
    + Writer<Partnership>
    + Writer<DeityProfile>
    + Writer<DeityDomain>
    + Writer<SpeciesProfile>
    + Writer<SpeciesHabitat>
    + Writer<PlaceProfile>
    + Writer<PlaceContainment>
    + Writer<PlaceBorder>
    + Writer<PlaceResource>
    + Writer<Census>
    + Writer<PolityProfile>
    + Writer<PolityLanguage>
    + Writer<PolitySymbol>
    + Writer<PolitySuccession>
    + Writer<Office>
    + Writer<OfficeTenure>
    + Writer<Citizenship>
    + Writer<DiplomaticRelation>
    + Writer<FactionProfile>
    + Writer<FactionParent>
    + Writer<Membership>
    + Writer<CulturePractice>
    + Writer<CultureLanguage>
    + Writer<LanguageProfile>
    + Writer<ReligionProfile>
    + Writer<ReligionTenet>
    + Writer<ReligionDeity>
    + Writer<HolyPlace>
    + Writer<Adherence>
    + Writer<ItemProfile>
    + Writer<ItemPower>
    + Writer<ItemOwnership>
    + Writer<EventCausality>
{
}

impl<T> ModelRepository for T where
    T: ModelReader
        + RecordRemoval
        + Writer<CanonState>
        + Writer<EntityType>
        + Writer<RelationshipType>
        + Writer<PropertyGroup>
        + Writer<PropertyDefinition>
        + Writer<AppearanceKind>
        + Writer<EditorialLevel>
        + Writer<Calendar>
        + Writer<Entity>
        + Writer<Relationship>
        + Writer<PropertyValue>
        + Writer<Appearance>
        + Writer<EventDetail>
        + Writer<EditorialNode>
        + Writer<AssetFile>
        + Writer<Attachment>
        + Writer<Source>
        + Writer<SourceReference>
        + Writer<Note>
        + Writer<CanonStateChange>
        + Writer<UniverseProfile>
        + Writer<VocabularyTerm>
        + Writer<DiplomaticStance>
        + Writer<BeingProfile>
        + Writer<BeingAppearance>
        + Writer<BeingMark>
        + Writer<BeingTrait>
        + Writer<BeingLanguage>
        + Writer<FamilyTie>
        + Writer<Partnership>
        + Writer<DeityProfile>
        + Writer<DeityDomain>
        + Writer<SpeciesProfile>
        + Writer<SpeciesHabitat>
        + Writer<PlaceProfile>
        + Writer<PlaceContainment>
        + Writer<PlaceBorder>
        + Writer<PlaceResource>
        + Writer<Census>
        + Writer<PolityProfile>
        + Writer<PolityLanguage>
        + Writer<PolitySymbol>
        + Writer<PolitySuccession>
        + Writer<Office>
        + Writer<OfficeTenure>
        + Writer<Citizenship>
        + Writer<DiplomaticRelation>
        + Writer<FactionProfile>
        + Writer<FactionParent>
        + Writer<Membership>
        + Writer<CulturePractice>
        + Writer<CultureLanguage>
        + Writer<LanguageProfile>
        + Writer<ReligionProfile>
        + Writer<ReligionTenet>
        + Writer<ReligionDeity>
        + Writer<HolyPlace>
        + Writer<Adherence>
        + Writer<ItemProfile>
        + Writer<ItemPower>
        + Writer<ItemOwnership>
        + Writer<EventCausality>
{
}

/// Readable names for the common reads, so callers do not spell out `Reader::<Entity>::list`.
pub trait ModelReaderExt: ModelReader {
    fn vocabulary_terms(&self) -> Result<Vec<VocabularyTerm>, Self::Error> {
        Reader::<VocabularyTerm>::list(self)
    }
    fn diplomatic_stances(&self) -> Result<Vec<DiplomaticStance>, Self::Error> {
        Reader::<DiplomaticStance>::list(self)
    }
    fn canon_states(&self) -> Result<Vec<CanonState>, Self::Error> {
        Reader::<CanonState>::list(self)
    }
    fn entity_types(&self) -> Result<Vec<EntityType>, Self::Error> {
        Reader::<EntityType>::list(self)
    }
    fn relationship_types(&self) -> Result<Vec<RelationshipType>, Self::Error> {
        Reader::<RelationshipType>::list(self)
    }
    fn property_groups(&self) -> Result<Vec<PropertyGroup>, Self::Error> {
        Reader::<PropertyGroup>::list(self)
    }
    fn property_definitions(&self) -> Result<Vec<PropertyDefinition>, Self::Error> {
        Reader::<PropertyDefinition>::list(self)
    }
    fn appearance_kinds(&self) -> Result<Vec<AppearanceKind>, Self::Error> {
        Reader::<AppearanceKind>::list(self)
    }
    fn editorial_levels(&self) -> Result<Vec<EditorialLevel>, Self::Error> {
        Reader::<EditorialLevel>::list(self)
    }
    fn calendars(&self) -> Result<Vec<Calendar>, Self::Error> {
        Reader::<Calendar>::list(self)
    }
    fn entities(&self) -> Result<Vec<Entity>, Self::Error> {
        Reader::<Entity>::list(self)
    }
    fn entity(&self, id: &EntityId) -> Result<Option<Entity>, Self::Error> {
        Reader::<Entity>::get(self, id)
    }
    fn relationships(&self) -> Result<Vec<Relationship>, Self::Error> {
        Reader::<Relationship>::list(self)
    }
    fn relationship(&self, id: &RelationshipId) -> Result<Option<Relationship>, Self::Error> {
        Reader::<Relationship>::get(self, id)
    }
    fn property_values(&self) -> Result<Vec<PropertyValue>, Self::Error> {
        Reader::<PropertyValue>::list(self)
    }
    fn appearances(&self) -> Result<Vec<Appearance>, Self::Error> {
        Reader::<Appearance>::list(self)
    }
    fn events(&self) -> Result<Vec<EventDetail>, Self::Error> {
        Reader::<EventDetail>::list(self)
    }
    fn event(&self, entity: &EntityId) -> Result<Option<EventDetail>, Self::Error> {
        Reader::<EventDetail>::get(self, entity)
    }
    fn editorial_nodes(&self) -> Result<Vec<EditorialNode>, Self::Error> {
        Reader::<EditorialNode>::list(self)
    }
    fn asset_files(&self) -> Result<Vec<AssetFile>, Self::Error> {
        Reader::<AssetFile>::list(self)
    }
    fn attachments(&self) -> Result<Vec<Attachment>, Self::Error> {
        Reader::<Attachment>::list(self)
    }
    fn sources(&self) -> Result<Vec<Source>, Self::Error> {
        Reader::<Source>::list(self)
    }
    fn citations(&self) -> Result<Vec<SourceReference>, Self::Error> {
        Reader::<SourceReference>::list(self)
    }
    fn notes(&self) -> Result<Vec<Note>, Self::Error> {
        Reader::<Note>::list(self)
    }
    /// The type and its ancestors, nearest first.
    fn entity_type_chain(
        &self,
        key: &TypeKey,
    ) -> Result<Vec<EntityType>, HierarchyError<Self::Error>>
    where
        Self: Sized,
    {
        hierarchy::entity_type_chain(self, key)
    }
    /// Every property that applies to entities of a type, inherited ones first.
    fn applicable_property_definitions(
        &self,
        key: &TypeKey,
    ) -> Result<Vec<PropertyDefinition>, HierarchyError<Self::Error>>
    where
        Self: Sized,
    {
        hierarchy::applicable_property_definitions(self, key)
    }
    /// The places a place sits inside, through one kind of containment, nearest first.
    fn places_containing(
        &self,
        place: &EntityId,
        kind: &TypeKey,
    ) -> Result<Vec<EntityId>, HierarchyError<Self::Error>>
    where
        Self: Sized,
    {
        hierarchy::places_containing(self, place, kind)
    }
    /// The places inside a place, and inside those, nearest first.
    fn places_within(
        &self,
        place: &EntityId,
        kind: &TypeKey,
    ) -> Result<Vec<EntityId>, HierarchyError<Self::Error>>
    where
        Self: Sized,
    {
        hierarchy::places_within(self, place, kind)
    }
    /// What the entity is inside of through a hierarchical relationship type, nearest first.
    fn containers_of(
        &self,
        entity: &EntityId,
        relationship_type: &TypeKey,
    ) -> Result<Vec<EntityId>, HierarchyError<Self::Error>>
    where
        Self: Sized,
    {
        hierarchy::containers_of(self, entity, relationship_type)
    }
    /// What is inside the entity through a hierarchical relationship type, nearest first.
    fn contents_of(
        &self,
        entity: &EntityId,
        relationship_type: &TypeKey,
    ) -> Result<Vec<EntityId>, HierarchyError<Self::Error>>
    where
        Self: Sized,
    {
        hierarchy::contents_of(self, entity, relationship_type)
    }
    fn universe_profile(&self) -> Result<UniverseProfile, Self::Error> {
        Ok(Reader::<UniverseProfile>::get(self, &())?.unwrap_or_default())
    }
}

impl<T: ModelReader> ModelReaderExt for T {}
