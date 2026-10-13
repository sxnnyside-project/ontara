#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    BeingAppearance, BeingAppearanceId, BeingLanguage, BeingMark, BeingMarkId, BeingProfile,
    BeingTrait, BeingTraitId, DeityDomain, DeityDomainId, DeityProfile, EntityId, FamilyTie,
    FamilyTieId, Origin, Partnership, PartnershipEndReason, PartnershipId, Proficiency, Reader,
    SpeciesHabitat, SpeciesProfile, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    int, opt_bool, opt_int, opt_real, opt_text, optional_parse, parse, read_instant, read_period,
    text, timestamp, type_key,
};
use crate::error::StorageError;

pub(super) const BEING_PROFILE: &str = "SELECT t.entity_id, t.gender, t.pronouns, t.species_id, i_birth_at.id AS birth_at__id, i_birth_at.day AS birth_at__day, i_birth_at.precision AS birth_at__precision, i_birth_at.earliest_day AS birth_at__earliest, i_birth_at.latest_day AS birth_at__latest, i_birth_at.calendar_id AS birth_at__calendar, i_birth_at.label AS birth_at__label, i_death_at.id AS death_at__id, i_death_at.day AS death_at__day, i_death_at.precision AS death_at__precision, i_death_at.earliest_day AS death_at__earliest, i_death_at.latest_day AS death_at__latest, i_death_at.calendar_id AS death_at__calendar, i_death_at.label AS death_at__label, t.birthplace_id, t.upbringing, t.occupation FROM being_profile AS t LEFT JOIN instant AS i_birth_at ON i_birth_at.id = t.birth_at LEFT JOIN instant AS i_death_at ON i_death_at.id = t.death_at";

pub(super) fn being_profile(row: &Row<'_>) -> Result<BeingProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("being_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let gender: Option<String> = row.get("gender")?;
    let pronouns: Option<String> = row.get("pronouns")?;
    let species = optional_parse::<EntityId>(
        "being_profile",
        &key,
        row.get::<_, Option<String>>("species_id")?,
    )?;
    let birth = read_instant(row, "birth_at")?;
    let death = read_instant(row, "death_at")?;
    let birthplace = optional_parse::<EntityId>(
        "being_profile",
        &key,
        row.get::<_, Option<String>>("birthplace_id")?,
    )?;
    let upbringing: Option<String> = row.get("upbringing")?;
    let occupation: Option<String> = row.get("occupation")?;
    Ok(BeingProfile {
        entity,
        gender,
        pronouns,
        species,
        birth,
        death,
        birthplace,
        upbringing,
        occupation,
    })
}

impl Reader<BeingProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<BeingProfile>, StorageError> {
        self.select_one(
            &format!("{BEING_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            being_profile,
        )
    }

    fn list(&self) -> Result<Vec<BeingProfile>, StorageError> {
        self.select(
            &format!("{BEING_PROFILE} ORDER BY t.entity_id"),
            [],
            being_profile,
        )
    }
}

impl Writer<BeingProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &BeingProfile) -> Result<(), StorageError> {
        let birth_at_id = self.write_instant_opt(record.birth.as_ref())?;
        let death_at_id = self.write_instant_opt(record.death.as_ref())?;
        self.execute(
            "INSERT INTO being_profile (entity_id, gender, pronouns, species_id, birth_at, death_at, birthplace_id, upbringing, occupation) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.gender.clone()),
            opt_text(record.pronouns.clone()),
            opt_text(record.species.map(|value| value.to_string())),
            opt_text(birth_at_id),
            opt_text(death_at_id),
            opt_text(record.birthplace.map(|value| value.to_string())),
            opt_text(record.upbringing.clone()),
            opt_text(record.occupation.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &BeingProfile) -> Result<(), StorageError> {
        let birth_at_id = self.write_instant_opt(record.birth.as_ref())?;
        let death_at_id = self.write_instant_opt(record.death.as_ref())?;
        self.execute(
            "UPDATE being_profile SET gender = ?2, pronouns = ?3, species_id = ?4, birth_at = ?5, death_at = ?6, birthplace_id = ?7, upbringing = ?8, occupation = ?9 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.gender.clone()),
            opt_text(record.pronouns.clone()),
            opt_text(record.species.map(|value| value.to_string())),
            opt_text(birth_at_id),
            opt_text(death_at_id),
            opt_text(record.birthplace.map(|value| value.to_string())),
            opt_text(record.upbringing.clone()),
            opt_text(record.occupation.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM being_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const BEING_APPEARANCE: &str = "SELECT t.id, t.being_id, t.label, t.hair_color, t.hair_style, t.eye_color, t.skin_tone, t.height_cm, t.weight_kg, t.build, t.voice, t.clothing_style, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM being_appearance AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn being_appearance(row: &Row<'_>) -> Result<BeingAppearance, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<BeingAppearanceId>("being_appearance", &key, &key)?;
    let being = parse::<EntityId>("being_appearance", &key, &row.get::<_, String>("being_id")?)?;
    let label: Option<String> = row.get("label")?;
    let hair_color: Option<String> = row.get("hair_color")?;
    let hair_style: Option<String> = row.get("hair_style")?;
    let eye_color: Option<String> = row.get("eye_color")?;
    let skin_tone: Option<String> = row.get("skin_tone")?;
    let height_cm: Option<f64> = row.get("height_cm")?;
    let weight_kg: Option<f64> = row.get("weight_kg")?;
    let build: Option<String> = row.get("build")?;
    let voice: Option<String> = row.get("voice")?;
    let clothing_style: Option<String> = row.get("clothing_style")?;
    let period = read_period(row, "being_appearance", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key(
        "being_appearance",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("being_appearance", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(BeingAppearance {
        id,
        being,
        label,
        hair_color,
        hair_style,
        eye_color,
        skin_tone,
        height_cm,
        weight_kg,
        build,
        voice,
        clothing_style,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<BeingAppearance> for SqliteModel<'_> {
    fn get(&self, id: &BeingAppearanceId) -> Result<Option<BeingAppearance>, StorageError> {
        self.select_one(
            &format!("{BEING_APPEARANCE} WHERE t.id = ?1"),
            [id.to_string()],
            being_appearance,
        )
    }

    fn list(&self) -> Result<Vec<BeingAppearance>, StorageError> {
        self.select(
            &format!("{BEING_APPEARANCE} ORDER BY t.created_at, t.id"),
            [],
            being_appearance,
        )
    }
}

impl Writer<BeingAppearance> for SqliteModel<'_> {
    fn insert(&mut self, record: &BeingAppearance) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO being_appearance (id, being_id, label, hair_color, hair_style, eye_color, skin_tone, height_cm, weight_kg, build, voice, clothing_style, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            opt_text(record.label.clone()),
            opt_text(record.hair_color.clone()),
            opt_text(record.hair_style.clone()),
            opt_text(record.eye_color.clone()),
            opt_text(record.skin_tone.clone()),
            opt_real(record.height_cm),
            opt_real(record.weight_kg),
            opt_text(record.build.clone()),
            opt_text(record.voice.clone()),
            opt_text(record.clothing_style.clone()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &BeingAppearance) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE being_appearance SET being_id = ?2, label = ?3, hair_color = ?4, hair_style = ?5, eye_color = ?6, skin_tone = ?7, height_cm = ?8, weight_kg = ?9, build = ?10, voice = ?11, clothing_style = ?12, time_state = ?13, starts_at = ?14, ends_at = ?15, note = ?16, canon_state = ?17, updated_at = ?18 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            opt_text(record.label.clone()),
            opt_text(record.hair_color.clone()),
            opt_text(record.hair_style.clone()),
            opt_text(record.eye_color.clone()),
            opt_text(record.skin_tone.clone()),
            opt_real(record.height_cm),
            opt_real(record.weight_kg),
            opt_text(record.build.clone()),
            opt_text(record.voice.clone()),
            opt_text(record.clothing_style.clone()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &BeingAppearanceId) -> Result<(), StorageError> {
        self.cascade("being_appearance", &id.to_string())
    }
}

pub(super) const BEING_MARK: &str = "SELECT t.id, t.being_id, t.description, t.body_location, i_acquired_at.id AS acquired_at__id, i_acquired_at.day AS acquired_at__day, i_acquired_at.precision AS acquired_at__precision, i_acquired_at.earliest_day AS acquired_at__earliest, i_acquired_at.latest_day AS acquired_at__latest, i_acquired_at.calendar_id AS acquired_at__calendar, i_acquired_at.label AS acquired_at__label, t.canon_state, t.origin, t.created_at, t.updated_at FROM being_mark AS t LEFT JOIN instant AS i_acquired_at ON i_acquired_at.id = t.acquired_at";

pub(super) fn being_mark(row: &Row<'_>) -> Result<BeingMark, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<BeingMarkId>("being_mark", &key, &key)?;
    let being = parse::<EntityId>("being_mark", &key, &row.get::<_, String>("being_id")?)?;
    let description: String = row.get("description")?;
    let body_location: Option<String> = row.get("body_location")?;
    let acquired = read_instant(row, "acquired_at")?;
    let canon_state = type_key("being_mark", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("being_mark", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(BeingMark {
        id,
        being,
        description,
        body_location,
        acquired,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<BeingMark> for SqliteModel<'_> {
    fn get(&self, id: &BeingMarkId) -> Result<Option<BeingMark>, StorageError> {
        self.select_one(
            &format!("{BEING_MARK} WHERE t.id = ?1"),
            [id.to_string()],
            being_mark,
        )
    }

    fn list(&self) -> Result<Vec<BeingMark>, StorageError> {
        self.select(
            &format!("{BEING_MARK} ORDER BY t.created_at, t.id"),
            [],
            being_mark,
        )
    }
}

impl Writer<BeingMark> for SqliteModel<'_> {
    fn insert(&mut self, record: &BeingMark) -> Result<(), StorageError> {
        let acquired_at_id = self.write_instant_opt(record.acquired.as_ref())?;
        self.execute(
            "INSERT INTO being_mark (id, being_id, description, body_location, acquired_at, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            text(&record.description),
            opt_text(record.body_location.clone()),
            opt_text(acquired_at_id),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &BeingMark) -> Result<(), StorageError> {
        let acquired_at_id = self.write_instant_opt(record.acquired.as_ref())?;
        self.execute(
            "UPDATE being_mark SET being_id = ?2, description = ?3, body_location = ?4, acquired_at = ?5, canon_state = ?6, updated_at = ?7 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            text(&record.description),
            opt_text(record.body_location.clone()),
            opt_text(acquired_at_id),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &BeingMarkId) -> Result<(), StorageError> {
        self.cascade("being_mark", &id.to_string())
    }
}

pub(super) const BEING_TRAIT: &str = "SELECT t.id, t.being_id, t.kind, t.text, t.intensity, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM being_trait AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn being_trait(row: &Row<'_>) -> Result<BeingTrait, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<BeingTraitId>("being_trait", &key, &key)?;
    let being = parse::<EntityId>("being_trait", &key, &row.get::<_, String>("being_id")?)?;
    let kind = type_key("being_trait", &key, &row.get::<_, String>("kind")?)?;
    let text: String = row.get("text")?;
    let intensity: Option<i32> = row.get("intensity")?;
    let period = read_period(row, "being_trait", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("being_trait", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("being_trait", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(BeingTrait {
        id,
        being,
        kind,
        text,
        intensity,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<BeingTrait> for SqliteModel<'_> {
    fn get(&self, id: &BeingTraitId) -> Result<Option<BeingTrait>, StorageError> {
        self.select_one(
            &format!("{BEING_TRAIT} WHERE t.id = ?1"),
            [id.to_string()],
            being_trait,
        )
    }

    fn list(&self) -> Result<Vec<BeingTrait>, StorageError> {
        self.select(
            &format!("{BEING_TRAIT} ORDER BY t.created_at, t.id"),
            [],
            being_trait,
        )
    }
}

impl Writer<BeingTrait> for SqliteModel<'_> {
    fn insert(&mut self, record: &BeingTrait) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO being_trait (id, being_id, kind, text, intensity, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            text(record.kind.as_str()),
            text(&record.text),
            opt_int(record.intensity.map(i64::from)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &BeingTrait) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE being_trait SET being_id = ?2, kind = ?3, text = ?4, intensity = ?5, time_state = ?6, starts_at = ?7, ends_at = ?8, note = ?9, canon_state = ?10, updated_at = ?11 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            text(record.kind.as_str()),
            text(&record.text),
            opt_int(record.intensity.map(i64::from)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &BeingTraitId) -> Result<(), StorageError> {
        self.cascade("being_trait", &id.to_string())
    }
}

pub(super) const BEING_LANGUAGE: &str =
    "SELECT t.being_id, t.language_id, t.proficiency, t.note FROM being_language AS t";

pub(super) fn being_language(row: &Row<'_>) -> Result<BeingLanguage, StorageError> {
    let key: String = row.get("being_id")?;
    let being = parse::<EntityId>("being_language", &key, &row.get::<_, String>("being_id")?)?;
    let language = parse::<EntityId>(
        "being_language",
        &key,
        &row.get::<_, String>("language_id")?,
    )?;
    let proficiency = parse::<Proficiency>(
        "being_language",
        &key,
        &row.get::<_, String>("proficiency")?,
    )?;
    let note: Option<String> = row.get("note")?;
    Ok(BeingLanguage {
        being,
        language,
        proficiency,
        note,
    })
}

impl Reader<BeingLanguage> for SqliteModel<'_> {
    fn get(&self, id: &(EntityId, EntityId)) -> Result<Option<BeingLanguage>, StorageError> {
        self.select_one(
            &format!("{BEING_LANGUAGE} WHERE t.being_id = ?1 AND t.language_id = ?2"),
            [id.0.to_string(), id.1.to_string()],
            being_language,
        )
    }

    fn list(&self) -> Result<Vec<BeingLanguage>, StorageError> {
        self.select(
            &format!("{BEING_LANGUAGE} ORDER BY t.being_id, t.language_id"),
            [],
            being_language,
        )
    }
}

impl Writer<BeingLanguage> for SqliteModel<'_> {
    fn insert(&mut self, record: &BeingLanguage) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO being_language (being_id, language_id, proficiency, note) VALUES (?1, ?2, ?3, ?4)",
            params_from_iter(vec![
            text(record.being.to_string()),
            text(record.language.to_string()),
            text(record.proficiency.as_str()),
            opt_text(record.note.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &BeingLanguage) -> Result<(), StorageError> {
        self.execute(
            "UPDATE being_language SET proficiency = ?3, note = ?4 WHERE being_id = ?1 AND language_id = ?2",
            params_from_iter(vec![
            text(record.being.to_string()),
            text(record.language.to_string()),
            text(record.proficiency.as_str()),
            opt_text(record.note.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &(EntityId, EntityId)) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM being_language WHERE being_id = ?1 AND language_id = ?2",
            [id.0.to_string(), id.1.to_string()],
        )
    }
}

pub(super) const FAMILY_TIE: &str = "SELECT t.id, t.parent_id, t.child_id, t.kind, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM family_tie AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn family_tie(row: &Row<'_>) -> Result<FamilyTie, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<FamilyTieId>("family_tie", &key, &key)?;
    let parent = parse::<EntityId>("family_tie", &key, &row.get::<_, String>("parent_id")?)?;
    let child = parse::<EntityId>("family_tie", &key, &row.get::<_, String>("child_id")?)?;
    let kind = type_key("family_tie", &key, &row.get::<_, String>("kind")?)?;
    let period = read_period(row, "family_tie", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("family_tie", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("family_tie", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(FamilyTie {
        id,
        parent,
        child,
        kind,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<FamilyTie> for SqliteModel<'_> {
    fn get(&self, id: &FamilyTieId) -> Result<Option<FamilyTie>, StorageError> {
        self.select_one(
            &format!("{FAMILY_TIE} WHERE t.id = ?1"),
            [id.to_string()],
            family_tie,
        )
    }

    fn list(&self) -> Result<Vec<FamilyTie>, StorageError> {
        self.select(
            &format!("{FAMILY_TIE} ORDER BY t.created_at, t.id"),
            [],
            family_tie,
        )
    }
}

impl Writer<FamilyTie> for SqliteModel<'_> {
    fn insert(&mut self, record: &FamilyTie) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO family_tie (id, parent_id, child_id, kind, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.parent.to_string()),
            text(record.child.to_string()),
            text(record.kind.as_str()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &FamilyTie) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE family_tie SET parent_id = ?2, child_id = ?3, kind = ?4, time_state = ?5, starts_at = ?6, ends_at = ?7, note = ?8, canon_state = ?9, updated_at = ?10 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.parent.to_string()),
            text(record.child.to_string()),
            text(record.kind.as_str()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &FamilyTieId) -> Result<(), StorageError> {
        self.cascade("family_tie", &id.to_string())
    }
}

pub(super) const PARTNERSHIP: &str = "SELECT t.id, t.first_id, t.second_id, t.kind, t.end_reason, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM partnership AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn partnership(row: &Row<'_>) -> Result<Partnership, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<PartnershipId>("partnership", &key, &key)?;
    let first = parse::<EntityId>("partnership", &key, &row.get::<_, String>("first_id")?)?;
    let second = parse::<EntityId>("partnership", &key, &row.get::<_, String>("second_id")?)?;
    let kind = type_key("partnership", &key, &row.get::<_, String>("kind")?)?;
    let end_reason = optional_parse::<PartnershipEndReason>(
        "partnership",
        &key,
        row.get::<_, Option<String>>("end_reason")?,
    )?;
    let period = read_period(row, "partnership", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("partnership", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("partnership", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(Partnership {
        id,
        first,
        second,
        kind,
        end_reason,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<Partnership> for SqliteModel<'_> {
    fn get(&self, id: &PartnershipId) -> Result<Option<Partnership>, StorageError> {
        self.select_one(
            &format!("{PARTNERSHIP} WHERE t.id = ?1"),
            [id.to_string()],
            partnership,
        )
    }

    fn list(&self) -> Result<Vec<Partnership>, StorageError> {
        self.select(
            &format!("{PARTNERSHIP} ORDER BY t.created_at, t.id"),
            [],
            partnership,
        )
    }
}

impl Writer<Partnership> for SqliteModel<'_> {
    fn insert(&mut self, record: &Partnership) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO partnership (id, first_id, second_id, kind, end_reason, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.first.to_string()),
            text(record.second.to_string()),
            text(record.kind.as_str()),
            opt_text(record.end_reason.map(PartnershipEndReason::as_str)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &Partnership) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE partnership SET first_id = ?2, second_id = ?3, kind = ?4, end_reason = ?5, time_state = ?6, starts_at = ?7, ends_at = ?8, note = ?9, canon_state = ?10, updated_at = ?11 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.first.to_string()),
            text(record.second.to_string()),
            text(record.kind.as_str()),
            opt_text(record.end_reason.map(PartnershipEndReason::as_str)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &PartnershipId) -> Result<(), StorageError> {
        self.cascade("partnership", &id.to_string())
    }
}

pub(super) const DEITY_PROFILE: &str =
    "SELECT t.entity_id, t.rank, t.alignment FROM deity_profile AS t";

pub(super) fn deity_profile(row: &Row<'_>) -> Result<DeityProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("deity_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let rank: Option<String> = row.get("rank")?;
    let alignment: Option<String> = row.get("alignment")?;
    Ok(DeityProfile {
        entity,
        rank,
        alignment,
    })
}

impl Reader<DeityProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<DeityProfile>, StorageError> {
        self.select_one(
            &format!("{DEITY_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            deity_profile,
        )
    }

    fn list(&self) -> Result<Vec<DeityProfile>, StorageError> {
        self.select(
            &format!("{DEITY_PROFILE} ORDER BY t.entity_id"),
            [],
            deity_profile,
        )
    }
}

impl Writer<DeityProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &DeityProfile) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO deity_profile (entity_id, rank, alignment) VALUES (?1, ?2, ?3)",
            params_from_iter(vec![
                text(record.entity.to_string()),
                opt_text(record.rank.clone()),
                opt_text(record.alignment.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &DeityProfile) -> Result<(), StorageError> {
        self.execute(
            "UPDATE deity_profile SET rank = ?2, alignment = ?3 WHERE entity_id = ?1",
            params_from_iter(vec![
                text(record.entity.to_string()),
                opt_text(record.rank.clone()),
                opt_text(record.alignment.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM deity_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const DEITY_DOMAIN: &str =
    "SELECT t.id, t.deity_id, t.domain, t.note FROM deity_domain AS t";

pub(super) fn deity_domain(row: &Row<'_>) -> Result<DeityDomain, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<DeityDomainId>("deity_domain", &key, &key)?;
    let deity = parse::<EntityId>("deity_domain", &key, &row.get::<_, String>("deity_id")?)?;
    let domain: String = row.get("domain")?;
    let note: Option<String> = row.get("note")?;
    Ok(DeityDomain {
        id,
        deity,
        domain,
        note,
    })
}

impl Reader<DeityDomain> for SqliteModel<'_> {
    fn get(&self, id: &DeityDomainId) -> Result<Option<DeityDomain>, StorageError> {
        self.select_one(
            &format!("{DEITY_DOMAIN} WHERE t.id = ?1"),
            [id.to_string()],
            deity_domain,
        )
    }

    fn list(&self) -> Result<Vec<DeityDomain>, StorageError> {
        self.select(&format!("{DEITY_DOMAIN} ORDER BY t.id"), [], deity_domain)
    }
}

impl Writer<DeityDomain> for SqliteModel<'_> {
    fn insert(&mut self, record: &DeityDomain) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO deity_domain (id, deity_id, domain, note) VALUES (?1, ?2, ?3, ?4)",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.deity.to_string()),
                text(&record.domain),
                opt_text(record.note.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &DeityDomain) -> Result<(), StorageError> {
        self.execute(
            "UPDATE deity_domain SET deity_id = ?2, domain = ?3, note = ?4 WHERE id = ?1",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.deity.to_string()),
                text(&record.domain),
                opt_text(record.note.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &DeityDomainId) -> Result<(), StorageError> {
        self.cascade("deity_domain", &id.to_string())
    }
}

pub(super) const SPECIES_PROFILE: &str = "SELECT t.entity_id, t.is_sapient, t.lifespan_min_years, t.lifespan_max_years, t.average_height_cm, t.diet FROM species_profile AS t";

pub(super) fn species_profile(row: &Row<'_>) -> Result<SpeciesProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("species_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let is_sapient: Option<bool> = row.get("is_sapient")?;
    let lifespan_min_years: Option<i64> = row.get("lifespan_min_years")?;
    let lifespan_max_years: Option<i64> = row.get("lifespan_max_years")?;
    let average_height_cm: Option<f64> = row.get("average_height_cm")?;
    let diet: Option<String> = row.get("diet")?;
    Ok(SpeciesProfile {
        entity,
        is_sapient,
        lifespan_min_years,
        lifespan_max_years,
        average_height_cm,
        diet,
    })
}

impl Reader<SpeciesProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<SpeciesProfile>, StorageError> {
        self.select_one(
            &format!("{SPECIES_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            species_profile,
        )
    }

    fn list(&self) -> Result<Vec<SpeciesProfile>, StorageError> {
        self.select(
            &format!("{SPECIES_PROFILE} ORDER BY t.entity_id"),
            [],
            species_profile,
        )
    }
}

impl Writer<SpeciesProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &SpeciesProfile) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO species_profile (entity_id, is_sapient, lifespan_min_years, lifespan_max_years, average_height_cm, diet) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_bool(record.is_sapient),
            opt_int(record.lifespan_min_years),
            opt_int(record.lifespan_max_years),
            opt_real(record.average_height_cm),
            opt_text(record.diet.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &SpeciesProfile) -> Result<(), StorageError> {
        self.execute(
            "UPDATE species_profile SET is_sapient = ?2, lifespan_min_years = ?3, lifespan_max_years = ?4, average_height_cm = ?5, diet = ?6 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_bool(record.is_sapient),
            opt_int(record.lifespan_min_years),
            opt_int(record.lifespan_max_years),
            opt_real(record.average_height_cm),
            opt_text(record.diet.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM species_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const SPECIES_HABITAT: &str =
    "SELECT t.species_id, t.place_id, t.note FROM species_habitat AS t";

pub(super) fn species_habitat(row: &Row<'_>) -> Result<SpeciesHabitat, StorageError> {
    let key: String = row.get("species_id")?;
    let species = parse::<EntityId>(
        "species_habitat",
        &key,
        &row.get::<_, String>("species_id")?,
    )?;
    let place = parse::<EntityId>("species_habitat", &key, &row.get::<_, String>("place_id")?)?;
    let note: Option<String> = row.get("note")?;
    Ok(SpeciesHabitat {
        species,
        place,
        note,
    })
}

impl Reader<SpeciesHabitat> for SqliteModel<'_> {
    fn get(&self, id: &(EntityId, EntityId)) -> Result<Option<SpeciesHabitat>, StorageError> {
        self.select_one(
            &format!("{SPECIES_HABITAT} WHERE t.species_id = ?1 AND t.place_id = ?2"),
            [id.0.to_string(), id.1.to_string()],
            species_habitat,
        )
    }

    fn list(&self) -> Result<Vec<SpeciesHabitat>, StorageError> {
        self.select(
            &format!("{SPECIES_HABITAT} ORDER BY t.species_id, t.place_id"),
            [],
            species_habitat,
        )
    }
}

impl Writer<SpeciesHabitat> for SqliteModel<'_> {
    fn insert(&mut self, record: &SpeciesHabitat) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO species_habitat (species_id, place_id, note) VALUES (?1, ?2, ?3)",
            params_from_iter(vec![
                text(record.species.to_string()),
                text(record.place.to_string()),
                opt_text(record.note.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &SpeciesHabitat) -> Result<(), StorageError> {
        self.execute(
            "UPDATE species_habitat SET note = ?3 WHERE species_id = ?1 AND place_id = ?2",
            params_from_iter(vec![
                text(record.species.to_string()),
                text(record.place.to_string()),
                opt_text(record.note.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &(EntityId, EntityId)) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM species_habitat WHERE species_id = ?1 AND place_id = ?2",
            [id.0.to_string(), id.1.to_string()],
        )
    }
}
