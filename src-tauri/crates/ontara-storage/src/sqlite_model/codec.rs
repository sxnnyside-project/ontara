use std::fmt::Display;
use std::str::FromStr;

use ontara_core::{CalendarId, Instant, Precision, Temporal, Timestamp, TypeKey};
use rusqlite::Row;
use rusqlite::types::Value;

use crate::error::StorageError;

pub(super) fn corrupt(table: &'static str, id: &str, reason: &dyn Display) -> StorageError {
    StorageError::CorruptRecord {
        table,
        id: id.to_owned(),
        reason: reason.to_string(),
    }
}

pub(super) fn parse<T>(table: &'static str, id: &str, raw: &str) -> Result<T, StorageError>
where
    T: FromStr,
    T::Err: Display,
{
    raw.parse()
        .map_err(|error: T::Err| corrupt(table, id, &error))
}

pub(super) fn type_key(table: &'static str, id: &str, raw: &str) -> Result<TypeKey, StorageError> {
    TypeKey::new(raw).map_err(|error| corrupt(table, id, &error))
}

pub(super) fn timestamp(millis: i64) -> Timestamp {
    Timestamp::from_unix_millis(millis)
}

// ---- values for statements

pub(super) fn text(value: impl Into<String>) -> Value {
    Value::Text(value.into())
}

pub(super) fn opt_text<T: Into<String>>(value: Option<T>) -> Value {
    value.map_or(Value::Null, |text| Value::Text(text.into()))
}

pub(super) fn int(value: i64) -> Value {
    Value::Integer(value)
}

pub(super) fn opt_int(value: Option<i64>) -> Value {
    value.map_or(Value::Null, Value::Integer)
}

pub(super) fn opt_real(value: Option<f64>) -> Value {
    value.map_or(Value::Null, Value::Real)
}

pub(super) fn boolean(value: bool) -> Value {
    Value::Integer(i64::from(value))
}

pub(super) fn opt_bool(value: Option<bool>) -> Value {
    value.map_or(Value::Null, |flag| Value::Integer(i64::from(flag)))
}

// ---- optional columns

pub(super) fn optional_parse<T>(
    table: &'static str,
    id: &str,
    raw: Option<String>,
) -> Result<Option<T>, StorageError>
where
    T: FromStr,
    T::Err: Display,
{
    raw.map(|text| parse::<T>(table, id, &text)).transpose()
}

pub(super) fn optional_key(
    table: &'static str,
    id: &str,
    raw: Option<String>,
) -> Result<Option<TypeKey>, StorageError> {
    raw.map(|text| type_key(table, id, &text)).transpose()
}

// ---- in-universe time

/// Reads the instant joined under `column` (see `select_sql` in the generated mappers).
pub(super) fn read_instant(row: &Row<'_>, column: &str) -> Result<Option<Instant>, StorageError> {
    let id: Option<String> = row.get(format!("{column}__id").as_str())?;
    let Some(id) = id else {
        return Ok(None);
    };
    let precision: String = row.get(format!("{column}__precision").as_str())?;
    let calendar: Option<String> = row.get(format!("{column}__calendar").as_str())?;
    Ok(Some(Instant {
        day: row.get(format!("{column}__day").as_str())?,
        precision: parse::<Precision>("instant", &id, &precision)?,
        earliest: row.get(format!("{column}__earliest").as_str())?,
        latest: row.get(format!("{column}__latest").as_str())?,
        calendar: optional_parse::<CalendarId>("instant", &id, calendar)?,
        label: row.get(format!("{column}__label").as_str())?,
    }))
}

/// Reads `time_state`, `starts_at` and `ends_at` into one value.
///
/// A moment with the same instant at both ends is a single point.
pub(super) fn read_period(
    row: &Row<'_>,
    table: &'static str,
    id: &str,
) -> Result<Temporal, StorageError> {
    let state: String = row.get("time_state")?;
    match state.as_str() {
        "unknown" => Ok(Temporal::Unknown),
        "unconstrained" => Ok(Temporal::Unconstrained),
        "known" => {
            let start_id: Option<String> = row.get("starts_at__id")?;
            let end_id: Option<String> = row.get("ends_at__id")?;
            let start = read_instant(row, "starts_at")?;
            let end = read_instant(row, "ends_at")?;
            match (start, end) {
                (Some(instant), Some(_)) if start_id == end_id => Ok(Temporal::At { instant }),
                (None, None) => Err(corrupt(table, id, &"a known period has no moments")),
                (start, end) => Ok(Temporal::Span { start, end }),
            }
        }
        other => Err(corrupt(table, id, &format!("unknown time state `{other}`"))),
    }
}

// ---- select fragments for moments

/// Columns of the instant joined under `column`, named the way `read_instant` expects.
pub(super) fn instant_columns(column: &str) -> String {
    let alias = format!("i_{column}");
    format!(
        "{alias}.id AS {column}__id, {alias}.day AS {column}__day, \
         {alias}.precision AS {column}__precision, {alias}.earliest_day AS {column}__earliest, \
         {alias}.latest_day AS {column}__latest, {alias}.calendar_id AS {column}__calendar, \
         {alias}.label AS {column}__label"
    )
}

/// The join that brings the instant referenced by `t.<column>` into the query.
pub(super) fn instant_join(column: &str) -> String {
    format!("LEFT JOIN instant AS i_{column} ON i_{column}.id = t.{column}")
}
