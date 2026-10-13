use super::{Failure, ModelEditor};
use crate::appearance::Appearance;
use crate::appearance_kind::AppearanceKind;
use crate::canon_state::CanonState;
use crate::editorial_level::EditorialLevel;
use crate::editorial_node::EditorialNode;
use crate::entity::Entity;
use crate::entity_type::EntityRole;
use crate::error::DomainError;
use crate::event_detail::EventDetail;
use crate::identifier::{AppearanceId, EntityId};
use crate::record_ref::{RecordKind, RecordRef};
use crate::repository::ModelRepository;
use crate::timestamp::Timestamp;

/// What restructuring or removing a section of the work would touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize)]
pub struct EditorialImpact {
    /// Sections nested under the node, at any depth.
    pub descendants: usize,
    /// Appearances that take place in the node or in its descendants.
    pub appearances: usize,
    /// Relationships that name the node or a descendant as their context.
    pub context_uses: usize,
}

impl<R: ModelRepository> ModelEditor<'_, R> {
    fn validate_appearance(&self, appearance: &Appearance) -> Result<(), Failure<R>> {
        appearance.validate()?;
        self.require::<Entity>("entity", &appearance.entity)?;
        self.narrative_context(&appearance.context)?;
        self.require_defined::<AppearanceKind>("appearance kind", &appearance.kind)?;
        self.require_defined::<CanonState>("canon state", &appearance.canon_state)?;
        if let Some(viewpoint) = &appearance.viewpoint {
            self.require::<Entity>("entity", viewpoint)?;
        }
        Ok(())
    }

    pub fn add_appearance(&mut self, appearance: Appearance) -> Result<Appearance, Failure<R>> {
        self.validate_appearance(&appearance)?;
        if self.find::<Appearance>(&appearance.id)?.is_some() {
            return Err(DomainError::duplicate("appearance", appearance.id).into());
        }
        self.insert(&appearance)?;
        Ok(appearance)
    }

    /// Replaces an appearance's editable fields. Identity, origin and creation time are kept.
    pub fn update_appearance(
        &mut self,
        mut appearance: Appearance,
        now: Timestamp,
    ) -> Result<Appearance, Failure<R>> {
        let stored = self.require::<Appearance>("appearance", &appearance.id)?;
        self.validate_appearance(&appearance)?;
        appearance.created_at = stored.created_at;
        appearance.origin = stored.origin;
        appearance.updated_at = now;
        self.update(&appearance)?;
        Ok(appearance)
    }

    pub fn remove_appearance(&mut self, id: &AppearanceId) -> Result<(), Failure<R>> {
        self.require::<Appearance>("appearance", id)?;
        self.remove_record(&RecordRef::new(RecordKind::Appearance, id.as_uuid()))
    }

    /// Creates or replaces the event detail of an entity whose type has the event role.
    pub fn set_event_detail(&mut self, detail: EventDetail) -> Result<EventDetail, Failure<R>> {
        detail.validate()?;
        self.entity_with_role(&detail.entity, EntityRole::Event, "event")?;
        if let Some(location) = &detail.location {
            self.require::<Entity>("entity", location)?;
        }
        for participant in &detail.participants {
            self.require::<Entity>("entity", &participant.entity)?;
        }
        if self.find::<EventDetail>(&detail.entity)?.is_some() {
            self.update(&detail)?;
        } else {
            self.insert(&detail)?;
        }
        Ok(detail)
    }

    pub fn clear_event_detail(&mut self, entity: &EntityId) -> Result<(), Failure<R>> {
        self.require::<EventDetail>("event detail", entity)?;
        self.delete::<EventDetail>(entity)
    }

    fn validate_placement(
        &self,
        node: &EditorialNode,
        level: &EditorialLevel,
    ) -> Result<(), Failure<R>> {
        let Some(parent_id) = &node.parent else {
            return Ok(());
        };
        if parent_id == &node.entity {
            return Err(DomainError::InvalidPlacement(
                "a section cannot contain itself".to_owned(),
            )
            .into());
        }
        let parent = self.require::<EditorialNode>("editorial node", parent_id)?;
        let parent_level =
            self.require_defined::<EditorialLevel>("editorial level", &parent.level)?;
        if parent_level.depth >= level.depth {
            return Err(DomainError::InvalidPlacement(format!(
                "a {} cannot be placed inside a {}",
                level.label, parent_level.label
            ))
            .into());
        }
        Ok(())
    }

    /// Places an editorial entity in the hierarchy of the work.
    pub fn place_editorial_node(
        &mut self,
        node: EditorialNode,
    ) -> Result<EditorialNode, Failure<R>> {
        self.entity_with_role(&node.entity, EntityRole::EditorialUnit, "editorial unit")?;
        let level = self.require_defined::<EditorialLevel>("editorial level", &node.level)?;
        if self.find::<EditorialNode>(&node.entity)?.is_some() {
            return Err(DomainError::duplicate("editorial node", node.entity).into());
        }
        self.validate_placement(&node, &level)?;
        self.insert(&node)?;
        Ok(node)
    }

    /// Moves a node under another parent, or to the top, at the given position.
    ///
    /// The node keeps its level, so a move can never put a section inside its own descendant.
    pub fn move_editorial_node(
        &mut self,
        entity: &EntityId,
        parent: Option<EntityId>,
        position: i32,
    ) -> Result<EditorialNode, Failure<R>> {
        let mut node = self.require::<EditorialNode>("editorial node", entity)?;
        let level = self.require_defined::<EditorialLevel>("editorial level", &node.level)?;
        node.parent = parent;
        node.position = position;
        self.validate_placement(&node, &level)?;
        self.update(&node)?;
        Ok(node)
    }

    /// Puts a node at `index` among the children of `parent` (or among the top-level sections)
    /// and renumbers its siblings, so the order is always a clean `0..n`. An index past the end
    /// means last.
    pub fn reorder_editorial_node(
        &mut self,
        entity: &EntityId,
        parent: Option<EntityId>,
        index: usize,
    ) -> Result<Vec<EditorialNode>, Failure<R>> {
        let mut node = self.require::<EditorialNode>("editorial node", entity)?;
        let level = self.require_defined::<EditorialLevel>("editorial level", &node.level)?;
        node.parent = parent;
        self.validate_placement(&node, &level)?;

        let mut siblings: Vec<EditorialNode> = self
            .query(|r| r.editorial_children(parent.as_ref()))?
            .into_iter()
            .filter(|sibling| sibling.entity != *entity)
            .collect();
        siblings.insert(index.min(siblings.len()), node);
        for (position, sibling) in siblings.iter_mut().enumerate() {
            sibling.position = i32::try_from(position).unwrap_or(i32::MAX);
            self.update(&*sibling)?;
        }
        Ok(siblings)
    }

    /// Takes a node out of the hierarchy. Its entity, appearances and relationships are untouched.
    pub fn remove_editorial_node(&mut self, entity: &EntityId) -> Result<(), Failure<R>> {
        self.require::<EditorialNode>("editorial node", entity)?;
        if !self
            .query(|r| r.editorial_children(Some(entity)))?
            .is_empty()
        {
            return Err(DomainError::EditorialChildrenRemain(*entity).into());
        }
        self.delete::<EditorialNode>(entity)
    }

    /// What restructuring or removing a section would touch, so it can be reviewed first.
    pub fn editorial_impact(&self, entity: &EntityId) -> Result<EditorialImpact, Failure<R>> {
        self.require::<EditorialNode>("editorial node", entity)?;
        let mut impact = EditorialImpact::default();
        let mut pending = vec![*entity];
        while let Some(current) = pending.pop() {
            impact.appearances += self.query(|r| r.appearances_in(&current))?.len();
            impact.context_uses += self.query(|r| r.relationships_in_context(&current))?.len();
            for child in self.query(|r| r.editorial_children(Some(&current)))? {
                impact.descendants += 1;
                pending.push(child.entity);
            }
        }
        Ok(impact)
    }
}
