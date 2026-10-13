const STORAGE_KEY = 'ontara.recent-projects';
const MAX_RECENT = 8;

export interface RecentProject {
  name: string;
  path: string;
}

/** Puts a project first, without duplicates, keeping the list short. */
export function remember(
  recent: readonly RecentProject[],
  project: RecentProject,
): RecentProject[] {
  return [project, ...recent.filter((entry) => entry.path !== project.path)].slice(0, MAX_RECENT);
}

export function forget(recent: readonly RecentProject[], path: string): RecentProject[] {
  return recent.filter((entry) => entry.path !== path);
}

function isRecentProject(value: unknown): value is RecentProject {
  return (
    typeof value === 'object' &&
    value !== null &&
    'name' in value &&
    'path' in value &&
    typeof value.name === 'string' &&
    typeof value.path === 'string'
  );
}

/** Reads the list a previous session kept. Anything unreadable is an empty list. */
export function loadRecent(storage: Pick<Storage, 'getItem'> | undefined): RecentProject[] {
  try {
    const raw = storage?.getItem(STORAGE_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? parsed.filter(isRecentProject).slice(0, MAX_RECENT) : [];
  } catch {
    return [];
  }
}

export function saveRecent(
  storage: Pick<Storage, 'setItem'> | undefined,
  recent: readonly RecentProject[],
): void {
  try {
    storage?.setItem(STORAGE_KEY, JSON.stringify(recent));
  } catch {
    // The list is a convenience; a blocked storage must not break opening a project.
  }
}
