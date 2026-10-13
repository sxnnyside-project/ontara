import { call } from '@/shared/backend';
import type { EntityRef } from '@/shared/model';

export type Severity = 'error' | 'warning' | 'info';

/** The checks that exist. Mirrors `Rule` in `ontara-core`. */
export type Rule =
  | 'participant_after_death'
  | 'participant_before_birth'
  | 'parent_not_older'
  | 'effect_before_cause'
  | 'presence_and_absence'
  | 'two_places_at_once'
  | 'canon_on_uncanon'
  | 'required_property_missing'
  | 'viewpoint_absent'
  | 'office_overfilled'
  | 'section_not_placed'
  | 'event_undated'
  | 'no_connections'
  | 'duplicate_name';

/** One finding with the entities it names resolved. Mirrors `FindingView`. */
export interface Finding {
  fingerprint: string;
  rule: Rule;
  code: string;
  severity: Severity;
  /** The entity to open to deal with the finding. */
  subject: EntityRef;
  related: EntityRef[];
  facts: Record<string, string>;
}

/** Mirrors `ReportView` in `src-tauri/src/consistency.rs`. */
export interface Report {
  findings: Finding[];
  counts: { errors: number; warnings: number; infos: number };
  examined: { entities: number; relationships: number; appearances: number; events: number };
}

export function checkConsistency(): Promise<Report> {
  return call('check_consistency');
}
