#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    BorderKind, Census, CensusId, EntityId, Origin, PlaceBorder, PlaceBorderId, PlaceContainment,
    PlaceContainmentId, PlaceProfile, PlaceResource, PlaceResourceId, PlaceStatus, Reader,
    ResourceAbundance, SourceId, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    boolean, corrupt, int, opt_real, opt_text, optional_parse, parse, read_instant, read_period,
    text, timestamp, type_key,
};
use crate::error::StorageError;

pub(super) const PLACE_PROFILE: &str = "SELECT t.entity_id, t.climate, t.terrain, t.area_km2, t.elevation_m, t.status, i_founded_at.id AS founded_at__id, i_founded_at.day AS founded_at__day, i_founded_at.precision AS founded_at__precision, i_founded_at.earliest_day AS founded_at__earliest, i_founded_at.latest_day AS founded_at__latest, i_founded_at.calendar_id AS founded_at__calendar, i_founded_at.label AS founded_at__label, i_dissolved_at.id AS dissolved_at__id, i_dissolved_at.day AS dissolved_at__day, i_dissolved_at.precision AS dissolved_at__precision, i_dissolved_at.earliest_day AS dissolved_at__earliest, i_dissolved_at.latest_day AS dissolved_at__latest, i_dissolved_at.calendar_id AS dissolved_at__calendar, i_dissolved_at.label AS dissolved_at__label FROM place_profile AS t LEFT JOIN instant AS i_founded_at ON i_founded_at.id = t.founded_at LEFT JOIN instant AS i_dissolved_at ON i_dissolved_at.id = t.dissolved_at";

pub(super) fn place_profile(row: &Row<'_>) -> Result<PlaceProfile, StorageError> {
    let key: String = row.get("entity_id")?;
    let entity = parse::<EntityId>("place_profile", &key, &row.get::<_, String>("entity_id")?)?;
    let climate: Option<String> = row.get("climate")?;
    let terrain: Option<String> = row.get("terrain")?;
    let area_km2: Option<f64> = row.get("area_km2")?;
    let elevation_m: Option<f64> = row.get("elevation_m")?;
    let status = optional_parse::<PlaceStatus>(
        "place_profile",
        &key,
        row.get::<_, Option<String>>("status")?,
    )?;
    let founded = read_instant(row, "founded_at")?;
    let dissolved = read_instant(row, "dissolved_at")?;
    Ok(PlaceProfile {
        entity,
        climate,
        terrain,
        area_km2,
        elevation_m,
        status,
        founded,
        dissolved,
    })
}

impl Reader<PlaceProfile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<PlaceProfile>, StorageError> {
        self.select_one(
            &format!("{PLACE_PROFILE} WHERE t.entity_id = ?1"),
            [id.to_string()],
            place_profile,
        )
    }

    fn list(&self) -> Result<Vec<PlaceProfile>, StorageError> {
        self.select(
            &format!("{PLACE_PROFILE} ORDER BY t.entity_id"),
            [],
            place_profile,
        )
    }
}

impl Writer<PlaceProfile> for SqliteModel<'_> {
    fn insert(&mut self, record: &PlaceProfile) -> Result<(), StorageError> {
        let founded_at_id = self.write_instant_opt(record.founded.as_ref())?;
        let dissolved_at_id = self.write_instant_opt(record.dissolved.as_ref())?;
        self.execute(
            "INSERT INTO place_profile (entity_id, climate, terrain, area_km2, elevation_m, status, founded_at, dissolved_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.climate.clone()),
            opt_text(record.terrain.clone()),
            opt_real(record.area_km2),
            opt_real(record.elevation_m),
            opt_text(record.status.map(PlaceStatus::as_str)),
            opt_text(founded_at_id),
            opt_text(dissolved_at_id),
            ]),
        )
    }

    fn update(&mut self, record: &PlaceProfile) -> Result<(), StorageError> {
        let founded_at_id = self.write_instant_opt(record.founded.as_ref())?;
        let dissolved_at_id = self.write_instant_opt(record.dissolved.as_ref())?;
        self.execute(
            "UPDATE place_profile SET climate = ?2, terrain = ?3, area_km2 = ?4, elevation_m = ?5, status = ?6, founded_at = ?7, dissolved_at = ?8 WHERE entity_id = ?1",
            params_from_iter(vec![
            text(record.entity.to_string()),
            opt_text(record.climate.clone()),
            opt_text(record.terrain.clone()),
            opt_real(record.area_km2),
            opt_real(record.elevation_m),
            opt_text(record.status.map(PlaceStatus::as_str)),
            opt_text(founded_at_id),
            opt_text(dissolved_at_id),
            ]),
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM place_profile WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const PLACE_CONTAINMENT: &str = "SELECT t.id, t.child_id, t.parent_id, t.kind, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM place_containment AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn place_containment(row: &Row<'_>) -> Result<PlaceContainment, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<PlaceContainmentId>("place_containment", &key, &key)?;
    let child = parse::<EntityId>(
        "place_containment",
        &key,
        &row.get::<_, String>("child_id")?,
    )?;
    let parent = parse::<EntityId>(
        "place_containment",
        &key,
        &row.get::<_, String>("parent_id")?,
    )?;
    let kind = type_key("place_containment", &key, &row.get::<_, String>("kind")?)?;
    let period = read_period(row, "place_containment", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key(
        "place_containment",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("place_containment", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(PlaceContainment {
        id,
        child,
        parent,
        kind,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<PlaceContainment> for SqliteModel<'_> {
    fn get(&self, id: &PlaceContainmentId) -> Result<Option<PlaceContainment>, StorageError> {
        self.select_one(
            &format!("{PLACE_CONTAINMENT} WHERE t.id = ?1"),
            [id.to_string()],
            place_containment,
        )
    }

    fn list(&self) -> Result<Vec<PlaceContainment>, StorageError> {
        self.select(
            &format!("{PLACE_CONTAINMENT} ORDER BY t.created_at, t.id"),
            [],
            place_containment,
        )
    }
}

impl Writer<PlaceContainment> for SqliteModel<'_> {
    fn insert(&mut self, record: &PlaceContainment) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO place_containment (id, child_id, parent_id, kind, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.child.to_string()),
            text(record.parent.to_string()),
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

    fn update(&mut self, record: &PlaceContainment) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE place_containment SET child_id = ?2, parent_id = ?3, kind = ?4, time_state = ?5, starts_at = ?6, ends_at = ?7, note = ?8, canon_state = ?9, updated_at = ?10 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.child.to_string()),
            text(record.parent.to_string()),
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

    fn delete(&mut self, id: &PlaceContainmentId) -> Result<(), StorageError> {
        self.cascade("place_containment", &id.to_string())
    }
}

pub(super) const PLACE_BORDER: &str = "SELECT t.id, t.first_id, t.second_id, t.kind, t.length_km, t.time_state, i_starts_at.id AS starts_at__id, i_starts_at.day AS starts_at__day, i_starts_at.precision AS starts_at__precision, i_starts_at.earliest_day AS starts_at__earliest, i_starts_at.latest_day AS starts_at__latest, i_starts_at.calendar_id AS starts_at__calendar, i_starts_at.label AS starts_at__label, i_ends_at.id AS ends_at__id, i_ends_at.day AS ends_at__day, i_ends_at.precision AS ends_at__precision, i_ends_at.earliest_day AS ends_at__earliest, i_ends_at.latest_day AS ends_at__latest, i_ends_at.calendar_id AS ends_at__calendar, i_ends_at.label AS ends_at__label, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM place_border AS t LEFT JOIN instant AS i_starts_at ON i_starts_at.id = t.starts_at LEFT JOIN instant AS i_ends_at ON i_ends_at.id = t.ends_at";

pub(super) fn place_border(row: &Row<'_>) -> Result<PlaceBorder, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<PlaceBorderId>("place_border", &key, &key)?;
    let first = parse::<EntityId>("place_border", &key, &row.get::<_, String>("first_id")?)?;
    let second = parse::<EntityId>("place_border", &key, &row.get::<_, String>("second_id")?)?;
    let kind = parse::<BorderKind>("place_border", &key, &row.get::<_, String>("kind")?)?;
    let length_km: Option<f64> = row.get("length_km")?;
    let period = read_period(row, "place_border", &key)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("place_border", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("place_border", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(PlaceBorder {
        id,
        first,
        second,
        kind,
        length_km,
        period,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<PlaceBorder> for SqliteModel<'_> {
    fn get(&self, id: &PlaceBorderId) -> Result<Option<PlaceBorder>, StorageError> {
        self.select_one(
            &format!("{PLACE_BORDER} WHERE t.id = ?1"),
            [id.to_string()],
            place_border,
        )
    }

    fn list(&self) -> Result<Vec<PlaceBorder>, StorageError> {
        self.select(
            &format!("{PLACE_BORDER} ORDER BY t.created_at, t.id"),
            [],
            place_border,
        )
    }
}

impl Writer<PlaceBorder> for SqliteModel<'_> {
    fn insert(&mut self, record: &PlaceBorder) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO place_border (id, first_id, second_id, kind, length_km, time_state, starts_at, ends_at, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.first.to_string()),
            text(record.second.to_string()),
            text(record.kind.as_str()),
            opt_real(record.length_km),
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

    fn update(&mut self, record: &PlaceBorder) -> Result<(), StorageError> {
        let (period_state, period_start, period_end) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE place_border SET first_id = ?2, second_id = ?3, kind = ?4, length_km = ?5, time_state = ?6, starts_at = ?7, ends_at = ?8, note = ?9, canon_state = ?10, updated_at = ?11 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.first.to_string()),
            text(record.second.to_string()),
            text(record.kind.as_str()),
            opt_real(record.length_km),
            text(period_state),
            opt_text(period_start.clone()),
            opt_text(period_end.clone()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &PlaceBorderId) -> Result<(), StorageError> {
        self.cascade("place_border", &id.to_string())
    }
}

pub(super) const PLACE_RESOURCE: &str =
    "SELECT t.id, t.place_id, t.resource, t.abundance, t.note FROM place_resource AS t";

pub(super) fn place_resource(row: &Row<'_>) -> Result<PlaceResource, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<PlaceResourceId>("place_resource", &key, &key)?;
    let place = parse::<EntityId>("place_resource", &key, &row.get::<_, String>("place_id")?)?;
    let resource: String = row.get("resource")?;
    let abundance = optional_parse::<ResourceAbundance>(
        "place_resource",
        &key,
        row.get::<_, Option<String>>("abundance")?,
    )?;
    let note: Option<String> = row.get("note")?;
    Ok(PlaceResource {
        id,
        place,
        resource,
        abundance,
        note,
    })
}

impl Reader<PlaceResource> for SqliteModel<'_> {
    fn get(&self, id: &PlaceResourceId) -> Result<Option<PlaceResource>, StorageError> {
        self.select_one(
            &format!("{PLACE_RESOURCE} WHERE t.id = ?1"),
            [id.to_string()],
            place_resource,
        )
    }

    fn list(&self) -> Result<Vec<PlaceResource>, StorageError> {
        self.select(
            &format!("{PLACE_RESOURCE} ORDER BY t.id"),
            [],
            place_resource,
        )
    }
}

impl Writer<PlaceResource> for SqliteModel<'_> {
    fn insert(&mut self, record: &PlaceResource) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO place_resource (id, place_id, resource, abundance, note) VALUES (?1, ?2, ?3, ?4, ?5)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.place.to_string()),
            text(&record.resource),
            opt_text(record.abundance.map(ResourceAbundance::as_str)),
            opt_text(record.note.clone()),
            ]),
        )
    }

    fn update(&mut self, record: &PlaceResource) -> Result<(), StorageError> {
        self.execute(
            "UPDATE place_resource SET place_id = ?2, resource = ?3, abundance = ?4, note = ?5 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.place.to_string()),
            text(&record.resource),
            opt_text(record.abundance.map(ResourceAbundance::as_str)),
            opt_text(record.note.clone()),
            ]),
        )
    }

    fn delete(&mut self, id: &PlaceResourceId) -> Result<(), StorageError> {
        self.cascade("place_resource", &id.to_string())
    }
}

pub(super) const CENSUS: &str = "SELECT t.id, t.place_id, i_taken_at.id AS taken_at__id, i_taken_at.day AS taken_at__day, i_taken_at.precision AS taken_at__precision, i_taken_at.earliest_day AS taken_at__earliest, i_taken_at.latest_day AS taken_at__latest, i_taken_at.calendar_id AS taken_at__calendar, i_taken_at.label AS taken_at__label, t.population, t.is_estimate, t.source_id, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM census AS t LEFT JOIN instant AS i_taken_at ON i_taken_at.id = t.taken_at";

pub(super) fn census(row: &Row<'_>) -> Result<Census, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<CensusId>("census", &key, &key)?;
    let place = parse::<EntityId>("census", &key, &row.get::<_, String>("place_id")?)?;
    let taken = read_instant(row, "taken_at")?
        .ok_or_else(|| corrupt("census", &key, &"it has no moment"))?;
    let population: i64 = row.get("population")?;
    let is_estimate: bool = row.get("is_estimate")?;
    let source =
        optional_parse::<SourceId>("census", &key, row.get::<_, Option<String>>("source_id")?)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key("census", &key, &row.get::<_, String>("canon_state")?)?;
    let origin = parse::<Origin>("census", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(Census {
        id,
        place,
        taken,
        population,
        is_estimate,
        source,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<Census> for SqliteModel<'_> {
    fn get(&self, id: &CensusId) -> Result<Option<Census>, StorageError> {
        self.select_one(
            &format!("{CENSUS} WHERE t.id = ?1"),
            [id.to_string()],
            census,
        )
    }

    fn list(&self) -> Result<Vec<Census>, StorageError> {
        self.select(&format!("{CENSUS} ORDER BY t.created_at, t.id"), [], census)
    }
}

impl Writer<Census> for SqliteModel<'_> {
    fn insert(&mut self, record: &Census) -> Result<(), StorageError> {
        let taken_at_id = self.write_instant(&record.taken)?;
        self.execute(
            "INSERT INTO census (id, place_id, taken_at, population, is_estimate, source_id, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.place.to_string()),
            text(taken_at_id),
            int(record.population),
            boolean(record.is_estimate),
            opt_text(record.source.map(|value| value.to_string())),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &Census) -> Result<(), StorageError> {
        let taken_at_id = self.write_instant(&record.taken)?;
        self.execute(
            "UPDATE census SET place_id = ?2, taken_at = ?3, population = ?4, is_estimate = ?5, source_id = ?6, note = ?7, canon_state = ?8, updated_at = ?9 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.place.to_string()),
            text(taken_at_id),
            int(record.population),
            boolean(record.is_estimate),
            opt_text(record.source.map(|value| value.to_string())),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &CensusId) -> Result<(), StorageError> {
        self.cascade("census", &id.to_string())
    }
}
