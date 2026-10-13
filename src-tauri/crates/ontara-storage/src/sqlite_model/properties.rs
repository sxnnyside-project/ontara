use ontara_core::{
    EntityId, Origin, PropertyContent, PropertyDefinitionId, PropertyValue, PropertyValueId,
    PropertyValueOwner, Reader, RelationshipId, Writer,
};
use rusqlite::{Row, params_from_iter};

use super::SqliteModel;
use super::codec::{
    corrupt, instant_columns, instant_join, int, opt_int, opt_real, opt_text, optional_parse,
    parse, read_instant, text, timestamp, type_key,
};
use crate::error::StorageError;

pub(super) fn property_value_sql() -> String {
    format!(
        "SELECT t.id, t.entity_id, t.relationship_id, t.definition_id, t.value_text, \
         t.value_integer, t.value_real, t.value_boolean, t.value_entity_id, t.value_choice, \
         t.note, t.position, t.canon_state, t.origin, t.created_at, t.updated_at, {}, {}, {} \
         FROM property_value AS t {} {} {}",
        instant_columns("value_instant"),
        instant_columns("valid_from"),
        instant_columns("valid_until"),
        instant_join("value_instant"),
        instant_join("valid_from"),
        instant_join("valid_until"),
    )
}

pub(super) fn property_value(row: &Row<'_>) -> Result<PropertyValue, StorageError> {
    let id: String = row.get("id")?;
    let entity: Option<String> = row.get("entity_id")?;
    let relationship: Option<String> = row.get("relationship_id")?;
    let owner = match (entity, relationship) {
        (Some(raw), None) => {
            PropertyValueOwner::Entity(parse::<EntityId>("property_value", &id, &raw)?)
        }
        (None, Some(raw)) => {
            PropertyValueOwner::Relationship(parse::<RelationshipId>("property_value", &id, &raw)?)
        }
        _ => return Err(corrupt("property_value", &id, &"it has no single owner")),
    };
    let definition: String = row.get("definition_id")?;
    let value_text: Option<String> = row.get("value_text")?;
    let value_integer: Option<i64> = row.get("value_integer")?;
    let value_real: Option<f64> = row.get("value_real")?;
    let value_boolean: Option<bool> = row.get("value_boolean")?;
    let value_entity: Option<String> = row.get("value_entity_id")?;
    let value_choice: Option<String> = row.get("value_choice")?;
    let value_instant = read_instant(row, "value_instant")?;
    let content = match (
        value_text,
        value_integer,
        value_real,
        value_boolean,
        value_instant,
        optional_parse::<EntityId>("property_value", &id, value_entity)?,
        value_choice,
    ) {
        (Some(value), None, None, None, None, None, None) => PropertyContent::Text(value),
        (None, Some(value), None, None, None, None, None) => PropertyContent::Integer(value),
        (None, None, Some(value), None, None, None, None) => PropertyContent::Real(value),
        (None, None, None, Some(value), None, None, None) => PropertyContent::Boolean(value),
        (None, None, None, None, Some(value), None, None) => PropertyContent::Date(value),
        (None, None, None, None, None, Some(value), None) => {
            PropertyContent::EntityReference(value)
        }
        (None, None, None, None, None, None, Some(value)) => {
            PropertyContent::Choice(type_key("property_value", &id, &value)?)
        }
        _ => {
            return Err(corrupt(
                "property_value",
                &id,
                &"it does not hold exactly one value",
            ));
        }
    };
    let canon_state: String = row.get("canon_state")?;
    let origin: String = row.get("origin")?;
    Ok(PropertyValue {
        id: parse::<PropertyValueId>("property_value", &id, &id)?,
        owner,
        definition: parse::<PropertyDefinitionId>("property_value", &id, &definition)?,
        content,
        valid_from: read_instant(row, "valid_from")?,
        valid_until: read_instant(row, "valid_until")?,
        note: row.get("note")?,
        position: row.get("position")?,
        canon_state: type_key("property_value", &id, &canon_state)?,
        origin: parse::<Origin>("property_value", &id, &origin)?,
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

impl Reader<PropertyValue> for SqliteModel<'_> {
    fn get(&self, id: &PropertyValueId) -> Result<Option<PropertyValue>, StorageError> {
        self.select_one(
            &format!("{} WHERE t.id = ?1", property_value_sql()),
            [id.to_string()],
            property_value,
        )
    }

    fn list(&self) -> Result<Vec<PropertyValue>, StorageError> {
        self.select(
            &format!("{} ORDER BY t.created_at, t.id", property_value_sql()),
            [],
            property_value,
        )
    }
}

/// The typed value columns of a property value, in the order
/// `value_text, value_integer, value_real, value_boolean, value_entity_id, value_choice`.
struct TypedColumns {
    text: Option<String>,
    integer: Option<i64>,
    real: Option<f64>,
    boolean: Option<bool>,
    entity: Option<String>,
    choice: Option<String>,
}

fn typed_columns(content: &PropertyContent) -> TypedColumns {
    let mut columns = TypedColumns {
        text: None,
        integer: None,
        real: None,
        boolean: None,
        entity: None,
        choice: None,
    };
    match content {
        PropertyContent::Text(value) => columns.text = Some(value.clone()),
        PropertyContent::Integer(value) => columns.integer = Some(*value),
        PropertyContent::Real(value) => columns.real = Some(*value),
        PropertyContent::Boolean(value) => columns.boolean = Some(*value),
        PropertyContent::EntityReference(value) => columns.entity = Some(value.to_string()),
        PropertyContent::Choice(value) => columns.choice = Some(value.to_string()),
        PropertyContent::Date(_) => {}
    }
    columns
}

impl Writer<PropertyValue> for SqliteModel<'_> {
    fn insert(&mut self, value: &PropertyValue) -> Result<(), StorageError> {
        let (entity_id, relationship_id) = match value.owner {
            PropertyValueOwner::Entity(id) => (Some(id.to_string()), None),
            PropertyValueOwner::Relationship(id) => (None, Some(id.to_string())),
        };
        let date = match &value.content {
            PropertyContent::Date(instant) => Some(self.write_instant(instant)?),
            _ => None,
        };
        let valid_from = self.write_instant_opt(value.valid_from.as_ref())?;
        let valid_until = self.write_instant_opt(value.valid_until.as_ref())?;
        let typed = typed_columns(&value.content);
        self.execute(
            "INSERT INTO property_value (id, entity_id, relationship_id, definition_id, \
             value_text, value_integer, value_real, value_boolean, value_instant, value_entity_id, \
             value_choice, valid_from, valid_until, note, position, canon_state, origin, \
             created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params_from_iter(vec![
                text(value.id.to_string()),
                opt_text(entity_id),
                opt_text(relationship_id),
                text(value.definition.to_string()),
                opt_text(typed.text),
                opt_int(typed.integer),
                opt_real(typed.real),
                typed.boolean.map_or(rusqlite::types::Value::Null, |flag| {
                    rusqlite::types::Value::Integer(i64::from(flag))
                }),
                opt_text(date),
                opt_text(typed.entity),
                opt_text(typed.choice),
                opt_text(valid_from),
                opt_text(valid_until),
                opt_text(value.note.clone()),
                int(i64::from(value.position)),
                text(value.canon_state.as_str()),
                text(value.origin.as_str()),
                int(value.created_at.as_unix_millis()),
                int(value.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn update(&mut self, value: &PropertyValue) -> Result<(), StorageError> {
        let date = match &value.content {
            PropertyContent::Date(instant) => Some(self.write_instant(instant)?),
            _ => None,
        };
        let valid_from = self.write_instant_opt(value.valid_from.as_ref())?;
        let valid_until = self.write_instant_opt(value.valid_until.as_ref())?;
        let typed = typed_columns(&value.content);
        self.execute(
            "UPDATE property_value SET value_text = ?2, value_integer = ?3, value_real = ?4, \
             value_boolean = ?5, value_instant = ?6, value_entity_id = ?7, value_choice = ?8, \
             valid_from = ?9, valid_until = ?10, note = ?11, position = ?12, canon_state = ?13, \
             updated_at = ?14 WHERE id = ?1",
            params_from_iter(vec![
                text(value.id.to_string()),
                opt_text(typed.text),
                opt_int(typed.integer),
                opt_real(typed.real),
                typed.boolean.map_or(rusqlite::types::Value::Null, |flag| {
                    rusqlite::types::Value::Integer(i64::from(flag))
                }),
                opt_text(date),
                opt_text(typed.entity),
                opt_text(typed.choice),
                opt_text(valid_from),
                opt_text(valid_until),
                opt_text(value.note.clone()),
                int(i64::from(value.position)),
                text(value.canon_state.as_str()),
                int(value.updated_at.as_unix_millis()),
            ]),
        )
    }

    fn delete(&mut self, id: &PropertyValueId) -> Result<(), StorageError> {
        self.cascade("property_value", &id.to_string())
    }
}
