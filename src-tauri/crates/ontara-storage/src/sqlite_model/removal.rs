use ontara_core::{CatalogEntry, RecordRef, RecordRemoval, ReferenceCount};
use rusqlite::Connection;

use super::SqliteModel;
use crate::error::StorageError;

/// Tables whose rows are part of the entity they point at, so they are not "references" to it:
/// its own names, traits, profile and the like. `(table, column)`.
#[rustfmt::skip]
const OWNED_BY_ENTITY: &[(&str, &str)] = &[
    ("entity_name", "entity_id"), ("entity_tag", "entity_id"), ("note", "entity_id"),
    ("being_profile", "entity_id"), ("being_appearance", "being_id"), ("being_mark", "being_id"),
    ("being_trait", "being_id"), ("being_language", "being_id"), ("deity_profile", "entity_id"),
    ("deity_domain", "deity_id"), ("species_profile", "entity_id"), ("place_profile", "entity_id"),
    ("place_resource", "place_id"), ("census", "place_id"), ("polity_profile", "entity_id"),
    ("polity_language", "polity_id"), ("polity_symbol", "polity_id"), ("office", "body_id"),
    ("faction_profile", "entity_id"), ("culture_practice", "culture_id"),
    ("culture_language", "culture_id"), ("language_profile", "entity_id"),
    ("religion_profile", "entity_id"), ("religion_tenet", "religion_id"),
    ("item_profile", "entity_id"), ("item_power", "item_id"), ("event_detail", "entity_id"),
    ("editorial_node", "entity_id"), ("asset_file", "entity_id"), ("property_value", "entity_id"),
    ("attachment", "owner_id"),
];

/// Tables whose rows belong to the catalog entry they point at, so they are not "uses" of it:
/// a relationship type's endpoint rules and the property definitions a type owns.
/// `(table, column)`.
const OWNED_BY_CATALOG_ENTRY: &[(&str, &str)] = &[
    ("relationship_type_endpoint", "relationship_type"),
    ("property_definition", "entity_type"),
    ("property_definition", "relationship_type"),
];

/// Optional columns that still cannot be cleared, because the row means nothing without them
/// (a property value must have exactly one value). The row is removed instead.
const REMOVE_ROW_WHEN_NULLABLE: &[(&str, &str)] = &[
    ("property_value", "entity_id"),
    ("property_value", "relationship_id"),
    ("property_value", "value_entity_id"),
];

struct ForeignKey {
    child_table: String,
    child_column: String,
    parent_column: String,
    nullable: bool,
}

impl SqliteModel<'_> {
    fn table_names(connection: &Connection) -> Result<Vec<String>, StorageError> {
        let mut statement = connection.prepare_cached(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )?;
        let names = statement.query_map([], |row| row.get::<_, String>(0))?;
        Ok(names.collect::<Result<_, _>>()?)
    }

    /// Foreign keys, in any table, that point at `parent`.
    fn foreign_keys_to(&self, parent: &str) -> Result<Vec<ForeignKey>, StorageError> {
        let mut found = Vec::new();
        for table in Self::table_names(self.connection)? {
            let mut columns = self
                .connection
                .prepare(&format!("PRAGMA table_info({table})"))?;
            let nullable: Vec<(String, bool)> = columns
                .query_map([], |row| {
                    Ok((row.get::<_, String>(1)?, row.get::<_, i64>(3)? == 0))
                })?
                .collect::<Result<_, _>>()?;
            let mut keys = self
                .connection
                .prepare(&format!("PRAGMA foreign_key_list({table})"))?;
            let rows = keys.query_map([], |row| {
                Ok((
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })?;
            for row in rows {
                let (target, from, to) = row?;
                if target == parent {
                    let is_nullable = nullable
                        .iter()
                        .find(|(name, _)| *name == from)
                        .is_some_and(|(_, n)| *n);
                    found.push(ForeignKey {
                        child_table: table.clone(),
                        child_column: from,
                        parent_column: to.unwrap_or_else(|| "id".to_owned()),
                        nullable: is_nullable,
                    });
                }
            }
        }
        Ok(found)
    }

    fn single_primary_key(&self, table: &str) -> Result<Option<String>, StorageError> {
        let mut statement = self
            .connection
            .prepare(&format!("PRAGMA table_info({table})"))?;
        let keys: Vec<String> = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, i64>(5)?))
            })?
            .filter_map(|row| match row {
                Ok((name, position)) if position > 0 => Some(Ok(name)),
                Ok(_) => None,
                Err(error) => Some(Err(error)),
            })
            .collect::<Result<_, _>>()?;
        Ok(if keys.len() == 1 {
            keys.into_iter().next()
        } else {
            None
        })
    }

    fn ids_where(
        &self,
        table: &str,
        key: &str,
        column: &str,
        ids: &[String],
    ) -> Result<Vec<String>, StorageError> {
        let marks = placeholders(ids.len());
        let mut statement = self.connection.prepare(&format!(
            "SELECT {key} FROM {table} WHERE {column} IN ({marks})"
        ))?;
        let rows = statement.query_map(rusqlite::params_from_iter(ids), |row| {
            row.get::<_, String>(0)
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Deletes rows, first deleting or clearing everything that points at them, and their
    /// citations. Driven by the schema's foreign keys, so a new table is covered automatically.
    pub(super) fn delete_rows(
        &self,
        table: &str,
        key: &str,
        ids: &[String],
    ) -> Result<(), StorageError> {
        if ids.is_empty() {
            return Ok(());
        }
        let marks = placeholders(ids.len());
        let mut parameters: Vec<String> = vec![table.to_owned()];
        parameters.extend(ids.iter().cloned());
        self.connection.execute(
            &format!(
                "DELETE FROM source_reference WHERE target_table = ?1 AND target_id IN ({})",
                placeholders_from(2, ids.len())
            ),
            rusqlite::params_from_iter(&parameters),
        )?;

        for foreign_key in self.foreign_keys_to(table)? {
            if foreign_key.parent_column != key {
                continue;
            }
            let ForeignKey {
                child_table,
                child_column,
                nullable,
                ..
            } = &foreign_key;
            let clears = *nullable
                && !REMOVE_ROW_WHEN_NULLABLE
                    .iter()
                    .any(|(t, c)| t == child_table && c == child_column);
            if clears {
                self.connection.execute(
                    &format!("UPDATE {child_table} SET {child_column} = NULL WHERE {child_column} IN ({marks})"),
                    rusqlite::params_from_iter(ids),
                )?;
            } else if let Some(child_key) = self.single_primary_key(child_table)? {
                let child_ids = self.ids_where(child_table, &child_key, child_column, ids)?;
                if child_table == table && child_column == &child_key {
                    continue;
                }
                self.delete_rows(child_table, &child_key, &child_ids)?;
            } else {
                self.connection.execute(
                    &format!("DELETE FROM {child_table} WHERE {child_column} IN ({marks})"),
                    rusqlite::params_from_iter(ids),
                )?;
            }
        }
        self.connection.execute(
            &format!("DELETE FROM {table} WHERE {key} IN ({marks})"),
            rusqlite::params_from_iter(ids),
        )?;
        Ok(())
    }

    /// Removes one row of a table keyed by `id`, with everything that depends on it.
    pub(super) fn cascade(&self, table: &str, id: &str) -> Result<(), StorageError> {
        self.delete_rows(table, "id", &[id.to_owned()])
    }

    /// Removes moments no record points at any more. Instants are rewritten on every edit of a
    /// record's period, so the old ones are left behind until here.
    pub(crate) fn prune_instants(&self) -> Result<(), StorageError> {
        let references: Vec<String> = self
            .foreign_keys_to("instant")?
            .into_iter()
            .map(|key| {
                format!(
                    "SELECT {} FROM {} WHERE {} IS NOT NULL",
                    key.child_column, key.child_table, key.child_column
                )
            })
            .collect();
        if references.is_empty() {
            return Ok(());
        }
        self.connection.execute(
            &format!(
                "DELETE FROM instant WHERE id NOT IN ({})",
                references.join(" UNION ")
            ),
            [],
        )?;
        Ok(())
    }
}

fn placeholders(count: usize) -> String {
    placeholders_from(1, count)
}

fn placeholders_from(first: usize, count: usize) -> String {
    (first..first + count)
        .map(|n| format!("?{n}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl RecordRemoval for SqliteModel<'_> {
    fn remove_record(&mut self, target: &RecordRef) -> Result<(), StorageError> {
        self.cascade(target.kind.as_str(), &target.id.to_string())
    }

    fn remove_catalog_entry(&mut self, entry: &CatalogEntry) -> Result<(), StorageError> {
        self.delete_rows(entry.table(), entry.key_column(), &[entry.key_text()])
    }
}

impl SqliteModel<'_> {
    pub(super) fn count_references(
        &self,
        entity: &str,
    ) -> Result<Vec<ReferenceCount>, StorageError> {
        let mut counts = Vec::new();
        for key in self.foreign_keys_to("entity")? {
            if OWNED_BY_ENTITY
                .iter()
                .any(|(table, column)| *table == key.child_table && *column == key.child_column)
            {
                continue;
            }
            let count: i64 = self.connection.query_row(
                &format!(
                    "SELECT COUNT(*) FROM {} WHERE {} = ?1",
                    key.child_table, key.child_column
                ),
                [entity],
                |row| row.get(0),
            )?;
            if count > 0 {
                counts.push(ReferenceCount {
                    kind: key.child_table,
                    count: usize::try_from(count).unwrap_or(usize::MAX),
                });
            }
        }
        Ok(counts)
    }
}

impl SqliteModel<'_> {
    pub(super) fn count_catalog_usage(
        &self,
        entry: &CatalogEntry,
    ) -> Result<Vec<ReferenceCount>, StorageError> {
        let key = entry.key_text();
        let mut counts = Vec::new();
        for foreign_key in self.foreign_keys_to(entry.table())? {
            if foreign_key.parent_column != entry.key_column()
                || OWNED_BY_CATALOG_ENTRY.iter().any(|(table, column)| {
                    *table == foreign_key.child_table && *column == foreign_key.child_column
                })
            {
                continue;
            }
            let count: i64 = self.connection.query_row(
                &format!(
                    "SELECT COUNT(*) FROM {} WHERE {} = ?1",
                    foreign_key.child_table, foreign_key.child_column
                ),
                [&key],
                |row| row.get(0),
            )?;
            if count > 0 {
                counts.push(ReferenceCount {
                    kind: foreign_key.child_table,
                    count: usize::try_from(count).unwrap_or(usize::MAX),
                });
            }
        }
        Ok(counts)
    }
}
