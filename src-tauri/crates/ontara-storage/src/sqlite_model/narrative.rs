use ontara_core::{
    Appearance, AppearanceId, EditorialNode, EntityId, EventDetail, EventParticipant, EventScale,
    Origin, Reader, Writer,
};
use rusqlite::{Row, params};

use super::SqliteModel;
use super::codec::{
    corrupt, instant_columns, instant_join, parse, read_period, timestamp, type_key,
};
use crate::error::StorageError;

pub(super) const APPEARANCE: &str = "SELECT id, entity_id, context_id, kind, position, \
    viewpoint_id, note, canon_state, origin, created_at, updated_at FROM appearance";

pub(super) fn appearance(row: &Row<'_>) -> Result<Appearance, StorageError> {
    let id: String = row.get("id")?;
    let entity: String = row.get("entity_id")?;
    let context: String = row.get("context_id")?;
    let kind: String = row.get("kind")?;
    let viewpoint: Option<String> = row.get("viewpoint_id")?;
    let canon_state: String = row.get("canon_state")?;
    let origin: String = row.get("origin")?;
    Ok(Appearance {
        id: parse::<AppearanceId>("appearance", &id, &id)?,
        entity: parse::<EntityId>("appearance", &id, &entity)?,
        context: parse::<EntityId>("appearance", &id, &context)?,
        kind: type_key("appearance", &id, &kind)?,
        position: row.get("position")?,
        viewpoint: viewpoint
            .map(|raw| parse::<EntityId>("appearance", &id, &raw))
            .transpose()?,
        note: row.get("note")?,
        canon_state: type_key("appearance", &id, &canon_state)?,
        origin: parse::<Origin>("appearance", &id, &origin)?,
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

impl Reader<Appearance> for SqliteModel<'_> {
    fn get(&self, id: &AppearanceId) -> Result<Option<Appearance>, StorageError> {
        self.select_one(
            &format!("{APPEARANCE} WHERE id = ?1"),
            [id.to_string()],
            appearance,
        )
    }

    fn list(&self) -> Result<Vec<Appearance>, StorageError> {
        self.select(
            &format!("{APPEARANCE} ORDER BY context_id, position, id"),
            [],
            appearance,
        )
    }
}

impl Writer<Appearance> for SqliteModel<'_> {
    fn insert(&mut self, value: &Appearance) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO appearance (id, entity_id, context_id, kind, position, viewpoint_id, \
             note, canon_state, origin, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                value.id.to_string(),
                value.entity.to_string(),
                value.context.to_string(),
                value.kind.as_str(),
                value.position,
                value.viewpoint.map(|entity| entity.to_string()),
                value.note,
                value.canon_state.as_str(),
                value.origin.as_str(),
                value.created_at.as_unix_millis(),
                value.updated_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, value: &Appearance) -> Result<(), StorageError> {
        self.execute(
            "UPDATE appearance SET entity_id = ?2, context_id = ?3, kind = ?4, position = ?5, \
             viewpoint_id = ?6, note = ?7, canon_state = ?8, updated_at = ?9 WHERE id = ?1",
            params![
                value.id.to_string(),
                value.entity.to_string(),
                value.context.to_string(),
                value.kind.as_str(),
                value.position,
                value.viewpoint.map(|entity| entity.to_string()),
                value.note,
                value.canon_state.as_str(),
                value.updated_at.as_unix_millis()
            ],
        )
    }

    fn delete(&mut self, id: &AppearanceId) -> Result<(), StorageError> {
        self.execute("DELETE FROM appearance WHERE id = ?1", [id.to_string()])
    }
}

fn event_detail_sql() -> String {
    format!(
        "SELECT t.entity_id, t.time_state, t.location_id, t.scale, t.consequences, {}, {} \
         FROM event_detail AS t {} {}",
        instant_columns("starts_at"),
        instant_columns("ends_at"),
        instant_join("starts_at"),
        instant_join("ends_at"),
    )
}

fn event_detail(row: &Row<'_>) -> Result<EventDetail, StorageError> {
    let id: String = row.get("entity_id")?;
    let location: Option<String> = row.get("location_id")?;
    let scale: Option<String> = row.get("scale")?;
    Ok(EventDetail {
        entity: parse::<EntityId>("event_detail", &id, &id)?,
        period: read_period(row, "event_detail", &id)?,
        location: location
            .map(|raw| parse::<EntityId>("event_detail", &id, &raw))
            .transpose()?,
        scale: scale
            .map(|raw| parse::<EventScale>("event_detail", &id, &raw))
            .transpose()?,
        consequences: row.get("consequences")?,
        participants: Vec::new(),
    })
}

impl SqliteModel<'_> {
    fn attach_participants(&self, events: &mut [EventDetail]) -> Result<(), StorageError> {
        for event in events {
            let id = event.entity.to_string();
            let mut statement = self.connection.prepare_cached(
                "SELECT participant_id, role, side FROM event_participant \
                 WHERE event_id = ?1 ORDER BY position, participant_id",
            )?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                let participant: String = row.get(0)?;
                event.participants.push(EventParticipant {
                    entity: parse::<EntityId>("event_participant", &id, &participant)?,
                    role: row.get(1)?,
                    side: row.get(2)?,
                });
            }
        }
        Ok(())
    }

    fn write_participants(&self, event: &EventDetail) -> Result<(), StorageError> {
        let id = event.entity.to_string();
        self.execute("DELETE FROM event_participant WHERE event_id = ?1", [&id])?;
        for (position, participant) in event.participants.iter().enumerate() {
            self.execute(
                "INSERT INTO event_participant (event_id, participant_id, role, side, position) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    id,
                    participant.entity.to_string(),
                    participant.role,
                    participant.side,
                    i64::try_from(position).unwrap_or(i64::MAX)
                ],
            )?;
        }
        Ok(())
    }

    pub(super) fn events_where(
        &self,
        clause: &str,
        parameters: impl rusqlite::Params,
    ) -> Result<Vec<EventDetail>, StorageError> {
        let mut events = self.select(
            &format!("{} {clause}", event_detail_sql()),
            parameters,
            event_detail,
        )?;
        self.attach_participants(&mut events)?;
        Ok(events)
    }
}

impl Reader<EventDetail> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<EventDetail>, StorageError> {
        Ok(self
            .events_where("WHERE t.entity_id = ?1", [id.to_string()])?
            .into_iter()
            .next())
    }

    fn list(&self) -> Result<Vec<EventDetail>, StorageError> {
        self.events_where("ORDER BY t.entity_id", [])
    }
}

impl Writer<EventDetail> for SqliteModel<'_> {
    fn insert(&mut self, value: &EventDetail) -> Result<(), StorageError> {
        let (state, starts, ends) = self.write_period(&value.period)?;
        self.execute(
            "INSERT INTO event_detail (entity_id, time_state, starts_at, ends_at, location_id, \
             scale, consequences) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                value.entity.to_string(),
                state,
                starts,
                ends,
                value.location.map(|entity| entity.to_string()),
                value.scale.map(EventScale::as_str),
                value.consequences
            ],
        )?;
        self.write_participants(value)
    }

    fn update(&mut self, value: &EventDetail) -> Result<(), StorageError> {
        let (state, starts, ends) = self.write_period(&value.period)?;
        self.execute(
            "UPDATE event_detail SET time_state = ?2, starts_at = ?3, ends_at = ?4, \
             location_id = ?5, scale = ?6, consequences = ?7 WHERE entity_id = ?1",
            params![
                value.entity.to_string(),
                state,
                starts,
                ends,
                value.location.map(|entity| entity.to_string()),
                value.scale.map(EventScale::as_str),
                value.consequences
            ],
        )?;
        self.write_participants(value)
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM event_participant WHERE event_id = ?1",
            [id.to_string()],
        )?;
        self.execute(
            "DELETE FROM event_detail WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const EDITORIAL_NODE: &str =
    "SELECT entity_id, level, parent_id, position FROM editorial_node";

pub(super) fn editorial_node(row: &Row<'_>) -> Result<EditorialNode, StorageError> {
    let id: String = row.get("entity_id")?;
    let level: String = row.get("level")?;
    let parent: Option<String> = row.get("parent_id")?;
    if parent.as_deref() == Some(id.as_str()) {
        return Err(corrupt("editorial_node", &id, &"a node is its own parent"));
    }
    Ok(EditorialNode {
        entity: parse::<EntityId>("editorial_node", &id, &id)?,
        level: type_key("editorial_node", &id, &level)?,
        parent: parent
            .map(|raw| parse::<EntityId>("editorial_node", &id, &raw))
            .transpose()?,
        position: row.get("position")?,
    })
}

impl Reader<EditorialNode> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<EditorialNode>, StorageError> {
        self.select_one(
            &format!("{EDITORIAL_NODE} WHERE entity_id = ?1"),
            [id.to_string()],
            editorial_node,
        )
    }

    fn list(&self) -> Result<Vec<EditorialNode>, StorageError> {
        self.select(
            &format!("{EDITORIAL_NODE} ORDER BY parent_id, position, entity_id"),
            [],
            editorial_node,
        )
    }
}

impl Writer<EditorialNode> for SqliteModel<'_> {
    fn insert(&mut self, value: &EditorialNode) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO editorial_node (entity_id, level, parent_id, position) \
             VALUES (?1, ?2, ?3, ?4)",
            params![
                value.entity.to_string(),
                value.level.as_str(),
                value.parent.map(|parent| parent.to_string()),
                value.position
            ],
        )
    }

    fn update(&mut self, value: &EditorialNode) -> Result<(), StorageError> {
        self.execute(
            "UPDATE editorial_node SET level = ?2, parent_id = ?3, position = ?4 \
             WHERE entity_id = ?1",
            params![
                value.entity.to_string(),
                value.level.as_str(),
                value.parent.map(|parent| parent.to_string()),
                value.position
            ],
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM editorial_node WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}
