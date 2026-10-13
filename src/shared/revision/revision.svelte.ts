/**
 * A counter that moves every time the project's data changes.
 *
 * Whatever shows data from the backend reads `revision.value` inside an effect, so it loads
 * again after any edit made anywhere in the interface.
 */
class Revision {
  value = $state(0);

  /** Announces that the project's data changed. */
  bump(): void {
    this.value += 1;
  }
}

export const revision = new Revision();
