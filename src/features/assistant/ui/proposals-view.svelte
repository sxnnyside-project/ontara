<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { getLocale, t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Banner, Button, CheckField, EmptyState, ViewFrame } from '@/shared/ui';
import {
  type ChangeView,
  decideProposal,
  discardProposal,
  type Proposal,
} from '../api/proposal-api';
import { danglingPositions, pendingChanges, targetName } from '../state/proposal-model';
import { proposals } from '../state/proposals.svelte';

interface Props {
  /** Follows an entity: shows its page. */
  onopen: (id: string) => void;
  /** Opens the assistant settings. */
  onsettings: () => void;
}

const { onopen, onsettings }: Props = $props();

let failure = $state<string | null>(null);
/** The changes the author took out of the selection, by proposal. Everything pending starts in. */
let excluded = $state<Record<string, number[]>>({});
let busy = $state(false);

$effect(() => {
  void revision.value;
  proposals
    .refresh()
    .then(() => {
      failure = null;
    })
    .catch((error) => {
      failure = toCommandError(error).message;
    });
});

// A client on this computer may have suggested something since the view opened.
$effect(() => {
  const timer = setInterval(() => {
    proposals.refresh().catch(() => {});
  }, 8000);
  return () => clearInterval(timer);
});

function isChosen(proposal: Proposal, change: ChangeView): boolean {
  return !(excluded[proposal.id] ?? []).includes(change.position);
}

function toggle(proposal: Proposal, change: ChangeView) {
  const current = excluded[proposal.id] ?? [];
  excluded = {
    ...excluded,
    [proposal.id]: current.includes(change.position)
      ? current.filter((position) => position !== change.position)
      : [...current, change.position],
  };
}

function chosen(proposal: Proposal): number[] {
  return pendingChanges(proposal)
    .filter((change) => isChosen(proposal, change))
    .map((change) => change.position);
}

async function decide(proposal: Proposal, approve: boolean) {
  const positions = chosen(proposal);
  busy = true;
  try {
    const result = await decideProposal(
      proposal.id,
      approve ? positions : [],
      approve ? [] : positions,
    );
    if (approve) {
      const applied = result.changes.filter(
        (change) => positions.includes(change.position) && change.status === 'applied',
      ).length;
      notices.success(t('proposals.appliedMessage', { count: applied }));
    }
    revision.bump();
  } catch (error) {
    notices.error(toCommandError(error).message);
  } finally {
    busy = false;
  }
}

async function discard(proposal: Proposal) {
  try {
    await discardProposal(proposal.id);
    await proposals.refresh();
  } catch (error) {
    notices.error(toCommandError(error).message);
  }
}

function when(millis: number): string {
  return new Date(millis).toLocaleString(getLocale(), { dateStyle: 'medium', timeStyle: 'short' });
}

function describe(proposal: Proposal, change: ChangeView): string {
  const missing = t('proposals.unknownEntity');
  const body = change.change;
  switch (body.op) {
    case 'create_entity':
      return t('proposals.create', { type: catalog.typeLabel(body.entity_type), name: body.name });
    case 'update_entity':
      return t('proposals.update', {
        name: change.before?.name ?? proposal.entities[body.entity]?.name ?? missing,
      });
    case 'add_relationship':
      return t('proposals.relate', {
        source: targetName(body.source, proposal.entities, missing),
        relationship: catalog.relationshipLabel(body.relationship_type, true).toLowerCase(),
        target: targetName(body.target, proposal.entities, missing),
      });
    case 'add_note':
      return t('proposals.note', {
        entity: targetName(body.entity, proposal.entities, missing),
        title: body.title,
      });
  }
}

/** The entity a change is about, when it already exists, so the author can look at it. */
function subject(change: ChangeView): string | null {
  const body = change.change;
  if (body.op === 'update_entity') return body.entity;
  if (body.op === 'add_note' && body.entity.kind === 'existing') return body.entity.id;
  if (body.op === 'add_relationship' && body.source.kind === 'existing') return body.source.id;
  return null;
}
</script>

<ViewFrame
  icon="i-mingcute:sparkles-line"
  title={t('proposals.title')}
  description={t('proposals.subtitle')}
>
  {#snippet actions()}
    <Button onclick={onsettings}>{t('assistant.open')}</Button>
  {/snippet}

  {#if failure}<Banner kind="error">{failure}</Banner>{/if}

  {#if proposals.items.length === 0}
    <EmptyState
      icon="i-mingcute:sparkles-line"
      title={t('proposals.none')}
      description={t('proposals.noneHint')}
    />
  {:else}
    <div class="flex flex-col gap-5 max-w-4xl">
      {#each proposals.items as proposal (proposal.id)}
        {@const pending = pendingChanges(proposal)}
        {@const picked = new Set(chosen(proposal))}
        <article class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] flex flex-col">
          <header class="px-4 py-3 flex items-start gap-3 border-b border-[var(--color-border-subtle)]">
            <div class="flex-1 min-w-0">
              <h3 class="m-0 text-sm font-serif font-medium">{proposal.title}</h3>
              <p class="m-0 mt-0.5 text-[11px] font-mono text-[var(--color-text-muted)]">
                {t(`proposals.source_${proposal.source}` as 'proposals.source_ai')} · {proposal.author} · {when(proposal.created_at)}
              </p>
              {#if proposal.rationale}
                <p class="m-0 mt-2 text-xs text-[var(--color-text-secondary)] whitespace-pre-wrap select-text">{proposal.rationale}</p>
              {/if}
            </div>
            <Button size="sm" variant="ghost" onclick={() => discard(proposal)}>{t('proposals.discard')}</Button>
          </header>

          <ul class="m-0 p-0 list-none flex flex-col divide-y divide-[var(--color-border-subtle)]">
            {#each proposal.changes as change (change.position)}
              <li class="px-4 py-2.5 flex items-start gap-3">
                {#if change.status === 'pending'}
                  <input
                    type="checkbox"
                    class="mt-1 accent-[var(--color-crown-red)]"
                    checked={isChosen(proposal, change)}
                    onchange={() => toggle(proposal, change)}
                    aria-label={describe(proposal, change)}
                  />
                {:else}
                  <span class="mt-0.5 w-4 h-4 shrink-0 {change.status === 'applied' ? 'i-mingcute:check-circle-line text-[var(--color-canon-established)]' : 'i-mingcute:close-circle-line text-[var(--color-text-muted)]'}" aria-hidden="true"></span>
                {/if}
                <div class="flex-1 min-w-0 flex flex-col gap-1.5">
                  <div class="flex items-baseline gap-2">
                    <span class="text-xs {change.status === 'pending' ? '' : 'text-[var(--color-text-muted)]'}">{describe(proposal, change)}</span>
                    {#if change.status !== 'pending'}
                      <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{t(`proposals.status_${change.status}` as 'proposals.status_applied')}</span>
                    {/if}
                    {#if subject(change)}
                      <button
                        type="button"
                        onclick={() => onopen(subject(change) ?? '')}
                        class="p-0 border-0 bg-transparent text-[10px] font-mono text-[var(--color-crown-red-text)] cursor-pointer hover:underline"
                      >
                        {t('graphView.openEntity')}
                      </button>
                    {/if}
                  </div>
                  {#if change.change.op === 'update_entity'}
                    {#if change.change.summary !== null}
                      {@render difference(t('proposals.summary'), change.before?.summary ?? '', change.change.summary)}
                    {/if}
                    {#if change.change.description !== null}
                      {@render difference(t('proposals.description'), change.before?.description ?? '', change.change.description)}
                    {/if}
                    {#if change.change.add_tags.length > 0}
                      <p class="m-0 text-[11px] text-[var(--color-text-secondary)]">{t('proposals.tags')}: {change.change.add_tags.join(', ')}</p>
                    {/if}
                  {:else if change.change.op === 'create_entity'}
                    {#if change.change.summary}
                      <p class="m-0 text-[11px] text-[var(--color-text-secondary)] select-text">{change.change.summary}</p>
                    {/if}
                    {#if change.change.description}
                      <p class="m-0 text-[11px] text-[var(--color-text-muted)] select-text whitespace-pre-wrap">{change.change.description}</p>
                    {/if}
                    <p class="m-0 text-[10px] font-mono text-[var(--color-text-muted)]">{t('proposals.nonCanon')}</p>
                  {:else if change.change.op === 'add_note'}
                    <p class="m-0 text-[11px] text-[var(--color-text-secondary)] whitespace-pre-wrap select-text">{change.change.body}</p>
                  {/if}
                  {#if change.error}
                    <p class="m-0 text-[11px] text-[var(--color-finding-error)]">{change.error}</p>
                  {/if}
                </div>
              </li>
            {/each}
          </ul>

          {#if pending.length > 0}
            <footer class="px-4 py-3 border-t border-[var(--color-border-subtle)] flex items-center gap-2">
              {#if danglingPositions(proposal, picked).length > 0}
                <span class="text-[11px] text-[var(--color-finding-warning)] flex-1">{t('proposals.dangling')}</span>
              {:else}
                <span class="text-[11px] font-mono text-[var(--color-text-muted)] flex-1">{t('proposals.selected', { count: picked.size })}</span>
              {/if}
              <Button size="sm" variant="ghost" disabled={busy || picked.size === 0} onclick={() => decide(proposal, false)}>
                {t('proposals.reject')}
              </Button>
              <Button size="sm" variant="primary" disabled={busy || picked.size === 0} onclick={() => decide(proposal, true)}>
                {t('proposals.apply')}
              </Button>
            </footer>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</ViewFrame>

{#snippet difference(label: string, before: string, after: string)}
  <div class="grid grid-cols-2 gap-2 text-[11px]">
    <div class="rounded border border-[var(--color-border-subtle)] p-2">
      <div class="font-mono uppercase text-[10px] text-[var(--color-text-muted)]">{label} · {t('proposals.now')}</div>
      <p class="m-0 mt-1 whitespace-pre-wrap text-[var(--color-text-secondary)] select-text">{before || '—'}</p>
    </div>
    <div class="rounded border border-[var(--color-canon-established)]/40 p-2">
      <div class="font-mono uppercase text-[10px] text-[var(--color-text-muted)]">{label} · {t('proposals.suggested')}</div>
      <p class="m-0 mt-1 whitespace-pre-wrap select-text">{after}</p>
    </div>
  </div>
{/snippet}
