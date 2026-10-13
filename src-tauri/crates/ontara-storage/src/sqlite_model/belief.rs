#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    Adherence, AdherenceId, CultureLanguage, CulturePractice, CulturePracticeId,
    CulturePracticeKind, Devotion, EntityId, HolyPlace, LanguageProfile, LanguageStatus, Origin,
    Reader, ReligionDeity, ReligionProfile, ReligionTenet, ReligionTenetId, TenetKind, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    int, opt_bool, opt_int, opt_text, optional_parse, parse, read_instant, read_period, text,
    timestamp, type_key,
};
use crate::error::StorageError;

pub(super) const CULTURE_PRACTICE: &str = "SELECT t.id, t.culture_id, t.kind, t.description, t.canon_state, t.origin, t.created_at, t.updated_at FROM culture_practice AS t";

pub(super) fn culture_practice(row: &Row<'_>) -> Result<CulturePractice, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<CulturePracticeId>("culture_practice", &key, &key)?;
    let culture = parse::<EntityId>(
        "culture_practice",
        &key,
        &row.get::<_, String>("culture_id")?,
    )?;
    let kind =
        parse::<CulturePracticeKind>("culture_practice", &key, &row.get::<_, String>("kind")?)?;
    let description: String = row.get("description")?;
    let canon_state = type_key(
        "culture_practice",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("culture_practice", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(CulturePractice {
        id,
        culture,
        kind,
        description,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<CulturePractice> for SqliteModel<'_> {
    fn get(&self, id: &CulturePracticeId) -> Result<Option<CulturePractice>, StorageError> {
        self.select_one(
            &format!("{CULTURE_PRACTICE} WHERE t.id = ?1"),
            [id.to_string()],
            culture_practice,
        )
    }

    fn list(&self) -> Result<Vec<CulturePractice>, StorageError> {
        self.select(
            &format!("{CULTURE_PRACTICE} ORDER BY t.created_at, t.id"),
            [],
            culture_practice,
        )
    }
}

impl Writer<CulturePractice> for SqliteModel<'_> {
    fn insert(&mut self, record: &CulturePractice) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO culture_practice (id, culture_id, kind, description, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.culture.to_string()),
            text(record.kind.as_str()),
            text(&record.description),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &CulturePractice) -> Result<(), StorageError> {
        self.execute(
            "UPDATE culture_practice SET culture_id = ?2, kind = ?3, description = ?4, canon_state = ?5, updated_at = ?6 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.culture.to_string()),
            text(record.kind.as_str()),
            text(&record.description),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &CulturePracticeId) -> Result<(), StorageError> {
        self.cascade("culture_practice", &id.to_string())
    }
}

pub(super) const CULTURE_LANGUAGE: &str =
    "SELECT t.culture_id, t.language_id FROM culture_language AS t";

pub(super) fn culture_language(row: &Row<'_>) -> Result<CultureLanguage, StorageError> {
    let key: String = row.get("culture_id")?;
    let culture = parse::<EntityId>(
        "culture_language",
        &key,
        &row.get::<_, String>("culture_id")?,
    )?;
    let language = parse::<EntityId>(
        "culture_language",
        &key,
        &row.get::<_, String>("language_id")?,
    )?;
    Ok(CultureLanguage { culture, language })
}

impl Reader<CultureLanguage> for SqliteModel<'_> {
    fn get(&self, id: &(EntityId, EntityId)) -> Result<Option<CultureLanguage>, StorageError> {
        self.select_one(
            &format!("{CULTURE_LANGUAGE} WHERE t.culture_id = ?1 AND t.language_id = ?2"),
            [id.0.to_string(), id.1.to_string()],
            culture_language,
        )
    }

    fn list(&self) -> Result<Vec<CultureLanguage>, StorageError> {
        self.select(
            &format!("{CULTURE_LANGUAGE} ORDER BY t.culture_id, t.language_id"),
            [],
            culture_language,
        )
    }
}

impl Writer<CultureLanguage> for SqliteModel<'_> {
    fn insert(&mut self, record: &CultureLanguage) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO culture_language (culture_id, language_id) VALUES (?1, ?2)",
            params_from_iter(vec![
                text(record.culture.to_string()),
                text(record.language.to_string()),
            ]),
        )
    }

    fn update(&mut self, record: &CultureLanguage) -> Result<(), StorageError> {
        let _ = record;
        Ok(())
    }

    fn delete(&mut self, id: &(EntityId, EntityId)) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM culture_language WHERE culture_id = ?1 AND language_id = ?2",
            [id.0.to_string(), id.1.to_string()],
        )
    }
}

pub(super) const LANGUAGE_PROFILE: &str = "SELECT t.entity_id, t.parent_language_id, t.writing_system, t.speakers_estimate, t.status FROM language_profile AS t";

pub(super) fn language_profile(row: &Row<'_>) -> Result<LanguageProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>(
        "language_profile",
        &key,
        &row.get::<_, String>("entity_id")?,
    )?;
    let parent_language = optional_parse::<EntityId>(
        "language_profile",
        &key,
        row.get::<_, Option<String>>("parent_language_id")?,
    )?;
    let writing_system: Option<String> = row.get("writing_system")?;
    let speakers_estimate: Option<i64> = row.get("speakers_estimate")?;
    let status = optional_parse::<LanguageStatus>(
        "language_profile",
        &key,
        row.get::<_, Option<String>>("status")?,
    )?;
    Ok(LanguageProfile {
        entity,
        parent_language,
        writing_system,
        speakers_estimate,
        status,
    })
}

impl Reader<LanguageProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<LanguageProfile>, StorageError> {
        self.select_one(
            &format!("{LANGUAGE_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            language_profile,
        )
    }

    fn list(&self) -> Result<Vec<LanguageProfile>, StorageError> {
        self.select(
            &format!("{LANGUAGE_PROFILE} ORDER BY t.entity_id"),
            [],
            language_profile,
        )
    }
}

impl Writer<LanguageProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &LanguageProfile) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO language_profile (entity_id, parent_language_id, writing_system, speakers_estimate, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.parent_language.map(|value| value.to_string())),
            opt_text(record.writing_system.clone()),
            opt_int(record.speakers_estimate),
            opt_text(record.status.map(LanguageStatus::as_str)),
            ]),
        )
    }

    fn update(&mut self, record: &LanguageProfile) -> Result<(), StorageError> {
        self.execute(
            "UPDATE language_profile SET parent_language_id = ?2, writing_system = ?3, speakers_estimate = ?4, status = ?5 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.parent_language.map(|value| value.to_string())),
            opt_text(record.writing_system.clone()),
            opt_int(record.speakers_estimate),
            opt_text(record.status.map(LanguageStatus::as_str)),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM language_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const RELIGION_PROFILE: &str = "SELECT t.entity_id, t.founder_id, i_founded_at.id AS founded_at__id, i_founded_at.day AS founded_at__day, i_founded_at.precision AS founded_at__precision, i_founded_at.earliest_day AS founded_at__earliest, i_founded_at.latest_day AS founded_at__latest, i_founded_at.calendar_id AS founded_at__calendar, i_founded_at.label AS founded_at__label, t.is_monotheistic FROM religion_profile AS t LEFT JOIN instant AS i_founded_at ON i_founded_at.id = t.founded_at";

pub(super) fn religion_profile(row: &Row<'_>) -> Result<ReligionProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>(
        "religion_profile",
        &key,
        &row.get::<_, String>("entity_id")?,
    )?;
    let founder = optional_parse::<EntityId>(
        "religion_profile",
        &key,
        row.get::<_, Option<String>>("founder_id")?,
    )?;
    let founded = read_instant(row, "founded_at")?;
    let is_monotheistic: Option<bool> = row.get("is_monotheistic")?;
    Ok(ReligionProfile {
        entity,
        founder,
        founded,
        is_monotheistic,
    })
}

impl Reader<ReligionProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<ReligionProfile>, StorageError> {
        self.select_one(
            &format!("{RELIGION_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            religion_profile,
        )
    }

    fn list(&self) -> Result<Vec<ReligionProfile>, StorageError> {
        self.select(
            &format!("{RELIGION_PROFILE} ORDER BY t.entity_id"),
            [],
            religion_profile,
        )
    }
}

impl Writer<ReligionProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &ReligionProfile) -> Result<(), StorageError> {
        let founded_at_id = self.write_instant_opt(record.founded.as_ref())?;
        self.execute(
            "INSERT INTO religion_profile (entity_id, founder_id, founded_at, is_monotheistic) VALUES (?1, ?2, ?3, ?4)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.founder.map(|value| value.to_string())),
            opt_text(founded_at_id),
            opt_bool(record.is_monotheistic),
            ]),
        )
    }

    fn update(&mut self, record: &ReligionProfile) -> Result<(), StorageError> {
        let founded_at_id = self.write_instant_opt(record.founded.as_ref())?;
        self.execute(
            "UPDATE religion_profile SET founder_id = ?2, founded_at = ?3, is_monotheistic = ?4 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.founder.map(|value| value.to_string())),
            opt_text(founded_at_id),
            opt_bool(record.is_monotheistic),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM religion_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const RELIGION_TENET: &str = "SELECT t.id, t.religion_id, t.kind, t.description, t.canon_state, t.origin, t.created_at, t.updated_at FROM religion_tenet AS t";

pub(super) fn religion_tenet(row: &Row<'_>) -> Result<ReligionTenet, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<ReligionTenetId>("religion_tenet", &key, &key)?;
    let religion = parse::<EntityId>(
        "religion_tenet",
        &key,
        &row.get::<_, String>("religion_id")?,
    )?;
    let kind = parse::<TenetKind>("religion_tenet", &key, &row.get::<_, String>("kind")?)?;
    let description: String = row.get("description")?;
    let canon_state = type_key(
        "religion_tenet",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("religion_tenet", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(ReligionTenet {
        id,
        religion,
        kind,
        description,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<ReligionTenet> for SqliteModel<'_> {
    fn get(&self, id: &ReligionTenetId) -> Result<Option<ReligionTenet>, StorageError> {
        self.select_one(
            &format!("{RELIGION_TENET} WHERE t.id = ?1"),
            [id.to_string()],
            religion_tenet,
        )
    }

    fn list(&self) -> Result<Vec<ReligionTenet>, StorageError> {
        self.select(
            &format!("{RELIGION_TENET} ORDER BY t.created_at, t.id"),
            [],
            religion_tenet,
        )
    }
}

impl Writer<ReligionTenet> for SqliteModel<'_> {
    fn insert(&mut self, record: &ReligionTenet) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO religion_tenet (id, religion_id, kind, description, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.religion.to_string()),
            text(record.kind.as_str()),
            text(&record.description),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &ReligionTenet) -> Result<(), StorageError> {
        self.execute(
            "UPDATE religion_tenet SET religion_id = ?2, kind = ?3, description = ?4, canon_state = ?5, updated_at = ?6 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.religion.to_string()),
            text(record.kind.as_str()),
            text(&record.description),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &ReligionTenetId) -> Result<(), StorageError> {
        self.cascade("religion_tenet", &id.to_string())
    }
}

pub(super) const RELIGION_DEITY: &str =
    "SELECT t.religion_id, t.deity_id, t.role FROM religion_deity AS t";

pub(super) fn religion_deity(row: &Row<'_>) -> Result<ReligionDeity, StorageError> {
    let key: String = row.get("religion_id")?;
    let religion = parse::<EntityId>(
        "religion_deity",
        &key,
        &row.get::<_, String>("religion_id")?,
    )?;
    let deity = parse::<EntityId>("religion_deity", &key, &row.get::<_, String>("deity_id")?)?;
    let role: Option<String> = row.get("role")?;
    Ok(ReligionDeity {
        religion,
        deity,
        role,
    })
}

impl Reader<ReligionDeity> for SqliteModel<'_> {
    fn get(&self, id: &(EntityId, EntityId)) -> Result<Option<ReligionDeity>, StorageError> {
        self.select_one(
            &format!("{RELIGION_DEITY} WHERE t.religion_id = ?1 AND t.deity_id = ?2"),
            [id.0.to_string(), id.1.to_string()],
            religion_deity,
        )
    }

    fn list(&self) -> Result<Vec<ReligionDeity>, StorageError> {
        self.select(
            &format!("{RELIGION_DEITY} ORDER BY t.religion_id, t.deity_id"),
            [],
            religion_deity,
        )
    }
}

impl Writer<ReligionDeity> for SqliteModel<'_> {
    fn insert(&mut self, record: &ReligionDeity) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO religion_deity (religion_id, deity_id, role) VALUES (?1, ?2, ?3)",
            params_from_iter(vec![
                text(record.religion.to_string()),
                text(record.deity.to_string()),
                opt_text(record.role.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &ReligionDeity) -> Result<(), StorageError> {
        self.execute(
            "UPDATE religion_deity SET role = ?3 WHERE religion_id = ?1 AND deity_id = ?2",
            params_from_iter(vec![
                text(record.religion.to_string()),
                text(record.deity.to_string()),
                opt_text(record.role.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &(EntityId, EntityId)) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM religion_deity WHERE religion_id = ?1 AND deity_id = ?2",
            [id.0.to_string(), id.1.to_string()],
        )
    }
}

pub(super) const HOLY_PLACE: &str =
    "SELECT t.religion_id, t.place_id, t.significance FROM holy_place AS t";

pub(super) fn holy_place(row: &Row<'_>) -> Result<HolyPlace, StorageError> {
    let key: String = row.get("religion_id")?;
    let religion = parse::<EntityId>("holy_place", &key, &row.get::<_, String>("religion_id")?)?;
    let place = parse::<EntityId>("holy_place", &key, &row.get::<_, String>("place_id")?)?;
    let significance: Option<String> = row.get("significance")?;
    Ok(HolyPlace {
        religion,
        place,
        significance,
    })
}

impl Reader<HolyPlace> for SqliteModel<'_> {
    fn get(&self, id: &(EntityId, EntityId)) -> Result<Option<HolyPlace>, StorageError> {
        self.select_one(
            &format!("{HOLY_PLACE} WHERE t.religion_id = ?1 AND t.place_id = ?2"),
            [id.0.to_string(), id.1.to_string()],
            holy_place,
        )
    }

    fn list(&self) -> Result<Vec<HolyPlace>, StorageError> {
        self.select(
            &format!("{HOLY_PLACE} ORDER BY t.religion_id, t.place_id"),
            [],
            holy_place,
        )
    }
}

impl Writer<HolyPlace> for SqliteModel<'_> {
    fn insert(&mut self, record: &HolyPlace) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO holy_place (religion_id, place_id, significance) VALUES (?1, ?2, ?3)",
            params_from_iter(vec![
                text(record.religion.to_string()),
                text(record.place.to_string()),
                opt_text(record.significance.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &HolyPlace) -> Result<(), StorageError> {
        self.execute(
            "UPDATE holy_place SET significance = ?3 WHERE religion_id = ?1 AND place_id = ?2",
            params_from_iter(vec![
                text(record.religion.to_string()),
                text(record.place.to_string()),
                opt_text(record.significance.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &(EntityId, EntityId)) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM holy_place WHERE religion_id = ?1 AND place_id = ?2",
            [id.0.to_string(), id.1.to_string()],
        )
    }
}

pub(super) const ADHERENCE: &str = "SELECT t.id, t.adherent_id, t.religion_id, t.devotion, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM adherence AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn adherence(row: &Row<'_>) -> Result<Adherence, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<AdherenceId>("adherence", &key, &key)?;
    let adherent = parse::<EntityId>("adherence", &key, &row.get::<_, String>("adherent_id")?)?;
    let religion = parse::<EntityId>("adherence", &key, &row.get::<_, String>("religion_id")?)?;
    let devotion =
        optional_parse::<Devotion>("adherence", &key, row.get::<_, Option<String>>("devotion")?)?;
    let period = read_period(row, "adherence", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("adherence", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("adherence", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(Adherence {
        id,
        adherent,
        religion,
        devotion,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<Adherence> for SqliteModel<'_> {
    fn get(&self, id: &AdherenceId) -> Result<Option<Adherence>, StorageError> {
        self.select_one(
            &format!("{ADHERENCE} WHERE t.id = ?1"),
            [id.to_string()],
            adherence,
        )
    }

    fn list(&self) -> Result<Vec<Adherence>, StorageError> {
        self.select(
            &format!("{ADHERENCE} ORDER BY t.created_at, t.id"),
            [],
            adherence,
        )
    }
}

impl Writer<Adherence> for SqliteModel<'_> {
    fn insert(&mut self, record: &Adherence) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO adherence (id, adherent_id, religion_id, devotion, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.adherent.to_string()),
            text(record.religion.to_string()),
            opt_text(record.devotion.map(Devotion::as_str)),
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

    fn update(&mut self, record: &Adherence) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE adherence SET adherent_id = ?2, religion_id = ?3, devotion = ?4, time_state = ?5, starts_at = ?6, ends_at = ?7, note = ?8, canon_state = ?9, updated_at = ?10 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.adherent.to_string()),
            text(record.religion.to_string()),
            opt_text(record.devotion.map(Devotion::as_str)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &AdherenceId) -> Result<(), StorageError> {
        self.cascade("adherence", &id.to_string())
    }
}
