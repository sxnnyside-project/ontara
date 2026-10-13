//! Ontara domain engine.
//!
//! This crate holds the authoritative domain rules. It must not depend on Tauri, the
//! frontend or any external provider, so every rule stays testable on its own.

mod app_info;
mod appearance;
mod appearance_kind;
mod appearance_matrix;
mod asset;
mod being;
mod belief;
mod calendar;
mod canon_state;
mod canon_state_change;
mod catalog_entry;
mod connections;
mod consistency;
mod diplomatic_stance;
mod editorial_level;
mod editorial_node;
mod entity;
mod entity_search;
mod entity_type;
mod error;
mod event;
mod event_detail;
mod faction;
mod graph;
mod hierarchy;
mod identifier;
mod item;
mod model_editor;
mod note;
mod origin;
mod place;
mod polity;
mod property_definition;
mod property_value;
mod record;
mod record_ref;
mod relationship;
mod relationship_type;
mod repository;
mod source;
mod temporal;
mod timeline;
mod timestamp;
mod type_key;
mod universe_profile;
mod vocabulary;

pub use app_info::{AppInfo, app_info};
pub use appearance::Appearance;
pub use appearance_kind::{AppearanceKind, builtin_appearance_kinds};
pub use appearance_matrix::{
    AppearanceMatrix, MatrixCell, MatrixColumn, MatrixColumns, MatrixQuery, appearance_matrix,
};
pub use asset::{AssetFile, Attachment};
pub use being::{
    BeingAppearance, BeingLanguage, BeingMark, BeingProfile, BeingTrait, DeityDomain, DeityProfile,
    FamilyTie, Partnership, PartnershipEndReason, Proficiency, SpeciesHabitat, SpeciesProfile,
};
pub use belief::{
    Adherence, CultureLanguage, CulturePractice, CulturePracticeKind, Devotion, HolyPlace,
    LanguageProfile, LanguageStatus, ReligionDeity, ReligionProfile, ReligionTenet, TenetKind,
};
pub use calendar::{
    Calendar, CalendarDate, CalendarDefinition, Era, EraDirection, LeapRule, Month,
    gregorian_definition,
};
pub use canon_state::{CanonState, builtin_canon_states, default_canon_state};
pub use canon_state_change::CanonStateChange;
pub use catalog_entry::CatalogEntry;
pub use connections::{Connection, ConnectionKind, connections_of};
pub use consistency::{
    ConsistencyReport, Examined, Finding, Rule, Severity, SeverityCounts, Snapshot,
    check_consistency, check_snapshot,
};
pub use diplomatic_stance::{DiplomaticStance, builtin_diplomatic_stances};
pub use editorial_level::{EditorialLevel, builtin_editorial_levels};
pub use editorial_node::{EditorialNode, ReadingStep, reading_order};
pub use entity::{Entity, EntityName, NameKind};
pub use entity_search::{EntityFilter, find_entities};
pub use entity_type::{EntityRole, EntityType, builtin_entity_types};
pub use error::DomainError;
pub use event::{CausalityKind, EventCausality};
pub use event_detail::{EventDetail, EventParticipant, EventScale};
pub use faction::{FactionParent, FactionProfile, Membership, MembershipEndReason};
pub use graph::{
    Graph, GraphEdge, GraphEdgeKind, GraphNode, GraphQuery, MAX_GRAPH_DEPTH, MAX_GRAPH_NODES,
    explore,
};
pub use hierarchy::{
    HierarchyError, applicable_property_definitions, containers_of, contents_of, entity_type_chain,
    is_entity_type, reaches,
};
pub use identifier::{
    AdherenceId, AppearanceId, AttachmentId, BeingAppearanceId, BeingMarkId, BeingTraitId,
    CalendarId, CanonStateChangeId, CensusId, CitizenshipId, CulturePracticeId, DeityDomainId,
    DiplomaticRelationId, EntityId, EventCausalityId, FactionParentId, FamilyTieId,
    ItemOwnershipId, ItemPowerId, MembershipId, NoteId, OfficeId, OfficeTenureId, PartnershipId,
    PlaceBorderId, PlaceContainmentId, PlaceResourceId, PolitySuccessionId, PolitySymbolId,
    PropertyDefinitionId, PropertyValueId, RelationshipId, ReligionTenetId, SourceId,
    SourceReferenceId,
};
pub use item::{ItemOwnership, ItemPower, ItemProfile, ItemRarity};
pub use model_editor::{EditError, EditorialImpact, ModelEditor, RemovalStrategy};
pub use note::Note;
pub use origin::Origin;
pub use place::{
    BorderKind, Census, PlaceBorder, PlaceContainment, PlaceProfile, PlaceResource, PlaceStatus,
    ResourceAbundance,
};
pub use polity::{
    Citizenship, DiplomaticRelation, Office, OfficeTenure, PolityLanguage, PolityLanguageStatus,
    PolityProfile, PolitySuccession, PolitySuccessionKind, PolitySymbol, PolitySymbolKind,
    SuccessionRule, TenureEndReason,
};
pub use property_definition::{
    BuiltinPropertySet, ChoiceOption, PropertyDataType, PropertyDefinition, PropertyGroup,
    PropertyOwner, builtin_property_set,
};
pub use property_value::{PropertyContent, PropertyValue, PropertyValueOwner};
pub use record::Record;
pub use record_ref::{RecordKind, RecordRef};
pub use relationship::{Lifecycle, Relationship};
pub use relationship_type::{Directionality, RelationshipType, builtin_relationship_types};
pub use repository::{
    ModelQueries, ModelReader, ModelReaderExt, ModelRepository, Reader, RecordRemoval,
    ReferenceCount, Repository, Writer,
};
pub use source::{Source, SourceKind, SourceReference};
pub use temporal::{Instant, Precision, Temporal};
pub use timeline::{
    Timeline, TimelineEntry, TimelineLink, TimelineQuery, TimelineSource, build_timeline, timeline,
};
pub use timestamp::Timestamp;
pub use type_key::TypeKey;
pub use universe_profile::UniverseProfile;
pub use vocabulary::{Vocabulary, VocabularyTerm, builtin_vocabulary_terms};
