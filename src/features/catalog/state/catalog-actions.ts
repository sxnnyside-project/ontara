import { revision } from '@/shared/revision';
import {
  type Catalog,
  type CatalogEntry,
  type CatalogRecord,
  defineCatalogEntry,
  removeCatalogEntry,
  removePropertyWithValues,
  updateCatalogEntry,
} from '../api/catalog-api';
import { catalog } from './catalog.svelte';

/** Takes the catalog the backend answered with and tells the interface its data changed. */
function adopt(next: Catalog): void {
  catalog.replace(next);
  revision.bump();
}

export async function defineEntry(record: CatalogRecord): Promise<void> {
  adopt(await defineCatalogEntry(record));
}

export async function updateEntry(record: CatalogRecord): Promise<void> {
  adopt(await updateCatalogEntry(record));
}

export async function removeEntry(entry: CatalogEntry): Promise<void> {
  adopt(await removeCatalogEntry(entry));
}

export async function removePropertyAndValues(id: string): Promise<void> {
  adopt(await removePropertyWithValues(id));
}
