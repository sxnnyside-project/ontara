use std::collections::{BTreeMap, BTreeSet};

use ontara_core::{
    AppearanceKind, Calendar, CalendarDefinition, CalendarId, CanonState, ChoiceOption,
    DiplomaticStance, Directionality, EditorialLevel, EntityRole, EntityType, Era, EraDirection,
    LeapRule, Month, PropertyDataType, PropertyDefinition, PropertyDefinitionId, PropertyGroup,
    PropertyOwner, Reader, RelationshipType, TypeKey, Vocabulary, VocabularyTerm, Writer,
};
use rusqlite::{Row, params};

use super::SqliteModel;
use super::codec::{corrupt, parse, timestamp, type_key};
use crate::error::StorageError;

const CANON_STATE: &str = "SELECT key, label, description, counts_as_canon, sort_order, builtin \
                           FROM canon_state";

fn canon_state(row: &Row<'_>) -> Result<CanonState, StorageError> {
    let key: String = row.get("key")?;
    Ok(CanonState {
        key: type_key("canon_state", &key, &key)?,
        label: row.get("label")?,
        description: row.get("description")?,
        counts_as_canon: row.get("counts_as_canon")?,
        sort_order: row.get("sort_order")?,
        builtin: row.get("builtin")?,
    })
}

impl Reader<CanonState> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<CanonState>, StorageError> {
        self.select_one(
            &format!("{CANON_STATE} WHERE key = ?1"),
            [key.as_str()],
            canon_state,
        )
    }

    fn list(&self) -> Result<Vec<CanonState>, StorageError> {
        self.select(
            &format!("{CANON_STATE} ORDER BY sort_order, key"),
            [],
            canon_state,
        )
    }
}

impl Writer<CanonState> for SqliteModel<'_> {
    fn insert(&mut self, state: &CanonState) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO canon_state (key, label, description, counts_as_canon, sort_order, builtin) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                state.key.as_str(),
                state.label,
                state.description,
                state.counts_as_canon,
                state.sort_order,
                state.builtin
            ],
        )
    }

    fn update(&mut self, state: &CanonState) -> Result<(), StorageError> {
        self.execute(
            "UPDATE canon_state SET label = ?2, description = ?3, counts_as_canon = ?4, \
             sort_order = ?5 WHERE key = ?1",
            params![
                state.key.as_str(),
                state.label,
                state.description,
                state.counts_as_canon,
                state.sort_order
            ],
        )
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute("DELETE FROM canon_state WHERE key = ?1", [key.as_str()])
    }
}

const ENTITY_TYPE: &str = "SELECT key, label, description, parent, is_abstract, role, sort_order, builtin FROM entity_type";

fn entity_type(row: &Row<'_>) -> Result<EntityType, StorageError> {
    let key: String = row.get("key")?;
    let role: String = row.get("role")?;
    let parent: Option<String> = row.get("parent")?;
    Ok(EntityType {
        key: type_key("entity_type", &key, &key)?,
        label: row.get("label")?,
        description: row.get("description")?,
        parent: parent
            .map(|raw| type_key("entity_type", &key, &raw))
            .transpose()?,
        is_abstract: row.get("is_abstract")?,
        role: parse::<EntityRole>("entity_type", &key, &role)?,
        sort_order: row.get("sort_order")?,
        builtin: row.get("builtin")?,
    })
}

impl Reader<EntityType> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<EntityType>, StorageError> {
        self.select_one(
            &format!("{ENTITY_TYPE} WHERE key = ?1"),
            [key.as_str()],
            entity_type,
        )
    }

    fn list(&self) -> Result<Vec<EntityType>, StorageError> {
        self.select(
            &format!("{ENTITY_TYPE} ORDER BY sort_order, key"),
            [],
            entity_type,
        )
    }
}

impl Writer<EntityType> for SqliteModel<'_> {
    fn insert(&mut self, value: &EntityType) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO entity_type (key, label, description, parent, is_abstract, role, \
             sort_order, builtin) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                value.key.as_str(),
                value.label,
                value.description,
                value.parent.as_ref().map(TypeKey::as_str),
                value.is_abstract,
                value.role.as_str(),
                value.sort_order,
                value.builtin
            ],
        )
    }

    fn update(&mut self, value: &EntityType) -> Result<(), StorageError> {
        self.execute(
            "UPDATE entity_type SET label = ?2, description = ?3, role = ?4, sort_order = ?5 \
             WHERE key = ?1",
            params![
                value.key.as_str(),
                value.label,
                value.description,
                value.role.as_str(),
                value.sort_order
            ],
        )
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute("DELETE FROM entity_type WHERE key = ?1", [key.as_str()])
    }
}

const RELATIONSHIP_TYPE: &str = "SELECT key, label, inverse_label, description, directionality, \
                                 acyclic, sort_order, builtin FROM relationship_type";

fn relationship_type(row: &Row<'_>) -> Result<RelationshipType, StorageError> {
    let key: String = row.get("key")?;
    let directionality: String = row.get("directionality")?;
    Ok(RelationshipType {
        key: type_key("relationship_type", &key, &key)?,
        label: row.get("label")?,
        inverse_label: row.get("inverse_label")?,
        description: row.get("description")?,
        directionality: parse::<Directionality>("relationship_type", &key, &directionality)?,
        acyclic: row.get("acyclic")?,
        allowed_sources: BTreeSet::new(),
        allowed_targets: BTreeSet::new(),
        sort_order: row.get("sort_order")?,
        builtin: row.get("builtin")?,
    })
}

impl SqliteModel<'_> {
    /// Fills the endpoint rules of relationship types, in one query for the whole set.
    fn attach_endpoints(&self, types: &mut [RelationshipType]) -> Result<(), StorageError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT relationship_type, side, entity_type FROM relationship_type_endpoint",
        )?;
        let mut rows = statement.query([])?;
        let mut rules: BTreeMap<String, (BTreeSet<TypeKey>, BTreeSet<TypeKey>)> = BTreeMap::new();
        while let Some(row) = rows.next()? {
            let owner: String = row.get(0)?;
            let side: String = row.get(1)?;
            let allowed: String = row.get(2)?;
            let key = type_key("relationship_type_endpoint", &owner, &allowed)?;
            let entry = rules.entry(owner).or_default();
            if side == "source" {
                entry.0.insert(key)
            } else {
                entry.1.insert(key)
            };
        }
        for relationship_type in types {
            if let Some((sources, targets)) = rules.remove(relationship_type.key.as_str()) {
                relationship_type.allowed_sources = sources;
                relationship_type.allowed_targets = targets;
            }
        }
        Ok(())
    }

    fn write_endpoints(&self, value: &RelationshipType) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM relationship_type_endpoint WHERE relationship_type = ?1",
            [value.key.as_str()],
        )?;
        for (side, allowed) in [
            ("source", &value.allowed_sources),
            ("target", &value.allowed_targets),
        ] {
            for entity_type in allowed {
                self.execute(
                    "INSERT INTO relationship_type_endpoint (relationship_type, side, entity_type) \
                     VALUES (?1, ?2, ?3)",
                    params![value.key.as_str(), side, entity_type.as_str()],
                )?;
            }
        }
        Ok(())
    }
}

impl Reader<RelationshipType> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<RelationshipType>, StorageError> {
        let mut found = self.select(
            &format!("{RELATIONSHIP_TYPE} WHERE key = ?1"),
            [key.as_str()],
            relationship_type,
        )?;
        self.attach_endpoints(&mut found)?;
        Ok(found.into_iter().next())
    }

    fn list(&self) -> Result<Vec<RelationshipType>, StorageError> {
        let mut all = self.select(
            &format!("{RELATIONSHIP_TYPE} ORDER BY sort_order, key"),
            [],
            relationship_type,
        )?;
        self.attach_endpoints(&mut all)?;
        Ok(all)
    }
}

impl Writer<RelationshipType> for SqliteModel<'_> {
    fn insert(&mut self, value: &RelationshipType) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO relationship_type \
             (key, label, inverse_label, description, directionality, acyclic, sort_order, builtin) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                value.key.as_str(),
                value.label,
                value.inverse_label,
                value.description,
                value.directionality.as_str(),
                value.acyclic,
                value.sort_order,
                value.builtin
            ],
        )?;
        self.write_endpoints(value)
    }

    fn update(&mut self, value: &RelationshipType) -> Result<(), StorageError> {
        self.execute(
            "UPDATE relationship_type SET label = ?2, inverse_label = ?3, description = ?4, \
             directionality = ?5, acyclic = ?6, sort_order = ?7 WHERE key = ?1",
            params![
                value.key.as_str(),
                value.label,
                value.inverse_label,
                value.description,
                value.directionality.as_str(),
                value.acyclic,
                value.sort_order
            ],
        )?;
        self.write_endpoints(value)
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM relationship_type_endpoint WHERE relationship_type = ?1",
            [key.as_str()],
        )?;
        self.execute(
            "DELETE FROM relationship_type WHERE key = ?1",
            [key.as_str()],
        )
    }
}

const PROPERTY_GROUP: &str = "SELECT key, label, sort_order FROM property_group";

fn property_group(row: &Row<'_>) -> Result<PropertyGroup, StorageError> {
    let key: String = row.get("key")?;
    Ok(PropertyGroup {
        key: type_key("property_group", &key, &key)?,
        label: row.get("label")?,
        sort_order: row.get("sort_order")?,
    })
}

impl Reader<PropertyGroup> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<PropertyGroup>, StorageError> {
        self.select_one(
            &format!("{PROPERTY_GROUP} WHERE key = ?1"),
            [key.as_str()],
            property_group,
        )
    }

    fn list(&self) -> Result<Vec<PropertyGroup>, StorageError> {
        self.select(
            &format!("{PROPERTY_GROUP} ORDER BY sort_order, key"),
            [],
            property_group,
        )
    }
}

impl Writer<PropertyGroup> for SqliteModel<'_> {
    fn insert(&mut self, group: &PropertyGroup) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO property_group (key, label, sort_order) VALUES (?1, ?2, ?3)",
            params![group.key.as_str(), group.label, group.sort_order],
        )
    }

    fn update(&mut self, group: &PropertyGroup) -> Result<(), StorageError> {
        self.execute(
            "UPDATE property_group SET label = ?2, sort_order = ?3 WHERE key = ?1",
            params![group.key.as_str(), group.label, group.sort_order],
        )
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute("DELETE FROM property_group WHERE key = ?1", [key.as_str()])
    }
}

const PROPERTY_DEFINITION: &str = "SELECT id, entity_type, relationship_type, key, label, \
                                   description, group_key, data_type, reference_entity_type, \
                                   required, multiple, sort_order, builtin FROM property_definition";

fn property_definition(row: &Row<'_>) -> Result<PropertyDefinition, StorageError> {
    let id: String = row.get("id")?;
    let entity_type: Option<String> = row.get("entity_type")?;
    let relationship_type: Option<String> = row.get("relationship_type")?;
    let owner = match (entity_type, relationship_type) {
        (Some(key), None) => PropertyOwner::EntityType(type_key("property_definition", &id, &key)?),
        (None, Some(key)) => {
            PropertyOwner::RelationshipType(type_key("property_definition", &id, &key)?)
        }
        _ => {
            return Err(corrupt(
                "property_definition",
                &id,
                &"it has no single owner",
            ));
        }
    };
    let key: String = row.get("key")?;
    let group: Option<String> = row.get("group_key")?;
    let data_type: String = row.get("data_type")?;
    let reference: Option<String> = row.get("reference_entity_type")?;
    let data_type = match data_type.as_str() {
        "text" => PropertyDataType::Text,
        "integer" => PropertyDataType::Integer,
        "real" => PropertyDataType::Real,
        "boolean" => PropertyDataType::Boolean,
        "date" => PropertyDataType::Date,
        "entity_reference" => PropertyDataType::EntityReference {
            entity_type: reference
                .map(|raw| type_key("property_definition", &id, &raw))
                .transpose()?,
        },
        "choice" => PropertyDataType::Choice {
            options: Vec::new(),
        },
        other => {
            return Err(corrupt(
                "property_definition",
                &id,
                &format!("unknown data type `{other}`"),
            ));
        }
    };
    Ok(PropertyDefinition {
        id: parse::<PropertyDefinitionId>("property_definition", &id, &id)?,
        owner,
        key: type_key("property_definition", &id, &key)?,
        label: row.get("label")?,
        description: row.get("description")?,
        group: group
            .map(|raw| type_key("property_definition", &id, &raw))
            .transpose()?,
        data_type,
        required: row.get("required")?,
        multiple: row.get("multiple")?,
        sort_order: row.get("sort_order")?,
        builtin: row.get("builtin")?,
    })
}

impl SqliteModel<'_> {
    fn attach_choice_options(
        &self,
        definitions: &mut [PropertyDefinition],
    ) -> Result<(), StorageError> {
        let mut statement = self.connection.prepare_cached(
            "SELECT definition_id, key, label FROM property_choice_option ORDER BY definition_id, position",
        )?;
        let mut rows = statement.query([])?;
        let mut options: BTreeMap<String, Vec<ChoiceOption>> = BTreeMap::new();
        while let Some(row) = rows.next()? {
            let owner: String = row.get(0)?;
            let key: String = row.get(1)?;
            options
                .entry(owner.clone())
                .or_default()
                .push(ChoiceOption {
                    key: type_key("property_choice_option", &owner, &key)?,
                    label: row.get(2)?,
                });
        }
        for definition in definitions {
            if let PropertyDataType::Choice { options: list } = &mut definition.data_type {
                *list = options
                    .remove(&definition.id.to_string())
                    .unwrap_or_default();
            }
        }
        Ok(())
    }

    fn write_choice_options(&self, definition: &PropertyDefinition) -> Result<(), StorageError> {
        let id = definition.id.to_string();
        self.execute(
            "DELETE FROM property_choice_option WHERE definition_id = ?1",
            [&id],
        )?;
        if let PropertyDataType::Choice { options } = &definition.data_type {
            for (position, option) in options.iter().enumerate() {
                self.execute(
                    "INSERT INTO property_choice_option (definition_id, key, label, position) \
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        id,
                        option.key.as_str(),
                        option.label,
                        i64::try_from(position).unwrap_or(i64::MAX)
                    ],
                )?;
            }
        }
        Ok(())
    }
}

/// The `data_type` column and the type a reference is restricted to, if any.
fn data_type_columns(data_type: &PropertyDataType) -> (&'static str, Option<&str>) {
    match data_type {
        PropertyDataType::Text => ("text", None),
        PropertyDataType::Integer => ("integer", None),
        PropertyDataType::Real => ("real", None),
        PropertyDataType::Boolean => ("boolean", None),
        PropertyDataType::Date => ("date", None),
        PropertyDataType::EntityReference { entity_type } => (
            "entity_reference",
            entity_type.as_ref().map(TypeKey::as_str),
        ),
        PropertyDataType::Choice { .. } => ("choice", None),
    }
}

impl Reader<PropertyDefinition> for SqliteModel<'_> {
    fn get(&self, id: &PropertyDefinitionId) -> Result<Option<PropertyDefinition>, StorageError> {
        let mut found = self.select(
            &format!("{PROPERTY_DEFINITION} WHERE id = ?1"),
            [id.to_string()],
            property_definition,
        )?;
        self.attach_choice_options(&mut found)?;
        Ok(found.into_iter().next())
    }

    fn list(&self) -> Result<Vec<PropertyDefinition>, StorageError> {
        let mut all = self.select(
            &format!(
                "{PROPERTY_DEFINITION} ORDER BY entity_type, relationship_type, sort_order, key"
            ),
            [],
            property_definition,
        )?;
        self.attach_choice_options(&mut all)?;
        Ok(all)
    }
}

impl Writer<PropertyDefinition> for SqliteModel<'_> {
    fn insert(&mut self, value: &PropertyDefinition) -> Result<(), StorageError> {
        let (entity_type, relationship_type) = match &value.owner {
            PropertyOwner::EntityType(key) => (Some(key.as_str()), None),
            PropertyOwner::RelationshipType(key) => (None, Some(key.as_str())),
        };
        let (data_type, reference) = data_type_columns(&value.data_type);
        self.execute(
            "INSERT INTO property_definition (id, entity_type, relationship_type, key, label, \
             description, group_key, data_type, reference_entity_type, required, multiple, \
             sort_order, builtin) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                value.id.to_string(),
                entity_type,
                relationship_type,
                value.key.as_str(),
                value.label,
                value.description,
                value.group.as_ref().map(TypeKey::as_str),
                data_type,
                reference,
                value.required,
                value.multiple,
                value.sort_order,
                value.builtin
            ],
        )?;
        self.write_choice_options(value)
    }

    fn update(&mut self, value: &PropertyDefinition) -> Result<(), StorageError> {
        let (data_type, reference) = data_type_columns(&value.data_type);
        self.execute(
            "UPDATE property_definition SET label = ?2, description = ?3, group_key = ?4, \
             data_type = ?5, reference_entity_type = ?6, required = ?7, multiple = ?8, \
             sort_order = ?9 WHERE id = ?1",
            params![
                value.id.to_string(),
                value.label,
                value.description,
                value.group.as_ref().map(TypeKey::as_str),
                data_type,
                reference,
                value.required,
                value.multiple,
                value.sort_order
            ],
        )?;
        self.write_choice_options(value)
    }

    fn delete(&mut self, id: &PropertyDefinitionId) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM property_choice_option WHERE definition_id = ?1",
            [id.to_string()],
        )?;
        self.execute(
            "DELETE FROM property_definition WHERE id = ?1",
            [id.to_string()],
        )
    }
}

const APPEARANCE_KIND: &str =
    "SELECT key, label, description, is_presence, sort_order, builtin FROM appearance_kind";

fn appearance_kind(row: &Row<'_>) -> Result<AppearanceKind, StorageError> {
    let key: String = row.get("key")?;
    Ok(AppearanceKind {
        key: type_key("appearance_kind", &key, &key)?,
        label: row.get("label")?,
        description: row.get("description")?,
        is_presence: row.get("is_presence")?,
        sort_order: row.get("sort_order")?,
        builtin: row.get("builtin")?,
    })
}

impl Reader<AppearanceKind> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<AppearanceKind>, StorageError> {
        self.select_one(
            &format!("{APPEARANCE_KIND} WHERE key = ?1"),
            [key.as_str()],
            appearance_kind,
        )
    }

    fn list(&self) -> Result<Vec<AppearanceKind>, StorageError> {
        self.select(
            &format!("{APPEARANCE_KIND} ORDER BY sort_order, key"),
            [],
            appearance_kind,
        )
    }
}

impl Writer<AppearanceKind> for SqliteModel<'_> {
    fn insert(&mut self, value: &AppearanceKind) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO appearance_kind (key, label, description, is_presence, sort_order, builtin) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                value.key.as_str(),
                value.label,
                value.description,
                value.is_presence,
                value.sort_order,
                value.builtin
            ],
        )
    }

    fn update(&mut self, value: &AppearanceKind) -> Result<(), StorageError> {
        self.execute(
            "UPDATE appearance_kind SET label = ?2, description = ?3, is_presence = ?4, \
             sort_order = ?5 WHERE key = ?1",
            params![
                value.key.as_str(),
                value.label,
                value.description,
                value.is_presence,
                value.sort_order
            ],
        )
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute("DELETE FROM appearance_kind WHERE key = ?1", [key.as_str()])
    }
}

const EDITORIAL_LEVEL: &str = "SELECT key, label, depth, builtin FROM editorial_level";

fn editorial_level(row: &Row<'_>) -> Result<EditorialLevel, StorageError> {
    let key: String = row.get("key")?;
    Ok(EditorialLevel {
        key: type_key("editorial_level", &key, &key)?,
        label: row.get("label")?,
        depth: row.get("depth")?,
        builtin: row.get("builtin")?,
    })
}

impl Reader<EditorialLevel> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<EditorialLevel>, StorageError> {
        self.select_one(
            &format!("{EDITORIAL_LEVEL} WHERE key = ?1"),
            [key.as_str()],
            editorial_level,
        )
    }

    fn list(&self) -> Result<Vec<EditorialLevel>, StorageError> {
        self.select(
            &format!("{EDITORIAL_LEVEL} ORDER BY depth"),
            [],
            editorial_level,
        )
    }
}

impl Writer<EditorialLevel> for SqliteModel<'_> {
    fn insert(&mut self, value: &EditorialLevel) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO editorial_level (key, label, depth, builtin) VALUES (?1, ?2, ?3, ?4)",
            params![value.key.as_str(), value.label, value.depth, value.builtin],
        )
    }

    fn update(&mut self, value: &EditorialLevel) -> Result<(), StorageError> {
        self.execute(
            "UPDATE editorial_level SET label = ?2, depth = ?3 WHERE key = ?1",
            params![value.key.as_str(), value.label, value.depth],
        )
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute("DELETE FROM editorial_level WHERE key = ?1", [key.as_str()])
    }
}

const CALENDAR: &str = "SELECT id, name, description, is_default, universe_offset, epoch_weekday, \
                        created_at, updated_at FROM calendar";

fn calendar(row: &Row<'_>) -> Result<Calendar, StorageError> {
    let id: String = row.get("id")?;
    Ok(Calendar {
        id: parse::<CalendarId>("calendar", &id, &id)?,
        name: row.get("name")?,
        description: row.get("description")?,
        is_default: row.get("is_default")?,
        definition: CalendarDefinition {
            months: Vec::new(),
            leap_rules: Vec::new(),
            weekdays: Vec::new(),
            epoch_weekday: row.get("epoch_weekday")?,
            eras: Vec::new(),
            universe_offset: row.get("universe_offset")?,
        },
        created_at: timestamp(row.get("created_at")?),
        updated_at: timestamp(row.get("updated_at")?),
    })
}

impl SqliteModel<'_> {
    /// Fills in the months, weekdays, leap rules and eras of the calendars.
    fn attach_calendar_parts(&self, calendars: &mut [Calendar]) -> Result<(), StorageError> {
        for calendar in calendars {
            let id = calendar.id.to_string();
            let definition = &mut calendar.definition;

            let mut statement = self.connection.prepare_cached(
                "SELECT label, days FROM calendar_month WHERE calendar_id = ?1 ORDER BY position",
            )?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                definition.months.push(Month {
                    label: row.get(0)?,
                    days: row.get(1)?,
                });
            }
            let mut statement = self.connection.prepare_cached(
                "SELECT label FROM calendar_weekday WHERE calendar_id = ?1 ORDER BY position",
            )?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                definition.weekdays.push(row.get(0)?);
            }
            let mut statement = self.connection.prepare_cached(
                "SELECT every_years, except_every_years, unless_every_years, month_position, \
                 extra_days FROM calendar_leap_rule WHERE calendar_id = ?1 ORDER BY id",
            )?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                definition.leap_rules.push(LeapRule {
                    every_years: row.get(0)?,
                    except_every_years: row.get(1)?,
                    unless_every_years: row.get(2)?,
                    month: row.get(3)?,
                    extra_days: row.get(4)?,
                });
            }
            let mut statement = self.connection.prepare_cached(
                "SELECT label, start_year, direction FROM calendar_era WHERE calendar_id = ?1 \
                 ORDER BY start_year",
            )?;
            let mut rows = statement.query([&id])?;
            while let Some(row) = rows.next()? {
                let direction: String = row.get(2)?;
                definition.eras.push(Era {
                    label: row.get(0)?,
                    start_year: row.get(1)?,
                    direction: match direction.as_str() {
                        "forward" => EraDirection::Forward,
                        "backward" => EraDirection::Backward,
                        other => {
                            return Err(corrupt(
                                "calendar_era",
                                &id,
                                &format!("unknown direction `{other}`"),
                            ));
                        }
                    },
                });
            }
        }
        Ok(())
    }

    fn write_calendar_parts(&self, calendar: &Calendar) -> Result<(), StorageError> {
        let id = calendar.id.to_string();
        for table in [
            "calendar_leap_rule",
            "calendar_month",
            "calendar_weekday",
            "calendar_era",
        ] {
            self.execute(
                &format!("DELETE FROM {table} WHERE calendar_id = ?1"),
                [&id],
            )?;
        }
        let definition = &calendar.definition;
        for (index, month) in definition.months.iter().enumerate() {
            self.execute(
                "INSERT INTO calendar_month (calendar_id, position, label, days) VALUES (?1, ?2, ?3, ?4)",
                params![id, i64::try_from(index + 1).unwrap_or(i64::MAX), month.label, month.days],
            )?;
        }
        for (index, weekday) in definition.weekdays.iter().enumerate() {
            self.execute(
                "INSERT INTO calendar_weekday (calendar_id, position, label) VALUES (?1, ?2, ?3)",
                params![id, i64::try_from(index).unwrap_or(i64::MAX), weekday],
            )?;
        }
        for rule in &definition.leap_rules {
            self.execute(
                "INSERT INTO calendar_leap_rule (calendar_id, every_years, except_every_years, \
                 unless_every_years, month_position, extra_days) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    rule.every_years,
                    rule.except_every_years,
                    rule.unless_every_years,
                    rule.month,
                    rule.extra_days
                ],
            )?;
        }
        for era in &definition.eras {
            let direction = match era.direction {
                EraDirection::Forward => "forward",
                EraDirection::Backward => "backward",
            };
            self.execute(
                "INSERT INTO calendar_era (calendar_id, start_year, label, direction) VALUES (?1, ?2, ?3, ?4)",
                params![id, era.start_year, era.label, direction],
            )?;
        }
        Ok(())
    }
}

impl Reader<Calendar> for SqliteModel<'_> {
    fn get(&self, id: &CalendarId) -> Result<Option<Calendar>, StorageError> {
        let mut found = self.select(
            &format!("{CALENDAR} WHERE id = ?1"),
            [id.to_string()],
            calendar,
        )?;
        self.attach_calendar_parts(&mut found)?;
        Ok(found.into_iter().next())
    }

    fn list(&self) -> Result<Vec<Calendar>, StorageError> {
        let mut all = self.select(&format!("{CALENDAR} ORDER BY name, id"), [], calendar)?;
        self.attach_calendar_parts(&mut all)?;
        Ok(all)
    }
}

impl Writer<Calendar> for SqliteModel<'_> {
    fn insert(&mut self, value: &Calendar) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO calendar (id, name, description, is_default, universe_offset, \
             epoch_weekday, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                value.id.to_string(),
                value.name,
                value.description,
                value.is_default,
                value.definition.universe_offset,
                value.definition.epoch_weekday,
                value.created_at.as_unix_millis(),
                value.updated_at.as_unix_millis()
            ],
        )?;
        self.write_calendar_parts(value)
    }

    fn update(&mut self, value: &Calendar) -> Result<(), StorageError> {
        self.execute(
            "UPDATE calendar SET name = ?2, description = ?3, is_default = ?4, \
             universe_offset = ?5, epoch_weekday = ?6, updated_at = ?7 WHERE id = ?1",
            params![
                value.id.to_string(),
                value.name,
                value.description,
                value.is_default,
                value.definition.universe_offset,
                value.definition.epoch_weekday,
                value.updated_at.as_unix_millis()
            ],
        )?;
        self.write_calendar_parts(value)
    }

    fn delete(&mut self, id: &CalendarId) -> Result<(), StorageError> {
        let id = id.to_string();
        for table in [
            "calendar_leap_rule",
            "calendar_month",
            "calendar_weekday",
            "calendar_era",
        ] {
            self.execute(
                &format!("DELETE FROM {table} WHERE calendar_id = ?1"),
                [&id],
            )?;
        }
        self.execute("DELETE FROM calendar WHERE id = ?1", [&id])
    }
}

// ---- vocabularies and diplomatic stances

fn vocabulary_term(
    vocabulary: Vocabulary,
) -> impl Fn(&Row<'_>) -> Result<VocabularyTerm, StorageError> {
    move |row| {
        let key: String = row.get("key")?;
        Ok(VocabularyTerm {
            vocabulary,
            key: type_key(vocabulary.as_str(), &key, &key)?,
            label: row.get("label")?,
            description: row.get("description")?,
            sort_order: row.get("sort_order")?,
            builtin: row.get("builtin")?,
        })
    }
}

impl Reader<VocabularyTerm> for SqliteModel<'_> {
    fn get(&self, id: &(Vocabulary, TypeKey)) -> Result<Option<VocabularyTerm>, StorageError> {
        let (vocabulary, key) = id;
        let build = vocabulary_term(*vocabulary);
        let mut statement = self.connection.prepare_cached(&format!(
            "SELECT key, label, description, sort_order, builtin FROM {} WHERE key = ?1",
            vocabulary.as_str()
        ))?;
        let mut rows = statement.query([key.as_str()])?;
        rows.next()?.map(build).transpose()
    }

    fn list(&self) -> Result<Vec<VocabularyTerm>, StorageError> {
        let mut terms = Vec::new();
        for vocabulary in Vocabulary::ALL {
            let build = vocabulary_term(vocabulary);
            let mut statement = self.connection.prepare_cached(&format!(
                "SELECT key, label, description, sort_order, builtin FROM {} ORDER BY sort_order, key",
                vocabulary.as_str()
            ))?;
            let mut rows = statement.query([])?;
            while let Some(row) = rows.next()? {
                terms.push(build(row)?);
            }
        }
        Ok(terms)
    }
}

impl Writer<VocabularyTerm> for SqliteModel<'_> {
    fn insert(&mut self, term: &VocabularyTerm) -> Result<(), StorageError> {
        self.execute(
            &format!(
                "INSERT INTO {} (key, label, description, sort_order, builtin) VALUES (?1, ?2, ?3, ?4, ?5)",
                term.vocabulary.as_str()
            ),
            params![term.key.as_str(), term.label, term.description, term.sort_order, term.builtin],
        )
    }

    fn update(&mut self, term: &VocabularyTerm) -> Result<(), StorageError> {
        self.execute(
            &format!(
                "UPDATE {} SET label = ?2, description = ?3, sort_order = ?4 WHERE key = ?1",
                term.vocabulary.as_str()
            ),
            params![
                term.key.as_str(),
                term.label,
                term.description,
                term.sort_order
            ],
        )
    }

    fn delete(&mut self, id: &(Vocabulary, TypeKey)) -> Result<(), StorageError> {
        self.execute(
            &format!("DELETE FROM {} WHERE key = ?1", id.0.as_str()),
            [id.1.as_str()],
        )
    }
}

const DIPLOMATIC_STANCE: &str =
    "SELECT key, label, description, is_hierarchical, sort_order, builtin FROM diplomatic_stance";

fn diplomatic_stance(row: &Row<'_>) -> Result<DiplomaticStance, StorageError> {
    let key: String = row.get("key")?;
    Ok(DiplomaticStance {
        key: type_key("diplomatic_stance", &key, &key)?,
        label: row.get("label")?,
        description: row.get("description")?,
        is_hierarchical: row.get("is_hierarchical")?,
        sort_order: row.get("sort_order")?,
        builtin: row.get("builtin")?,
    })
}

impl Reader<DiplomaticStance> for SqliteModel<'_> {
    fn get(&self, key: &TypeKey) -> Result<Option<DiplomaticStance>, StorageError> {
        self.select_one(
            &format!("{DIPLOMATIC_STANCE} WHERE key = ?1"),
            [key.as_str()],
            diplomatic_stance,
        )
    }

    fn list(&self) -> Result<Vec<DiplomaticStance>, StorageError> {
        self.select(
            &format!("{DIPLOMATIC_STANCE} ORDER BY sort_order, key"),
            [],
            diplomatic_stance,
        )
    }
}

impl Writer<DiplomaticStance> for SqliteModel<'_> {
    fn insert(&mut self, value: &DiplomaticStance) -> Result<(), StorageError> {
        self.execute(
            "INSERT INTO diplomatic_stance (key, label, description, is_hierarchical, sort_order, \
             builtin) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                value.key.as_str(),
                value.label,
                value.description,
                value.is_hierarchical,
                value.sort_order,
                value.builtin
            ],
        )
    }

    fn update(&mut self, value: &DiplomaticStance) -> Result<(), StorageError> {
        self.execute(
            "UPDATE diplomatic_stance SET label = ?2, description = ?3, is_hierarchical = ?4, \
             sort_order = ?5 WHERE key = ?1",
            params![
                value.key.as_str(),
                value.label,
                value.description,
                value.is_hierarchical,
                value.sort_order
            ],
        )
    }

    fn delete(&mut self, key: &TypeKey) -> Result<(), StorageError> {
        self.execute(
            "DELETE FROM diplomatic_stance WHERE key = ?1",
            [key.as_str()],
        )
    }
}
