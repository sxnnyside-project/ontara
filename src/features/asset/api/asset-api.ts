import { call } from '@/shared/backend';
import type { AttachmentId, EntityId, EntityRef, Timestamp } from '@/shared/model';

/** The file behind an asset entity. Mirrors `AssetFile` in `ontara-core`. */
export interface AssetFile {
  entity: EntityId;
  relative_path: string;
  media_type: string | null;
  size_bytes: number;
  content_hash: string | null;
  imported_at: Timestamp;
}

/** Mirrors `AssetView` in `src-tauri/src/asset.rs`. */
export interface AssetView {
  entity: EntityRef;
  file: AssetFile;
  missing: boolean;
  attached_to: EntityRef[];
}

export interface ImportedAsset {
  entity: EntityRef;
  file: AssetFile;
  already_imported: boolean;
}

export interface Attachment {
  id: AttachmentId;
  owner: EntityId;
  asset: EntityId;
  purpose: string | null;
  position: number;
  note: string | null;
  created_at: Timestamp;
}

/** An asset attached to an entity. Mirrors `AttachmentView`. */
export interface AttachmentView {
  attachment: Attachment;
  asset: EntityRef;
  file: AssetFile | null;
  missing: boolean;
}

export interface NewAttachment {
  owner: EntityId;
  asset: EntityId;
  purpose: string | null;
  note: string | null;
}

export function importAsset(path: string, name: string | null): Promise<ImportedAsset> {
  return call('import_asset', { path, name });
}

export function listAssets(): Promise<AssetView[]> {
  return call('list_assets');
}

export function listAttachments(entity: EntityId): Promise<AttachmentView[]> {
  return call('list_attachments', { entity });
}

export function attachAsset(attachment: NewAttachment): Promise<Attachment> {
  return call('attach_asset', { attachment });
}

export function detachAsset(id: AttachmentId): Promise<void> {
  return call('detach_asset', { id });
}

/** The bytes of an asset, for previewing it. */
export function readAsset(entity: EntityId): Promise<ArrayBuffer> {
  return call('read_asset', { entity });
}
