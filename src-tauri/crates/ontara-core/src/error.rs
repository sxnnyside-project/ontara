use thiserror::Error;

use crate::identifier::EntityId;
use crate::type_key::TypeKey;

/// A domain rule was violated. Nothing was changed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error(
        "`{0}` is not a valid key: use lowercase letters, digits and underscores, starting with a letter (max 64)"
    )]
    InvalidTypeKey(String),

    #[error("`{1}` is not a valid {0}")]
    UnknownValue(&'static str, String),

    #[error("a {0} must not be blank")]
    BlankText(&'static str),

    #[error("{what} `{key}` is not defined in this project")]
    NotDefined { what: &'static str, key: String },

    #[error("{what} `{key}` is already defined")]
    AlreadyDefined { what: &'static str, key: String },

    #[error("{what} {id} does not exist")]
    NotFound { what: &'static str, id: String },

    #[error("{what} {id} already exists")]
    Duplicate { what: &'static str, id: String },

    #[error("invalid calendar: {0}")]
    InvalidCalendar(String),

    #[error("invalid time: {0}")]
    InvalidTime(String),

    #[error("the date is outside the range this calendar can compute")]
    DateOutOfRange,

    #[error("property `{property}` is not valid for this record: {reason}")]
    InvalidProperty { property: String, reason: String },

    #[error("relationship type `{relationship_type}` does not allow a {entity_type} as its {side}")]
    EndpointNotAllowed {
        relationship_type: TypeKey,
        side: &'static str,
        entity_type: TypeKey,
    },

    #[error("entity {entity} is a `{actual}`, but this requires a type with the {required} role")]
    RoleMismatch {
        entity: EntityId,
        actual: TypeKey,
        required: &'static str,
    },

    #[error("invalid editorial placement: {0}")]
    InvalidPlacement(String),

    #[error("`{0}` is not a valid project-relative path")]
    InvalidPath(String),

    #[error(
        "entity {entity} is still referenced ({references} reference(s)); choose how to resolve them"
    )]
    StillReferenced { entity: EntityId, references: usize },

    #[error("{0}")]
    Invalid(String),

    #[error("entity {entity} is a `{actual}`, but a `{expected}` is required here")]
    WrongEntityType {
        entity: EntityId,
        actual: TypeKey,
        expected: TypeKey,
    },

    #[error("entity type `{0}` only groups other types; choose one of its subtypes")]
    AbstractType(TypeKey),

    #[error("invalid type hierarchy: {0}")]
    InvalidTypeHierarchy(String),

    #[error(
        "a `{relationship_type}` relationship here would make an entity contain or descend from itself"
    )]
    WouldCreateCycle { relationship_type: TypeKey },

    #[error("the {0} of a record cannot be changed after it is created")]
    Immutable(&'static str),

    #[error("entity {0} still has editorial children; move or remove them first")]
    EditorialChildrenRemain(EntityId),

    #[error("{what} `{key}` ships with every project and cannot be removed; rename it instead")]
    BuiltinCatalogEntry { what: &'static str, key: String },

    #[error("{what} `{key}` is still in use ({uses} use(s)); stop using it before removing it")]
    StillInUse {
        what: &'static str,
        key: String,
        uses: usize,
    },
}

// The call sites hand over owned, `Copy` identifiers; borrowing each one would only add noise.
#[allow(clippy::needless_pass_by_value)]
impl DomainError {
    pub(crate) fn not_defined(what: &'static str, key: impl ToString) -> Self {
        Self::NotDefined {
            what,
            key: key.to_string(),
        }
    }

    pub(crate) fn already_defined(what: &'static str, key: impl ToString) -> Self {
        Self::AlreadyDefined {
            what,
            key: key.to_string(),
        }
    }

    pub(crate) fn not_found(what: &'static str, id: impl ToString) -> Self {
        Self::NotFound {
            what,
            id: id.to_string(),
        }
    }

    pub(crate) fn duplicate(what: &'static str, id: impl ToString) -> Self {
        Self::Duplicate {
            what,
            id: id.to_string(),
        }
    }
}
