//! The table of sections. Each entry names a record type, the fields that tie it to entities and
//! the editor operations that write it; a macro per shape turns that into the functions the
//! dossier calls.

use ontara_core::{
    Adherence, BeingAppearance, BeingLanguage, BeingMark, BeingProfile, BeingTrait, Census,
    Citizenship, CultureLanguage, CulturePractice, DeityDomain, DeityProfile, DiplomaticRelation,
    FactionParent, FactionProfile, FamilyTie, HolyPlace, ItemOwnership, ItemPower, ItemProfile,
    LanguageProfile, Membership, Office, OfficeTenure, Partnership, PlaceBorder, PlaceContainment,
    PlaceProfile, PlaceResource, PolityLanguage, PolityProfile, PolitySuccession, PolitySymbol,
    Reader, ReligionDeity, ReligionProfile, ReligionTenet, SpeciesHabitat, SpeciesProfile,
};

use super::{Section, Shape, decode, encode};
use crate::workspace::CommandError;

fn values<T: serde::Serialize>(
    records: Result<Vec<T>, ontara_storage::StorageError>,
) -> Result<Vec<serde_json::Value>, CommandError> {
    records?
        .iter()
        .map(|record| {
            serde_json::to_value(record)
                .map_err(|error| CommandError::new("storage", error.to_string()))
        })
        .collect()
}

fn invalid(error: &impl std::fmt::Display) -> CommandError {
    CommandError::new("rejected", format!("the record is not valid: {error}"))
}

/// One record per entity, keyed by it: written with `set_`, removed by the entity.
macro_rules! profile {
    ($key:literal, $owners:expr, $ty:ty, $set:ident, $remove:ident) => {
        Section {
            key: $key,
            shape: Shape::Profile,
            owners: $owners,
            list: |project| values(Reader::<$ty>::list(&project.reader())),
            exists: |_, _| Ok(false),
            save: |editor, value, _, _| encode(&editor.$set(decode::<$ty>(value)?)?),
            remove: |editor, value| editor.$remove(&decode::<$ty>(value)?.entity),
        }
    };
}

/// Calls an update that takes the time (`now`) or one that does not.
macro_rules! update_call {
    ($editor:ident, $update:ident, $record:ident, $at:ident, now) => {
        $editor.$update($record, $at)?
    };
    ($editor:ident, $update:ident, $record:ident, $at:ident) => {{
        let _ = $at;
        $editor.$update($record)?
    }};
}

/// Records with their own identifier: added, updated and removed.
macro_rules! items {
    ($key:literal, $owners:expr, $ty:ty, $add:ident, $update:ident, $remove:ident $(, $now:ident)?) => {
        Section {
            key: $key,
            shape: Shape::Items,
            owners: $owners,
            list: |project| values(Reader::<$ty>::list(&project.reader())),
            exists: |project, value| {
                let record: $ty = serde_json::from_value(value.clone()).map_err(|e| invalid(&e))?;
                Ok(Reader::<$ty>::get(&project.reader(), &record.id)?.is_some())
            },
            save: |editor, value, exists, at| {
                let record: $ty = decode(value)?;
                let saved = if exists {
                    update_call!(editor, $update, record, at $(, $now)?)
                } else {
                    editor.$add(record)?
                };
                encode(&saved)
            },
            remove: |editor, value| editor.$remove(&decode::<$ty>(value)?.id),
        }
    };
}

/// Records keyed by the pair of entities they link: written with `set_`, removed by the pair.
macro_rules! links {
    ($key:literal, $owners:expr, $ty:ty, $first:ident, $second:ident, $set:ident, $remove:ident) => {
        Section {
            key: $key,
            shape: Shape::Links,
            owners: $owners,
            list: |project| values(Reader::<$ty>::list(&project.reader())),
            exists: |_, _| Ok(false),
            save: |editor, value, _, _| encode(&editor.$set(decode::<$ty>(value)?)?),
            remove: |editor, value| {
                let record = decode::<$ty>(value)?;
                editor.$remove(&(record.$first, record.$second))
            },
        }
    };
}

const BEING: &[&str] = &["being"];
const PLACE: &[&str] = &["place"];
const POLITY: &[&str] = &["polity"];
const FACTION: &[&str] = &["faction"];
const LANGUAGE: &[&str] = &["language"];
const RELIGION: &[&str] = &["religion"];
const DEITY: &[&str] = &["deity"];
const ITEM: &[&str] = &["item"];
const BODY: &[&str] = &["polity", "faction"];
const ANYONE: &[&str] = &["being", "faction", "polity", "culture"];

/// Every section, in the order the interface shows them.
pub(super) static ALL: &[Section] = &[
    profile!(
        "being_profile",
        &[("entity", BEING)],
        BeingProfile,
        set_being_profile,
        remove_being_profile
    ),
    items!(
        "being_appearance",
        &[("being", BEING)],
        BeingAppearance,
        add_being_appearance,
        update_being_appearance,
        remove_being_appearance,
        now
    ),
    items!(
        "being_trait",
        &[("being", BEING)],
        BeingTrait,
        add_being_trait,
        update_being_trait,
        remove_being_trait,
        now
    ),
    items!(
        "being_mark",
        &[("being", BEING)],
        BeingMark,
        add_being_mark,
        update_being_mark,
        remove_being_mark,
        now
    ),
    links!(
        "being_language",
        &[("being", BEING), ("language", LANGUAGE)],
        BeingLanguage,
        being,
        language,
        set_being_language,
        remove_being_language
    ),
    items!(
        "family_tie",
        &[("parent", BEING), ("child", BEING)],
        FamilyTie,
        add_family_tie,
        update_family_tie,
        remove_family_tie,
        now
    ),
    items!(
        "partnership",
        &[("first", BEING), ("second", BEING)],
        Partnership,
        add_partnership,
        update_partnership,
        remove_partnership,
        now
    ),
    items!(
        "citizenship",
        &[("being", BEING), ("polity", POLITY)],
        Citizenship,
        add_citizenship,
        update_citizenship,
        remove_citizenship,
        now
    ),
    items!(
        "office_tenure",
        &[("holder", BEING)],
        OfficeTenure,
        add_office_tenure,
        update_office_tenure,
        remove_office_tenure,
        now
    ),
    profile!(
        "deity_profile",
        &[("entity", DEITY)],
        DeityProfile,
        set_deity_profile,
        remove_deity_profile
    ),
    items!(
        "deity_domain",
        &[("deity", DEITY)],
        DeityDomain,
        add_deity_domain,
        update_deity_domain,
        remove_deity_domain
    ),
    profile!(
        "species_profile",
        &[("entity", &["species"])],
        SpeciesProfile,
        set_species_profile,
        remove_species_profile
    ),
    links!(
        "species_habitat",
        &[("species", &["species"]), ("place", PLACE)],
        SpeciesHabitat,
        species,
        place,
        set_species_habitat,
        remove_species_habitat
    ),
    profile!(
        "place_profile",
        &[("entity", PLACE)],
        PlaceProfile,
        set_place_profile,
        remove_place_profile
    ),
    items!(
        "place_containment",
        &[("child", PLACE), ("parent", PLACE)],
        PlaceContainment,
        add_place_containment,
        update_place_containment,
        remove_place_containment,
        now
    ),
    items!(
        "place_border",
        &[("first", PLACE), ("second", PLACE)],
        PlaceBorder,
        add_place_border,
        update_place_border,
        remove_place_border,
        now
    ),
    items!(
        "place_resource",
        &[("place", PLACE)],
        PlaceResource,
        add_place_resource,
        update_place_resource,
        remove_place_resource
    ),
    items!(
        "census",
        &[("place", PLACE)],
        Census,
        add_census,
        update_census,
        remove_census,
        now
    ),
    profile!(
        "polity_profile",
        &[("entity", POLITY)],
        PolityProfile,
        set_polity_profile,
        remove_polity_profile
    ),
    links!(
        "polity_language",
        &[("polity", POLITY), ("language", LANGUAGE)],
        PolityLanguage,
        polity,
        language,
        set_polity_language,
        remove_polity_language
    ),
    items!(
        "polity_symbol",
        &[("polity", POLITY)],
        PolitySymbol,
        add_polity_symbol,
        update_polity_symbol,
        remove_polity_symbol
    ),
    items!(
        "polity_succession",
        &[("predecessor", POLITY), ("successor", POLITY)],
        PolitySuccession,
        add_polity_succession,
        update_polity_succession,
        remove_polity_succession,
        now
    ),
    items!(
        "office",
        &[("body", BODY)],
        Office,
        add_office,
        update_office,
        remove_office,
        now
    ),
    items!(
        "diplomatic_relation",
        &[("first", BODY), ("second", BODY)],
        DiplomaticRelation,
        add_diplomatic_relation,
        update_diplomatic_relation,
        remove_diplomatic_relation,
        now
    ),
    profile!(
        "faction_profile",
        &[("entity", FACTION)],
        FactionProfile,
        set_faction_profile,
        remove_faction_profile
    ),
    items!(
        "faction_parent",
        &[("child", FACTION), ("parent", FACTION)],
        FactionParent,
        add_faction_parent,
        update_faction_parent,
        remove_faction_parent,
        now
    ),
    items!(
        "membership",
        &[("faction", FACTION), ("member", ANYONE)],
        Membership,
        add_membership,
        update_membership,
        remove_membership,
        now
    ),
    profile!(
        "language_profile",
        &[("entity", LANGUAGE)],
        LanguageProfile,
        set_language_profile,
        remove_language_profile
    ),
    profile!(
        "religion_profile",
        &[("entity", RELIGION)],
        ReligionProfile,
        set_religion_profile,
        remove_religion_profile
    ),
    items!(
        "religion_tenet",
        &[("religion", RELIGION)],
        ReligionTenet,
        add_religion_tenet,
        update_religion_tenet,
        remove_religion_tenet,
        now
    ),
    links!(
        "religion_deity",
        &[("religion", RELIGION), ("deity", DEITY)],
        ReligionDeity,
        religion,
        deity,
        set_religion_deity,
        remove_religion_deity
    ),
    links!(
        "holy_place",
        &[("religion", RELIGION), ("place", PLACE)],
        HolyPlace,
        religion,
        place,
        set_holy_place,
        remove_holy_place
    ),
    items!(
        "adherence",
        &[("adherent", ANYONE), ("religion", RELIGION)],
        Adherence,
        add_adherence,
        update_adherence,
        remove_adherence,
        now
    ),
    items!(
        "culture_practice",
        &[("culture", &["culture"])],
        CulturePractice,
        add_culture_practice,
        update_culture_practice,
        remove_culture_practice,
        now
    ),
    links!(
        "culture_language",
        &[("culture", &["culture"]), ("language", LANGUAGE)],
        CultureLanguage,
        culture,
        language,
        set_culture_language,
        remove_culture_language
    ),
    profile!(
        "item_profile",
        &[("entity", ITEM)],
        ItemProfile,
        set_item_profile,
        remove_item_profile
    ),
    items!(
        "item_power",
        &[("item", ITEM)],
        ItemPower,
        add_item_power,
        update_item_power,
        remove_item_power,
        now
    ),
    items!(
        "item_ownership",
        &[("item", ITEM), ("owner", ANYONE)],
        ItemOwnership,
        add_item_ownership,
        update_item_ownership,
        remove_item_ownership,
        now
    ),
];
