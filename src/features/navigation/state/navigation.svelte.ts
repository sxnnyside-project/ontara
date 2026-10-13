import type { EntityId } from '@/shared/model';

/** Longest trail kept for going back. */
const MAX_TRAIL = 100;

/** The ways of looking at the universe. The same entity can be followed through any of them. */
export type ViewKey =
  | 'universe'
  | 'structure'
  | 'graph'
  | 'matrix'
  | 'timeline'
  | 'consistency'
  | 'library'
  | 'configuration';

/** Where the author is: which view, looking at which entity. */
interface Place {
  entity: EntityId | null;
  view: ViewKey;
}

/**
 * Which entity the author is looking at and through which view, and the trail of places they came
 * through, so any link can be followed and undone. The entity is shared by every view: moving
 * from the graph to the timeline keeps looking at the same entity.
 */
class Navigation {
  current = $state<EntityId | null>(null);
  view = $state<ViewKey>('universe');
  private trail = $state<Place[]>([]);
  private position = $state(-1);

  get canGoBack(): boolean {
    return this.position > 0;
  }

  get canGoForward(): boolean {
    return this.position >= 0 && this.position < this.trail.length - 1;
  }

  /**
   * Goes to an entity, in the current view unless another is given. Going where the author
   * already is changes nothing.
   */
  open(id: EntityId, view: ViewKey = this.view): void {
    this.go({ entity: id, view });
  }

  /** Looks at the same entity through another view. */
  show(view: ViewKey): void {
    this.go({ entity: this.current, view });
  }

  back(): void {
    if (!this.canGoBack) return;
    this.position -= 1;
    this.land();
  }

  forward(): void {
    if (!this.canGoForward) return;
    this.position += 1;
    this.land();
  }

  /** Forgets an entity that no longer exists, so back and forward never land on it. */
  forget(id: EntityId): void {
    const here = this.trail[this.position];
    this.trail = this.trail.filter((place) => place.entity !== id);
    const index = here === undefined ? -1 : this.trail.lastIndexOf(here);
    if (this.current === id) {
      this.position = Math.min(this.position, this.trail.length - 1);
      this.land();
    } else {
      this.position = index;
    }
  }

  clear(): void {
    this.current = null;
    this.view = 'universe';
    this.trail = [];
    this.position = -1;
  }

  private go(place: Place): void {
    if (this.current === place.entity && this.view === place.view) return;
    const kept = this.trail.slice(0, this.position + 1);
    kept.push(place);
    this.trail = kept.slice(-MAX_TRAIL);
    this.position = this.trail.length - 1;
    this.current = place.entity;
    this.view = place.view;
  }

  private land(): void {
    const place = this.trail[this.position];
    this.current = place?.entity ?? null;
    if (place) this.view = place.view;
  }
}

export const navigation = new Navigation();
