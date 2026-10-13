<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { EntityRef } from '@/shared/model';
import { describeTemporal } from '@/shared/model';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Button, CanonBadge, ConfirmDialog, EmptyState, entityIcon } from '@/shared/ui';
import {
  listRelationships,
  removeRelationship,
  type RelationshipView,
} from '../api/relationship-api';
import RelationshipDialog from './relationship-dialog.svelte';

interface Props {
  entity: EntityRef;
  onopen: (id: string) => void;
}

const { entity, onopen }: Props = $props();

let views = $state<RelationshipView[]>([]);
let adding = $state(false);
let editing = $state<string | null>(null);
let removing = $state<RelationshipView | null>(null);

$effect(() => {
  void revision.value;
  const id = entity.id;
  let cancelled = false;
  listRelationships(id)
    .then((found) => {
      if (!cancelled) views = found;
    })
    .catch((failure) => {
      if (!cancelled) notices.error(toCommandError(failure).message);
    });
  return () => {
    cancelled = true;
  };
});

function otherEnd(view: RelationshipView): { ref: EntityRef; outgoing: boolean } {
  const outgoing = view.relationship.source === entity.id;
  return { ref: outgoing ? view.target : view.source, outgoing };
}

async function remove() {
  if (!removing) return;
  try {
    await removeRelationship(removing.relationship.id);
    removing = null;
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}
</script>

<div class="flex flex-col gap-4 max-w-3xl">
  <div>
    <Button icon="i-mingcute:add-line" onclick={() => (adding = true)}>{t('relationship.add')}</Button>
  </div>

  {#each views as view (view.relationship.id)}
    {@const end = otherEnd(view)}
    <article class="rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-3 flex items-start gap-3">
      <span class="i-mingcute:git-branch-line w-4 h-4 mt-0.5 text-[var(--color-crown-red-text)] shrink-0"></span>
      <div class="flex-1 min-w-0 flex flex-col gap-1">
        <div class="flex items-center gap-1.5 flex-wrap text-xs">
          <span class="font-mono text-[var(--color-text-secondary)]">
            {catalog.relationshipLabel(view.relationship.relationship_type, end.outgoing)}
          </span>
          <button
            type="button"
            onclick={() => onopen(end.ref.id)}
            class="inline-flex items-center gap-1 bg-transparent border-0 p-0 cursor-pointer text-[var(--color-text-primary)] font-medium hover:underline"
          >
            <span class="{entityIcon(catalog.chain(end.ref.entity_type))} w-3.5 h-3.5 text-[var(--color-text-muted)]"></span>
            {end.ref.name}
          </button>
          {#if view.relationship.qualifier}
            <span class="text-[var(--color-text-muted)]">· {view.relationship.qualifier}</span>
          {/if}
        </div>
        <div class="flex items-center gap-2 flex-wrap text-[11px] font-mono text-[var(--color-text-muted)]">
          <span>{describeTemporal(view.relationship.period)}</span>
          {#if view.relationship.lifecycle === 'historical'}
            <span class="px-1 rounded bg-[var(--color-surface-elevated)]">{t('relationship.historical')}</span>
          {/if}
          {#if view.context}
            <span>· {t('relationship.inContext', { name: view.context.name })}</span>
          {/if}
        </div>
        {#if view.relationship.note}
          <p class="m-0 text-xs text-[var(--color-text-secondary)] select-text">{view.relationship.note}</p>
        {/if}
      </div>
      {#if view.relationship.canon_state !== 'established'}
        <CanonBadge
          state={view.relationship.canon_state}
          label={catalog.canonLabel(view.relationship.canon_state)}
        />
      {/if}
      <Button size="sm" variant="ghost" icon="i-mingcute:edit-line" title={t('common.edit')} onclick={() => (editing = view.relationship.id)} />
      <Button size="sm" variant="ghost" icon="i-mingcute:delete-2-line" title={t('common.remove')} onclick={() => (removing = view)} />
    </article>
  {:else}
    <EmptyState
      icon="i-mingcute:git-branch-line"
      title={t('relationship.noneTitle')}
      description={t('relationship.noneHint')}
    />
  {/each}
</div>

{#if adding}
  <RelationshipDialog {entity} onclose={() => (adding = false)} />
{/if}
{#if editing}
  <RelationshipDialog {entity} relationshipId={editing} onclose={() => (editing = null)} />
{/if}
{#if removing}
  <ConfirmDialog
    title={t('relationship.removeTitle')}
    confirmLabel={t('common.remove')}
    danger
    onconfirm={remove}
    oncancel={() => (removing = null)}
  >
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('relationship.removeBody')}</p>
  </ConfirmDialog>
{/if}
