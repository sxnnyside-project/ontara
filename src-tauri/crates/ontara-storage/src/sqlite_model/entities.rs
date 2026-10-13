use std::collections::{BTreeMap, BTreeSet};

use ontara_core::{
    Entity, EntityId, EntityName, Lifecycle, NameKind, Origin, Reader, Relationship,
    RelationshipId, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    boolean, instant_columns, instant_join, int, opt_text, parse, read_period, text, timestamp,
    type_key,
};
use crate::error::StorageError;

const ENTITY: &str = "SELECT t.id, t.entity_type, t.name, t.sort_name, t.summary, t.description, \
                      t.canon_state, t.archived, t.origin, t.created_at, t.updated_at \
                      FROM entity AS t";

fn entity(row: &Row<'_>) -> Result<Entity, StorageError> {
    let id: String = row.get("id")?;
    let entity_type: String = row.get("entity_type")?;
    let canon_state: String = row.get("canon_state")?;
    let origin: String = row.get("origin")?;
    Ok(Entity {
        id: parse::<EntityId>("entity", &id, &id)?,
        entity_type: type_key("entity", &id, &entity_type)?,
        name: row.get("name")?,
        sort_name: row.get("sort_name")?,
        names: Vec::new(),
        summary: row.get("summary")?,
        description: row.get("description")?,
        tags: BTreeSet::new(),
        canon_state: type_key("entity", &id, &canon_state)?,
        archived: row.get("archived")?,
        origin: parse::<Origin>("entity", &id, &origin)?,
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

fn name_sql() -> String {
    format!(
        "SELECT t.entity_id, t.name, t.kind, t.language_id, t.is_preferred, t.note, t.time_state, \
         {}, {} FROM entity_name AS t {} {} ORDER BY t.entity_id, t.rowid",
        instant_columns("starts_at"),
        instant_columns("ends_at"),
        instant_join("starts_at"),
        instant_join("ends_at"),
    )
}

impl SqliteModel<'_> {
    /// Loads the names and tags of the given entities in two queries, not two per entity.
    fn attach_children(&self, entities: &mut [Entity]) -> Result<(), StorageError> {
        if entities.is_empty() {
            return Ok(());
        }
        let mut names: BTreeMap<String, Vec<EntityName>> = BTreeMap::new();
        let mut statement = self.connection.prepare_cached(&name_sql())?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let owner: String = row.get("entity_id")?;
            let kind: String = row.get("kind")?;
            let language: Option<String> = row.get("language_id")?;
            names.entry(owner.clone()).or_default().push(EntityName {
                name: row.get("name")?,
                kind: parse::<NameKind>("entity_name", &owner, &kind)?,
                language: language
                    .map(|raw| parse::<EntityId>("entity_name", &owner, &raw))
                    .transpose()?,
                is_preferred: row.get("is_preferred")?,
                period: read_period(row, "entity_name", &owner)?,
                note: row.get("note")?,
            });
        }

        let mut tags: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut statement = self.connection.prepare_cached(
            "SELECT entity_tag.entity_id, tag.name FROM entity_tag \
             JOIN tag ON tag.id = entity_tag.tag_id",
        )?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let owner: String = row.get(0)?;
            tags.entry(owner).or_default().insert(row.get(1)?);
        }

        for record in entities {
            let key = record.id.to_string();
            record.names = names.remove(&key).unwrap_or_default();
            record.tags = tags.remove(&key).unwrap_or_default();
        }
        Ok(())
    }

    fn write_names(&self, record: &Entity) -> Result<(), StorageError> {
        let id = record.id.to_string();
        self.execute("DELETE FROM entity_name WHERE entity_id = ?1", [&id])?;
        for name in &record.names {
            let (state, starts, ends) = self.write_period(&name.period)?;
            self.execute(
                "INSERT INTO entity_name (id, entity_id, name, kind, language_id, is_preferred, \
                 note, time_state, starts_at, ends_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params_from_iter(vec![
                    text(uuid::Uuid::now_v7().to_string()),
                    text(&id),
                    text(&name.name),
                    text(name.kind.as_str()),
                    opt_text(name.language.map(|language| language.to_string())),
                    boolean(name.is_preferred),
                    opt_text(name.note.clone()),
                    text(state),
                    opt_text(starts),
                    opt_text(ends),
                ]),
            )?;
        }
        Ok(())
    }

    fn write_tags(&self, record: &Entity) -> Result<(), StorageError> {
        let id = record.id.to_string();
        self.execute("DELETE FROM entity_tag WHERE entity_id = ?1", [&id])?;
        for tag in &record.tags {
            self.execute("INSERT OR IGNORE INTO tag (name) VALUES (?1)", [tag])?;
            self.execute(
                "INSERT INTO entity_tag (entity_id, tag_id) SELECT ?1, id FROM tag WHERE name = ?2",
                [&id, tag],
            )?;
        }
        self.execute(
            "DELETE FROM tag WHERE id NOT IN (SELECT tag_id FROM entity_tag)",
            [],
        )
    }
}

impl Reader<Entity> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<Entity>, StorageError> {
        let mut found = self.select(
            &format!("{ENTITY} WHERE t.id = ?1"),
            [id.to_string()],
            entity,
        )?;
        if found.is_empty() {
            return Ok(None);
        }
        self.attach_children(&mut found)?;
        Ok(found.into_iter().next())
    }

    fn list(&self) -> Result<Vec<Entity>, StorageError> {
        let mut all = self.select(&format!("{ENTITY} ORDER BY t.created_at, t.id"), [], entity)?;
        self.attach_children(&mut all)?;
        Ok(all)
    }
}

impl Writer<Entity> for SqliteModel<'_> {
    fn insert(&mut self, record: &Entity) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO entity (id, entity_type, name, sort_name, summary, description, \
             archived, canon_state, origin, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.entity_type.as_str()),
                text(&record.name),
                opt_text(record.sort_name.clone()),
                text(&record.summary),
                text(&record.description),
                boolean(record.archived),
                text(record.canon_state.as_str()),
                text(record.origin.as_str()),
                int(record.created_at.as_unix_millis()),
                int(record.updated_at.as_unix_millis()),
            ]),
        )?;
        self.write_names(record)?;
        self.write_tags(record)
    }

    fn update(&mut self, record: &Entity) -> Result<(), StorageError> {
        self.execute(
            "UPDATE entity SET name = ?2, sort_name = ?3, summary = ?4, description = ?5, \
             archived = ?6, canon_state = ?7, updated_at = ?8 WHERE id = ?1",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(&record.name),
                opt_text(record.sort_name.clone()),
                text(&record.summary),
                text(&record.description),
                boolean(record.archived),
                text(record.canon_state.as_str()),
                int(record.updated_at.as_unix_millis()),
            ]),
        )?;
        self.write_names(record)?;
        self.write_tags(record)
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.cascade("entity", &id.to_string())?;
        self.execute(
            "DELETE FROM tag WHERE id NOT IN (SELECT tag_id FROM entity_tag)",
            [],
        )
    }
}

pub(super) fn relationship_sql() -> String {
    format!(
        "SELECT t.id, t.relationship_type, t.source_id, t.target_id, t.qualifier, t.context_id, \
         t.lifecycle, t.time_state, t.note, t.canon_state, t.origin, t.created_at, t.updated_at, \
         {}, {} FROM relationship AS t {} {}",
        instant_columns("starts_at"),
        instant_columns("ends_at"),
        instant_join("starts_at"),
        instant_join("ends_at"),
    )
}

pub(super) fn relationship(row: &Row<'_>) -> Result<Relationship, StorageError> {
    let id: String = row.get("id")?;
    let relationship_type: String = row.get("relationship_type")?;
    let source: String = row.get("source_id")?;
    let target: String = row.get("target_id")?;
    let context: Option<String> = row.get("context_id")?;
    let lifecycle: String = row.get("lifecycle")?;
    let canon_state: String = row.get("canon_state")?;
    let origin: String = row.get("origin")?;
    Ok(Relationship {
        id: parse::<RelationshipId>("relationship", &id, &id)?,
        relationship_type: type_key("relationship", &id, &relationship_type)?,
        source: parse::<EntityId>("relationship", &id, &source)?,
        target: parse::<EntityId>("relationship", &id, &target)?,
        qualifier: row.get("qualifier")?,
        context: context
            .map(|raw| parse::<EntityId>("relationship", &id, &raw))
            .transpose()?,
        lifecycle: parse::<Lifecycle>("relationship", &id, &lifecycle)?,
        period: read_period(row, "relationship", &id)?,
        note: row.get("note")?,
        canon_state: type_key("relationship", &id, &canon_state)?,
        origin: parse::<Origin>("relationship", &id, &origin)?,
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

impl Reader<Relationship> for SqliteModel<'_> {
    fn get(&self, id: &RelationshipId) -> Result<Option<Relationship>, StorageError> {
        self.select_one(
            &format!("{} WHERE t.id = ?1", relationship_sql()),
            [id.to_string()],
            relationship,
        )
    }

    fn list(&self) -> Result<Vec<Relationship>, StorageError> {
        self.select(
            &format!("{} ORDER BY t.created_at, t.id", relationship_sql()),
            [],
            relationship,
        )
    }
}

impl Writer<Relationship> for SqliteModel<'_> {
    fn insert(&mut self, record: &Relationship) -> Result<(), StorageError> {
        let (state, starts, ends) = self.write_period(&record.period)?;
        self.execute(
            "INSERT INTO relationship (id, relationship_type, source_id, target_id, qualifier, \
             context_id, lifecycle, time_state, starts_at, ends_at, note, canon_state, origin, \
             created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.relationship_type.as_str()),
                text(record.source.to_string()),
                text(record.target.to_string()),
                opt_text(record.qualifier.clone()),
                opt_text(record.context.map(|context| context.to_string())),
                text(record.lifecycle.as_str()),
                text(state),
                opt_text(starts),
                opt_text(ends),
                opt_text(record.note.clone()),
                text(record.canon_state.as_str()),
                text(record.origin.as_str()),
                int(record.created_at.as_unix_millis()),
                int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, record: &Relationship) -> Result<(), StorageError> {
        let (state, starts, ends) = self.write_period(&record.period)?;
        self.execute(
            "UPDATE relationship SET relationship_type = ?2, source_id = ?3, target_id = ?4, \
             qualifier = ?5, context_id = ?6, lifecycle = ?7, time_state = ?8, starts_at = ?9, \
             ends_at = ?10, note = ?11, canon_state = ?12, updated_at = ?13 WHERE id = ?1",
            params_from_iter(vec![
                text(record.id.to_string()),
                text(record.relationship_type.as_str()),
                text(record.source.to_string()),
                text(record.target.to_string()),
                opt_text(record.qualifier.clone()),
                opt_text(record.context.map(|context| context.to_string())),
                text(record.lifecycle.as_str()),
                text(state),
                opt_text(starts),
                opt_text(ends),
                opt_text(record.note.clone()),
                text(record.canon_state.as_str()),
                int(record.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &RelationshipId) -> Result<(), StorageError> {
        self.cascade("relationship", &id.to_string())
    }
}
