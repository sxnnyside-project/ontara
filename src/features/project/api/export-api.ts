import { call } from '@/shared/backend';

export type ExportFormat = 'markdown' | 'json';

/**
 * The words a Markdown export is written with. Mirrors `ExportWords` in
 * `src-tauri/src/export.rs`.
 */
export interface ExportWords {
  contents: string;
  summary: string;
  description: string;
  tags: string;
  aliases: string;
  properties: string;
  relationships: string;
  notes: string;
  structure: string;
  canon: string;
  types: Record<string, string>;
  relationship_types: Record<string, { label: string; inverse: string | null }>;
  canon_states: Record<string, string>;
  property_names: Record<string, string>;
}

/** The file an export wrote. Mirrors `ExportedFile`. */
export interface ExportedFile {
  name: string;
  /** Absolute path. */
  path: string;
  size_bytes: number;
  entities: number;
}

export function exportUniverse(format: ExportFormat, words: ExportWords): Promise<ExportedFile> {
  return call('export_universe', { format, words });
}
