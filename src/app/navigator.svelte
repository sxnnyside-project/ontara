<script lang="ts">
import { catalog } from '@/features/catalog';
import { getEditorialTree, SectionTree } from '@/features/editorial';
import { listEntities, type EntitySummary } from '@/features/entity';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { typeLabelOf } from '@/shared/model';
import { revision } from '@/shared/revision';
import {
  Button,
  CanonBadge,
  EmptyState,
  InlineSelect,
  SearchInput,
  Segmented,
  entityIcon,
} from '@/shared/ui';

interface Props {
  selectedId: string | null;
  onselect: (id: string) => void;
  oncreate: () => void;
  onmanageStructure: () => void;
  /** Hides the navigator; the author brings it back from the edge of the window. */
  oncollapse: () => void;
}

const { selectedId, onselect, oncreate, onmanageStructure, oncollapse }: Props = $props();

let tab = $state<'entities' | 'structure'>('entities');
let text = $state('');
let typeFilter = $state('');
let items = $state<EntitySummary[]>([]);
let sectionCount = $state(0);
let failure = $state<string | null>(null);

/** Only the elements of the universe: sections live in the structure, assets in the library. */
function isUniverseElement(key: string): boolean {
  const role = catalog.data.entity_types.find((entry) => entry.key === key)?.role;
  return role === 'generic' || role === 'event';
}

$effect(() => {
  void revision.value;
  const filter = { text: text.trim() || null, entity_type: typeFilter || null };
  let cancelled = false;
  listEntities(filter)
    .then((found) => {
      if (cancelled) return;
      items = found.filter((entry) => isUniverseElement(entry.entity_type));
      failure = null;
    })
    .catch((error) => {
      if (!cancelled) failure = toCommandError(error).message;
    });
  return () => {
    cancelled = true;
  };
});

$effect(() => {
  void revision.value;
  getEditorialTree()
    .then((found) => {
      sectionCount = found.length;
    })
    .catch(() => {
      sectionCount = 0;
    });
});

const filtering = $derived(text.trim() !== '' || typeFilter !== '');

const typeOptions = $derived(
  catalog.data.entity_types
    .filter((entry) => entry.role === 'generic' || entry.role === 'event')
    .map((entry) => ({
      value: entry.key,
      label: entry.is_abstract ? `${entry.label} ${t('entity.andSubtypes')}` : entry.label,
    })),
);
</script>

<div class="h-full min-h-0 flex flex-col">
  <div class="p-2 flex items-center gap-1 border-b border-[var(--color-border-subtle)]">
    <div class="flex-1 min-w-0">
      <Segmented
        bind:value={tab as string}
        label={t('navigator.entities')}
        options={[
          { value: 'entities', label: t('navigator.entities') },
          { value: 'structure', label: t('navigator.structure') },
        ]}
      />
    </div>
    <Button size="sm" variant="ghost" icon="i-mingcute:left-line" title={t('navigator.hide')} onclick={oncollapse} />
  </div>

  {#if tab === 'entities'}
    <div class="p-2 flex flex-col gap-2 border-b border-[var(--color-border-subtle)]">
      <SearchInput bind:value={text} label={t('common.search')} placeholder={t('navigator.searchPlaceholder')} />
      <div class="flex items-center gap-2">
        <div class="flex-1 min-w-0">
          <InlineSelect label={t('navigator.type')} bind:value={typeFilter} options={typeOptions} placeholder={t('navigator.allTypes')} />
        </div>
        <Button size="sm" variant="primary" icon="i-mingcute:add-line" title={t('navigator.newEntity')} onclick={oncreate}>
          {t('navigator.new')}
        </Button>
      </div>
    </div>
    <div class="flex-1 min-h-0 overflow-y-auto p-1 flex flex-col gap-0.5">
      {#if failure}
        <p class="m-2 text-xs text-[var(--color-finding-error)]" role="alert">{failure}</p>
      {/if}
      {#each items as item (item.id)}
        <button
          type="button"
          onclick={() => onselect(item.id)}
          class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left border-0 cursor-pointer transition-colors
          {selectedId === item.id
            ? 'bg-[var(--color-surface-elevated)] shadow-[inset_2px_0_0_var(--color-crown-red)]'
            : 'bg-transparent hover:bg-[var(--color-surface-hover)]'}"
        >
          <span class="{entityIcon(catalog.chain(item.entity_type))} w-4 h-4 shrink-0 text-[var(--color-text-muted)]"></span>
          <span class="flex-1 min-w-0">
            <span class="block text-xs truncate {item.archived ? 'line-through text-[var(--color-text-muted)]' : 'text-[var(--color-text-primary)]'}">{item.name}</span>
            <span class="block text-[10px] font-mono text-[var(--color-text-muted)] truncate">{typeLabelOf(item)}</span>
          </span>
          {#if item.canon_state !== 'established'}
            <CanonBadge state={item.canon_state} label={catalog.canonLabel(item.canon_state)} size="sm" />
          {/if}
        </button>
      {:else}
        {#if !failure}
          {#if filtering}
            <div class="p-4 flex flex-col items-center gap-2 text-center">
              <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('navigator.noResults')}</p>
              <Button size="sm" variant="ghost" onclick={() => { text = ''; typeFilter = ''; }}>
                {t('navigator.clearFilters')}
              </Button>
            </div>
          {:else}
            <EmptyState icon="i-mingcute:box-line" title={t('navigator.emptyTitle')} description={t('navigator.emptyHint')}>
              <Button variant="primary" icon="i-mingcute:add-line" onclick={oncreate}>{t('navigator.newEntity')}</Button>
            </EmptyState>
          {/if}
        {/if}
      {/each}
    </div>
    <div class="px-3 py-1.5 border-t border-[var(--color-border-subtle)] text-[10px] font-mono text-[var(--color-text-muted)]">
      {t('navigator.count', { count: items.length })}
    </div>
  {:else}
    <div class="flex-1 min-h-0 overflow-y-auto p-2">
      {#if sectionCount === 0}
        <EmptyState icon="i-mingcute:book-2-line" title={t('navigator.noStructure')}>
          <Button variant="primary" onclick={onmanageStructure}>{t('navigator.manageStructure')}</Button>
        </EmptyState>
      {:else}
        <SectionTree selectedId={selectedId} onselect={onselect} />
      {/if}
    </div>
    <div class="p-2 border-t border-[var(--color-border-subtle)]">
      <Button size="sm" variant="ghost" icon="i-mingcute:settings-3-line" onclick={onmanageStructure}>
        {t('navigator.manageStructure')}
      </Button>
    </div>
  {/if}
</div>
