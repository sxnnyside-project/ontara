/** Whether a media type is an image the interface can show as a picture. */
export function isPreviewableImage(mediaType: string | null): boolean {
  return (
    mediaType !== null &&
    ['image/png', 'image/jpeg', 'image/gif', 'image/webp', 'image/svg+xml'].includes(mediaType)
  );
}

/** A file size the way a person reads it. */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB'];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(value < 10 ? 1 : 0)} ${units[unit]}`;
}
