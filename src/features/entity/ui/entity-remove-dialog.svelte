<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t, tDynamic } from '@/shared/i18n';

import type { ReferenceCount } from '@/shared/model';
import { Banner, Button, Dialog } from '@/shared/ui';
import { entityReferences, removeEntity } from '../api/entity-api';

interface Props {
  id: string;
  name: string;
  onclose: () => void;
  onremoved: () => void;
}

const { id, name, onclose, onremoved }: Props = $props();

let references = $state<ReferenceCount[] | null>(null);
let busy = $state(false);
let error = $state<string | null>(null);

$effect(() => {
  entityReferences(id)
    .then((found) => {
      references = found;
    })
    .catch((failure) => {
      error = toCommandError(failure).message;
    });
});

const total = $derived((references ?? []).reduce((sum, entry) => sum + entry.count, 0));

async function remove() {
  busy = true;
  try {
    await removeEntity(id, true);
    onremoved();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function kindLabel(kind: string): string {
  return tDynamic(`references.${kind}`, kind.replaceAll('_', ' '));
}
</script>

<Dialog title={t('entity.removeTitle', { name })} width="sm" {onclose}>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  {#if references === null && !error}
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('common.loading')}</p>
  {:else if total === 0}
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('entity.removeNothing')}</p>
  {:else}
    <Banner kind="warning">{t('entity.removeReferenced', { count: total })}</Banner>
    <ul class="m-0 pl-4 text-xs text-[var(--color-text-secondary)] flex flex-col gap-1">
      {#each references ?? [] as entry (entry.kind)}
        <li><span class="font-mono tabular-nums">{entry.count}</span> {kindLabel(entry.kind)}</li>
      {/each}
    </ul>
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('entity.removeExplain')}</p>
  {/if}
  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="danger" disabled={busy || references === null} onclick={remove}>
      {t('entity.removeConfirm')}
    </Button>
  {/snippet}
</Dialog>
