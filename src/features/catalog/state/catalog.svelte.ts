import type { TypeKey } from '@/shared/model';
import { type Catalog, getCatalog } from '../api/catalog-api';
import { emptyCatalog, labelOf, typeChain } from './catalog-model';

/** The configuration of the open project, loaded once and replaced after every change to it. */
class CatalogState {
  data = $state<Catalog>(emptyCatalog());
  loaded = $state(false);

  async load(): Promise<void> {
    this.data = await getCatalog();
    this.loaded = true;
  }

  /** Takes a catalog the backend returned after a configuration change. */
  replace(catalog: Catalog): void {
    this.data = catalog;
    this.loaded = true;
  }

  clear(): void {
    this.data = emptyCatalog();
    this.loaded = false;
  }

  typeLabel(key: TypeKey): string {
    return labelOf(this.data.entity_types, key);
  }

  canonLabel(key: TypeKey): string {
    return labelOf(this.data.canon_states, key);
  }

  levelLabel(key: TypeKey): string {
    return labelOf(this.data.editorial_levels, key);
  }

  /** The label of a relationship type, read from the side the entity stands on. */
  relationshipLabel(key: TypeKey, outgoing: boolean): string {
    const found = this.data.relationship_types.find((candidate) => candidate.key === key);
    if (!found) return key;
    return outgoing ? found.label : (found.inverse_label ?? found.label);
  }

  /** The type and its ancestors, nearest first. */
  chain(key: TypeKey): TypeKey[] {
    return typeChain(this.data.entity_types, key);
  }
}

export const catalog = new CatalogState();
