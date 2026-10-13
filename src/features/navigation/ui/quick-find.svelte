<script lang="ts">
import { catalog } from '@/features/catalog';
import { t } from '@/shared/i18n';
import { Dialog, entityIcon } from '@/shared/ui';
import { listEntities } from '@/features/entity';
import type { EntitySummary } from '@/features/entity';

interface Props {
  onselect: (id: string) => void;
  onclose: () => void;
}

const { onselect, onclose }: Props = $props();

let text = $state('');
let results = $state<EntitySummary[]>([]);
let active = $state(0);

$effect(() => {
  const query = text.trim();
  let cancelled = false;
  listEntities({ text: query === '' ? null : query, include_archived: true })
    .then((found) => {
      if (cancelled) return;
      results = found.slice(0, 12);
      active = 0;
    })
    .catch(() => {
      if (!cancelled) results = [];
    });
  return () => {
    cancelled = true;
  };
});

function onkeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    active = Math.min(active + 1, results.length - 1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    active = Math.max(active - 1, 0);
  } else if (event.key === 'Enter') {
    const chosen = results[active];
    if (chosen) onselect(chosen.id);
  }
}
</script>

<Dialog title={t('quickFind.title')} {onclose}>
  <input
    bind:value={text}
    {onkeydown}
    placeholder={t('quickFind.placeholder')}
    aria-label={t('quickFind.title')}
    class="h-9 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-3 text-sm outline-none focus:border-[var(--color-border-focus)] select-text"
  />
  <div class="flex flex-col gap-0.5 -mx-1">
    {#each results as result, index (result.id)}
      <button
        type="button"
        onclick={() => onselect(result.id)}
        class="flex items-center gap-2 px-2 py-1.5 rounded border-0 text-left cursor-pointer
        {index === active ? 'bg-[var(--color-surface-elevated)]' : 'bg-transparent hover:bg-[var(--color-surface-hover)]'}"
      >
        <span class="{entityIcon(catalog.chain(result.entity_type))} w-4 h-4 text-[var(--color-text-muted)]"></span>
        <span class="flex-1 min-w-0 text-xs truncate">{result.name}</span>
        <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{result.type_label}</span>
      </button>
    {:else}
      <p class="m-2 text-xs text-[var(--color-text-muted)]">{t('entity.noneFound')}</p>
    {/each}
  </div>
</Dialog>
