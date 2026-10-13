#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    Citizenship, CitizenshipId, DiplomaticRelation, DiplomaticRelationId, EntityId, Office,
    OfficeId, OfficeTenure, OfficeTenureId, Origin, PolityLanguage, PolityLanguageStatus,
    PolityProfile, PolitySuccession, PolitySuccessionId, PolitySuccessionKind, PolitySymbol,
    PolitySymbolId, PolitySymbolKind, Reader, SuccessionRule, TenureEndReason, TypeKey, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    boolean, int, opt_text, optional_key, optional_parse, parse, read_instant, read_period, text,
    timestamp, type_key,
};
use crate::error::StorageError;

pub(super) const POLITY_PROFILE: &str = "SELECT t.entity_id, t.government_type, t.succession_rule, t.currency, t.motto, t.capital_id, t.state_religion_id FROM polity_profile AS t";

pub(super) fn polity_profile(row: &Row<'_>) -> Result<PolityProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("polity_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let government_type: Option<String> = row.get("government_type")?;
    let succession_rule = optional_parse::<SuccessionRule>(
        "polity_profile",
        &key,
        row.get::<_, Option<String>>("succession_rule")?,
    )?;
    let currency: Option<String> = row.get("currency")?;
    let motto: Option<String> = row.get("motto")?;
    let capital = optional_parse::<EntityId>(
        "polity_profile",
        &key,
        row.get::<_, Option<String>>("capital_id")?,
    )?;
    let state_religion = optional_parse::<EntityId>(
        "polity_profile",
        &key,
        row.get::<_, Option<String>>("state_religion_id")?,
    )?;
    Ok(PolityProfile {
        entity,
        government_type,
        succession_rule,
        currency,
        motto,
        capital,
        state_religion,
    })
}

impl Reader<PolityProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<PolityProfile>, StorageError> {
        self.select_one(
            &format!("{POLITY_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            polity_profile,
        )
    }

    fn list(&self) -> Result<Vec<PolityProfile>, StorageError> {
        self.select(
            &format!("{POLITY_PROFILE} ORDER BY t.entity_id"),
            [],
            polity_profile,
        )
    }
}

impl Writer<PolityProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &PolityProfile) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO polity_profile (entity_id, government_type, succession_rule, currency, motto, capital_id, state_religion_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.government_type.clone()),
            opt_text(record.succession_rule.map(SuccessionRule::as_str)),
            opt_text(record.currency.clone()),
            opt_text(record.motto.clone()),
            opt_text(record.capital.map(|value| value.to_string())),
            opt_text(record.state_religion.map(|value| value.to_string())),
            ]),
        )
    }

    fn update(&mut self, record: &PolityProfile) -> Result<(), StorageError> {
        self.execute(
            "UPDATE polity_profile SET government_type = ?2, succession_rule = ?3, currency = ?4, motto = ?5, capital_id = ?6, state_religion_id = ?7 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.government_type.clone()),
            opt_text(record.succession_rule.map(SuccessionRule::as_str)),
            opt_text(record.currency.clone()),
            opt_text(record.motto.clone()),
            opt_text(record.capital.map(|value| value.to_string())),
            opt_text(record.state_religion.map(|value| value.to_string())),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM polity_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const POLITY_LANGUAGE: &str =
    "SELECT t.polity_id, t.language_id, t.status FROM polity_language AS t";

pub(super) fn polity_language(row: &Row<'_>) -> Result<PolityLanguage, StorageError> {
    let key: String = row.get("polity_id")?;
    let polity = parse::<EntityId>("polity_language", &key, &row.get::<_, String>("polity_id")?)?;
    let language = parse::<EntityId>(
        "polity_language",
        &key,
        &row.get::<_, String>("language_id")?,
    )?;
    let status =
        parse::<PolityLanguageStatus>("polity_language", &key, &row.get::<_, String>("status")?)?;
    Ok(PolityLanguage {
        polity,
        language,
        status,
    })
}

impl Reader<PolityLanguage> for SqliteModel<'_> {
    fn get(&self, id: &(EntityId, EntityId)) -> Result<Option<PolityLanguage>, StorageError> {
        self.select_one(
            &format!("{POLITY_LANGUAGE} WHERE t.polity_id = ?1 AND t.language_id = ?2"),
            [id.0.to_string(), id.1.to_string()],
            polity_language,
        )
    }

    fn list(&self) -> Result<Vec<PolityLanguage>, StorageError> {
        self.select(
            &format!("{POLITY_LANGUAGE} ORDER BY t.polity_id, t.language_id"),
            [],
            polity_language,
        )
    }
}

impl Writer<PolityLanguage> for SqliteModel<'_> {
    fn insert(&mut self, record: &PolityLanguage) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO polity_language (polity_id, language_id, status) VALUES (?1, ?2, ?3)",
            params_from_iter(vec![
                text(record.polity.to_string()),
                text(record.language.to_string()),
                text(record.status.as_str()),
            ]),
        )
    }

    fn update(&mut self, record: &PolityLanguage) -> Result<(), StorageError> {
        self.execute(
            "UPDATE polity_language SET status = ?3 WHERE polity_id = ?1 AND language_id = ?2",
            params_from_iter(vec![
                text(record.polity.to_string()),
                text(record.language.to_string()),
                text(record.status.as_str()),
            ]),
        )
    }

    fn delete(&mut self, id: &(EntityId, EntityId)) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM polity_language WHERE polity_id = ?1 AND language_id = ?2",
            [id.0.to_string(), id.1.to_string()],
        )
    }
}

pub(super) const POLITY_SYMBOL: &str =
    "SELECT t.id, t.polity_id, t.kind, t.description FROM polity_symbol AS t";

pub(super) fn polity_symbol(row: &Row<'_>) -> Result<PolitySymbol, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<PolitySymbolId>("polity_symbol", &key, &key)?;
    let polity = parse::<EntityId>("polity_symbol", &key, &row.get::<_, String>("polity_id")?)?;
    let kind = parse::<PolitySymbolKind>("polity_symbol", &key, &row.get::<_, String>("kind")?)?;
    let description: String = row.get("description")?;
    Ok(PolitySymbol {
        id,
        polity,
        kind,
        description,
    })
}

impl Reader<PolitySymbol> for SqliteModel<'_> {
    fn get(&self, id: &PolitySymbolId) -> Result<Option<PolitySymbol>, StorageError> {
        self.select_one(
            &format!("{POLITY_SYMBOL} WHERE t.id = ?1"),
            [id.to_string()],
            polity_symbol,
        )
    }

    fn list(&self) -> Result<Vec<PolitySymbol>, StorageError> {
        self.select(&format!("{POLITY_SYMBOL} ORDER BY t.id"), [], polity_symbol)
    }
}

impl Writer<PolitySymbol> for SqliteModel<'_> {
    fn insert(&mut self, record: &PolitySymbol) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO polity_symbol (id, polity_id, kind, description) VALUES (?1, ?2, ?3, ?4)",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.polity.to_string()),
                text(record.kind.as_str()),
                text(&record.description),
            ]),
        )
    }

    fn update(&mut self, record: &PolitySymbol) -> Result<(), StorageError> {
        self.execute(
            "UPDATE polity_symbol SET polity_id = ?2, kind = ?3, description = ?4 WHERE id = ?1",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.polity.to_string()),
                text(record.kind.as_str()),
                text(&record.description),
            ]),
        )
    }

    fn delete(&mut self, id: &PolitySymbolId) -> Result<(), StorageError> {
        self.cascade("polity_symbol", &id.to_string())
    }
}

pub(super) const POLITY_SUCCESSION: &str = "SELECT t.id, t.predecessor_id, t.successor_id, t.kind, i_occurred_at.id AS occurred_at__id, i_occurred_at.day AS occurred_at__day, i_occurred_at.precision AS occurred_at__precision, i_occurred_at.earliest_day AS occurred_at__earliest, i_occurred_at.latest_day AS occurred_at__latest, i_occurred_at.calendar_id AS occurred_at__calendar, i_occurred_at.label AS occurred_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM polity_succession AS t LEFT JOIN instant AS i_occurred_at ON i_occurred_at.id = t.occurred_at";

pub(super) fn polity_succession(row: &Row<'_>) -> Result<PolitySuccession, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<PolitySuccessionId>("polity_succession", &key, &key)?;
    let predecessor = parse::<EntityId>(
        "polity_succession",
        &key,
        &row.get::<_, String>("predecessor_id")?,
    )?;
    let successor = parse::<EntityId>(
        "polity_succession",
        &key,
        &row.get::<_, String>("successor_id")?,
    )?;
    let kind =
        parse::<PolitySuccessionKind>("polity_succession", &key, &row.get::<_, String>("kind")?)?;
    let occurred = read_instant(row, "occurred_at")?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key(
        "polity_succession",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("polity_succession", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(PolitySuccession {
        id,
        predecessor,
        successor,
        kind,
        occurred,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<PolitySuccession> for SqliteModel<'_> {
    fn get(&self, id: &PolitySuccessionId) -> Result<Option<PolitySuccession>, StorageError> {
        self.select_one(
            &format!("{POLITY_SUCCESSION} WHERE t.id = ?1"),
            [id.to_string()],
            polity_succession,
        )
    }

    fn list(&self) -> Result<Vec<PolitySuccession>, StorageError> {
        self.select(
            &format!("{POLITY_SUCCESSION} ORDER BY t.created_at, t.id"),
            [],
            polity_succession,
        )
    }
}

impl Writer<PolitySuccession> for SqliteModel<'_> {
    fn insert(&mut self, record: &PolitySuccession) -> Result<(), StorageError> {
        let occurred_at_id = self.write_instant_opt(record.occurred.as_ref())?;
        self.execute(
            "INSERT INTO polity_succession (id, predecessor_id, successor_id, kind, occurred_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.predecessor.to_string()),
            text(record.successor.to_string()),
            text(record.kind.as_str()),
            opt_text(occurred_at_id),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &PolitySuccession) -> Result<(), StorageError> {
        let occurred_at_id = self.write_instant_opt(record.occurred.as_ref())?;
        self.execute(
            "UPDATE polity_succession SET predecessor_id = ?2, successor_id = ?3, kind = ?4, occurred_at = ?5, note = ?6, canon_state = ?7, updated_at = ?8 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.predecessor.to_string()),
            text(record.successor.to_string()),
            text(record.kind.as_str()),
            opt_text(occurred_at_id),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &PolitySuccessionId) -> Result<(), StorageError> {
        self.cascade("polity_succession", &id.to_string())
    }
}

pub(super) const OFFICE: &str = "SELECT t.id, t.body_id, t.title, t.description, t.is_head, t.seats, t.created_at, t.updated_at FROM office AS t";

pub(super) fn office(row: &Row<'_>) -> Result<Office, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<OfficeId>("office", &key, &key)?;
    let body = parse::<EntityId>("office", &key, &row.get::<_, String>("body_id")?)?;
    let title: String = row.get("title")?;
    let description: String = row.get("description")?;
    let is_head: bool = row.get("is_head")?;
    let seats: i32 = row.get("seats")?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(Office {
        id,
        body,
        title,
        description,
        is_head,
        seats,
        created_at,
        updated_at,
    })
}

impl Reader<Office> for SqliteModel<'_> {
    fn get(&self, id: &OfficeId) -> Result<Option<Office>, StorageError> {
        self.select_one(
            &format!("{OFFICE} WHERE t.id = ?1"),
            [id.to_string()],
            office,
        )
    }

    fn list(&self) -> Result<Vec<Office>, StorageError> {
        self.select(&format!("{OFFICE} ORDER BY t.created_at, t.id"), [], office)
    }
}

impl Writer<Office> for SqliteModel<'_> {
    fn insert(&mut self, record: &Office) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO office (id, body_id, title, description, is_head, seats, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.body.to_string()),
            text(&record.title),
            text(&record.description),
            boolean(record.is_head),
            int(i64::from(record.seats)),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &Office) -> Result<(), StorageError> {
        self.execute(
            "UPDATE office SET body_id = ?2, title = ?3, description = ?4, is_head = ?5, seats = ?6, updated_at = ?7 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.body.to_string()),
            text(&record.title),
            text(&record.description),
            boolean(record.is_head),
            int(i64::from(record.seats)),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &OfficeId) -> Result<(), StorageError> {
        self.cascade("office", &id.to_string())
    }
}

pub(super) const OFFICE_TENURE: &str = "SELECT t.id, t.office_id, t.holder_id, t.acquisition, t.end_reason, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM office_tenure AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn office_tenure(row: &Row<'_>) -> Result<OfficeTenure, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<OfficeTenureId>("office_tenure", &key, &key)?;
    let office = parse::<OfficeId>("office_tenure", &key, &row.get::<_, String>("office_id")?)?;
    let holder = parse::<EntityId>("office_tenure", &key, &row.get::<_, String>("holder_id")?)?;
    let acquisition = optional_key(
        "office_tenure",
        &key,
        row.get::<_, Option<String>>("acquisition")?,
    )?;
    let end_reason = optional_parse::<TenureEndReason>(
        "office_tenure",
        &key,
        row.get::<_, Option<String>>("end_reason")?,
    )?;
    let period = read_period(row, "office_tenure", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("office_tenure", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("office_tenure", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(OfficeTenure {
        id,
        office,
        holder,
        acquisition,
        end_reason,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<OfficeTenure> for SqliteModel<'_> {
    fn get(&self, id: &OfficeTenureId) -> Result<Option<OfficeTenure>, StorageError> {
        self.select_one(
            &format!("{OFFICE_TENURE} WHERE t.id = ?1"),
            [id.to_string()],
            office_tenure,
        )
    }

    fn list(&self) -> Result<Vec<OfficeTenure>, StorageError> {
        self.select(
            &format!("{OFFICE_TENURE} ORDER BY t.created_at, t.id"),
            [],
            office_tenure,
        )
    }
}

impl Writer<OfficeTenure> for SqliteModel<'_> {
    fn insert(&mut self, record: &OfficeTenure) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO office_tenure (id, office_id, holder_id, acquisition, end_reason, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.office.to_string()),
            text(record.holder.to_string()),
            opt_text(record.acquisition.as_ref().map(TypeKey::as_str)),
            opt_text(record.end_reason.map(TenureEndReason::as_str)),
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

    fn update(&mut self, record: &OfficeTenure) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE office_tenure SET office_id = ?2, holder_id = ?3, acquisition = ?4, end_reason = ?5, time_state = ?6, starts_at = ?7, ends_at = ?8, note = ?9, canon_state = ?10, updated_at = ?11 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.office.to_string()),
            text(record.holder.to_string()),
            opt_text(record.acquisition.as_ref().map(TypeKey::as_str)),
            opt_text(record.end_reason.map(TenureEndReason::as_str)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &OfficeTenureId) -> Result<(), StorageError> {
        self.cascade("office_tenure", &id.to_string())
    }
}

pub(super) const CITIZENSHIP: &str = "SELECT t.id, t.being_id, t.polity_id, t.status, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM citizenship AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn citizenship(row: &Row<'_>) -> Result<Citizenship, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<CitizenshipId>("citizenship", &key, &key)?;
    let being = parse::<EntityId>("citizenship", &key, &row.get::<_, String>("being_id")?)?;
    let polity = parse::<EntityId>("citizenship", &key, &row.get::<_, String>("polity_id")?)?;
    let status = type_key("citizenship", &key, &row.get::<_, String>("status")?)?;
    let period = read_period(row, "citizenship", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("citizenship", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("citizenship", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(Citizenship {
        id,
        being,
        polity,
        status,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<Citizenship> for SqliteModel<'_> {
    fn get(&self, id: &CitizenshipId) -> Result<Option<Citizenship>, StorageError> {
        self.select_one(
            &format!("{CITIZENSHIP} WHERE t.id = ?1"),
            [id.to_string()],
            citizenship,
        )
    }

    fn list(&self) -> Result<Vec<Citizenship>, StorageError> {
        self.select(
            &format!("{CITIZENSHIP} ORDER BY t.created_at, t.id"),
            [],
            citizenship,
        )
    }
}

impl Writer<Citizenship> for SqliteModel<'_> {
    fn insert(&mut self, record: &Citizenship) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO citizenship (id, being_id, polity_id, status, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            text(record.polity.to_string()),
            text(record.status.as_str()),
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

    fn update(&mut self, record: &Citizenship) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE citizenship SET being_id = ?2, polity_id = ?3, status = ?4, time_state = ?5, starts_at = ?6, ends_at = ?7, note = ?8, canon_state = ?9, updated_at = ?10 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.being.to_string()),
            text(record.polity.to_string()),
            text(record.status.as_str()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &CitizenshipId) -> Result<(), StorageError> {
        self.cascade("citizenship", &id.to_string())
    }
}

pub(super) const DIPLOMATIC_RELATION: &str = "SELECT t.id, t.first_id, t.second_id, t.stance, t.senior_id, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM diplomatic_relation AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn diplomatic_relation(row: &Row<'_>) -> Result<DiplomaticRelation, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<DiplomaticRelationId>("diplomatic_relation", &key, &key)?;
    let first = parse::<EntityId>(
        "diplomatic_relation",
        &key,
        &row.get::<_, String>("first_id")?,
    )?;
    let second = parse::<EntityId>(
        "diplomatic_relation",
        &key,
        &row.get::<_, String>("second_id")?,
    )?;
    let stance = type_key(
        "diplomatic_relation",
        &key,
        &row.get::<_, String>("stance")?,
    )?;
    let senior = optional_parse::<EntityId>(
        "diplomatic_relation",
        &key,
        row.get::<_, Option<String>>("senior_id")?,
    )?;
    let period = read_period(row, "diplomatic_relation", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key(
        "diplomatic_relation",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>(
        "diplomatic_relation",
        &key,
        &row.get::<_, String>("origin")?,
    )?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(DiplomaticRelation {
        id,
        first,
        second,
        stance,
        senior,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<DiplomaticRelation> for SqliteModel<'_> {
    fn get(&self, id: &DiplomaticRelationId) -> Result<Option<DiplomaticRelation>, StorageError> {
        self.select_one(
            &format!("{DIPLOMATIC_RELATION} WHERE t.id = ?1"),
            [id.to_string()],
            diplomatic_relation,
        )
    }

    fn list(&self) -> Result<Vec<DiplomaticRelation>, StorageError> {
        self.select(
            &format!("{DIPLOMATIC_RELATION} ORDER BY t.created_at, t.id"),
            [],
            diplomatic_relation,
        )
    }
}

impl Writer<DiplomaticRelation> for SqliteModel<'_> {
    fn insert(&mut self, record: &DiplomaticRelation) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO diplomatic_relation (id, first_id, second_id, stance, senior_id, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.first.to_string()),
            text(record.second.to_string()),
            text(record.stance.as_str()),
            opt_text(record.senior.map(|value| value.to_string())),
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

    fn update(&mut self, record: &DiplomaticRelation) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE diplomatic_relation SET first_id = ?2, second_id = ?3, stance = ?4, senior_id = ?5, time_state = ?6, starts_at = ?7, ends_at = ?8, note = ?9, canon_state = ?10, updated_at = ?11 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.first.to_string()),
            text(record.second.to_string()),
            text(record.stance.as_str()),
            opt_text(record.senior.map(|value| value.to_string())),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &DiplomaticRelationId) -> Result<(), StorageError> {
        self.cascade("diplomatic_relation", &id.to_string())
    }
}
