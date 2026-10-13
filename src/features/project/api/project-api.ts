import { call } from '@/shared/backend';
import type { Timestamp } from '@/shared/model';

/** Mirrors `ProjectSummary` in `src-tauri/src/project.rs`. */
export interface ProjectSummary {
  id: string;
  name: string;
  /** Absolute path of the project folder. */
  path: string;
  created_at: Timestamp;
  universe: { summary: string; description: string; updated_at: Timestamp | null };
}

export function createProject(path: string, name: string): Promise<ProjectSummary> {
  return call('create_project', { path, name });
}

export function openProject(path: string, allowMigration: boolean): Promise<ProjectSummary> {
  return call('open_project', { path, allowMigration });
}

export function closeProject(): Promise<void> {
  return call('close_project');
}

export function currentProject(): Promise<ProjectSummary | null> {
  return call('current_project');
}

export function renameProject(name: string): Promise<ProjectSummary> {
  return call('rename_project', { name });
}

export function updateUniverse(summary: string, description: string): Promise<ProjectSummary> {
  return call('update_universe', { summary, description });
}

/** Writes a verified backup and returns where it was written. */
export function createProjectBackup(): Promise<string> {
  return call('create_project_backup');
}
