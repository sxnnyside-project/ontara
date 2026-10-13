#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    EntityId, ItemOwnership, ItemOwnershipId, ItemPower, ItemPowerId, ItemProfile, ItemRarity,
    Origin, Reader, TypeKey, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    int, opt_text, optional_key, optional_parse, parse, read_instant, read_period, text, timestamp,
    type_key,
};
use crate::error::StorageError;

pub(super) const ITEM_PROFILE: &str = "SELECT t.entity_id, t.material, t.rarity, t.creator_id, i_made_at.id AS made_at__id, i_made_at.day AS made_at__day, i_made_at.precision AS made_at__precision, i_made_at.earliest_day AS made_at__earliest, i_made_at.latest_day AS made_at__latest, i_made_at.calendar_id AS made_at__calendar, i_made_at.label AS made_at__label, i_destroyed_at.id AS destroyed_at__id, i_destroyed_at.day AS destroyed_at__day, i_destroyed_at.precision AS destroyed_at__precision, i_destroyed_at.earliest_day AS destroyed_at__earliest, i_destroyed_at.latest_day AS destroyed_at__latest, i_destroyed_at.calendar_id AS destroyed_at__calendar, i_destroyed_at.label AS destroyed_at__label FROM item_profile AS t LEFT JOIN instant AS i_made_at ON i_made_at.id = t.made_at LEFT JOIN instant AS i_destroyed_at ON i_destroyed_at.id = t.destroyed_at";

pub(super) fn item_profile(row: &Row<'_>) -> Result<ItemProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("item_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let material: Option<String> = row.get("material")?;
    let rarity = optional_parse::<ItemRarity>(
        "item_profile",
        &key,
        row.get::<_, Option<String>>("rarity")?,
    )?;
    let creator = optional_parse::<EntityId>(
        "item_profile",
        &key,
        row.get::<_, Option<String>>("creator_id")?,
    )?;
    let made = read_instant(row, "made_at")?;
    let destroyed = read_instant(row, "destroyed_at")?;
    Ok(ItemProfile {
        entity,
        material,
        rarity,
        creator,
        made,
        destroyed,
    })
}

impl Reader<ItemProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<ItemProfile>, StorageError> {
        self.select_one(
            &format!("{ITEM_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            item_profile,
        )
    }

    fn list(&self) -> Result<Vec<ItemProfile>, StorageError> {
        self.select(
            &format!("{ITEM_PROFILE} ORDER BY t.entity_id"),
            [],
            item_profile,
        )
    }
}

impl Writer<ItemProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &ItemProfile) -> Result<(), StorageError> {
        let made_at_id = self.write_instant_opt(record.made.as_ref())?;
        let destroyed_at_id = self.write_instant_opt(record.destroyed.as_ref())?;
        self.execute(
            "INSERT INTO item_profile (entity_id, material, rarity, creator_id, made_at, destroyed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.material.clone()),
            opt_text(record.rarity.map(ItemRarity::as_str)),
            opt_text(record.creator.map(|value| value.to_string())),
            opt_text(made_at_id),
            opt_text(destroyed_at_id),
            ]),
        )
    }

    fn update(&mut self, record: &ItemProfile) -> Result<(), StorageError> {
        let made_at_id = self.write_instant_opt(record.made.as_ref())?;
        let destroyed_at_id = self.write_instant_opt(record.destroyed.as_ref())?;
        self.execute(
            "UPDATE item_profile SET material = ?2, rarity = ?3, creator_id = ?4, made_at = ?5, destroyed_at = ?6 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.material.clone()),
            opt_text(record.rarity.map(ItemRarity::as_str)),
            opt_text(record.creator.map(|value| value.to_string())),
            opt_text(made_at_id),
            opt_text(destroyed_at_id),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM item_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const ITEM_POWER: &str = "SELECT t.id, t.item_id, t.description, t.cost, t.canon_state, t.origin, t.created_at, t.updated_at FROM item_power AS t";

pub(super) fn item_power(row: &Row<'_>) -> Result<ItemPower, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<ItemPowerId>("item_power", &key, &key)?;
    let item = parse::<EntityId>("item_power", &key, &row.get::<_, String>("item_id")?)?;
    let description: String = row.get("description")?;
    let cost: Option<String> = row.get("cost")?;
    let canon_state = type_key("item_power", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("item_power", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(ItemPower {
        id,
        item,
        description,
        cost,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<ItemPower> for SqliteModel<'_> {
    fn get(&self, id: &ItemPowerId) -> Result<Option<ItemPower>, StorageError> {
        self.select_one(
            &format!("{ITEM_POWER} WHERE t.id = ?1"),
            [id.to_string()],
            item_power,
        )
    }

    fn list(&self) -> Result<Vec<ItemPower>, StorageError> {
        self.select(
            &format!("{ITEM_POWER} ORDER BY t.created_at, t.id"),
            [],
            item_power,
        )
    }
}

impl Writer<ItemPower> for SqliteModel<'_> {
    fn insert(&mut self, record: &ItemPower) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO item_power (id, item_id, description, cost, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.item.to_string()),
            text(&record.description),
            opt_text(record.cost.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &ItemPower) -> Result<(), StorageError> {
        self.execute(
            "UPDATE item_power SET item_id = ?2, description = ?3, cost = ?4, canon_state = ?5, updated_at = ?6 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.item.to_string()),
            text(&record.description),
            opt_text(record.cost.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &ItemPowerId) -> Result<(), StorageError> {
        self.cascade("item_power", &id.to_string())
    }
}

pub(super) const ITEM_OWNERSHIP: &str = "SELECT t.id, t.item_id, t.owner_id, t.acquisition, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM item_ownership AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn item_ownership(row: &Row<'_>) -> Result<ItemOwnership, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<ItemOwnershipId>("item_ownership", &key, &key)?;
    let item = parse::<EntityId>("item_ownership", &key, &row.get::<_, String>("item_id")?)?;
    let owner = parse::<EntityId>("item_ownership", &key, &row.get::<_, String>("owner_id")?)?;
    let acquisition = optional_key(
        "item_ownership",
        &key,
        row.get::<_, Option<String>>("acquisition")?,
    )?;
    let period = read_period(row, "item_ownership", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key(
        "item_ownership",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("item_ownership", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(ItemOwnership {
        id,
        item,
        owner,
        acquisition,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<ItemOwnership> for SqliteModel<'_> {
    fn get(&self, id: &ItemOwnershipId) -> Result<Option<ItemOwnership>, StorageError> {
        self.select_one(
            &format!("{ITEM_OWNERSHIP} WHERE t.id = ?1"),
            [id.to_string()],
            item_ownership,
        )
    }

    fn list(&self) -> Result<Vec<ItemOwnership>, StorageError> {
        self.select(
            &format!("{ITEM_OWNERSHIP} ORDER BY t.created_at, t.id"),
            [],
            item_ownership,
        )
    }
}

impl Writer<ItemOwnership> for SqliteModel<'_> {
    fn insert(&mut self, record: &ItemOwnership) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO item_ownership (id, item_id, owner_id, acquisition, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.item.to_string()),
            text(record.owner.to_string()),
            opt_text(record.acquisition.as_ref().map(TypeKey::as_str)),
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

    fn update(&mut self, record: &ItemOwnership) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE item_ownership SET item_id = ?2, owner_id = ?3, acquisition = ?4, time_state = ?5, starts_at = ?6, ends_at = ?7, note = ?8, canon_state = ?9, updated_at = ?10 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.item.to_string()),
            text(record.owner.to_string()),
            opt_text(record.acquisition.as_ref().map(TypeKey::as_str)),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &ItemOwnershipId) -> Result<(), StorageError> {
        self.cascade("item_ownership", &id.to_string())
    }
}
