import { call } from '@/shared/backend';
import type { EntityId, EntityRef, Temporal, TypeKey } from '@/shared/model';

export type EventScale = 'personal' | 'local' | 'regional' | 'national' | 'world';

export type CausalityKind = 'direct' | 'contributing' | 'enabling' | 'prevented' | 'triggered';

/** What to show of the chronology. Mirrors `TimelineQuery`. */
export interface TimelineQuery {
  involving?: EntityId | null;
  from_day?: number | null;
  to_day?: number | null;
}

/** An event on the timeline. Mirrors `TimelineEntryView`. */
export interface TimelineEntry {
  entity: EntityRef;
  period: Temporal;
  start_day: number | null;
  end_day: number | null;
  location: EntityRef | null;
  scale: EventScale | null;
  participants: number;
  presented_in: EntityRef[];
  narrative_rank: number | null;
  out_of_order: boolean;
}

export interface TimelineLink {
  cause: EntityId;
  effect: EntityId;
  kind: CausalityKind;
}

/** Mirrors `TimelineView` in `src-tauri/src/timeline.rs`. */
export interface Timeline {
  dated: TimelineEntry[];
  undated: TimelineEntry[];
  links: TimelineLink[];
}

export interface EventParticipant {
  entity: EntityId;
  role: string;
  side: string | null;
}

/** What makes an entity an event. Mirrors `EventDetail` in `ontara-core`. */
export interface EventDetail {
  entity: EntityId;
  period: Temporal;
  location: EntityId | null;
  scale: EventScale | null;
  consequences: string;
  participants: EventParticipant[];
}

export type ParticipantView = EventParticipant & { entity: EntityRef };

export interface CausalityView {
  id: string;
  other: EntityRef;
  kind: CausalityKind;
  note: string | null;
}

/** Mirrors `EventView`. */
export interface EventView {
  detail: EventDetail | null;
  location: EntityRef | null;
  participants: Array<Omit<EventParticipant, 'entity'> & { entity: EntityRef }>;
  causes: CausalityView[];
  effects: CausalityView[];
}

export function getTimeline(query: TimelineQuery = {}): Promise<Timeline> {
  return call('get_timeline', { query });
}

export function getEvent(entity: EntityId): Promise<EventView> {
  return call('get_event', { entity });
}

export function saveEventDetail(detail: EventDetail): Promise<EventDetail> {
  return call('save_event_detail', { detail });
}

export function addEventCausality(causality: {
  cause: EntityId;
  effect: EntityId;
  kind: CausalityKind;
  note: string | null;
}): Promise<unknown> {
  return call('add_event_causality', { causality });
}

export function removeEventCausality(id: string): Promise<void> {
  return call('remove_event_causality', { id });
}

export function eventTypeKeys(): Promise<TypeKey[]> {
  return call('event_type_keys');
}
