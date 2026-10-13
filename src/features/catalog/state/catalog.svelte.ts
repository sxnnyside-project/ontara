import type { TypeKey } from '@/shared/model';
import { getLocale, tDefault } from '@/shared/i18n';
import { type Catalog, getCatalog } from '../api/catalog-api';
import { emptyCatalog, labelOf, mapCatalogLabels, typeChain } from './catalog-model';

/** The configuration of the open project, loaded once and replaced after every change to it. */
class CatalogState {
  /** What the project stores: built-in labels are the English defaults until renamed. */
  stored = $state<Catalog>(emptyCatalog());
  loaded = $state(false);

  /** What the interface shows: the defaults in the interface language, renames as written. */
  data = $derived.by(() => {
    void getLocale();
    return mapCatalogLabels(this.stored, tDefault);
  });

  async load(): Promise<void> {
    this.stored = await getCatalog();
    this.loaded = true;
  }

  /** Takes a catalog the backend returned after a configuration change. */
  replace(catalog: Catalog): void {
    this.stored = catalog;
    this.loaded = true;
  }

  clear(): void {
    this.stored = emptyCatalog();
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
