#![allow(clippy::similar_names)] // Bindings are the model's own field names.

use ontara_core::{
    CausalityKind, EntityId, EventCausality, EventCausalityId, Origin, Reader, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{int, opt_text, parse, text, timestamp, type_key};
use crate::error::StorageError;

pub(super) const EVENT_CAUSALITY: &str = "SELECT t.id, t.cause_id, t.effect_id, t.kind, t.note, t.canon_state, t.origin, t.created_at, t.updated_at FROM event_causality AS t";

pub(super) fn event_causality(row: &Row<'_>) -> Result<EventCausality, StorageError> {
    let key: String = row.get("id")?;
    let id = parse::<EventCausalityId>("event_causality", &key, &key)?;
    let cause = parse::<EntityId>("event_causality", &key, &row.get::<_, String>("cause_id")?)?;
    let effect = parse::<EntityId>("event_causality", &key, &row.get::<_, String>("effect_id")?)?;
    let kind = parse::<CausalityKind>("event_causality", &key, &row.get::<_, String>("kind")?)?;
    let note: Option<String> = row.get("note")?;
    let canon_state = type_key(
        "event_causality",
        &key,
        &row.get::<_, String>("canon_state")?,
    )?;
    let origin = parse::<Origin>("event_causality", &key, &row.get::<_, String>("origin")?)?;
    let created_at = timestamp(row.get("created_at")?);
    let updated_at = timestamp(row.get("updated_at")?);
    Ok(EventCausality {
        id,
        cause,
        effect,
        kind,
        note,
        canon_state,
        origin,
        created_at,
        updated_at,
    })
}

impl Reader<EventCausality> for SqliteModel<'_> {
    fn get(&self, id: &EventCausalityId) -> Result<Option<EventCausality>, StorageError> {
        self.select_one(
            &format!("{EVENT_CAUSALITY} WHERE t.id = ?1"),
            [id.to_string()],
            event_causality,
        )
    }

    fn list(&self) -> Result<Vec<EventCausality>, StorageError> {
        self.select(
            &format!("{EVENT_CAUSALITY} ORDER BY t.created_at, t.id"),
            [],
            event_causality,
        )
    }
}

impl Writer<EventCausality> for SqliteModel<'_> {
    fn insert(&mut self, record: &EventCausality) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO event_causality (id, cause_id, effect_id, kind, note, canon_state, origin, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.cause.to_string()),
            text(record.effect.to_string()),
            text(record.kind.as_str()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            text(record.origin.as_str()),
            int(record.created_at.as_unix_millis()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &EventCausality) -> Result<(), StorageError> {
        self.execute(
            "UPDATE event_causality SET cause_id = ?2, effect_id = ?3, kind = ?4, note = ?5, canon_state = ?6, updated_at = ?7 WHERE id = ?1",
            params_from_iter(vec![
            text(record.id.to_string()),
            text(record.cause.to_string()),
            text(record.effect.to_string()),
            text(record.kind.as_str()),
            opt_text(record.note.clone()),
            text(record.canon_state.as_str()),
            int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &EventCausalityId) -> Result<(), StorageError> {
        self.cascade("event_causality", &id.to_string())
    }
}
