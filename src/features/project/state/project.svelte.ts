import {
  closeProject,
  createProject,
  currentProject,
  openProject,
  type ProjectSummary,
} from '../api/project-api';
import { forget, loadRecent, type RecentProject, remember, saveRecent } from './recent-projects';

const storage = typeof localStorage === 'undefined' ? undefined : localStorage;

/** The project the author has open. */
class ProjectState {
  current = $state<ProjectSummary | null>(null);
  /** True until the first check for an already open project has finished. */
  starting = $state(true);
  recent = $state<RecentProject[]>(loadRecent(storage));

  /** Picks up the project the backend already has open, as after a window reload. */
  async restore(): Promise<void> {
    try {
      this.current = await currentProject();
    } finally {
      this.starting = false;
    }
  }

  async create(path: string, name: string): Promise<void> {
    this.adopt(await createProject(path, name));
  }

  async open(path: string, allowMigration = false): Promise<void> {
    try {
      this.adopt(await openProject(path, allowMigration));
    } catch (error) {
      if (error instanceof Error && 'code' in error && error.code === 'not_a_project') {
        this.recent = forget(this.recent, path);
        saveRecent(storage, this.recent);
      }
      throw error;
    }
  }

  async close(): Promise<void> {
    await closeProject();
    this.current = null;
  }

  /** Replaces what is shown of the project after it was renamed or described. */
  refresh(summary: ProjectSummary): void {
    this.current = summary;
    this.recent = remember(this.recent, { name: summary.name, path: summary.path });
    saveRecent(storage, this.recent);
  }

  private adopt(summary: ProjectSummary): void {
    this.refresh(summary);
  }
}

export const project = new ProjectState();
