import { expect, test } from 'bun:test';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { historyTexts } from '@/shared/i18n/texts/history';

const BACKEND = join(import.meta.dir, '../../../../src-tauri/src');

function rustFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return rustFiles(path);
    return entry.name.endsWith('.rs') ? [path] : [];
  });
}

test('every edit the backend records has a name in the history', () => {
  const labels = new Set<string>();
  for (const file of rustFiles(BACKEND)) {
    for (const match of readFileSync(file, 'utf8').matchAll(/\.record\(\s*"([a-z_]+\.[a-z_]+)"/g)) {
      const label = match[1];
      if (label) labels.add(label);
    }
  }

  expect(labels.size).toBeGreaterThan(20);
  const known = historyTexts.historyLabels as Record<string, unknown>;
  for (const label of labels) {
    expect(known[label.replace('.', '_')]).toBeDefined();
  }
});
