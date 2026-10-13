use rusqlite::Connection;

use crate::error::StorageError;

/// Outcome of checking a project database. An empty list of problems means it is sound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    pub problems: Vec<String>,
}

impl IntegrityReport {
    #[must_use]
    pub fn is_sound(&self) -> bool {
        self.problems.is_empty()
    }
}

/// `thorough` runs the `SQLite` full check; otherwise the faster structural check used when opening.
pub(crate) fn check(
    connection: &Connection,
    thorough: bool,
) -> Result<IntegrityReport, StorageError> {
    let pragma = if thorough {
        "integrity_check"
    } else {
        "quick_check"
    };
    let mut problems = Vec::new();

    let mut statement = connection.prepare(&format!("PRAGMA {pragma}"))?;
    for line in statement.query_map([], |row| row.get::<_, String>(0))? {
        let line = line?;
        if line != "ok" {
            problems.push(line);
        }
    }

    let mut statement = connection.prepare("PRAGMA foreign_key_check")?;
    let violations = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(2)?))
    })?;
    for violation in violations {
        let (table, parent) = violation?;
        problems.push(format!(
            "a record in {table} references a missing record in {}",
            parent.as_deref().unwrap_or("an unknown table")
        ));
    }

    // Citations point at records of many tables, so no foreign key can guard them. Every table a
    // citation may name has a single `id` column.
    let mut statement =
        connection.prepare("SELECT id, target_table, target_id FROM source_reference")?;
    let citations: Vec<(String, String, String)> = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<_, _>>()?;
    for (id, table, target) in citations {
        let exists = connection.prepare(&format!("SELECT 1 FROM {table} WHERE id = ?1"));
        match exists {
            Ok(mut lookup) => {
                if !lookup.exists([&target])? {
                    problems.push(format!(
                        "citation {id} points at a {table} record that no longer exists"
                    ));
                }
            }
            Err(_) => problems.push(format!("citation {id} names an unknown table `{table}`")),
        }
    }

    Ok(IntegrityReport { problems })
}
