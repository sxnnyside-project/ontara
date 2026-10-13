mod being;
mod belief;
mod catalog;
mod codec;
mod entities;
mod event;
mod faction;
mod item;
mod narrative;
mod place;
mod polity;
mod properties;
mod queries;
mod removal;
mod search;
mod supporting;

use ontara_core::{Instant, Repository, Temporal};
use rusqlite::{Connection, Params, Row};

use crate::error::StorageError;

/// `SQLite` implementation of the domain's repository contracts.
///
/// It performs raw record operations. Domain rules are enforced by `ModelEditor`, which callers
/// reach through `Project::edit`.
pub struct SqliteModel<'connection> {
    connection: &'connection Connection,
}

impl<'connection> SqliteModel<'connection> {
    pub(crate) fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    /// Runs a query and builds one record per row.
    fn select<T>(
        &self,
        sql: &str,
        parameters: impl Params,
        build: fn(&Row<'_>) -> Result<T, StorageError>,
    ) -> Result<Vec<T>, StorageError> {
        let mut statement = self.connection.prepare_cached(sql)?;
        let mut rows = statement.query(parameters)?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(build(row)?);
        }
        Ok(records)
    }

    fn select_one<T>(
        &self,
        sql: &str,
        parameters: impl Params,
        build: fn(&Row<'_>) -> Result<T, StorageError>,
    ) -> Result<Option<T>, StorageError> {
        Ok(self.select(sql, parameters, build)?.into_iter().next())
    }

    /// Writes a moment and returns the identifier of its row.
    fn write_instant(&self, instant: &Instant) -> Result<String, StorageError> {
        let id = uuid::Uuid::now_v7().to_string();
        self.connection.execute(
            "INSERT INTO instant (id, day, precision, earliest_day, latest_day, calendar_id, label) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                id,
                instant.day,
                instant.precision.as_str(),
                instant.earliest,
                instant.latest,
                instant.calendar.map(|calendar| calendar.to_string()),
                instant.label
            ],
        )?;
        Ok(id)
    }

    fn write_instant_opt(&self, instant: Option<&Instant>) -> Result<Option<String>, StorageError> {
        instant.map(|value| self.write_instant(value)).transpose()
    }

    /// Writes the moments of a period and returns `(time_state, starts_at, ends_at)`.
    /// A single point is stored as one moment at both ends.
    fn write_period(
        &self,
        period: &Temporal,
    ) -> Result<(&'static str, Option<String>, Option<String>), StorageError> {
        Ok(match period {
            Temporal::Unknown => ("unknown", None, None),
            Temporal::Unconstrained => ("unconstrained", None, None),
            Temporal::At { instant } => {
                let id = self.write_instant(instant)?;
                ("known", Some(id.clone()), Some(id))
            }
            Temporal::Span { start, end } => (
                "known",
                self.write_instant_opt(start.as_ref())?,
                self.write_instant_opt(end.as_ref())?,
            ),
        })
    }

    fn execute(&self, sql: &str, parameters: impl Params) -> Result<(), StorageError> {
        self.connection.prepare_cached(sql)?.execute(parameters)?;
        Ok(())
    }
}

impl Repository for SqliteModel<'_> {
    type Error = StorageError;
}
