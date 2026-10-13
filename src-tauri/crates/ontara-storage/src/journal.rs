//! Undo and redo: a journal of row changes kept by the database itself.
//!
//! Every table of the project is watched by three triggers that, while a change set is open,
//! append the statement that would reverse the row change. Nothing in the domain knows about it:
//! a table added by a later migration is covered by installing the triggers again, and what an
//! undo restores is exactly what the edit touched, whatever records it spans.

use ontara_core::Timestamp;
use rusqlite::{Connection, Transaction, params};
use serde::Serialize;

use crate::error::StorageError;

/// Most undoable edits kept. Older ones are dropped as new ones arrive.
pub const MAX_HISTORY: u32 = 200;

/// Tables the journal never records: its own, and the search index the triggers maintain.
const UNWATCHED: &[&str] = &[
    "change_set",
    "change_entry",
    "journal_state",
    "search_index",
    "search_index_data",
    "search_index_idx",
    "search_index_content",
    "search_index_docsize",
    "search_index_config",
];

/// One edit that can be undone or redone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChangeSummary {
    /// Stable machine name of the operation, such as `entity.update`.
    pub label: String,
    /// Milliseconds since the Unix epoch.
    pub at: i64,
}

/// What can be undone and redone right now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct History {
    /// Newest first.
    pub undo: Vec<ChangeSummary>,
    /// The next redo first.
    pub redo: Vec<ChangeSummary>,
}

/// Which way a change set is applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Undo,
    Redo,
}

impl Direction {
    fn stack(self) -> &'static str {
        match self {
            Self::Undo => "undo",
            Self::Redo => "redo",
        }
    }

    fn opposite(self) -> &'static str {
        match self {
            Self::Undo => "redo",
            Self::Redo => "undo",
        }
    }
}

/// Creates, or recreates, the triggers of every table. Called after migrations, so a table a
/// migration added is covered and one it changed is not recorded with stale columns.
pub(crate) fn install(connection: &Connection) -> Result<(), StorageError> {
    let tables = watched_tables(connection)?;
    let stale: Vec<String> = {
        let mut statement = connection.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'trigger' AND name LIKE 'journal_%'",
        )?;
        statement
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, _>>()?
    };
    for name in stale {
        connection.execute_batch(&format!("DROP TRIGGER \"{name}\""))?;
    }
    for table in tables {
        let columns = columns_of(connection, &table)?;
        connection.execute_batch(&triggers_for(&table, &columns))?;
    }
    Ok(())
}

/// Forgets every recorded change. A migration may change what a statement means, so history from
/// before it is not replayed.
pub(crate) fn clear(connection: &Connection) -> Result<(), StorageError> {
    connection.execute("DELETE FROM change_set", [])?;
    connection.execute("UPDATE journal_state SET open_set = NULL", [])?;
    Ok(())
}

struct Column {
    name: String,
    /// The column is the table's row id under another name.
    is_row_id_alias: bool,
}

fn watched_tables(connection: &Connection) -> Result<Vec<String>, StorageError> {
    let mut statement = connection.prepare(
        "SELECT name FROM sqlite_master
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let names: Vec<String> = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(names
        .into_iter()
        .filter(|name| !UNWATCHED.contains(&name.as_str()))
        .collect())
}

fn columns_of(connection: &Connection, table: &str) -> Result<Vec<Column>, StorageError> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info(\"{table}\")"))?;
    let rows: Vec<(String, String, i64)> = statement
        .query_map([], |row| Ok((row.get(1)?, row.get(2)?, row.get(5)?)))?
        .collect::<Result<_, _>>()?;
    let key_columns = rows.iter().filter(|(_, _, key)| *key > 0).count();
    Ok(rows
        .into_iter()
        .map(|(name, declared, key)| Column {
            is_row_id_alias: key == 1
                && key_columns == 1
                && declared.eq_ignore_ascii_case("INTEGER"),
            name,
        })
        .collect())
}

/// The three triggers that record how to reverse an insert, an update and a delete of `table`.
fn triggers_for(table: &str, columns: &[Column]) -> String {
    const GUARD: &str = "WHEN (SELECT open_set FROM journal_state) IS NOT NULL";
    let record = |statement: &str| {
        format!(
            "INSERT INTO change_entry (change_set, statement) \
             SELECT open_set, {statement} FROM journal_state;"
        )
    };

    let remove = format!("'DELETE FROM \"{table}\" WHERE rowid = ' || NEW.rowid");

    let aliased = columns.iter().any(|column| column.is_row_id_alias);
    let mut names: Vec<String> = Vec::new();
    let mut values: Vec<String> = Vec::new();
    if !aliased {
        names.push("rowid".to_owned());
        values.push("OLD.rowid".to_owned());
    }
    for column in columns {
        names.push(format!("\"{}\"", column.name));
        values.push(format!("quote(OLD.\"{}\")", column.name));
    }
    let restore = format!(
        "'INSERT INTO \"{table}\" ({}) VALUES (' || {} || ')'",
        names.join(", "),
        values.join(" || ', ' || ")
    );

    let assignments: Vec<String> = columns
        .iter()
        .map(|column| format!("\"{0}\" = ' || quote(OLD.\"{0}\") || '", column.name))
        .collect();
    let revert = format!(
        "'UPDATE \"{table}\" SET {} WHERE rowid = ' || NEW.rowid",
        assignments.join(", ")
    );

    format!(
        "CREATE TRIGGER \"journal_{table}_insert\" AFTER INSERT ON \"{table}\" {GUARD}\n\
         BEGIN {} END;\n\
         CREATE TRIGGER \"journal_{table}_update\" AFTER UPDATE ON \"{table}\" {GUARD}\n\
         BEGIN {} END;\n\
         CREATE TRIGGER \"journal_{table}_delete\" AFTER DELETE ON \"{table}\" {GUARD}\n\
         BEGIN {} END;\n",
        record(&remove),
        record(&revert),
        record(&restore),
    )
}

/// Starts recording into a new set. Returns its identifier.
pub(crate) fn open_set(
    transaction: &Transaction<'_>,
    label: &str,
    now: Timestamp,
) -> Result<i64, StorageError> {
    let id = insert_set(transaction, label, Direction::Undo.stack(), now)?;
    transaction.execute("UPDATE journal_state SET open_set = ?1", [id])?;
    Ok(id)
}

/// Stops recording. An edit that changed nothing leaves no set behind; one that changed
/// something clears the redo stack and keeps the history within its limit.
pub(crate) fn close_set(transaction: &Transaction<'_>, set: i64) -> Result<(), StorageError> {
    transaction.execute("UPDATE journal_state SET open_set = NULL", [])?;
    let entries: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM change_entry WHERE change_set = ?1",
        [set],
        |row| row.get(0),
    )?;
    if entries == 0 {
        transaction.execute("DELETE FROM change_set WHERE id = ?1", [set])?;
        return Ok(());
    }
    transaction.execute("DELETE FROM change_set WHERE stack = 'redo'", [])?;
    transaction.execute(
        "DELETE FROM change_set WHERE stack = 'undo' AND id NOT IN
         (SELECT id FROM change_set WHERE stack = 'undo' ORDER BY id DESC LIMIT ?1)",
        [MAX_HISTORY],
    )?;
    Ok(())
}

fn insert_set(
    transaction: &Transaction<'_>,
    label: &str,
    stack: &str,
    now: Timestamp,
) -> Result<i64, StorageError> {
    transaction.execute(
        "INSERT INTO change_set (label, stack, created_at) VALUES (?1, ?2, ?3)",
        params![label, stack, now.as_unix_millis()],
    )?;
    Ok(transaction.last_insert_rowid())
}

/// Applies the newest set of the stack `direction` names, returning what it was, or `None` when
/// that stack is empty. What the application does is recorded as a set of the opposite stack.
pub(crate) fn apply(
    connection: &mut Connection,
    direction: Direction,
    now: Timestamp,
) -> Result<Option<ChangeSummary>, StorageError> {
    let transaction = connection.transaction()?;
    let found: Option<(i64, String, i64)> = transaction
        .query_row(
            "SELECT id, label, created_at FROM change_set WHERE stack = ?1 ORDER BY id DESC LIMIT 1",
            [direction.stack()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(other),
        })?;
    let Some((set, label, at)) = found else {
        return Ok(None);
    };

    let statements: Vec<String> = {
        let mut statement = transaction
            .prepare("SELECT statement FROM change_entry WHERE change_set = ?1 ORDER BY id DESC")?;
        statement
            .query_map([set], |row| row.get(0))?
            .collect::<Result<_, _>>()?
    };

    // The statements restore states that existed in order, but a cascade may log a child's row
    // before its parent's; the references are checked once the whole set is applied.
    transaction.pragma_update(None, "defer_foreign_keys", true)?;
    let mirror = insert_set(&transaction, &label, direction.opposite(), now)?;
    transaction.execute("UPDATE journal_state SET open_set = ?1", [mirror])?;
    for statement in &statements {
        transaction.execute_batch(statement)?;
    }
    transaction.execute("UPDATE journal_state SET open_set = NULL", [])?;
    transaction.execute("DELETE FROM change_set WHERE id = ?1", [set])?;
    transaction.commit()?;
    Ok(Some(ChangeSummary { label, at }))
}

/// The edits that can be undone and redone, with the newest or next first.
pub(crate) fn history(connection: &Connection) -> Result<History, StorageError> {
    let read = |stack: &str| -> Result<Vec<ChangeSummary>, StorageError> {
        let mut statement = connection.prepare(
            "SELECT label, created_at FROM change_set WHERE stack = ?1 ORDER BY id DESC",
        )?;
        let summaries = statement
            .query_map([stack], |row| {
                Ok(ChangeSummary {
                    label: row.get(0)?,
                    at: row.get(1)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(summaries)
    };
    Ok(History {
        undo: read("undo")?,
        redo: read("redo")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: Timestamp = Timestamp::from_unix_millis(2);

    fn database() -> Result<Connection, StorageError> {
        let mut connection = Connection::open_in_memory()?;
        connection.pragma_update(None, "foreign_keys", true)?;
        crate::migration::migrate(&mut connection)?;
        Ok(connection)
    }

    fn edit(connection: &mut Connection, label: &str, sql: &str) -> Result<(), StorageError> {
        let transaction = connection.transaction()?;
        let set = open_set(&transaction, label, Timestamp::from_unix_millis(1))?;
        transaction.execute_batch(sql)?;
        close_set(&transaction, set)?;
        transaction.commit()?;
        Ok(())
    }

    fn names(connection: &Connection) -> Result<Vec<String>, StorageError> {
        let mut statement = connection.prepare("SELECT name FROM tag ORDER BY name")?;
        Ok(statement
            .query_map([], |row| row.get(0))?
            .collect::<Result<_, _>>()?)
    }

    #[test]
    fn an_insert_an_update_and_a_delete_are_undone_and_redone() -> Result<(), StorageError> {
        let mut connection = database()?;

        edit(
            &mut connection,
            "tag.add",
            "INSERT INTO tag (name) VALUES ('war')",
        )?;
        edit(
            &mut connection,
            "tag.rename",
            "UPDATE tag SET name = 'peace' WHERE name = 'war'",
        )?;
        edit(
            &mut connection,
            "tag.remove",
            "DELETE FROM tag WHERE name = 'peace'",
        )?;
        assert!(names(&connection)?.is_empty());

        apply(&mut connection, Direction::Undo, NOW)?;
        assert_eq!(names(&connection)?, ["peace"]);
        apply(&mut connection, Direction::Undo, NOW)?;
        assert_eq!(names(&connection)?, ["war"]);
        apply(&mut connection, Direction::Undo, NOW)?;
        assert!(names(&connection)?.is_empty());
        assert!(apply(&mut connection, Direction::Undo, NOW)?.is_none());

        apply(&mut connection, Direction::Redo, NOW)?;
        apply(&mut connection, Direction::Redo, NOW)?;
        apply(&mut connection, Direction::Redo, NOW)?;
        assert!(names(&connection)?.is_empty());
        assert_eq!(history(&connection)?.undo.len(), 3);
        Ok(())
    }

    #[test]
    fn a_new_edit_discards_what_could_have_been_redone() -> Result<(), StorageError> {
        let mut connection = database()?;
        edit(
            &mut connection,
            "tag.add",
            "INSERT INTO tag (name) VALUES ('war')",
        )?;
        apply(&mut connection, Direction::Undo, NOW)?;
        assert_eq!(history(&connection)?.redo.len(), 1);

        edit(
            &mut connection,
            "tag.add",
            "INSERT INTO tag (name) VALUES ('peace')",
        )?;

        assert!(history(&connection)?.redo.is_empty());
        assert!(apply(&mut connection, Direction::Redo, NOW)?.is_none());
        Ok(())
    }

    #[test]
    fn an_edit_that_changes_nothing_leaves_no_history() -> Result<(), StorageError> {
        let mut connection = database()?;

        edit(&mut connection, "tag.look", "SELECT 1")?;

        assert!(history(&connection)?.undo.is_empty());
        Ok(())
    }

    #[test]
    fn the_search_index_follows_an_undo_but_is_not_journaled() -> Result<(), StorageError> {
        let mut connection = database()?;
        // Only the index is under test, not the catalog an entity's type refers to.
        connection.pragma_update(None, "foreign_keys", false)?;
        let hits = |connection: &Connection, word: &str| -> Result<i64, StorageError> {
            Ok(connection.query_row(
                "SELECT COUNT(*) FROM search_index WHERE search_index MATCH ?1",
                [word],
                |row| row.get(0),
            )?)
        };
        edit(
            &mut connection,
            "entity.create",
            "INSERT INTO entity (id, entity_type, name, summary, description, archived,
                canon_state, origin, created_at, updated_at)
             VALUES ('e1', 'character', 'Ayla', 'A lighthouse keeper', '', 0, 'canon', 'user', 1, 1)",
        )?;
        assert_eq!(hits(&connection, "lighthouse")?, 1);

        apply(&mut connection, Direction::Undo, NOW)?;
        assert_eq!(hits(&connection, "lighthouse")?, 0);
        apply(&mut connection, Direction::Redo, NOW)?;
        assert_eq!(hits(&connection, "lighthouse")?, 1);
        Ok(())
    }
}
