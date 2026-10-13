// @unocss-include
// The icon classes below are written out in full so UnoCSS can find and generate them.

/** The icon of a type that has no better one. */
export const DEFAULT_ENTITY_ICON = 'i-mingcute:box-line';

/** Icon of the nearest ancestor (or the type itself) with an entry. */
const ICONS: Record<string, string> = {
  being: 'i-mingcute:user-3-line',
  character: 'i-mingcute:user-3-line',
  deity: 'i-mingcute:sun-line',
  species: 'i-mingcute:paw-line',
  faction: 'i-mingcute:group-line',
  culture: 'i-mingcute:group-2-line',
  language: 'i-mingcute:translate-2-line',
  religion: 'i-mingcute:candle-line',
  item: 'i-mingcute:diamond-2-line',
  place: 'i-mingcute:map-pin-line',
  world: 'i-mingcute:earth-2-line',
  continent: 'i-mingcute:earth-line',
  region: 'i-mingcute:map-line',
  polity: 'i-mingcute:flag-3-line',
  settlement: 'i-mingcute:building-2-line',
  stronghold: 'i-mingcute:safe-shield-2-line',
  event: 'i-mingcute:calendar-line',
  series: 'i-mingcute:bookmarks-line',
  volume: 'i-mingcute:book-2-line',
  part: 'i-mingcute:book-5-line',
  chapter: 'i-mingcute:file-line',
  scene: 'i-mingcute:movie-line',
  asset: 'i-mingcute:attachment-line',
};

/**
 * The icon for an entity type, given its key and its ancestors' keys nearest first (as the
 * backend sends them), so a project's own type inherits the icon of what it specializes.
 */
export function entityIcon(typeChain: readonly string[]): string {
  for (const key of typeChain) {
    const icon = ICONS[key];
    if (icon) return icon;
  }
  return DEFAULT_ENTITY_ICON;
}
