#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    EntityId, FactionParent, FactionParentId, FactionProfile, Membership, MembershipEndReason,
    MembershipId, Origin, Reader, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    int, opt_int, opt_text, optional_parse, parse, read_instant, read_period, text, timestamp,
    type_key,
};
use crate::error::StorageError;

pub(super) const FACTION_PROFILE: &str = "SELECT t.entity_id, t.ideology, t.size_estimate, t.headquarters_id, i_founded_at.id AS founded_at__id, i_founded_at.day AS founded_at__day, i_founded_at.precision AS founded_at__precision, i_founded_at.earliest_day AS founded_at__earliest, i_founded_at.latest_day AS founded_at__latest, i_founded_at.calendar_id AS founded_at__calendar, i_founded_at.label AS founded_at__label, i_dissolved_at.id AS dissolved_at__id, i_dissolved_at.day AS dissolved_at__day, i_dissolved_at.precision AS dissolved_at__precision, i_dissolved_at.earliest_day AS dissolved_at__earliest, i_dissolved_at.latest_day AS dissolved_at__latest, i_dissolved_at.calendar_id AS dissolved_at__calendar, i_dissolved_at.label AS dissolved_at__label FROM faction_profile AS t LEFT JOIN instant AS i_founded_at ON i_founded_at.id = t.founded_at LEFT JOIN instant AS i_dissolved_at ON i_dissolved_at.id = t.dissolved_at";

pub(super) fn faction_profile(row: &Row<'_>) -> Result<FactionProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("faction_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let ideology: Option<String> = row.get("ideology")?;
    let size_estimate: Option<i64> = row.get("size_estimate")?;
    let headquarters = optional_parse::<EntityId>(
        "faction_profile",
        &key,
        row.get::<_, Option<String>>("headquarters_id")?,
    )?;
    let founded = read_instant(row, "founded_at")?;
    let dissolved = read_instant(row, "dissolved_at")?;
    Ok(FactionProfile {
        entity,
        ideology,
        size_estimate,
        headquarters,
        founded,
        dissolved,
    })
}

impl Reader<FactionProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<FactionProfile>, StorageError> {
        self.select_one(
            &format!("{FACTION_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            faction_profile,
        )
    }

    fn list(&self) -> Result<Vec<FactionProfile>, StorageError> {
        self.select(
            &format!("{FACTION_PROFILE} ORDER BY t.entity_id"),
            [],
            faction_profile,
        )
    }
}

impl Writer<FactionProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &FactionProfile) -> Result<(), StorageError> {
        let founded_at_id = self.write_instant_opt(record.founded.as_ref())?;
        let dissolved_at_id = self.write_instant_opt(record.dissolved.as_ref())?;
        self.execute(
            "INSERT INTO faction_profile (entity_id, ideology, size_estimate, headquarters_id, founded_at, dissolved_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.ideology.clone()),
            opt_int(record.size_estimate),
            opt_text(record.headquarters.map(|value| value.to_string())),
            opt_text(founded_at_id),
            opt_text(dissolved_at_id),
            ]),
        )
    }

    fn update(&mut self, record: &FactionProfile) -> Result<(), StorageError> {
        let founded_at_id = self.write_instant_opt(record.founded.as_ref())?;
        let dissolved_at_id = self.write_instant_opt(record.dissolved.as_ref())?;
        self.execute(
            "UPDATE faction_profile SET ideology = ?2, size_estimate = ?3, headquarters_id = ?4, founded_at = ?5, dissolved_at = ?6 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.ideology.clone()),
            opt_int(record.size_estimate),
            opt_text(record.headquarters.map(|value| value.to_string())),
            opt_text(founded_at_id),
            opt_text(dissolved_at_id),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM faction_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const FACTION_PARENT: &str = "SELECT t.id, t.child_id, t.parent_id, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.canon_state, t.origin, t.created_at, t.updated_at FROM faction_parent AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn faction_parent(row: &Row<'_>) -> Result<FactionParent, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<FactionParentId>("faction_parent", &key, &key)?;
    let child = parse::<EntityId>("faction_parent", &key, &row.get::<_, String>("child_id")?)?;
    let parent = parse::<EntityId>("faction_parent", &key, &row.get::<_, String>("parent_id")?)?;
    let period = read_period(row, "faction_parent", &key)?;
    let canon_state = type_key(
        "faction_parent",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("faction_parent", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(FactionParent {
        id,
        child,
        parent,
        period,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<FactionParent> for SqliteModel<'_> {
    fn get(&self, id: &FactionParentId) -> Result<Option<FactionParent>, StorageError> {
        self.select_one(
            &format!("{FACTION_PARENT} WHERE t.id = ?1"),
            [id.to_string()],
            faction_parent,
        )
    }

    fn list(&self) -> Result<Vec<FactionParent>, StorageError> {
        self.select(
            &format!("{FACTION_PARENT} ORDER BY t.created_at, t.id"),
            [],
            faction_parent,
        )
    }
}

impl Writer<FactionParent> for SqliteModel<'_> {
    fn insert(&mut self, record: &FactionParent) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO faction_parent (id, child_id, parent_id, time_state, starts_at, ends_at, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.child.to_string()),
            text(record.parent.to_string()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &FactionParent) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE faction_parent SET child_id = ?2, parent_id = ?3, time_state = ?4, starts_at = ?5, ends_at = ?6, canon_state = ?7, updated_at = ?8 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.child.to_string()),
            text(record.parent.to_string()),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &FactionParentId) -> Result<(), StorageError> {
        self.cascade("faction_parent", &id.to_string())
    }
}

pub(super) const MEMBERSHIP: &str = "SELECT t.id, t.faction_id, t.member_id, t.role, t.rank, t.end_reason, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM membership AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn membership(row: &Row<'_>) -> Result<Membership, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<MembershipId>("membership", &key, &key)?;
    let faction = parse::<EntityId>("membership", &key, &row.get::<_, String>("faction_id")?)?;
    let member = parse::<EntityId>("membership", &key, &row.get::<_, String>("member_id")?)?;
    let role: Option<String> = row.get("role")?;
    let rank: Option<String> = row.get("rank")?;
    let end_reason = optional_parse::<MembershipEndReason>(
        "membership",
        &key,
        row.get::<_, Option<String>>("end_reason")?,
    )?;
    let period = read_period(row, "membership", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("membership", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("membership", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(Membership {
        id,
        faction,
        member,
        role,
        rank,
        end_reason,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<Membership> for SqliteModel<'_> {
    fn get(&self, id: &MembershipId) -> Result<Option<Membership>, StorageError> {
        self.select_one(
            &format!("{MEMBERSHIP} WHERE t.id = ?1"),
            [id.to_string()],
            membership,
        )
    }

    fn list(&self) -> Result<Vec<Membership>, StorageError> {
        self.select(
            &format!("{MEMBERSHIP} ORDER BY t.created_at, t.id"),
            [],
            membership,
        )
    }
}

impl Writer<Membership> for SqliteModel<'_> {
    fn insert(&mut self, record: &Membership) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO membership (id, faction_id, member_id, role, rank, end_reason, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.faction.to_string()),
            text(record.member.to_string()),
            opt_text(record.role.clone()),
            opt_text(record.rank.clone()),
            opt_text(record.end_reason.map(MembershipEndReason::as_str)),
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

    fn update(&mut self, record: &Membership) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE membership SET faction_id = ?2, member_id = ?3, role = ?4, rank = ?5, end_reason = ?6, time_state = ?7, starts_at = ?8, ends_at = ?9, note = ?10, canon_state = ?11, updated_at = ?12 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.faction.to_string()),
            text(record.member.to_string()),
            opt_text(record.role.clone()),
            opt_text(record.rank.clone()),
            opt_text(record.end_reason.map(MembershipEndReason::as_str)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &MembershipId) -> Result<(), StorageError> {
        self.cascade("membership", &id.to_string())
    }
}
