use rusqlite::Connection;

use crate::error::StorageError;

struct Migration {
    version: u32,
    sql: &'static str,
}

/// Ordered, append-only. A shipped migration is never edited; changes are new entries.
const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    sql: include_str!("../migrations/0001_initial_schema.sql"),
}];

/// The schema version this build of Ontara writes.
#[must_use]
pub fn latest_schema_version() -> u32 {
    MIGRATIONS.last().map_or(0, |migration| migration.version)
}

pub(crate) fn current_schema_version(connection: &Connection) -> Result<u32, StorageError> {
    Ok(connection.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Applies every pending migration, each in its own transaction. Returns the new version.
pub(crate) fn migrate(connection: &mut Connection) -> Result<u32, StorageError> {
    let current = current_schema_version(connection)?;
    for migration in MIGRATIONS
        .iter()
        .filter(|migration| migration.version > current)
    {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.pragma_update(None, "user_version", migration.version)?;
        transaction.commit()?;
    }
    current_schema_version(connection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_strictly_increasing_from_one() {
        let versions: Vec<u32> = MIGRATIONS.iter().map(|m| m.version).collect();
        let expected: Vec<u32> = (1..=u32::try_from(versions.len()).unwrap_or(0)).collect();

        assert_eq!(versions, expected);
    }

    #[test]
    fn a_fresh_database_reaches_the_latest_version_and_is_idempotent() -> Result<(), StorageError> {
        let mut connection = Connection::open_in_memory()?;

        assert_eq!(migrate(&mut connection)?, latest_schema_version());
        assert_eq!(migrate(&mut connection)?, latest_schema_version());
        Ok(())
    }
}
