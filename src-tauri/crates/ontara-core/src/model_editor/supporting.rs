use super::{Failure, ModelEditor};
use crate::asset::{AssetFile, Attachment};
use crate::entity::Entity;
use crate::entity_type::EntityRole;
use crate::error::DomainError;
use crate::identifier::{AttachmentId, NoteId, SourceId, SourceReferenceId};
use crate::note::Note;
use crate::repository::ModelRepository;
use crate::source::{Source, SourceReference};
use crate::timestamp::Timestamp;
use crate::universe_profile::UniverseProfile;

impl<R: ModelRepository> ModelEditor<'_, R> {
    /// Registers the file behind an entity of an asset type.
    pub fn register_asset_file(&mut self, file: AssetFile) -> Result<AssetFile, Failure<R>> {
        file.validate()?;
        self.entity_with_role(&file.entity, EntityRole::Asset, "asset")?;
        if self.find::<AssetFile>(&file.entity)?.is_some() {
            self.update(&file)?;
        } else {
            self.insert(&file)?;
        }
        Ok(file)
    }

    /// Associates an asset with an entity without copying the asset.
    pub fn attach_asset(&mut self, attachment: Attachment) -> Result<Attachment, Failure<R>> {
        attachment.validate()?;
        self.require::<Entity>("entity", &attachment.owner)?;
        self.entity_with_role(&attachment.asset, EntityRole::Asset, "asset")?;
        if self.find::<Attachment>(&attachment.id)?.is_some() {
            return Err(DomainError::duplicate("attachment", attachment.id).into());
        }
        self.insert(&attachment)?;
        Ok(attachment)
    }

    /// Removes the association only; the asset itself is never deleted by this.
    pub fn detach_asset(&mut self, id: &AttachmentId) -> Result<(), Failure<R>> {
        self.require::<Attachment>("attachment", id)?;
        self.delete::<Attachment>(id)
    }

    pub fn add_source(&mut self, source: Source) -> Result<Source, Failure<R>> {
        source.validate()?;
        if self.find::<Source>(&source.id)?.is_some() {
            return Err(DomainError::duplicate("source", source.id).into());
        }
        self.insert(&source)?;
        Ok(source)
    }

    pub fn update_source(
        &mut self,
        mut source: Source,
        now: Timestamp,
    ) -> Result<Source, Failure<R>> {
        let stored = self.require::<Source>("source", &source.id)?;
        source.validate()?;
        source.created_at = stored.created_at;
        source.updated_at = now;
        self.update(&source)?;
        Ok(source)
    }

    /// Removes a source together with every citation of it.
    pub fn remove_source(&mut self, id: &SourceId) -> Result<(), Failure<R>> {
        let source = self.require::<Source>("source", id)?;
        for citation in self.query(|r| r.citations_from(&source))? {
            self.delete::<SourceReference>(&citation.id)?;
        }
        self.delete::<Source>(id)
    }

    /// Cites a source as support for a record.
    pub fn cite(&mut self, reference: SourceReference) -> Result<SourceReference, Failure<R>> {
        reference.validate()?;
        self.require::<Source>("source", &reference.source)?;
        self.require_record(&reference.target)?;
        if self.find::<SourceReference>(&reference.id)?.is_some() {
            return Err(DomainError::duplicate("citation", reference.id).into());
        }
        self.insert(&reference)?;
        Ok(reference)
    }

    pub fn remove_citation(&mut self, id: &SourceReferenceId) -> Result<(), Failure<R>> {
        self.require::<SourceReference>("citation", id)?;
        self.delete::<SourceReference>(id)
    }

    pub fn add_note(&mut self, note: Note) -> Result<Note, Failure<R>> {
        note.validate()?;
        self.require::<Entity>("entity", &note.entity)?;
        if self.find::<Note>(&note.id)?.is_some() {
            return Err(DomainError::duplicate("note", note.id).into());
        }
        self.insert(&note)?;
        Ok(note)
    }

    pub fn update_note(&mut self, mut note: Note, now: Timestamp) -> Result<Note, Failure<R>> {
        let stored = self.require::<Note>("note", &note.id)?;
        if stored.entity != note.entity {
            return Err(DomainError::Immutable("note owner").into());
        }
        note.validate()?;
        note.created_at = stored.created_at;
        note.updated_at = now;
        self.update(&note)?;
        Ok(note)
    }

    pub fn remove_note(&mut self, id: &NoteId) -> Result<(), Failure<R>> {
        self.require::<Note>("note", id)?;
        self.delete::<Note>(id)
    }

    pub fn update_universe_profile(
        &mut self,
        mut profile: UniverseProfile,
        now: Timestamp,
    ) -> Result<UniverseProfile, Failure<R>> {
        profile.updated_at = Some(now);
        if self.find::<UniverseProfile>(&())?.is_some() {
            self.update(&profile)?;
        } else {
            self.insert(&profile)?;
        }
        Ok(profile)
    }
}
