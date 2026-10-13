// <generated imports>
use super::being::{BEING_APPEARANCE, being_appearance};
use super::being::{BEING_MARK, being_mark};
use super::being::{BEING_TRAIT, being_trait};
use super::being::{DEITY_DOMAIN, deity_domain};
use super::being::{FAMILY_TIE, family_tie};
use super::being::{PARTNERSHIP, partnership};
use super::belief::{ADHERENCE, adherence};
use super::belief::{CULTURE_PRACTICE, culture_practice};
use super::belief::{RELIGION_TENET, religion_tenet};
use super::event::{EVENT_CAUSALITY, event_causality};
use super::faction::{FACTION_PARENT, faction_parent};
use super::faction::{MEMBERSHIP, membership};
use super::item::{ITEM_OWNERSHIP, item_ownership};
use super::item::{ITEM_POWER, item_power};
use super::place::{CENSUS, census};
use super::place::{PLACE_BORDER, place_border};
use super::place::{PLACE_CONTAINMENT, place_containment};
use super::place::{PLACE_RESOURCE, place_resource};
use super::polity::{CITIZENSHIP, citizenship};
use super::polity::{DIPLOMATIC_RELATION, diplomatic_relation};
use super::polity::{OFFICE, office};
use super::polity::{OFFICE_TENURE, office_tenure};
use super::polity::{POLITY_SUCCESSION, polity_succession};
use super::polity::{POLITY_SYMBOL, polity_symbol};
use ontara_core::{
    Adherence, BeingAppearance, BeingMark, BeingTrait, Census, Citizenship, CulturePractice,
    DeityDomain, DiplomaticRelation, EventCausality, FactionParent, FamilyTie, ItemOwnership,
    ItemPower, Membership, Office, OfficeTenure, Partnership, PlaceBorder, PlaceContainment,
    PlaceResource, PolitySuccession, PolitySymbol, ReligionTenet,
};
// </generated imports>
use ontara_core::{
    Appearance, Attachment, CanonStateChange, CatalogEntry, EditorialNode, EntityId, EventDetail,
    ModelQueries, Note, PropertyDefinition, PropertyOwner, PropertyValue, PropertyValueOwner,
    RecordRef, ReferenceCount, Relationship, Source, SourceReference,
};

use super::SqliteModel;
use super::entities::{relationship, relationship_sql};
use super::narrative::{APPEARANCE, EDITORIAL_NODE, appearance, editorial_node};
use super::properties::{property_value, property_value_sql};
use super::supporting::{
    ATTACHMENT, CANON_STATE_CHANGE, NOTE, SOURCE_REFERENCE, attachment, canon_state_change, note,
    source_reference,
};
use crate::error::StorageError;

impl ModelQueries for SqliteModel<'_> {
    fn relationships_of(&self, entity: &EntityId) -> Result<Vec<Relationship>, StorageError> {
        self.select(
            &format!(
                "{} WHERE t.source_id = ?1 OR t.target_id = ?1 ORDER BY t.created_at, t.id",
                relationship_sql()
            ),
            [entity.to_string()],
            relationship,
        )
    }

    fn relationships_in_context(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<Relationship>, StorageError> {
        self.select(
            &format!(
                "{} WHERE t.context_id = ?1 ORDER BY t.created_at, t.id",
                relationship_sql()
            ),
            [entity.to_string()],
            relationship,
        )
    }

    fn property_definitions_of(
        &self,
        owner: &PropertyOwner,
    ) -> Result<Vec<PropertyDefinition>, StorageError> {
        let all = ontara_core::Reader::<PropertyDefinition>::list(self)?;
        Ok(all
            .into_iter()
            .filter(|definition| &definition.owner == owner)
            .collect())
    }

    fn property_values_of(
        &self,
        owner: &PropertyValueOwner,
    ) -> Result<Vec<PropertyValue>, StorageError> {
        let (column, id) = match owner {
            PropertyValueOwner::Entity(id) => ("entity_id", id.to_string()),
            PropertyValueOwner::Relationship(id) => ("relationship_id", id.to_string()),
        };
        self.select(
            &format!(
                "{} WHERE t.{column} = ?1 ORDER BY t.definition_id, t.position, t.id",
                property_value_sql()
            ),
            [id],
            property_value,
        )
    }

    fn property_values_referencing(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PropertyValue>, StorageError> {
        self.select(
            &format!(
                "{} WHERE t.value_entity_id = ?1 ORDER BY t.created_at, t.id",
                property_value_sql()
            ),
            [entity.to_string()],
            property_value,
        )
    }

    fn appearances_of(&self, entity: &EntityId) -> Result<Vec<Appearance>, StorageError> {
        self.select(
            &format!("{APPEARANCE} WHERE entity_id = ?1 ORDER BY context_id, position, id"),
            [entity.to_string()],
            appearance,
        )
    }

    fn appearances_in(&self, context: &EntityId) -> Result<Vec<Appearance>, StorageError> {
        self.select(
            &format!("{APPEARANCE} WHERE context_id = ?1 ORDER BY position, id"),
            [context.to_string()],
            appearance,
        )
    }

    fn appearances_with_viewpoint(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<Appearance>, StorageError> {
        self.select(
            &format!("{APPEARANCE} WHERE viewpoint_id = ?1 ORDER BY context_id, position, id"),
            [entity.to_string()],
            appearance,
        )
    }

    fn events_involving(&self, entity: &EntityId) -> Result<Vec<EventDetail>, StorageError> {
        self.events_where(
            "WHERE t.location_id = ?1 OR t.entity_id IN \
             (SELECT event_id FROM event_participant WHERE participant_id = ?1) \
             ORDER BY t.entity_id",
            [entity.to_string()],
        )
    }

    fn editorial_children(
        &self,
        parent: Option<&EntityId>,
    ) -> Result<Vec<EditorialNode>, StorageError> {
        self.select(
            &format!("{EDITORIAL_NODE} WHERE parent_id IS ?1 ORDER BY position, entity_id"),
            [parent.map(ToString::to_string)],
            editorial_node,
        )
    }

    fn attachments_of(&self, entity: &EntityId) -> Result<Vec<Attachment>, StorageError> {
        self.select(
            &format!("{ATTACHMENT} WHERE owner_id = ?1 OR asset_id = ?1 ORDER BY position, id"),
            [entity.to_string()],
            attachment,
        )
    }

    fn notes_of(&self, entity: &EntityId) -> Result<Vec<Note>, StorageError> {
        self.select(
            &format!("{NOTE} WHERE entity_id = ?1 ORDER BY created_at, id"),
            [entity.to_string()],
            note,
        )
    }

    fn citations_of(&self, target: &RecordRef) -> Result<Vec<SourceReference>, StorageError> {
        self.select(
            &format!(
                "{SOURCE_REFERENCE} WHERE target_table = ?1 AND target_id = ?2 ORDER BY created_at, id"
            ),
            [target.kind.as_str().to_owned(), target.id.to_string()],
            source_reference,
        )
    }

    fn citations_from(&self, source: &Source) -> Result<Vec<SourceReference>, StorageError> {
        self.select(
            &format!("{SOURCE_REFERENCE} WHERE source_id = ?1 ORDER BY created_at, id"),
            [source.id.to_string()],
            source_reference,
        )
    }

    fn canon_history(&self, target: &RecordRef) -> Result<Vec<CanonStateChange>, StorageError> {
        self.select(
            &format!(
                "{CANON_STATE_CHANGE} WHERE target_table = ?1 AND target_id = ?2 \
                 ORDER BY changed_at, rowid"
            ),
            [target.kind.as_str().to_owned(), target.id.to_string()],
            canon_state_change,
        )
    }

    fn references_to(&self, entity: &EntityId) -> Result<Vec<ReferenceCount>, StorageError> {
        self.count_references(&entity.to_string())
    }

    fn catalog_usage(&self, entry: &CatalogEntry) -> Result<Vec<ReferenceCount>, StorageError> {
        self.count_catalog_usage(entry)
    }

    // <generated queries>
    fn being_appearance_by_being(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<BeingAppearance>, StorageError> {
        self.select(
            &format!("{BEING_APPEARANCE} WHERE t.being_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            being_appearance,
        )
    }

    fn being_mark_by_being(&self, entity: &EntityId) -> Result<Vec<BeingMark>, StorageError> {
        self.select(
            &format!("{BEING_MARK} WHERE t.being_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            being_mark,
        )
    }

    fn being_trait_by_being(&self, entity: &EntityId) -> Result<Vec<BeingTrait>, StorageError> {
        self.select(
            &format!("{BEING_TRAIT} WHERE t.being_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            being_trait,
        )
    }

    fn family_tie_by_parent(&self, entity: &EntityId) -> Result<Vec<FamilyTie>, StorageError> {
        self.select(
            &format!("{FAMILY_TIE} WHERE t.parent_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            family_tie,
        )
    }

    fn family_tie_by_child(&self, entity: &EntityId) -> Result<Vec<FamilyTie>, StorageError> {
        self.select(
            &format!("{FAMILY_TIE} WHERE t.child_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            family_tie,
        )
    }

    fn partnership_by_first(&self, entity: &EntityId) -> Result<Vec<Partnership>, StorageError> {
        self.select(
            &format!("{PARTNERSHIP} WHERE t.first_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            partnership,
        )
    }

    fn deity_domain_by_deity(&self, entity: &EntityId) -> Result<Vec<DeityDomain>, StorageError> {
        self.select(
            &format!("{DEITY_DOMAIN} WHERE t.deity_id = ?1 ORDER BY t.id"),
            [entity.to_string()],
            deity_domain,
        )
    }

    fn place_containment_by_child(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PlaceContainment>, StorageError> {
        self.select(
            &format!("{PLACE_CONTAINMENT} WHERE t.child_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            place_containment,
        )
    }

    fn place_containment_by_parent(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PlaceContainment>, StorageError> {
        self.select(
            &format!("{PLACE_CONTAINMENT} WHERE t.parent_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            place_containment,
        )
    }

    fn place_border_by_first(&self, entity: &EntityId) -> Result<Vec<PlaceBorder>, StorageError> {
        self.select(
            &format!("{PLACE_BORDER} WHERE t.first_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            place_border,
        )
    }

    fn place_resource_by_place(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PlaceResource>, StorageError> {
        self.select(
            &format!("{PLACE_RESOURCE} WHERE t.place_id = ?1 ORDER BY t.id"),
            [entity.to_string()],
            place_resource,
        )
    }

    fn census_by_place(&self, entity: &EntityId) -> Result<Vec<Census>, StorageError> {
        self.select(
            &format!("{CENSUS} WHERE t.place_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            census,
        )
    }

    fn polity_symbol_by_polity(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PolitySymbol>, StorageError> {
        self.select(
            &format!("{POLITY_SYMBOL} WHERE t.polity_id = ?1 ORDER BY t.id"),
            [entity.to_string()],
            polity_symbol,
        )
    }

    fn polity_succession_by_predecessor(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PolitySuccession>, StorageError> {
        self.select(
            &format!("{POLITY_SUCCESSION} WHERE t.predecessor_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            polity_succession,
        )
    }

    fn polity_succession_by_successor(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<PolitySuccession>, StorageError> {
        self.select(
            &format!("{POLITY_SUCCESSION} WHERE t.successor_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            polity_succession,
        )
    }

    fn office_by_body(&self, entity: &EntityId) -> Result<Vec<Office>, StorageError> {
        self.select(
            &format!("{OFFICE} WHERE t.body_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            office,
        )
    }

    fn office_tenure_by_holder(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<OfficeTenure>, StorageError> {
        self.select(
            &format!("{OFFICE_TENURE} WHERE t.holder_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            office_tenure,
        )
    }

    fn citizenship_by_being(&self, entity: &EntityId) -> Result<Vec<Citizenship>, StorageError> {
        self.select(
            &format!("{CITIZENSHIP} WHERE t.being_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            citizenship,
        )
    }

    fn diplomatic_relation_by_first(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<DiplomaticRelation>, StorageError> {
        self.select(
            &format!("{DIPLOMATIC_RELATION} WHERE t.first_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            diplomatic_relation,
        )
    }

    fn faction_parent_by_child(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<FactionParent>, StorageError> {
        self.select(
            &format!("{FACTION_PARENT} WHERE t.child_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            faction_parent,
        )
    }

    fn faction_parent_by_parent(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<FactionParent>, StorageError> {
        self.select(
            &format!("{FACTION_PARENT} WHERE t.parent_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            faction_parent,
        )
    }

    fn membership_by_faction(&self, entity: &EntityId) -> Result<Vec<Membership>, StorageError> {
        self.select(
            &format!("{MEMBERSHIP} WHERE t.faction_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            membership,
        )
    }

    fn culture_practice_by_culture(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<CulturePractice>, StorageError> {
        self.select(
            &format!("{CULTURE_PRACTICE} WHERE t.culture_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            culture_practice,
        )
    }

    fn religion_tenet_by_religion(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<ReligionTenet>, StorageError> {
        self.select(
            &format!("{RELIGION_TENET} WHERE t.religion_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            religion_tenet,
        )
    }

    fn adherence_by_adherent(&self, entity: &EntityId) -> Result<Vec<Adherence>, StorageError> {
        self.select(
            &format!("{ADHERENCE} WHERE t.adherent_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            adherence,
        )
    }

    fn item_power_by_item(&self, entity: &EntityId) -> Result<Vec<ItemPower>, StorageError> {
        self.select(
            &format!("{ITEM_POWER} WHERE t.item_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            item_power,
        )
    }

    fn item_ownership_by_item(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<ItemOwnership>, StorageError> {
        self.select(
            &format!("{ITEM_OWNERSHIP} WHERE t.item_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            item_ownership,
        )
    }

    fn event_causality_by_cause(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<EventCausality>, StorageError> {
        self.select(
            &format!("{EVENT_CAUSALITY} WHERE t.cause_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            event_causality,
        )
    }

    fn event_causality_by_effect(
        &self,
        entity: &EntityId,
    ) -> Result<Vec<EventCausality>, StorageError> {
        self.select(
            &format!("{EVENT_CAUSALITY} WHERE t.effect_id = ?1 ORDER BY t.created_at, t.id"),
            [entity.to_string()],
            event_causality,
        )
    }

    // </generated queries>
}
