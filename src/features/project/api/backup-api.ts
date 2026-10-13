import { call } from '@/shared/backend';
import type { ProjectSummary } from './project-api';

/** One backup of the project. Mirrors `BackupSummary` in `src-tauri/src/backup.rs`. */
export interface BackupSummary {
  name: string;
  /** What made it: `backup` (the author), `auto`, `pre-migration-v1` or `pre-restore`. */
  kind: string;
  /** Milliseconds since the Unix epoch. */
  created_at: number;
  size_bytes: number;
}

export function listBackups(): Promise<BackupSummary[]> {
  return call('list_backups');
}

/** Replaces the project with one of its backups and returns the project as it now is. */
export function restoreProjectBackup(name: string): Promise<ProjectSummary> {
  return call('restore_project_backup', { name });
}
