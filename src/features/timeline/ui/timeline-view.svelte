<script lang="ts">
import { catalog } from '@/features/catalog';
import { EntityCreateDialog } from '@/features/entity';
import { navigation, ShowIn } from '@/features/navigation';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import {
  Banner,
  Button,
  CheckField,
  EmptyState,
  InlineSelect,
  SearchInput,
  Segmented,
  ViewFrame,
} from '@/shared/ui';
import { type Timeline, type TimelineEntry, getTimeline } from '../api/timeline-api';
import {
  type TimelineOrder,
  causesOf,
  countOutOfOrder,
  inOrder,
  spanLabel,
} from '../state/timeline-model';
import EventPanel from './event-panel.svelte';

interface Props {
  /** Follows an entity: shows its page. */
  onopen: (id: string) => void;
}

const { onopen }: Props = $props();

let order = $state<TimelineOrder>('chronological');
let onlyFocus = $state(false);
let from = $state('');
let to = $state('');
let timeline = $state<Timeline | null>(null);
let failure = $state<string | null>(null);
let selected = $state<string | null>(null);
let creating = $state(false);

const parse = (raw: string) => {
  const value = Number.parseInt(raw, 10);
  return Number.isFinite(value) ? value : null;
};

$effect(() => {
  void revision.value;
  let cancelled = false;
  getTimeline({
    involving: onlyFocus ? navigation.current : null,
    from_day: parse(from),
    to_day: parse(to),
  })
    .then((found) => {
      if (cancelled) return;
      timeline = found;
      failure = null;
    })
    .catch((error) => {
      if (!cancelled) failure = toCommandError(error).message;
    });
  return () => {
    cancelled = true;
  };
});

const entries = $derived(timeline ? inOrder(timeline, order) : []);
const names = $derived(
  new Map(
    [...(timeline?.dated ?? []), ...(timeline?.undated ?? [])].map((entry): [string, string] => [
      entry.entity.id,
      entry.entity.name,
    ]),
  ),
);
const current = $derived(
  [...(timeline?.dated ?? []), ...(timeline?.undated ?? [])].find(
    (entry) => entry.entity.id === selected,
  ) ?? null,
);

function choose(entry: TimelineEntry) {
  selected = entry.entity.id;
  navigation.open(entry.entity.id);
}
</script>

<ViewFrame
  icon="i-mingcute:history-line"
  title={t('timeline.title')}
  description={t('timeline.subtitle')}
  asideWidth={384}
  asideOpen={current !== null}
  asideTitle={t('timeline.event')}
  onasideclose={() => (selected = null)}
  flush
>
  {#snippet actions()}
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => (creating = true)}>
      {t('timeline.newEvent')}
    </Button>
  {/snippet}
  {#snippet toolbar()}
    <Segmented
      bind:value={order as string}
      label={t('timeline.order')}
      options={[
        { value: 'chronological', label: t('timeline.chronological'), icon: 'i-mingcute:history-line' },
        { value: 'narrative', label: t('timeline.narrative'), icon: 'i-mingcute:book-2-line' },
      ]}
    />
    <label class="inline-flex items-center gap-1.5 text-[11px] font-mono text-[var(--color-text-muted)]">
      {t('timeline.from')}
      <input type="number" bind:value={from} class="h-7 w-20 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-1.5 text-xs text-[var(--color-text-primary)] outline-none focus:border-[var(--color-border-focus)]" />
    </label>
    <label class="inline-flex items-center gap-1.5 text-[11px] font-mono text-[var(--color-text-muted)]">
      {t('timeline.to')}
      <input type="number" bind:value={to} class="h-7 w-20 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-1.5 text-xs text-[var(--color-text-primary)] outline-none focus:border-[var(--color-border-focus)]" />
    </label>
    {#if navigation.current}
      <CheckField label={t('timeline.onlyFocus')} bind:checked={onlyFocus} />
    {/if}
    <span class="flex-1"></span>
    {#if timeline}
      <span class="text-[11px] font-mono text-[var(--color-text-muted)]">
        {t('timeline.summary', {
          dated: timeline.dated.length,
          undated: timeline.undated.length,
          jumps: countOutOfOrder(timeline),
        })}
      </span>
    {/if}
    <ShowIn views={['universe', 'graph', 'matrix']} />
  {/snippet}
  {#snippet aside()}
    {#if current}
      <div class="p-4">
        <h3 class="m-0 mb-3 text-base font-serif">{current.entity.name}</h3>
        {#key current.entity.id}
          <EventPanel entity={current.entity} {onopen} />
        {/key}
      </div>
    {/if}
  {/snippet}

  <div class="p-6">
    {#if failure}<Banner kind="error">{failure}</Banner>{/if}
    {#if timeline && entries.length === 0 && timeline.undated.length === 0}
      <EmptyState
        icon="i-mingcute:history-line"
        title={t('timeline.emptyTitle')}
        description={t('timeline.emptyHint')}
        steps={[
          { icon: 'i-mingcute:calendar-line', title: t('timeline.stepCreate'), text: t('timeline.stepCreateText') },
          { icon: 'i-mingcute:time-line', title: t('timeline.stepDate'), text: t('timeline.stepDateText') },
          { icon: 'i-mingcute:book-2-line', title: t('timeline.stepTell'), text: t('timeline.stepTellText') },
        ]}
      >
        <Button variant="primary" icon="i-mingcute:add-line" onclick={() => (creating = true)}>
          {t('timeline.newEvent')}
        </Button>
      </EmptyState>
    {/if}
    <ol class="m-0 p-0 list-none relative flex flex-col gap-3 border-l border-[var(--color-border-strong)] ml-20">
      {#each entries as entry (entry.entity.id)}
        {@const causes = timeline ? causesOf(entry, timeline.links, names) : []}
        <li class="relative pl-6">
          <span class="absolute -left-[5px] top-3 w-2.5 h-2.5 rounded-full border-2 border-[var(--color-meridian-blue)] bg-[var(--color-surface-base)]
            {selected === entry.entity.id ? '!bg-[var(--color-crown-red)] !border-[var(--color-crown-red)]' : ''}"></span>
          <span class="absolute -left-20 top-2 w-16 text-right font-mono text-[11px] tabular-nums text-[var(--color-text-secondary)]">
            {spanLabel(entry)}
          </span>
          <button
            type="button"
            onclick={() => choose(entry)}
            class="w-full text-left rounded-lg border px-3 py-2 cursor-pointer bg-[var(--color-surface-panel)] transition-colors
            {selected === entry.entity.id ? 'border-[var(--color-crown-red)]' : 'border-[var(--color-border-subtle)] hover:border-[var(--color-border-strong)]'}"
          >
            <span class="flex items-center gap-2">
              <span class="text-sm font-serif font-medium">{entry.entity.name}</span>
              {#if entry.out_of_order}
                <span class="inline-flex items-center gap-1 text-[10px] font-mono text-[var(--color-canon-speculative)]" title={t('timeline.outOfOrderHint')}>
                  <span class="i-mingcute:history-line w-3 h-3"></span>
                  {t('timeline.outOfOrder')}
                </span>
              {/if}
            </span>
            <span class="flex items-center gap-3 flex-wrap text-[11px] font-mono text-[var(--color-text-muted)]">
              {#if entry.location}<span>{entry.location.name}</span>{/if}
              {#if entry.participants > 0}<span>{t('timeline.participants', { count: entry.participants })}</span>{/if}
              {#if entry.presented_in.length > 0}
                <span>{t('timeline.toldIn', { sections: entry.presented_in.map((section) => section.name).join(', ') })}</span>
              {:else}
                <span>{t('timeline.notTold')}</span>
              {/if}
            </span>
            {#if causes.length > 0}
              <span class="block text-[11px] font-mono text-[var(--color-text-muted)]">
                {t('timeline.causedBy', { events: causes.join(', ') })}
              </span>
            {/if}
          </button>
        </li>
      {/each}
    </ol>

    {#if timeline && timeline.undated.length > 0}
      <h3 class="mt-8 mb-2 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
        {t('timeline.undated')} ({timeline.undated.length})
      </h3>
      <div class="flex flex-col gap-1.5">
        {#each timeline.undated as entry (entry.entity.id)}
          <button
            type="button"
            onclick={() => choose(entry)}
            class="text-left rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] px-3 py-1.5 text-xs cursor-pointer hover:border-[var(--color-border-strong)]"
          >
            {entry.entity.name}
          </button>
        {/each}
      </div>
    {/if}
  </div>
</ViewFrame>

{#if creating}
  <EntityCreateDialog
    initialType={catalog.data.entity_types.find((entry) => entry.role === 'event' && !entry.is_abstract)?.key}
    onclose={() => (creating = false)}
    oncreated={(entity) => {
      creating = false;
      selected = entity.id;
      navigation.open(entity.id);
    }}
  />
{/if}
