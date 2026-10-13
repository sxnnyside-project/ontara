import { call } from '@/shared/backend';
import type { EntityRef } from '@/shared/model';

/**
 * An entity whose text matched, and the passage that matched, with each matching word between
 * U+0002 and U+0003. Mirrors `SearchResult` in `src-tauri/src/search.rs`.
 */
export interface SearchResult {
  entity: EntityRef;
  excerpt: string;
}

export function searchEntities(query: string, includeArchived = false): Promise<SearchResult[]> {
  return call('search_entities', { query, includeArchived });
}
