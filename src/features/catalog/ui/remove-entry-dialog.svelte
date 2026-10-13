<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t, tDynamic } from '@/shared/i18n';

import type { ReferenceCount } from '@/shared/model';
import { Banner, Button, Dialog } from '@/shared/ui';
import { type CatalogEntry, catalogEntryUsage } from '../api/catalog-api';
import { removeEntry, removePropertyAndValues } from '../state/catalog-actions';

interface Props {
  entry: CatalogEntry;
  name: string;
  onclose: () => void;
}

const { entry, name, onclose }: Props = $props();

let usage = $state<ReferenceCount[] | null>(null);
let busy = $state(false);
let error = $state<string | null>(null);

$effect(() => {
  catalogEntryUsage(entry)
    .then((found) => {
      usage = found;
    })
    .catch((failure) => {
      error = toCommandError(failure).message;
    });
});

const total = $derived((usage ?? []).reduce((sum, item) => sum + item.count, 0));
const isProperty = $derived(entry.catalog === 'property_definition');

async function remove(withValues: boolean) {
  busy = true;
  error = null;
  try {
    if (entry.catalog === 'property_definition' && withValues) {
      await removePropertyAndValues(entry.id);
    } else {
      await removeEntry(entry);
    }
    onclose();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}
</script>

<Dialog title={t('config.removeTitle', { name })} width="sm" {onclose}>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  {#if usage === null && !error}
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('common.loading')}</p>
  {:else if total === 0}
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('config.removeUnused')}</p>
  {:else}
    <Banner kind="warning">
      {isProperty ? t('config.removePropertyInUse', { count: total }) : t('config.removeInUse', { count: total })}
    </Banner>
    <ul class="m-0 pl-4 text-xs text-[var(--color-text-secondary)] flex flex-col gap-1">
      {#each usage ?? [] as item (item.kind)}
        <li>
          <span class="font-mono tabular-nums">{item.count}</span>
          {tDynamic(`references.${item.kind}`, item.kind.replaceAll('_', ' '))}
        </li>
      {/each}
    </ul>
  {/if}
  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    {#if isProperty && total > 0}
      <Button variant="danger" disabled={busy} onclick={() => remove(true)}>
        {t('config.removeWithValues')}
      </Button>
    {:else}
      <Button variant="danger" disabled={busy || usage === null || total > 0} onclick={() => remove(false)}>
        {t('common.remove')}
      </Button>
    {/if}
  {/snippet}
</Dialog>
