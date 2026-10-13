use serde::{Deserialize, Serialize};

use crate::identifier::CanonStateChangeId;
use crate::origin::Origin;
use crate::record::Record;
use crate::record_ref::RecordRef;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

/// One change of a record's canon state, kept so the history of a decision is never lost.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonStateChange {
    pub id: CanonStateChangeId,
    pub target: RecordRef,
    pub from: TypeKey,
    pub to: TypeKey,
    pub reason: Option<String>,
    pub origin: Origin,
    pub changed_at: Timestamp,
}

impl Record for CanonStateChange {
    type Id = CanonStateChangeId;

    fn id(&self) -> CanonStateChangeId {
        self.id
    }
}
