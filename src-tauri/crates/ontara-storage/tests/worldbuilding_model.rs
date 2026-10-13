//! A worked universe through the real database: every table of the relational model holds
//! something, and everything must come back identical after close, move and reopen.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;
use std::fs;

use ontara_core::{
    Adherence, Appearance, AssetFile, Attachment, BeingAppearance, BeingLanguage, BeingMark,
    BeingProfile, BeingTrait, BorderKind, CalendarDate, CausalityKind, Census, Citizenship,
    CultureLanguage, CulturePractice, CulturePracticeKind, DeityDomain, DeityProfile,
    DiplomaticRelation, EditorialNode, Entity, EntityId, EventCausality, EventDetail,
    EventParticipant, FactionParent, FactionProfile, FamilyTie, HolyPlace, Instant, ItemOwnership,
    ItemPower, ItemProfile, LanguageProfile, Membership, ModelQueries, ModelReaderExt, Note,
    Office, OfficeTenure, Origin, Partnership, PlaceBorder, PlaceContainment, PlaceProfile,
    PlaceResource, PolityLanguage, PolityLanguageStatus, PolityProfile, PolitySuccession,
    PolitySuccessionKind, PolitySymbol, PolitySymbolKind, Proficiency, Reader, RecordKind,
    RecordRef, ReligionDeity, ReligionProfile, ReligionTenet, Source, SourceKind, SourceReference,
    SpeciesHabitat, SpeciesProfile, Temporal, TenetKind, Timestamp, TypeKey,
};
use ontara_storage::{MigrationPolicy, Project};

type TestResult = Result<(), Box<dyn Error>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);
const T1: Timestamp = Timestamp::from_unix_millis(2_000);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn span(from: i64, until: Option<i64>) -> Temporal {
    Temporal::Span {
        start: Some(Instant::at_day(from)),
        end: until.map(Instant::at_day),
    }
}

/// Every table of the model, as text, so two states can be compared whole.
fn snapshot(project: &Project) -> Result<Vec<String>, Box<dyn Error>> {
    let reader = project.reader();
    macro_rules! table {
        ($($record:ty),* $(,)?) => {
            vec![$(format!("{:?}", Reader::<$record>::list(&reader)?)),*]
        };
    }
    Ok(table![
        Entity,
        ontara_core::Relationship,
        ontara_core::PropertyValue,
        Appearance,
        EventDetail,
        EditorialNode,
        AssetFile,
        Attachment,
        Source,
        SourceReference,
        Note,
        BeingProfile,
        BeingAppearance,
        BeingMark,
        BeingTrait,
        BeingLanguage,
        FamilyTie,
        Partnership,
        DeityProfile,
        DeityDomain,
        SpeciesProfile,
        SpeciesHabitat,
        PlaceProfile,
        PlaceContainment,
        PlaceBorder,
        PlaceResource,
        Census,
        PolityProfile,
        PolityLanguage,
        PolitySymbol,
        PolitySuccession,
        Office,
        OfficeTenure,
        Citizenship,
        DiplomaticRelation,
        FactionProfile,
        FactionParent,
        Membership,
        CulturePractice,
        CultureLanguage,
        LanguageProfile,
        ReligionProfile,
        ReligionTenet,
        ReligionDeity,
        HolyPlace,
        Adherence,
        ItemProfile,
        ItemPower,
        ItemOwnership,
        EventCausality,
        ontara_core::Calendar,
        ontara_core::PropertyDefinition,
        ontara_core::CanonStateChange,
    ])
}

struct Cast {
    ayla: EntityId,
    orrin: EntityId,
    north: EntityId,
    ashmere: EntityId,
    chapter: EntityId,
}

#[test]
#[allow(clippy::too_many_lines)] // One continuous scenario reads better than fragments.
fn a_whole_universe_round_trips_through_close_move_and_reopen() -> TestResult {
    let sandbox = tempfile::tempdir()?;
    let root = sandbox.path().join("Aurea.ontara");
    let mut project = Project::create(&root, "Aurea", T0)?;

    let cast = project.edit(|editor| {
        // Entities of many types, most of them with names in other languages and periods.
        let new = |editor: &mut ontara_core::ModelEditor<'_, _>, kind: &str, name: &str| {
            editor
                .add_entity(Entity::new(key(kind), name, T0))
                .map(|entity| entity.id)
        };
        let ayla = new(editor, "character", "Ayla")?;
        let orrin = new(editor, "character", "Orrin")?;
        let bren = new(editor, "character", "Bren")?;
        let sun = new(editor, "deity", "Solis")?;
        let elf = new(editor, "species", "Elf")?;
        let world = new(editor, "world", "Aurea")?;
        let continent = new(editor, "continent", "Velmar")?;
        let north = new(editor, "realm", "The Northern Realm")?;
        let south = new(editor, "realm", "The Southern Realm")?;
        let old_empire = new(editor, "empire", "The Old Empire")?;
        let ashmere = new(editor, "city", "Ashmere")?;
        let order = new(editor, "faction", "The Order")?;
        let guild = new(editor, "faction", "The Cartographers")?;
        let culture = new(editor, "culture", "Reedfolk")?;
        let sylvan = new(editor, "language", "Sylvan")?;
        let old_tongue = new(editor, "language", "Old Tongue")?;
        let faith = new(editor, "religion", "The Red Moon")?;
        let sword = new(editor, "item", "Dawnbreaker")?;
        let fall = new(editor, "event", "The Fall of the Citadel")?;
        let war = new(editor, "event", "The War of Ash")?;
        let book = new(editor, "volume", "Book One")?;
        let chapter = new(editor, "chapter", "Chapter 1")?;
        let portrait = new(editor, "asset", "Ayla portrait")?;

        // A being: profile, how she looks over time, marks, traits, languages.
        let mut profile = BeingProfile::new(ayla);
        profile.gender = Some("female".to_owned());
        profile.species = Some(elf);
        profile.birth = Some(Instant::told("the winter the river froze"));
        profile.birthplace = Some(ashmere);
        editor.set_being_profile(profile)?;
        let mut young = BeingAppearance::new(ayla, T0);
        young.label = Some("Young".to_owned());
        young.hair_color = Some("black".to_owned());
        young.eye_color = Some("violet".to_owned());
        young.height_cm = Some(172.0);
        young.period = Temporal::Span {
            start: None,
            end: Some(Instant::at_day(140)),
        };
        editor.add_being_appearance(young)?;
        let mut aged = BeingAppearance::new(ayla, T0);
        aged.label = Some("Aged".to_owned());
        aged.hair_color = Some("white".to_owned());
        aged.period = span(140, None);
        editor.add_being_appearance(aged)?;
        let mut scar = BeingMark::new(ayla, "A burn across the left palm", T0);
        scar.acquired = Some(Instant::told("the night the library burned"));
        editor.add_being_mark(scar)?;
        let mut fear = BeingTrait::new(ayla, key("fear"), "Open water", T0);
        fear.intensity = Some(4);
        editor.add_being_trait(fear)?;
        editor.add_being_trait(BeingTrait::new(
            ayla,
            key("quirk"),
            "Counts steps aloud",
            T0,
        ))?;
        editor.set_being_language(BeingLanguage::new(ayla, sylvan, Proficiency::Native))?;
        editor.set_deity_profile(DeityProfile::new(sun))?;
        editor.add_deity_domain(DeityDomain::new(sun, "Light"))?;
        editor.set_species_profile(SpeciesProfile::new(elf))?;
        editor.set_species_habitat(SpeciesHabitat::new(elf, ashmere))?;

        // What has no table of its own: a typed relationship and traits an author defines.
        let mut mentor = ontara_core::Relationship::new(key("mentor_of"), orrin, ayla, T0);
        mentor.qualifier = Some("Taught her to read maps".to_owned());
        mentor.period = span(60, Some(100));
        editor.add_relationship(mentor)?;
        let mut occupation = ontara_core::PropertyValue::new(
            ontara_core::PropertyValueOwner::Entity(ayla),
            ontara_core::PropertyDefinitionId::builtin("being", "occupation"),
            ontara_core::PropertyContent::Text("Cartographer".to_owned()),
            T0,
        );
        occupation.valid_from = Some(Instant::at_day(110));
        editor.add_property_value(occupation)?;
        editor.add_property_value(ontara_core::PropertyValue::new(
            ontara_core::PropertyValueOwner::Entity(ayla),
            ontara_core::PropertyDefinitionId::builtin("being", "death"),
            ontara_core::PropertyContent::Date(Instant::told("in her sleep, they say")),
            T0,
        ))?;

        // Family and partnership.
        editor.add_family_tie(FamilyTie::new(orrin, ayla, key("biological"), T0))?;
        editor.add_family_tie(FamilyTie::new(ayla, bren, key("adoptive"), T0))?;
        let mut marriage = Partnership::new(ayla, orrin, key("marriage"), T0);
        marriage.period = span(100, Some(120));
        editor.add_partnership(marriage)?;

        // Places: profile, nesting over time, borders, resources and census.
        let mut place = PlaceProfile::new(ashmere);
        place.climate = Some("Cold and dry".to_owned());
        editor.set_place_profile(place)?;
        for (inner, outer) in [
            (continent, world),
            (north, continent),
            (south, continent),
            (old_empire, continent),
        ] {
            editor.add_place_containment(PlaceContainment::new(
                inner,
                outer,
                key("political"),
                T0,
            ))?;
        }
        let mut was = PlaceContainment::new(ashmere, old_empire, key("political"), T0);
        was.period = Temporal::Span {
            start: None,
            end: Some(Instant::at_day(100)),
        };
        editor.add_place_containment(was)?;
        let mut now = PlaceContainment::new(ashmere, north, key("political"), T0);
        now.period = span(100, None);
        editor.add_place_containment(now)?;
        editor.add_place_border(PlaceBorder::new(north, south, BorderKind::River, T0))?;
        editor.add_place_resource(PlaceResource::new(ashmere, "Silver"))?;
        let mut count = Census::new(ashmere, Instant::at_day(120), 9_500, T0);
        count.is_estimate = true;
        editor.add_census(count)?;

        // Polities: profile, languages, symbols, succession, offices and tenures, citizens.
        let mut polity = PolityProfile::new(north);
        polity.government_type = Some("Monarchy".to_owned());
        polity.capital = Some(ashmere);
        polity.state_religion = Some(faith);
        editor.set_polity_profile(polity)?;
        editor.set_polity_language(PolityLanguage::new(
            north,
            sylvan,
            PolityLanguageStatus::Official,
        ))?;
        editor.add_polity_symbol(PolitySymbol::new(
            north,
            PolitySymbolKind::Flag,
            "A white tower on blue",
        ))?;
        editor.add_polity_succession(PolitySuccession::new(
            old_empire,
            north,
            PolitySuccessionKind::Split,
            T0,
        ))?;
        let mut throne = Office::new(north, "King of the North", T0);
        throne.is_head = true;
        let throne = editor.add_office(throne)?;
        let mut first = OfficeTenure::new(throne.id, orrin, T0);
        first.acquisition = Some(key("inheritance"));
        first.period = span(100, Some(140));
        editor.add_office_tenure(first)?;
        let mut second = OfficeTenure::new(throne.id, ayla, T0);
        second.acquisition = Some(key("usurpation"));
        second.period = span(140, None);
        editor.add_office_tenure(second)?;
        let mut citizen = Citizenship::new(ayla, north, key("citizen"), T0);
        citizen.period = span(110, None);
        editor.add_citizenship(citizen)?;
        let mut stance = DiplomaticRelation::new(north, south, key("vassalage"), T0);
        stance.senior = Some(north);
        editor.add_diplomatic_relation(stance)?;

        // Organizations, culture and belief.
        editor.set_faction_profile(FactionProfile::new(order))?;
        editor.add_faction_parent(FactionParent::new(guild, order, T0))?;
        let mut membership = Membership::new(order, ayla, T0);
        membership.role = Some("Cartographer".to_owned());
        membership.period = span(120, Some(150));
        editor.add_membership(membership)?;
        editor.add_culture_practice(CulturePractice::new(
            culture,
            CulturePracticeKind::Festival,
            "The Night of Lanterns",
            T0,
        ))?;
        editor.set_culture_language(CultureLanguage::new(culture, sylvan))?;
        let mut language = LanguageProfile::new(sylvan);
        language.parent_language = Some(old_tongue);
        editor.set_language_profile(language)?;
        editor.set_religion_profile(ReligionProfile::new(faith))?;
        editor.add_religion_tenet(ReligionTenet::new(
            faith,
            TenetKind::Taboo,
            "Never speak the moon's name",
            T0,
        ))?;
        editor.set_religion_deity(ReligionDeity::new(faith, sun))?;
        editor.set_holy_place(HolyPlace::new(faith, ashmere))?;
        editor.add_adherence(Adherence::new(ayla, faith, T0))?;

        // Items and events.
        editor.set_item_profile(ItemProfile::new(sword))?;
        editor.add_item_power(ItemPower::new(sword, "Burns without fuel", T0))?;
        let mut owned = ItemOwnership::new(sword, ayla, T0);
        owned.acquisition = Some(key("inherited"));
        owned.period = span(140, None);
        editor.add_item_ownership(owned)?;
        let mut detail = EventDetail::new(fall);
        detail.period = Temporal::At {
            instant: Instant::at_day(150),
        };
        detail.location = Some(ashmere);
        detail.participants.push(EventParticipant::new(ayla));
        editor.set_event_detail(detail)?;
        editor.add_event_causality(EventCausality::new(war, fall, CausalityKind::Direct, T0))?;

        // Narrative, assets, sources and notes.
        editor.place_editorial_node(EditorialNode {
            entity: book,
            level: key("volume"),
            parent: None,
            position: 0,
        })?;
        editor.place_editorial_node(EditorialNode {
            entity: chapter,
            level: key("chapter"),
            parent: Some(book),
            position: 0,
        })?;
        let mut told = Appearance::new(ayla, chapter, key("physical_presence"), T0);
        told.viewpoint = Some(ayla);
        editor.add_appearance(told)?;
        editor.register_asset_file(AssetFile {
            entity: portrait,
            relative_path: "assets/characters/ayla.png".to_owned(),
            media_type: Some("image/png".to_owned()),
            size_bytes: 2_048,
            content_hash: Some("sha256:abc".to_owned()),
            imported_at: T0,
        })?;
        editor.attach_asset(Attachment::new(ayla, portrait, T0))?;
        let manuscript =
            editor.add_source(Source::new(SourceKind::Manuscript, "Book One draft", T0))?;
        let mut citation = SourceReference::new(
            manuscript.id,
            RecordRef::new(RecordKind::Entity, ayla.as_uuid()),
            T0,
        );
        citation.locator = Some("p. 14".to_owned());
        editor.cite(citation)?;
        editor.add_note(Note::new(
            ayla,
            "Backstory",
            "Born under a **red moon**.",
            T0,
        ))?;
        editor.change_canon_state(
            RecordRef::new(RecordKind::Entity, bren.as_uuid()),
            key("provisional"),
            Some("Only in the second draft.".to_owned()),
            Origin::User,
            T1,
        )?;
        let _ = CalendarDate {
            year: 0,
            month: 1,
            day: 1,
        };
        Ok(Cast {
            ayla,
            orrin,
            north,
            ashmere,
            chapter,
        })
    })?;

    let before = snapshot(&project)?;
    assert!(
        before.iter().all(|table| table != "[]"),
        "every table holds something: {before:#?}"
    );
    assert!(project.check_integrity()?.is_sound());
    project.close()?;

    // Move the whole folder somewhere else and reopen.
    let moved = sandbox.path().join("elsewhere").join("Aurea Moved.ontara");
    fs::create_dir_all(moved.parent().ok_or("a parent exists")?)?;
    fs::rename(&root, &moved)?;
    let reopened = Project::open(&moved, MigrationPolicy::Refuse, T1)?;

    assert_eq!(
        before,
        snapshot(&reopened)?,
        "nothing changes across close, move and reopen"
    );
    assert!(reopened.check_integrity()?.is_sound());

    // What the model answers without re-deriving anything.
    let reader = reopened.reader();
    assert_eq!(
        reader
            .places_containing(&cast.ashmere, &key("political"))?
            .len(),
        3,
        "the containment that ended with the old empire is history, so it is not followed"
    );
    assert_eq!(reader.appearances_in(&cast.chapter)?.len(), 1);
    let tenures = ontara_core::Reader::<OfficeTenure>::list(&reader)?;
    assert_eq!(
        tenures
            .iter()
            .filter(|tenure| tenure.holder == cast.orrin)
            .count(),
        1
    );
    assert_eq!(
        reader.entity(&cast.north)?.map(|entity| entity.name),
        Some("The Northern Realm".to_owned())
    );
    assert!(reader.entity(&cast.ayla)?.is_some());
    Ok(())
}
