<script lang="ts">
import { catalog } from '@/features/catalog';
import { getEditorialTree, type Section } from '@/features/editorial';
import { navigation, ShowIn } from '@/features/navigation';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { EntityRef } from '@/shared/model';
import { typeLabelOf } from '@/shared/model';
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
import {
  type Matrix,
  type MatrixCell,
  getAppearanceMatrix,
  setAppearanceKind,
} from '../api/matrix-api';
import {
  cellKey,
  indexCells,
  kindLook,
  presenceByColumn,
  presenceByRow,
} from '../state/matrix-model';
import AppearanceDialog from './appearance-dialog.svelte';

interface Props {
  /** Follows an entity: shows its page. */
  onopen: (id: string) => void;
}

const { onopen }: Props = $props();

let source = $state<'structure' | 'events'>('structure');
let level = $state('');
let section = $state('');
let entityType = $state('');
let text = $state('');
let includeEmpty = $state(false);
let matrix = $state<Matrix | null>(null);
let sections = $state<Section[]>([]);
let failure = $state<string | null>(null);
let editing = $state<{ entity: EntityRef; context: EntityRef; cell: MatrixCell } | null>(null);

$effect(() => {
  void revision.value;
  getEditorialTree()
    .then((found) => {
      sections = found;
    })
    .catch(() => {
      sections = [];
    });
});

$effect(() => {
  void revision.value;
  let cancelled = false;
  getAppearanceMatrix({
    rows: {
      entity_type: entityType === '' ? null : entityType,
      text: text.trim() === '' ? null : text.trim(),
    },
    columns:
      source === 'events'
        ? { source: 'events' }
        : {
            source: 'structure',
            section: section === '' ? null : section,
            level: level === '' ? null : level,
          },
    include_empty_rows: includeEmpty,
  })
    .then((found) => {
      if (cancelled) return;
      matrix = found;
      failure = null;
    })
    .catch((error) => {
      if (!cancelled) failure = toCommandError(error).message;
    });
  return () => {
    cancelled = true;
  };
});

const cells = $derived(indexCells(matrix?.cells ?? []));
const perRow = $derived(presenceByRow(matrix?.cells ?? []));
const perColumn = $derived(presenceByColumn(matrix?.cells ?? []));
const defaultKind = $derived(
  catalog.data.appearance_kinds.find((entry) => entry.is_presence)?.key ?? 'physical_presence',
);
const kindLabels = $derived(
  new Map(catalog.data.appearance_kinds.map((entry) => [entry.key, entry.label])),
);
const kindPresence = $derived(
  new Map(catalog.data.appearance_kinds.map((entry) => [entry.key, entry.is_presence])),
);

const typeOptions = $derived(
  catalog.data.entity_types
    .filter((entry) => entry.role === 'generic')
    .map((entry) => ({ value: entry.key, label: entry.label })),
);
const levelOptions = $derived(
  catalog.data.editorial_levels.map((entry) => ({ value: entry.key, label: entry.label })),
);
const sectionOptions = $derived(
  sections.map((entry) => ({ value: entry.entity.id, label: entry.entity.name })),
);

async function place(row: EntityRef, context: EntityRef) {
  try {
    await setAppearanceKind(row.id, context.id, defaultKind);
    revision.bump();
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

function describe(found: MatrixCell[]): string {
  return found.map((cell) => kindLabels.get(cell.kind) ?? cell.kind).join(', ');
}
</script>

<ViewFrame
  icon="i-mingcute:grid-line"
  title={t('matrixView.title')}
  description={t('matrixView.subtitle')}
  flush
>
  {#snippet toolbar()}
    <Segmented
      bind:value={source as string}
      label={t('matrixView.columns')}
      options={[
        { value: 'structure', label: t('matrixView.byStructure'), icon: 'i-mingcute:book-2-line' },
        { value: 'events', label: t('matrixView.byEvents'), icon: 'i-mingcute:calendar-line' },
      ]}
    />
    {#if source === 'structure'}
      <InlineSelect label={t('matrixView.level')} bind:value={level} options={levelOptions} placeholder={t('matrixView.innermost')} />
      <InlineSelect label={t('matrixView.section')} bind:value={section} options={sectionOptions} placeholder={t('matrixView.wholeWork')} />
    {/if}
    <span class="w-px h-5 bg-[var(--color-border-subtle)]"></span>
    <InlineSelect label={t('matrixView.rows')} bind:value={entityType} options={typeOptions} placeholder={t('matrixView.everyone')} />
    <div class="w-48"><SearchInput bind:value={text} label={t('common.search')} placeholder={t('common.search')} /></div>
    <CheckField label={t('matrixView.includeEmpty')} bind:checked={includeEmpty} />
    <span class="flex-1"></span>
    <ShowIn views={['universe', 'graph', 'timeline']} />
  {/snippet}

  <div class="p-5">
    {#if failure}
      <Banner kind="error">{failure}</Banner>
    {/if}
    {#if matrix && matrix.columns.length === 0}
      <EmptyState icon="i-mingcute:grid-line" title={t('matrixView.noColumnsTitle')} description={t('matrixView.noColumnsHint')} />
    {:else if matrix && matrix.rows.length === 0}
      <EmptyState icon="i-mingcute:grid-line" title={t('matrixView.noRowsTitle')} description={t('matrixView.noRowsHint')} />
    {:else if matrix}
      <table class="w-full border-separate border-spacing-0 text-xs">
        <thead>
          <tr>
            <th class="sticky left-0 top-0 z-20 bg-[var(--color-surface-base)] text-left px-2 py-2 font-mono uppercase tracking-wider text-[10px] text-[var(--color-text-muted)] border-b border-[var(--color-border-subtle)]">
              {t('matrixView.entityHeader')}
            </th>
            {#each matrix.columns as column (column.id)}
              <th class="sticky top-0 z-10 bg-[var(--color-surface-base)] px-1 py-2 border-b border-[var(--color-border-subtle)] align-bottom">
                <button
                  type="button"
                  onclick={() => onopen(column.id)}
                  title={column.name}
                  class="w-10 h-28 flex items-end justify-center cursor-pointer bg-transparent border-0 p-0 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
                >
                  <span class="block max-h-28 truncate text-[11px]" style="writing-mode: vertical-rl; transform: rotate(180deg);">
                    {column.name}
                  </span>
                </button>
              </th>
            {/each}
            <th class="sticky top-0 bg-[var(--color-surface-base)] px-2 py-2 border-b border-[var(--color-border-subtle)] font-mono text-[10px] text-[var(--color-text-muted)]">
              {t('matrixView.total')}
            </th>
          </tr>
        </thead>
        <tbody>
          {#each matrix.rows as row (row.id)}
            <tr class={navigation.current === row.id ? 'bg-[var(--color-crown-red-subtle)]' : ''}>
              <th class="sticky left-0 z-10 bg-[var(--color-surface-base)] text-left px-2 py-1 border-b border-[var(--color-border-subtle)] max-w-56">
                <button
                  type="button"
                  onclick={() => navigation.open(row.id)}
                  ondblclick={() => onopen(row.id)}
                  class="flex flex-col items-start cursor-pointer bg-transparent border-0 p-0 text-left"
                >
                  <span class="text-xs font-medium text-[var(--color-text-primary)] truncate max-w-52">{row.name}</span>
                  <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{typeLabelOf(row)}</span>
                </button>
              </th>
              {#each matrix.columns as column (column.id)}
                {@const found = cells.get(cellKey(row.id, column.id)) ?? []}
                {@const first = found[0]}
                <td class="p-0.5 border-b border-[var(--color-border-subtle)] text-center">
                  {#if first}
                    {@const look = kindLook(first.kind, kindPresence.get(first.kind) ?? first.is_presence)}
                    <button
                      type="button"
                      title={describe(found)}
                      onclick={() => (editing = { entity: row, context: column, cell: first })}
                      class="w-8 h-8 flex items-center justify-center border-0 bg-transparent cursor-pointer transition-colors hover:bg-[var(--color-surface-hover)] {look.text}"
                    >
                      <span class="{look.icon} w-4 h-4"></span>
                      {#if first.viewpoint}
                        <span class="i-mingcute:eye-line w-2.5 h-2.5 absolute translate-x-3 -translate-y-3"></span>
                      {/if}
                    </button>
                  {:else}
                    <button
                      type="button"
                      title={t('matrixView.place')}
                      onclick={() => place(row, column)}
                      class="w-8 h-8 flex items-center justify-center border-0 cursor-pointer bg-transparent hover:bg-[var(--color-surface-hover)]"
                    >
                      <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-border-strong)]/30"></span>
                    </button>
                  {/if}
                </td>
              {/each}
              <td class="px-2 border-b border-[var(--color-border-subtle)] font-mono tabular-nums text-center text-[var(--color-text-secondary)]">
                {perRow.get(row.id) ?? 0}
              </td>
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr>
            <th class="sticky left-0 bg-[var(--color-surface-base)] text-left px-2 py-1 font-mono text-[10px] text-[var(--color-text-muted)]">
              {t('matrixView.total')}
            </th>
            {#each matrix.columns as column (column.id)}
              <td class="text-center font-mono tabular-nums text-[var(--color-text-secondary)] py-1">{perColumn.get(column.id) ?? 0}</td>
            {/each}
            <td></td>
          </tr>
        </tfoot>
      </table>
    {/if}
  </div>
</ViewFrame>

{#if editing && matrix}
  <AppearanceDialog
    entity={editing.entity}
    context={editing.context}
    cell={editing.cell}
    viewpoints={matrix.rows}
    onclose={() => (editing = null)}
  />
{/if}
