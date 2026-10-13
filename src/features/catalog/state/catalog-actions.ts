import { storedDefault } from '@/shared/i18n';
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
import { storeCatalogLabels } from './catalog-model';

/** Takes the catalog the backend answered with and tells the interface its data changed. */
function adopt(next: Catalog): void {
  catalog.replace(next);
  revision.bump();
}

export async function defineEntry(record: CatalogRecord): Promise<void> {
  adopt(await defineCatalogEntry(storeCatalogLabels(record, storedDefault)));
}

export async function updateEntry(record: CatalogRecord): Promise<void> {
  adopt(await updateCatalogEntry(storeCatalogLabels(record, storedDefault)));
}

export async function removeEntry(entry: CatalogEntry): Promise<void> {
  adopt(await removeCatalogEntry(entry));
}

export async function removePropertyAndValues(id: string): Promise<void> {
  adopt(await removePropertyWithValues(id));
}
