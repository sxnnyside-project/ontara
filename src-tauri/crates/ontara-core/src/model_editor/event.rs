use std::collections::BTreeSet;

use super::{Failure, ModelEditor};
use crate::canon_state::CanonState;
use crate::entity_type::EntityRole;
use crate::error::DomainError;
use crate::event::EventCausality;
use crate::identifier::EventCausalityId;
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;
use crate::type_key::TypeKey;

impl<R: ModelRepository> ModelEditor<'_, R> {
    pub fn add_event_causality(
        &mut self,
        record: EventCausality,
    ) -> Result<EventCausality, Failure<R>> {
        record.validate()?;
        self.require_entity_with_role(&record.cause, EntityRole::Event, "event")?;
        self.require_entity_with_role(&record.effect, EntityRole::Event, "event")?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.event_causality_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("event_causality"),
            }
            .into());
        }
        if self.find::<EventCausality>(&record.id)?.is_some() {
            return Err(DomainError::duplicate("event causality", record.id).into());
        }
        self.insert(&record)?;
        Ok(record)
    }

    pub fn update_event_causality(
        &mut self,
        mut record: EventCausality,
        now: Timestamp,
    ) -> Result<EventCausality, Failure<R>> {
        let stored = self.require::<EventCausality>("event causality", &record.id)?;
        record.validate()?;
        self.require_entity_with_role(&record.cause, EntityRole::Event, "event")?;
        self.require_entity_with_role(&record.effect, EntityRole::Event, "event")?;
        self.require_defined::<CanonState>("canon state", &record.canon_state)?;
        if self.event_causality_would_loop(&record)? {
            return Err(DomainError::WouldCreateCycle {
                relationship_type: TypeKey::from_static("event_causality"),
            }
            .into());
        }
        record.created_at = stored.created_at;
        record.origin = stored.origin;
        record.updated_at = now;
        self.update(&record)?;
        Ok(record)
    }

    pub fn remove_event_causality(&mut self, id: &EventCausalityId) -> Result<(), Failure<R>> {
        self.require::<EventCausality>("event causality", id)?;
        self.remove_record(&RecordRef::new(RecordKind::EventCausality, id.as_uuid()))
    }

    /// Whether adding the record would let the causality lead back to where it started. Only
    /// open-ended records count: one that has ended is history and cannot make a loop today.
    fn event_causality_would_loop(&self, candidate: &EventCausality) -> Result<bool, Failure<R>> {
        let mut seen = BTreeSet::from([candidate.effect]);
        let mut pending = vec![candidate.effect];
        while let Some(current) = pending.pop() {
            if current == candidate.cause {
                return Ok(true);
            }
            for edge in self.query(|repository| repository.event_causality_by_cause(&current))? {
                if edge.id == candidate.id {
                    continue;
                }
                if seen.insert(edge.effect) {
                    pending.push(edge.effect);
                }
            }
        }
        Ok(false)
    }
}
