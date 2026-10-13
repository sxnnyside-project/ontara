<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Button, CanonBadge, SelectField, TextField, entityIcon } from '@/shared/ui';
import { type EntitySummary, listEntities } from '../api/entity-api';

interface Props {
  selectedId: string | null;
  onselect: (id: string) => void;
  oncreate: () => void;
}

const { selectedId, onselect, oncreate }: Props = $props();

let text = $state('');
let typeFilter = $state('');
let items = $state<EntitySummary[]>([]);
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

const typeOptions = $derived(
  catalog.data.entity_types
    .filter((entry) => entry.role === 'generic' || entry.role === 'event')
    .map((entry) => ({
      value: entry.key,
      label: entry.is_abstract ? `${entry.label} ${t('entity.andSubtypes')}` : entry.label,
    })),
);
</script>

<div class="flex flex-col gap-2 min-h-0 flex-1">
  <div class="flex items-center gap-1.5 px-2">
    <span class="i-mingcute:box-line w-3.5 h-3.5 text-[var(--color-meridian-blue)]"></span>
    <span class="flex-1 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('sidebar.entities')}
    </span>
    <Button size="sm" variant="ghost" icon="i-mingcute:add-line" title={t('sidebar.addEntity')} onclick={oncreate} />
  </div>

  <div class="px-2 flex flex-col gap-2">
    <TextField label={t('common.search')} bind:value={text} type="search" placeholder={t('entity.searchPlaceholder')} />
    <SelectField label={t('entity.type')} bind:value={typeFilter} options={typeOptions} placeholder={t('entity.allTypes')} />
  </div>

  <div class="flex-1 min-h-0 overflow-y-auto px-1 flex flex-col gap-0.5">
    {#if failure}
      <p class="m-2 text-xs text-[var(--color-finding-error)]" role="alert">{failure}</p>
    {/if}
    {#each items as item (item.id)}
      <button
        type="button"
        onclick={() => onselect(item.id)}
        class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left border-0 cursor-pointer transition-colors
        {selectedId === item.id
          ? 'bg-[var(--color-surface-elevated)] border-l-2 border-l-[var(--color-crown-red)]'
          : 'bg-transparent hover:bg-[var(--color-surface-hover)]'}"
      >
        <span class="{entityIcon(catalog.chain(item.entity_type))} w-4 h-4 shrink-0 text-[var(--color-text-muted)]"></span>
        <span class="flex-1 min-w-0">
          <span class="block text-xs truncate {item.archived ? 'line-through text-[var(--color-text-muted)]' : 'text-[var(--color-text-primary)]'}">{item.name}</span>
          <span class="block text-[10px] font-mono text-[var(--color-text-muted)] truncate">{item.type_label}</span>
        </span>
        {#if item.canon_state !== 'established'}
          <CanonBadge state={item.canon_state} label={catalog.canonLabel(item.canon_state)} size="sm" />
        {/if}
      </button>
    {:else}
      {#if !failure}
        <p class="m-2 text-xs text-[var(--color-text-muted)]">{t('entity.noneFound')}</p>
      {/if}
    {/each}
  </div>
</div>
