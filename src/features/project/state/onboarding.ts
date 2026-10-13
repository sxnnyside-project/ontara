import type { UniverseCounts } from '../api/project-api';

/** One thing a new universe does before it is really working. */
export type StepKey = 'entities' | 'relationships' | 'structure' | 'appearances' | 'events';

export interface OnboardingStep {
  key: StepKey;
  done: boolean;
}

/** Enough entities for the rest to have something to connect. */
export const FIRST_ENTITIES = 3;

/** The steps of getting started, in order, with which are done. */
export function onboardingSteps(counts: UniverseCounts): OnboardingStep[] {
  return [
    { key: 'entities', done: counts.entities >= FIRST_ENTITIES },
    { key: 'relationships', done: counts.relationships >= 1 },
    { key: 'structure', done: counts.sections >= 1 },
    { key: 'appearances', done: counts.appearances >= 1 },
    { key: 'events', done: counts.events >= 1 },
  ];
}

const DISMISSED_KEY = 'ontara.onboarding-dismissed';

/** Whether the author closed the getting-started list for this project. */
export function isDismissed(
  storage: Pick<Storage, 'getItem'> | undefined,
  project: string,
): boolean {
  try {
    const raw = storage?.getItem(DISMISSED_KEY);
    if (!raw) return false;
    const list: unknown = JSON.parse(raw);
    return Array.isArray(list) && list.includes(project);
  } catch {
    return false;
  }
}

export function dismiss(
  storage: Pick<Storage, 'getItem' | 'setItem'> | undefined,
  project: string,
): void {
  try {
    const raw = storage?.getItem(DISMISSED_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    const list = Array.isArray(parsed) ? parsed.filter((item) => typeof item === 'string') : [];
    if (!list.includes(project)) list.push(project);
    storage?.setItem(DISMISSED_KEY, JSON.stringify(list));
  } catch {
    // Not being able to remember a dismissal only means the list shows again.
  }
}
