import { expect, test } from 'bun:test';
import { formatBytes, isPreviewableImage } from '@/features/asset/state/media';

test('only common image formats are previewed', () => {
  expect(isPreviewableImage('image/png')).toBe(true);
  expect(isPreviewableImage('application/pdf')).toBe(false);
  expect(isPreviewableImage(null)).toBe(false);
});

test('sizes read the way a person says them', () => {
  expect(formatBytes(512)).toBe('512 B');
  expect(formatBytes(1536)).toBe('1.5 KB');
  expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MB');
  expect(formatBytes(120 * 1024 * 1024)).toBe('120 MB');
});
