use serde::{Deserialize, Serialize};

use crate::editorial_node::EditorialNode;
use crate::event_detail::EventDetail;
use crate::identifier::{
    AppearanceId, AttachmentId, EntityId, PropertyDefinitionId, PropertyValueId, RelationshipId,
};
use crate::property_value::PropertyValueOwner;
use crate::repository::{ModelReader, Reader};
use crate::type_key::TypeKey;

/// How two entities are connected, seen from the entity being looked at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "via", rename_all = "snake_case")]
pub enum ConnectionKind {
    /// A relationship; `outgoing` when the entity is its source.
    Relationship {
        relationship: RelationshipId,
        relationship_type: TypeKey,
        outgoing: bool,
    },
    /// The entity shows up in `other`, a chapter, a scene or an event.
    AppearsIn {
        appearance: AppearanceId,
        kind: TypeKey,
    },
    /// `other` shows up in the entity, which is a chapter, a scene or an event.
    Hosts {
        appearance: AppearanceId,
        kind: TypeKey,
    },
    /// The entity takes part in `other`, an event.
    Participates { role: String },
    /// The entity, an event, has `other` as participant.
    HasParticipant { role: String },
    /// The event happens at `other`.
    TakesPlaceAt,
    /// The entity is the place where `other` happens.
    HostsEvent,
    /// The entity carries `other`, an asset.
    HasAsset { attachment: AttachmentId },
    /// The entity, an asset, is attached to `other`.
    AttachedTo { attachment: AttachmentId },
    /// A property of `other` points at the entity.
    PropertyOf {
        value: PropertyValueId,
        definition: PropertyDefinitionId,
    },
    /// The section is placed inside `other`.
    InsideSection,
    /// The section contains `other`.
    ContainsSection,
}

/// One link between the entity being looked at and another entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub other: EntityId,
    #[serde(flatten)]
    pub kind: ConnectionKind,
}

/// Everything that links an entity to the rest of the universe, in both directions, so an author
/// can move from any entity to what mentions it and what it mentions.
pub fn connections_of<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
) -> Result<Vec<Connection>, R::Error> {
    let mut found = Vec::new();
    relationship_connections(reader, entity, &mut found)?;
    appearance_connections(reader, entity, &mut found)?;
    event_connections(reader, entity, &mut found)?;
    asset_connections(reader, entity, &mut found)?;
    property_connections(reader, entity, &mut found)?;
    editorial_connections(reader, entity, &mut found)?;
    Ok(found)
}

fn relationship_connections<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    found: &mut Vec<Connection>,
) -> Result<(), R::Error> {
    for relationship in reader.relationships_of(entity)? {
        let outgoing = relationship.source == *entity;
        found.push(Connection {
            other: if outgoing {
                relationship.target
            } else {
                relationship.source
            },
            kind: ConnectionKind::Relationship {
                relationship: relationship.id,
                relationship_type: relationship.relationship_type,
                outgoing,
            },
        });
    }
    Ok(())
}

fn appearance_connections<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    found: &mut Vec<Connection>,
) -> Result<(), R::Error> {
    for appearance in reader.appearances_of(entity)? {
        found.push(Connection {
            other: appearance.context,
            kind: ConnectionKind::AppearsIn {
                appearance: appearance.id,
                kind: appearance.kind,
            },
        });
    }
    for appearance in reader.appearances_in(entity)? {
        found.push(Connection {
            other: appearance.entity,
            kind: ConnectionKind::Hosts {
                appearance: appearance.id,
                kind: appearance.kind,
            },
        });
    }
    Ok(())
}

fn event_connections<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    found: &mut Vec<Connection>,
) -> Result<(), R::Error> {
    for event in reader.events_involving(entity)? {
        if event.location.as_ref() == Some(entity) {
            found.push(Connection {
                other: event.entity,
                kind: ConnectionKind::HostsEvent,
            });
        }
        for participant in event.participants.iter().filter(|p| p.entity == *entity) {
            found.push(Connection {
                other: event.entity,
                kind: ConnectionKind::Participates {
                    role: participant.role.clone(),
                },
            });
        }
    }
    if let Some(detail) = Reader::<EventDetail>::get(reader, entity)? {
        if let Some(location) = detail.location {
            found.push(Connection {
                other: location,
                kind: ConnectionKind::TakesPlaceAt,
            });
        }
        for participant in detail.participants {
            found.push(Connection {
                other: participant.entity,
                kind: ConnectionKind::HasParticipant {
                    role: participant.role,
                },
            });
        }
    }
    Ok(())
}

fn asset_connections<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    found: &mut Vec<Connection>,
) -> Result<(), R::Error> {
    for attachment in reader.attachments_of(entity)? {
        let (other, kind) = if attachment.owner == *entity {
            (
                attachment.asset,
                ConnectionKind::HasAsset {
                    attachment: attachment.id,
                },
            )
        } else {
            (
                attachment.owner,
                ConnectionKind::AttachedTo {
                    attachment: attachment.id,
                },
            )
        };
        found.push(Connection { other, kind });
    }
    Ok(())
}

fn property_connections<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    found: &mut Vec<Connection>,
) -> Result<(), R::Error> {
    for value in reader.property_values_referencing(entity)? {
        if let PropertyValueOwner::Entity(owner) = value.owner {
            found.push(Connection {
                other: owner,
                kind: ConnectionKind::PropertyOf {
                    value: value.id,
                    definition: value.definition,
                },
            });
        }
    }
    Ok(())
}

fn editorial_connections<R: ModelReader>(
    reader: &R,
    entity: &EntityId,
    found: &mut Vec<Connection>,
) -> Result<(), R::Error> {
    let parent = Reader::<EditorialNode>::get(reader, entity)?.and_then(|node| node.parent);
    if let Some(parent) = parent {
        found.push(Connection {
            other: parent,
            kind: ConnectionKind::InsideSection,
        });
    }
    for child in reader.editorial_children(Some(entity))? {
        found.push(Connection {
            other: child.entity,
            kind: ConnectionKind::ContainsSection,
        });
    }
    Ok(())
}
