use serde::{Deserialize, Serialize};

use crate::record::Record;
use crate::timestamp::Timestamp;

/// The description of the universe a project holds. There is exactly one per project.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UniverseProfile {
    /// One-line premise.
    pub summary: String,
    pub description: String,
    pub updated_at: Option<Timestamp>,
}

impl Record for UniverseProfile {
    type Id = ();

    fn id(&self) {}
}
