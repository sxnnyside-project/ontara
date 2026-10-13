//! The domain rules, exercised through a real project.

#![allow(clippy::expect_used)] // Fixtures use literal type keys that are known to be valid.

use std::error::Error;
use std::fs;

use ontara_core::{
    Appearance, AssetFile, Attachment, BorderKind, Calendar, CalendarId, CanonState,
    Directionality, DomainError, EditError, EditorialNode, Entity, EntityId, EntityRole,
    EntityType, EventDetail, EventParticipant, Instant, ModelQueries, ModelReaderExt, Note, Office,
    OfficeTenure, Origin, PlaceBorder, PlaceContainment, PropertyContent, PropertyDataType,
    PropertyDefinition, PropertyDefinitionId, PropertyOwner, PropertyValue, PropertyValueOwner,
    RecordKind, RecordRef, Relationship, RelationshipType, RemovalStrategy, Source, SourceKind,
    SourceReference, Temporal, Timestamp, TypeKey, Vocabulary, VocabularyTerm,
};
use ontara_storage::{Project, StorageError};

type TestResult = Result<(), Box<dyn Error>>;
type EditResult<T> = Result<T, EditError<StorageError>>;

const T0: Timestamp = Timestamp::from_unix_millis(1_000);
const T1: Timestamp = Timestamp::from_unix_millis(2_000);

fn key(raw: &str) -> TypeKey {
    TypeKey::new(raw).expect("literal test keys are valid")
}

fn project() -> Result<(tempfile::TempDir, Project), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let project = Project::create(&directory.path().join("Aurea.ontara"), "Aurea", T0)?;
    Ok((directory, project))
}

fn domain<T: std::fmt::Debug>(result: EditResult<T>) -> DomainError {
    match result {
        Err(EditError::Domain(error)) => error,
        other => panic!("expected a domain error, got {other:?}"),
    }
}

fn add(project: &mut Project, entity_type: &str, name: &str) -> Result<EntityId, Box<dyn Error>> {
    let id = project.edit(|editor| {
        Ok(editor
            .add_entity(Entity::new(key(entity_type), name, T0))?
            .id)
    })?;
    Ok(id)
}

fn builtin(owner: &str, property: &str) -> PropertyDefinitionId {
    PropertyDefinitionId::builtin(owner, property)
}

// ---- entities and types

#[test]
fn entities_need_a_defined_type_and_canon_state() -> TestResult {
    let (_directory, mut project) = project()?;

    let unknown_type =
        project.edit(|editor| editor.add_entity(Entity::new(key("dragon"), "Smaug", T0)));
    assert!(matches!(
        domain(unknown_type),
        DomainError::NotDefined {
            what: "entity type",
            ..
        }
    ));

    let mut entity = Entity::new(key("character"), "Ayla", T0);
    entity.canon_state = key("apocryphal");
    let unknown_state = project.edit(|editor| editor.add_entity(entity));
    assert!(matches!(
        domain(unknown_state),
        DomainError::NotDefined {
            what: "canon state",
            ..
        }
    ));
    assert!(project.reader().entities()?.is_empty());
    Ok(())
}

#[test]
fn abstract_types_group_subtypes_but_cannot_be_instantiated() -> TestResult {
    let (_directory, mut project) = project()?;

    let error =
        project.edit(|editor| editor.add_entity(Entity::new(key("settlement"), "Somewhere", T0)));

    assert!(matches!(domain(error), DomainError::AbstractType(_)));
    add(&mut project, "city", "Ashmere")?;
    Ok(())
}

#[test]
fn a_project_can_define_its_own_canon_state_and_use_it() -> TestResult {
    let (_directory, mut project) = project()?;

    project.edit(|editor| {
        editor.define_canon_state(CanonState::custom(
            key("alternate_timeline"),
            "Alternate timeline",
            "",
            false,
        ))?;
        let mut entity = Entity::new(key("character"), "Ayla", T0);
        entity.canon_state = key("alternate_timeline");
        editor.add_entity(entity)?;
        Ok(())
    })?;
    Ok(())
}

#[test]
fn updating_keeps_identity_type_origin_and_creation_time() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let mut edited = project.reader().entity(&ayla)?.ok_or("Ayla must exist")?;
    edited.name = "Ayla the Wanderer".to_owned();
    edited.created_at = T1;
    edited.origin = Origin::Ai;

    let saved = project.edit(|editor| editor.update_entity(edited, T1))?;

    assert_eq!(
        (saved.id, saved.created_at, saved.updated_at, saved.origin),
        (ayla, T0, T1, Origin::User)
    );
    let mut retyped = saved;
    retyped.entity_type = key("deity");
    let error = project.edit(|editor| editor.update_entity(retyped, T1));
    assert_eq!(domain(error), DomainError::Immutable("entity type"));
    Ok(())
}

#[test]
fn names_keep_their_kind_language_and_period() -> TestResult {
    let (_directory, mut project) = project()?;
    let sylvan = add(&mut project, "language", "Sylvan")?;
    let mut ayla = Entity::new(key("character"), "Ayla", T0);
    let mut former =
        ontara_core::EntityName::new("Ayla of the Reeds", ontara_core::NameKind::FormerName);
    former.language = Some(sylvan);
    former.period = Temporal::Span {
        start: None,
        end: Some(Instant::told("before the crowning")),
    };
    ayla.names.push(former.clone());
    let id = ayla.id;
    project.edit(|editor| editor.add_entity(ayla))?;

    let stored = project.reader().entity(&id)?.ok_or("must exist")?;

    assert_eq!(stored.names, vec![former]);
    Ok(())
}

// ---- relationships

#[test]
fn relationship_types_can_restrict_what_they_connect_including_subtypes() -> TestResult {
    let (_directory, mut project) = project()?;
    let mut rule =
        RelationshipType::custom(key("sworn_to"), "Is sworn to", Directionality::Directed);
    rule.allowed_sources.insert(key("being"));
    rule.allowed_targets.insert(key("faction"));
    project.edit(|editor| editor.define_relationship_type(rule))?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let order = add(&mut project, "faction", "The Order")?;
    let citadel = add(&mut project, "city", "Ashmere")?;

    project.edit(|editor| {
        editor.add_relationship(Relationship::new(key("sworn_to"), ayla, order, T0))
    })?;
    let error = project.edit(|editor| {
        editor.add_relationship(Relationship::new(key("sworn_to"), ayla, citadel, T0))
    });

    assert!(matches!(
        domain(error),
        DomainError::EndpointNotAllowed { side: "target", .. }
    ));
    Ok(())
}

#[test]
fn hierarchical_relationship_types_cannot_loop() -> TestResult {
    let (_directory, mut project) = project()?;
    let first = add(&mut project, "character", "Ayla")?;
    let second = add(&mut project, "character", "Ayla Reborn")?;
    project.edit(|editor| {
        editor.add_relationship(Relationship::new(
            key("reincarnation_of"),
            second,
            first,
            T0,
        ))
    })?;

    let error = project.edit(|editor| {
        editor.add_relationship(Relationship::new(
            key("reincarnation_of"),
            first,
            second,
            T0,
        ))
    });

    assert!(matches!(
        domain(error),
        DomainError::WouldCreateCycle { .. }
    ));
    Ok(())
}

// ---- properties

#[test]
fn a_character_and_a_deity_inherit_the_traits_of_a_being() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let sun = add(&mut project, "deity", "Solis")?;
    let keep = add(&mut project, "stronghold", "The Keep")?;
    let eyes = builtin("being", "eye_color");
    let value = |owner: EntityId| {
        PropertyValue::new(
            PropertyValueOwner::Entity(owner),
            eyes,
            PropertyContent::Text("violet".to_owned()),
            T0,
        )
    };

    project.edit(|editor| {
        editor.add_property_value(value(ayla))?;
        editor.add_property_value(value(sun))?;
        Ok(())
    })?;
    let on_a_place = project.edit(|editor| editor.add_property_value(value(keep)));

    assert!(matches!(
        domain(on_a_place),
        DomainError::InvalidProperty { .. }
    ));
    Ok(())
}

#[test]
fn a_trait_changes_over_time_when_each_value_has_a_period() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let owner = PropertyValueOwner::Entity(ayla);
    let hair = builtin("being", "hair_color");
    let value = |text: &str, from: Option<i64>, until: Option<i64>| {
        let mut value = PropertyValue::new(owner, hair, PropertyContent::Text(text.to_owned()), T0);
        value.valid_from = from.map(Instant::at_day);
        value.valid_until = until.map(Instant::at_day);
        value
    };

    project.edit(|editor| editor.add_property_value(value("black", None, Some(100))))?;
    let unbounded = project.edit(|editor| editor.add_property_value(value("white", None, None)));
    assert!(matches!(
        domain(unbounded),
        DomainError::InvalidProperty { .. }
    ));
    project.edit(|editor| editor.add_property_value(value("white", Some(101), None)))?;
    Ok(())
}

#[test]
fn property_values_are_checked_against_their_definition() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let elf = add(&mut project, "species", "Elf")?;
    let keep = add(&mut project, "stronghold", "The Keep")?;
    let owner = PropertyValueOwner::Entity(ayla);

    let wrong_type = PropertyValue::new(
        owner,
        builtin("being", "occupation"),
        PropertyContent::Integer(3),
        T0,
    );
    assert!(matches!(
        domain(project.edit(|editor| editor.add_property_value(wrong_type))),
        DomainError::InvalidProperty { .. }
    ));
    let wrong_reference = PropertyValue::new(
        owner,
        builtin("character", "species"),
        PropertyContent::EntityReference(keep),
        T0,
    );
    assert!(matches!(
        domain(project.edit(|editor| editor.add_property_value(wrong_reference))),
        DomainError::InvalidProperty { .. }
    ));
    let right = PropertyValue::new(
        owner,
        builtin("character", "species"),
        PropertyContent::EntityReference(elf),
        T0,
    );
    project.edit(|editor| editor.add_property_value(right))?;
    Ok(())
}

#[test]
fn a_choice_property_only_accepts_its_options() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let allegiance = project.edit(|editor| {
        editor.define_property_definition(PropertyDefinition::custom(
            PropertyOwner::EntityType(key("character")),
            key("allegiance"),
            "Allegiance",
            PropertyDataType::Choice {
                options: vec![
                    ontara_core::ChoiceOption {
                        key: key("light"),
                        label: "Light".to_owned(),
                    },
                    ontara_core::ChoiceOption {
                        key: key("dark"),
                        label: "Dark".to_owned(),
                    },
                ],
            },
        ))
    })?;
    let owner = PropertyValueOwner::Entity(ayla);

    let bad = PropertyValue::new(
        owner,
        allegiance.id,
        PropertyContent::Choice(key("grey")),
        T0,
    );
    assert!(matches!(
        domain(project.edit(|editor| editor.add_property_value(bad))),
        DomainError::InvalidProperty { .. }
    ));
    let good = PropertyValue::new(
        owner,
        allegiance.id,
        PropertyContent::Choice(key("dark")),
        T0,
    );
    project.edit(|editor| editor.add_property_value(good))?;
    let reloaded = project.reader().property_definitions()?;
    let stored = reloaded
        .iter()
        .find(|d| d.id == allegiance.id)
        .ok_or("must exist")?;
    assert_eq!(
        stored.data_type, allegiance.data_type,
        "choice options survive storage"
    );
    Ok(())
}

#[test]
fn a_property_key_cannot_be_declared_twice_along_a_type_hierarchy() -> TestResult {
    let (_directory, mut project) = project()?;

    let clash = PropertyDefinition::custom(
        PropertyOwner::EntityType(key("character")),
        key("hair_color"),
        "Hair color",
        PropertyDataType::Text,
    );
    let error = project.edit(|editor| editor.define_property_definition(clash));

    assert!(matches!(domain(error), DomainError::AlreadyDefined { .. }));
    Ok(())
}

// ---- the relational facts

#[test]
fn places_nest_by_kind_and_cannot_loop_but_history_does_not_block() -> TestResult {
    let (_directory, mut project) = project()?;
    let world = add(&mut project, "world", "Aurea")?;
    let continent = add(&mut project, "continent", "Velmar")?;
    let realm = add(&mut project, "realm", "The Northern Realm")?;
    let city = add(&mut project, "city", "Ashmere")?;
    project.edit(|editor| {
        for (inner, outer) in [(city, realm), (realm, continent), (continent, world)] {
            editor.add_place_containment(PlaceContainment::new(
                inner,
                outer,
                key("political"),
                T0,
            ))?;
        }
        Ok(())
    })?;

    let chain = project
        .reader()
        .places_containing(&city, &key("political"))?;
    assert_eq!(chain, vec![realm, continent, world]);
    assert_eq!(
        project
            .reader()
            .places_within(&world, &key("political"))?
            .len(),
        3
    );

    let loop_back = project.edit(|editor| {
        editor.add_place_containment(PlaceContainment::new(world, city, key("political"), T0))
    });
    assert!(matches!(
        domain(loop_back),
        DomainError::WouldCreateCycle { .. }
    ));
    // A different kind of containment is a different hierarchy.
    project.edit(|editor| {
        editor.add_place_containment(PlaceContainment::new(world, city, key("religious"), T0))
    })?;
    Ok(())
}

#[test]
fn a_containment_that_ended_does_not_block_a_later_one() -> TestResult {
    let (_directory, mut project) = project()?;
    let old = add(&mut project, "realm", "The Old Realm")?;
    let new = add(&mut project, "realm", "The New Realm")?;
    let mut was = PlaceContainment::new(old, new, key("political"), T0);
    was.period = Temporal::Span {
        start: None,
        end: Some(Instant::at_day(100)),
    };
    project.edit(|editor| editor.add_place_containment(was))?;

    project.edit(|editor| {
        editor.add_place_containment(PlaceContainment::new(new, old, key("political"), T0))
    })?;
    Ok(())
}

#[test]
fn family_ties_cannot_make_someone_their_own_ancestor() -> TestResult {
    let (_directory, mut project) = project()?;
    let parent = add(&mut project, "character", "Orrin")?;
    let child = add(&mut project, "character", "Ayla")?;
    let grandchild = add(&mut project, "character", "Bren")?;
    project.edit(|editor| {
        editor.add_family_tie(ontara_core::FamilyTie::new(
            parent,
            child,
            key("biological"),
            T0,
        ))?;
        editor.add_family_tie(ontara_core::FamilyTie::new(
            child,
            grandchild,
            key("adoptive"),
            T0,
        ))?;
        Ok(())
    })?;

    let error = project.edit(|editor| {
        editor.add_family_tie(ontara_core::FamilyTie::new(
            grandchild,
            parent,
            key("biological"),
            T0,
        ))
    });

    assert!(matches!(
        domain(error),
        DomainError::WouldCreateCycle { .. }
    ));
    Ok(())
}

#[test]
fn symmetric_facts_are_stored_once_in_order() -> TestResult {
    let (_directory, mut project) = project()?;
    let a = add(&mut project, "realm", "North")?;
    let b = add(&mut project, "realm", "South")?;
    let (low, high) = if a < b { (a, b) } else { (b, a) };

    let border = project.edit(|editor| {
        editor.add_place_border(PlaceBorder::new(high, low, BorderKind::River, T0))
    })?;

    assert_eq!((border.first, border.second), (low, high));
    Ok(())
}

#[test]
fn rule_of_a_polity_is_a_sequence_of_tenures_of_an_office() -> TestResult {
    let (_directory, mut project) = project()?;
    let realm = add(&mut project, "realm", "The Northern Realm")?;
    let orrin = add(&mut project, "character", "Orrin")?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let office = project.edit(|editor| {
        let mut throne = Office::new(realm, "King of the North", T0);
        throne.is_head = true;
        editor.add_office(throne)
    })?;

    project.edit(|editor| {
        let mut first = OfficeTenure::new(office.id, orrin, T0);
        first.acquisition = Some(key("inheritance"));
        first.period = Temporal::Span {
            start: Some(Instant::at_day(100)),
            end: Some(Instant::at_day(140)),
        };
        editor.add_office_tenure(first)?;
        let mut second = OfficeTenure::new(office.id, ayla, T0);
        second.acquisition = Some(key("usurpation"));
        second.period = Temporal::Span {
            start: Some(Instant::at_day(140)),
            end: None,
        };
        editor.add_office_tenure(second)?;
        Ok(())
    })?;

    let tenures = ontara_core::Reader::<OfficeTenure>::list(&project.reader())?;
    assert_eq!(tenures.len(), 2);
    let missing_term = project.edit(|editor| {
        let mut tenure = OfficeTenure::new(office.id, ayla, T0);
        tenure.acquisition = Some(key("divine_lottery"));
        editor.add_office_tenure(tenure)
    });
    assert!(matches!(
        domain(missing_term),
        DomainError::NotDefined { .. }
    ));
    Ok(())
}

#[test]
fn a_project_can_extend_a_vocabulary_and_use_the_new_term() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    project.edit(|editor| {
        editor.define_vocabulary_term(VocabularyTerm::custom(
            Vocabulary::TraitKind,
            key("oath"),
            "Oath",
        ))?;
        editor.add_being_trait(ontara_core::BeingTrait::new(
            ayla,
            key("oath"),
            "Never lie",
            T0,
        ))?;
        Ok(())
    })?;

    let unknown = project.edit(|editor| {
        editor.add_being_trait(ontara_core::BeingTrait::new(ayla, key("hunch"), "x", T0))
    });
    assert!(matches!(domain(unknown), DomainError::NotDefined { .. }));
    Ok(())
}

#[test]
fn range_and_sign_checks_reject_nonsense() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let city = add(&mut project, "city", "Ashmere")?;

    let mut trait_ = ontara_core::BeingTrait::new(ayla, key("fear"), "Water", T0);
    trait_.intensity = Some(9);
    assert!(matches!(
        domain(project.edit(|editor| editor.add_being_trait(trait_))),
        DomainError::Invalid(_)
    ));
    let census = ontara_core::Census::new(city, Instant::told("the last winter"), -5, T0);
    assert!(matches!(
        domain(project.edit(|editor| editor.add_census(census))),
        DomainError::Invalid(_)
    ));
    Ok(())
}

// ---- narrative

#[test]
fn appearances_need_a_narrative_context_and_distinguish_absence() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let bren = add(&mut project, "character", "Bren")?;
    let chapter = add(&mut project, "chapter", "Chapter 1")?;

    let in_a_character = project
        .edit(|editor| editor.add_appearance(Appearance::new(ayla, bren, key("dialogue"), T0)));
    assert!(matches!(
        domain(in_a_character),
        DomainError::RoleMismatch { .. }
    ));

    project.edit(|editor| {
        editor.add_appearance(Appearance::new(ayla, chapter, key("physical_presence"), T0))?;
        editor.add_appearance(Appearance::new(bren, chapter, key("confirmed_absence"), T0))?;
        Ok(())
    })?;
    assert_eq!(project.reader().appearances_in(&chapter)?.len(), 2);
    Ok(())
}

#[test]
fn only_event_typed_entities_have_event_details() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let battle = add(&mut project, "event", "The Battle")?;
    let keep = add(&mut project, "stronghold", "The Keep")?;

    let on_a_character = project.edit(|editor| editor.set_event_detail(EventDetail::new(ayla)));
    assert!(matches!(
        domain(on_a_character),
        DomainError::RoleMismatch { .. }
    ));

    project.edit(|editor| {
        let mut detail = EventDetail::new(battle);
        detail.location = Some(keep);
        detail.period = Temporal::At {
            instant: Instant::at_day(500),
        };
        detail.participants.push(EventParticipant {
            entity: ayla,
            role: "Commander".to_owned(),
            side: Some("Defenders".to_owned()),
        });
        editor.set_event_detail(detail)
    })?;
    let stored = project.reader().event(&battle)?.ok_or("must exist")?;
    assert_eq!(stored.participants.len(), 1);
    assert_eq!(stored.period.start_day(), Some(500));
    Ok(())
}

#[test]
fn an_event_cannot_cause_itself_through_a_chain() -> TestResult {
    let (_directory, mut project) = project()?;
    let war = add(&mut project, "event", "The War")?;
    let siege = add(&mut project, "event", "The Siege")?;
    project.edit(|editor| {
        editor.add_event_causality(ontara_core::EventCausality::new(
            war,
            siege,
            ontara_core::CausalityKind::Direct,
            T0,
        ))
    })?;

    let error = project.edit(|editor| {
        editor.add_event_causality(ontara_core::EventCausality::new(
            siege,
            war,
            ontara_core::CausalityKind::Direct,
            T0,
        ))
    });

    assert!(matches!(
        domain(error),
        DomainError::WouldCreateCycle { .. }
    ));
    Ok(())
}

#[test]
fn the_editorial_hierarchy_respects_depth() -> TestResult {
    let (_directory, mut project) = project()?;
    let book = add(&mut project, "volume", "Book One")?;
    let chapter = add(&mut project, "chapter", "Chapter 1")?;
    let scene = add(&mut project, "scene", "Scene 1")?;
    let node = |entity: EntityId, level: &str, parent: Option<EntityId>| EditorialNode {
        entity,
        level: key(level),
        parent,
        position: 0,
    };
    project.edit(|editor| {
        editor.place_editorial_node(node(book, "volume", None))?;
        editor.place_editorial_node(node(chapter, "chapter", Some(book)))?;
        editor.place_editorial_node(node(scene, "scene", Some(chapter)))?;
        Ok(())
    })?;

    let backwards = project.edit(|editor| editor.move_editorial_node(&book, Some(chapter), 0));
    assert!(matches!(
        domain(backwards),
        DomainError::InvalidPlacement(_)
    ));
    let to_itself = project.edit(|editor| editor.move_editorial_node(&scene, Some(scene), 0));
    assert!(matches!(
        domain(to_itself),
        DomainError::InvalidPlacement(_)
    ));
    let impact = project.edit(|editor| editor.editorial_impact(&book))?;
    assert_eq!(impact.descendants, 2);
    Ok(())
}

// ---- removal, canon, assets, calendars

#[test]
fn removing_a_referenced_entity_requires_an_explicit_strategy() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let order = add(&mut project, "faction", "The Order")?;
    project.edit(|editor| editor.add_membership(ontara_core::Membership::new(order, ayla, T0)))?;

    let blocked =
        project.edit(|editor| editor.remove_entity(&ayla, RemovalStrategy::RejectIfReferenced));
    assert!(matches!(
        domain(blocked),
        DomainError::StillReferenced { references: 1, .. }
    ));
    assert_eq!(project.reader().entities()?.len(), 2);

    let removed =
        project.edit(|editor| editor.remove_entity(&ayla, RemovalStrategy::RemoveDependents))?;
    assert_eq!(
        removed
            .iter()
            .map(|reference| reference.count)
            .sum::<usize>(),
        1
    );
    assert_eq!(project.reader().entities()?.len(), 1);
    assert!(project.check_integrity()?.is_sound());
    Ok(())
}

#[test]
fn removing_with_dependents_clears_everything_that_hung_off_the_entity() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let bren = add(&mut project, "character", "Bren")?;
    let chapter = add(&mut project, "chapter", "Chapter 1")?;
    let city = add(&mut project, "city", "Ashmere")?;
    let source = project.edit(|editor| {
        editor.add_source(Source::new(SourceKind::Manuscript, "Chapter 1 draft", T0))
    })?;

    project.edit(|editor| {
        editor.add_relationship(Relationship::new(key("mentor_of"), ayla, bren, T0))?;
        editor.add_property_value(PropertyValue::new(
            PropertyValueOwner::Entity(ayla),
            builtin("being", "occupation"),
            PropertyContent::Text("Cartographer".to_owned()),
            T0,
        ))?;
        editor.add_property_value(PropertyValue::new(
            PropertyValueOwner::Entity(bren),
            builtin("being", "birthplace"),
            PropertyContent::EntityReference(city),
            T0,
        ))?;
        editor.add_appearance(Appearance::new(ayla, chapter, key("dialogue"), T0))?;
        let mut told = Appearance::new(bren, chapter, key("physical_presence"), T0);
        told.viewpoint = Some(ayla);
        editor.add_appearance(told)?;
        editor.add_being_trait(ontara_core::BeingTrait::new(ayla, key("fear"), "Water", T0))?;
        editor.add_note(Note::new(ayla, "Backstory", "Born under a red moon.", T0))?;
        editor.cite(SourceReference::new(
            source.id,
            RecordRef::new(RecordKind::Entity, ayla.as_uuid()),
            T0,
        ))?;
        Ok(())
    })?;

    project.edit(|editor| editor.remove_entity(&ayla, RemovalStrategy::RemoveDependents))?;

    let reader = project.reader();
    assert!(reader.entity(&ayla)?.is_none());
    assert!(reader.relationships()?.is_empty());
    assert_eq!(
        reader.property_values()?.len(),
        1,
        "values about others survive"
    );
    assert_eq!(reader.appearances()?.len(), 1);
    assert_eq!(
        reader.appearances()?[0].viewpoint,
        None,
        "told-by appearances survive, unattributed"
    );
    assert!(reader.notes()?.is_empty());
    assert!(reader.citations()?.is_empty());
    drop(reader);
    assert!(project.check_integrity()?.is_sound());
    Ok(())
}

#[test]
fn removing_a_polity_removes_its_offices_and_their_tenures() -> TestResult {
    let (_directory, mut project) = project()?;
    let realm = add(&mut project, "realm", "The Northern Realm")?;
    let orrin = add(&mut project, "character", "Orrin")?;
    project.edit(|editor| {
        let office = editor.add_office(Office::new(realm, "King", T0))?;
        editor.add_office_tenure(OfficeTenure::new(office.id, orrin, T0))?;
        Ok(())
    })?;

    project.edit(|editor| editor.remove_entity(&realm, RemovalStrategy::RemoveDependents))?;

    let reader = project.reader();
    assert!(ontara_core::Reader::<Office>::list(&reader)?.is_empty());
    assert!(ontara_core::Reader::<OfficeTenure>::list(&reader)?.is_empty());
    assert_eq!(
        reader.entity(&orrin)?.map(|e| e.name),
        Some("Orrin".to_owned())
    );
    Ok(())
}

#[test]
fn changing_canon_state_keeps_reason_and_history() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let target = RecordRef::new(RecordKind::Entity, ayla.as_uuid());

    let change = project.edit(|editor| {
        editor.change_canon_state(
            target.clone(),
            key("disestablished"),
            Some("Retconned in the second edition.".to_owned()),
            Origin::User,
            T1,
        )
    })?;

    let change = change.ok_or("the state changed")?;
    assert_eq!(
        (change.from, change.to),
        (key("established"), key("disestablished"))
    );
    assert_eq!(project.reader().canon_history(&target)?.len(), 1);
    let unchanged = project.edit(|editor| {
        editor.change_canon_state(
            target.clone(),
            key("disestablished"),
            None,
            Origin::User,
            T1,
        )
    })?;
    assert!(unchanged.is_none());
    Ok(())
}

#[test]
fn canon_changes_work_for_the_new_fact_tables_too() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let the_trait = project.edit(|editor| {
        editor.add_being_trait(ontara_core::BeingTrait::new(
            ayla,
            key("secret"),
            "Forged the map",
            T0,
        ))
    })?;

    let change = project.edit(|editor| {
        editor.change_canon_state(
            RecordRef::new(RecordKind::BeingTrait, the_trait.id.as_uuid()),
            key("disputed"),
            None,
            Origin::Ai,
            T1,
        )
    })?;

    assert!(change.is_some());
    let stored =
        ontara_core::Reader::<ontara_core::BeingTrait>::get(&project.reader(), &the_trait.id)?;
    assert_eq!(stored.map(|t| t.canon_state), Some(key("disputed")));
    Ok(())
}

#[test]
fn asset_files_stay_inside_the_project_and_attach_without_copying() -> TestResult {
    let (directory, mut project) = project()?;
    let portrait = add(&mut project, "asset", "Ayla portrait")?;
    let ayla = add(&mut project, "character", "Ayla")?;
    let bren = add(&mut project, "character", "Bren")?;
    let file = |path: &str| AssetFile {
        entity: portrait,
        relative_path: path.to_owned(),
        media_type: Some("image/png".to_owned()),
        size_bytes: 10,
        content_hash: None,
        imported_at: T0,
    };

    let escaping = project.edit(|editor| editor.register_asset_file(file("../secret.png")));
    assert!(matches!(domain(escaping), DomainError::InvalidPath(_)));
    project.edit(|editor| {
        editor.register_asset_file(file("assets/characters/ayla.png"))?;
        editor.attach_asset(Attachment::new(ayla, portrait, T0))?;
        editor.attach_asset(Attachment::new(bren, portrait, T0))?;
        Ok(())
    })?;

    assert_eq!(project.missing_assets()?.len(), 1);
    fs::create_dir_all(directory.path().join("Aurea.ontara/assets/characters"))?;
    fs::write(
        directory
            .path()
            .join("Aurea.ontara/assets/characters/ayla.png"),
        b"png",
    )?;
    assert!(project.missing_assets()?.is_empty());
    Ok(())
}

#[test]
fn a_new_default_calendar_demotes_the_previous_one() -> TestResult {
    let (_directory, mut project) = project()?;
    let mut second = Calendar::new("Second", ontara_core::gregorian_definition(), T0);
    second.is_default = true;

    project.edit(|editor| editor.define_calendar(second.clone()))?;

    let calendars = project.reader().calendars()?;
    let defaults: Vec<_> = calendars.iter().filter(|c| c.is_default).collect();
    assert_eq!(defaults.len(), 1);
    assert_eq!(defaults[0].id, second.id);
    assert!(
        calendars
            .iter()
            .any(|c| c.id == CalendarId::builtin_default() && !c.is_default)
    );
    let stored = calendars
        .iter()
        .find(|c| c.id == second.id)
        .ok_or("must exist")?;
    assert_eq!(
        stored.definition, second.definition,
        "normalized calendar tables round-trip"
    );
    Ok(())
}

#[test]
fn defining_a_key_twice_is_rejected_for_every_catalog() -> TestResult {
    let (_directory, mut project) = project()?;

    let entity_type = EntityType::custom(key("character"), "Avatar", EntityRole::Generic);
    assert!(matches!(
        domain(project.edit(|editor| editor.define_entity_type(entity_type))),
        DomainError::AlreadyDefined { .. }
    ));
    let relationship_type =
        RelationshipType::custom(key("mentor_of"), "Mentor", Directionality::Directed);
    assert!(matches!(
        domain(project.edit(|editor| editor.define_relationship_type(relationship_type))),
        DomainError::AlreadyDefined { .. }
    ));
    let term = VocabularyTerm::custom(Vocabulary::TraitKind, key("fear"), "Fear");
    assert!(matches!(
        domain(project.edit(|editor| editor.define_vocabulary_term(term))),
        DomainError::AlreadyDefined { .. }
    ));
    Ok(())
}

#[test]
fn an_unplaced_moment_survives_storage_and_can_be_dated_later() -> TestResult {
    let (_directory, mut project) = project()?;
    let ayla = add(&mut project, "character", "Ayla")?;
    project.edit(|editor| {
        let mut profile = ontara_core::BeingProfile::new(ayla);
        profile.birth = Some(Instant::told("the winter the river froze"));
        editor.set_being_profile(profile)
    })?;

    let stored = ontara_core::Reader::<ontara_core::BeingProfile>::get(&project.reader(), &ayla)?
        .ok_or("must exist")?;
    let birth = stored.birth.clone().ok_or("birth was set")?;
    assert!(!birth.is_placed());
    assert_eq!(birth.label.as_deref(), Some("the winter the river froze"));

    let mut dated = stored;
    dated.birth = Some(Instant {
        day: Some(4_200),
        ..birth
    });
    project.edit(|editor| editor.set_being_profile(dated))?;
    let again = ontara_core::Reader::<ontara_core::BeingProfile>::get(&project.reader(), &ayla)?
        .ok_or("must exist")?;
    assert_eq!(again.birth.and_then(|moment| moment.day), Some(4_200));
    Ok(())
}
