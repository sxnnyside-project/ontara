use ontara_core::{
    AssetFile, Attachment, AttachmentId, CanonStateChange, CanonStateChangeId, EntityId, Note,
    NoteId, Origin, Reader, RecordKind, RecordRef, Source, SourceId, SourceKind, SourceReference,
    SourceReferenceId, UniverseProfile, Writer,
};
use rusqlite::{Row, params};
use uuid::Uuid;

use super::SqliteModel;
use super::codec::{parse, timestamp, type_key};
use crate::error::StorageError;

const ASSET_FILE: &str = "SELECT entity_id, relative_path, media_type, size_bytes, content_hash, \
                          imported_at FROM asset_file";

fn asset_file(row: &Row<'_>) -> Result<AssetFile, StorageError> {
    let id: String = row.get("entity_id")?;
    let size: i64 = row.get("size_bytes")?;
    Ok(AssetFile {
        entity: parse::<EntityId>("asset_file", &id, &id)?,
        relative_path: row.get("relative_path")?,
        media_type: row.get("media_type")?,
        size_bytes: u64::try_from(size).unwrap_or(0),
        content_hash: row.get("content_hash")?,
        imported_at: timestamp(row.get("imported_at")?),
    })
}

impl Reader<AssetFile> for SqliteModel<'_> {
    fn get(&self, id: &EntityId) -> Result<Option<AssetFile>, StorageError> {
        self.select_one(
            &format!("{ASSET_FILE} WHERE entity_id = ?1"),
            [id.to_string()],
            asset_file,
        )
    }

    fn list(&self) -> Result<Vec<AssetFile>, StorageError> {
        self.select(
            &format!("{ASSET_FILE} ORDER BY relative_path"),
            [],
            asset_file,
        )
    }
}

impl Writer<AssetFile> for SqliteModel<'_> {
    fn insert(&mut self, value: &AssetFile) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO asset_file (entity_id, relative_path, media_type, size_bytes, \
             content_hash, imported_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                value.entity.to_string(),
                value.relative_path,
                value.media_type,
                i64::try_from(value.size_bytes).unwrap_or(i64::MAX),
                value.content_hash,
                value.imported_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, value: &AssetFile) -> Result<(), StorageError> {
        self.execute(
            "UPDATE asset_file SET relative_path = ?2, media_type = ?3, size_bytes = ?4, \
             content_hash = ?5, imported_at = ?6 WHERE entity_id = ?1",
            params![
                value.entity.to_string(),
                value.relative_path,
                value.media_type,
                i64::try_from(value.size_bytes).unwrap_or(i64::MAX),
                value.content_hash,
                value.imported_at.as_unix_millis()
            ],
        )
    }

    fn delete(&mut self, id: &EntityId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM asset_file WHERE entity_id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const ATTACHMENT: &str =
    "SELECT id, owner_id, asset_id, purpose, position, note, created_at FROM attachment";

pub(super) fn attachment(row: &Row<'_>) -> Result<Attachment, StorageError> {
    let id: String = row.get("id")?;
    let owner: String = row.get("owner_id")?;
    let asset: String = row.get("asset_id")?;
    Ok(Attachment {
        id: parse::<AttachmentId>("attachment", &id, &id)?,
        owner: parse::<EntityId>("attachment", &id, &owner)?,
        asset: parse::<EntityId>("attachment", &id, &asset)?,
        purpose: row.get("purpose")?,
        position: row.get("position")?,
        note: row.get("note")?,
        created_at: timestamp(row.get("created_at")?),
    })
}

impl Reader<Attachment> for SqliteModel<'_> {
    fn get(&self, id: &AttachmentId) -> Result<Option<Attachment>, StorageError> {
        self.select_one(
            &format!("{ATTACHMENT} WHERE id = ?1"),
            [id.to_string()],
            attachment,
        )
    }

    fn list(&self) -> Result<Vec<Attachment>, StorageError> {
        self.select(
            &format!("{ATTACHMENT} ORDER BY owner_id, position, id"),
            [],
            attachment,
        )
    }
}

impl Writer<Attachment> for SqliteModel<'_> {
    fn insert(&mut self, value: &Attachment) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO attachment (id, owner_id, asset_id, purpose, position, note, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                value.id.to_string(),
                value.owner.to_string(),
                value.asset.to_string(),
                value.purpose,
                value.position,
                value.note,
                value.created_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, value: &Attachment) -> Result<(), StorageError> {
        self.execute(
            "UPDATE attachment SET purpose = ?2, position = ?3, note = ?4 WHERE id = ?1",
            params![
                value.id.to_string(),
                value.purpose,
                value.position,
                value.note
            ],
        )
    }

    fn delete(&mut self, id: &AttachmentId) -> Result<(), StorageError> {
        self.execute("DELETE FROM attachment WHERE id = ?1", [id.to_string()])
    }
}

const SOURCE: &str = "SELECT id, kind, title, locator, note, created_at, updated_at FROM source";

fn source(row: &Row<'_>) -> Result<Source, StorageError> {
    let id: String = row.get("id")?;
    let kind: String = row.get("kind")?;
    Ok(Source {
        id: parse::<SourceId>("source", &id, &id)?,
        kind: parse::<SourceKind>("source", &id, &kind)?,
        title: row.get("title")?,
        locator: row.get("locator")?,
        note: row.get("note")?,
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

impl Reader<Source> for SqliteModel<'_> {
    fn get(&self, id: &SourceId) -> Result<Option<Source>, StorageError> {
        self.select_one(&format!("{SOURCE} WHERE id = ?1"), [id.to_string()], source)
    }

    fn list(&self) -> Result<Vec<Source>, StorageError> {
        self.select(&format!("{SOURCE} ORDER BY title, id"), [], source)
    }
}

impl Writer<Source> for SqliteModel<'_> {
    fn insert(&mut self, value: &Source) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO source (id, kind, title, locator, note, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                value.id.to_string(),
                value.kind.as_str(),
                value.title,
                value.locator,
                value.note,
                value.created_at.as_unix_millis(),
                value.updated_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, value: &Source) -> Result<(), StorageError> {
        self.execute(
            "UPDATE source SET kind = ?2, title = ?3, locator = ?4, note = ?5, updated_at = ?6 \
             WHERE id = ?1",
            params![
                value.id.to_string(),
                value.kind.as_str(),
                value.title,
                value.locator,
                value.note,
                value.updated_at.as_unix_millis()
            ],
        )
    }

    fn delete(&mut self, id: &SourceId) -> Result<(), StorageError> {
        self.execute("DELETE FROM source WHERE id = ?1", [id.to_string()])
    }
}

pub(super) const SOURCE_REFERENCE: &str = "SELECT id, source_id, target_table, target_id, locator, quote, created_at FROM source_reference";

fn record_ref(
    table: &'static str,
    id: &str,
    kind: &str,
    target: &str,
) -> Result<RecordRef, StorageError> {
    Ok(RecordRef::new(
        parse::<RecordKind>(table, id, kind)?,
        parse::<Uuid>(table, id, target)?,
    ))
}

pub(super) fn source_reference(row: &Row<'_>) -> Result<SourceReference, StorageError> {
    let id: String = row.get("id")?;
    let source: String = row.get("source_id")?;
    let kind: String = row.get("target_table")?;
    let target: String = row.get("target_id")?;
    Ok(SourceReference {
        id: parse::<SourceReferenceId>("source_reference", &id, &id)?,
        source: parse::<SourceId>("source_reference", &id, &source)?,
        target: record_ref("source_reference", &id, &kind, &target)?,
        locator: row.get("locator")?,
        quote: row.get("quote")?,
        created_at: timestamp(row.get("created_at")?),
    })
}

impl Reader<SourceReference> for SqliteModel<'_> {
    fn get(&self, id: &SourceReferenceId) -> Result<Option<SourceReference>, StorageError> {
        self.select_one(
            &format!("{SOURCE_REFERENCE} WHERE id = ?1"),
            [id.to_string()],
            source_reference,
        )
    }

    fn list(&self) -> Result<Vec<SourceReference>, StorageError> {
        self.select(
            &format!("{SOURCE_REFERENCE} ORDER BY created_at, id"),
            [],
            source_reference,
        )
    }
}

impl Writer<SourceReference> for SqliteModel<'_> {
    fn insert(&mut self, value: &SourceReference) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO source_reference (id, source_id, target_table, target_id, locator, quote, \
             created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                value.id.to_string(),
                value.source.to_string(),
                value.target.kind.as_str(),
                value.target.id.to_string(),
                value.locator,
                value.quote,
                value.created_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, value: &SourceReference) -> Result<(), StorageError> {
        self.execute(
            "UPDATE source_reference SET locator = ?2, quote = ?3 WHERE id = ?1",
            params![value.id.to_string(), value.locator, value.quote],
        )
    }

    fn delete(&mut self, id: &SourceReferenceId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM source_reference WHERE id = ?1",
            [id.to_string()],
        )
    }
}

pub(super) const NOTE: &str = "SELECT id, entity_id, title, body, created_at, updated_at FROM note";

pub(super) fn note(row: &Row<'_>) -> Result<Note, StorageError> {
    let id: String = row.get("id")?;
    let entity: String = row.get("entity_id")?;
    Ok(Note {
        id: parse::<NoteId>("note", &id, &id)?,
        entity: parse::<EntityId>("note", &id, &entity)?,
        title: row.get("title")?,
        body: row.get("body")?,
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

impl Reader<Note> for SqliteModel<'_> {
    fn get(&self, id: &NoteId) -> Result<Option<Note>, StorageError> {
        self.select_one(&format!("{NOTE} WHERE id = ?1"), [id.to_string()], note)
    }

    fn list(&self) -> Result<Vec<Note>, StorageError> {
        self.select(&format!("{NOTE} ORDER BY created_at, id"), [], note)
    }
}

impl Writer<Note> for SqliteModel<'_> {
    fn insert(&mut self, value: &Note) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO note (id, entity_id, title, body, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                value.id.to_string(),
                value.entity.to_string(),
                value.title,
                value.body,
                value.created_at.as_unix_millis(),
                value.updated_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, value: &Note) -> Result<(), StorageError> {
        self.execute(
            "UPDATE note SET title = ?2, body = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                value.id.to_string(),
                value.title,
                value.body,
                value.updated_at.as_unix_millis()
            ],
        )
    }

    fn delete(&mut self, id: &NoteId) -> Result<(), StorageError> {
        self.execute("DELETE FROM note WHERE id = ?1", [id.to_string()])
    }
}

pub(super) const CANON_STATE_CHANGE: &str = "SELECT id, target_table, target_id, from_state, \
    to_state, reason, origin, changed_at FROM canon_state_change";

pub(super) fn canon_state_change(row: &Row<'_>) -> Result<CanonStateChange, StorageError> {
    let id: String = row.get("id")?;
    let kind: String = row.get("target_table")?;
    let target: String = row.get("target_id")?;
    let from: String = row.get("from_state")?;
    let to: String = row.get("to_state")?;
    let origin: String = row.get("origin")?;
    Ok(CanonStateChange {
        id: parse::<CanonStateChangeId>("canon_state_change", &id, &id)?,
        target: record_ref("canon_state_change", &id, &kind, &target)?,
        from: type_key("canon_state_change", &id, &from)?,
        to: type_key("canon_state_change", &id, &to)?,
        reason: row.get("reason")?,
        origin: parse::<Origin>("canon_state_change", &id, &origin)?,
        changed_at: timestamp(row.get("changed_at")?),
    })
}

impl Reader<CanonStateChange> for SqliteModel<'_> {
    fn get(&self, id: &CanonStateChangeId) -> Result<Option<CanonStateChange>, StorageError> {
        self.select_one(
            &format!("{CANON_STATE_CHANGE} WHERE id = ?1"),
            [id.to_string()],
            canon_state_change,
        )
    }

    fn list(&self) -> Result<Vec<CanonStateChange>, StorageError> {
        self.select(
            &format!("{CANON_STATE_CHANGE} ORDER BY changed_at, id"),
            [],
            canon_state_change,
        )
    }
}

/// History is append-only: a change can be recorded, never edited.
impl Writer<CanonStateChange> for SqliteModel<'_> {
    fn insert(&mut self, value: &CanonStateChange) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO canon_state_change (id, target_table, target_id, from_state, to_state, \
             reason, origin, changed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                value.id.to_string(),
                value.target.kind.as_str(),
                value.target.id.to_string(),
                value.from.as_str(),
                value.to.as_str(),
                value.reason,
                value.origin.as_str(),
                value.changed_at.as_unix_millis()
            ],
        )
    }

    fn update(&mut self, _: &CanonStateChange) -> Result<(), StorageError> {
        Err(StorageError::Encoding(
            "canon state history cannot be edited".to_owned(),
        ))
    }

    fn delete(&mut self, _: &CanonStateChangeId) -> Result<(), StorageError> {
        Err(StorageError::Encoding(
            "canon state history cannot be deleted".to_owned(),
        ))
    }
}

impl Reader<UniverseProfile> for SqliteModel<'_> {
    fn get(&self, (): &()) -> Result<Option<UniverseProfile>, StorageError> {
        self.select_one(
            "SELECT summary, description, updated_at FROM universe_profile WHERE id = 1",
            [],
            |row| {
                Ok(UniverseProfile {
                    summary: row.get("summary")?,
                    description: row.get("description")?,
                    updated_at: row.get::<_, Option<i64>>("updated_at")?.map(timestamp),
                })
            },
        )
    }

    fn list(&self) -> Result<Vec<UniverseProfile>, StorageError> {
        Ok(Reader::<UniverseProfile>::get(self, &())?
            .into_iter()
            .collect())
    }
}

impl Writer<UniverseProfile> for SqliteModel<'_> {
    fn insert(&mut self, value: &UniverseProfile) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO universe_profile (id, summary, description, updated_at) \
             VALUES (1, ?1, ?2, ?3) \
             ON CONFLICT (id) DO UPDATE SET summary = ?1, description = ?2, updated_at = ?3",
            params![
                value.summary,
                value.description,
                value.updated_at.map(ontara_core::Timestamp::as_unix_millis)
            ],
        )
    }

    fn update(&mut self, value: &UniverseProfile) -> Result<(), StorageError> {
        Writer::<UniverseProfile>::insert(self, value)
    }

    fn delete(&mut self, (): &()) -> Result<(), StorageError> {
        self.execute("DELETE FROM universe_profile WHERE id = 1", [])
    }
}
