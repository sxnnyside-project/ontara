<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { entityIcon } from '@/shared/ui';
import { getEditorialTree, type Section } from '../api/editorial-api';
import { buildTree, flatten } from '../state/section-tree';

interface Props {
  selectedId: string | null;
  onselect: (id: string) => void;
}

const { selectedId, onselect }: Props = $props();

let sections = $state<Section[]>([]);
let collapsed = $state<Set<string>>(new Set());

$effect(() => {
  void revision.value;
  let cancelled = false;
  getEditorialTree()
    .then((found) => {
      if (!cancelled) sections = found;
    })
    .catch((failure) => {
      if (!cancelled) notices.error(toCommandError(failure).message);
    });
  return () => {
    cancelled = true;
  };
});

const rows = $derived(flatten(buildTree(sections), collapsed));

function toggle(id: string) {
  const next = new Set(collapsed);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  collapsed = next;
}
</script>

<div class="flex flex-col gap-0.5">
  {#each rows as { node, depth } (node.section.entity.id)}
    {@const id = node.section.entity.id}
    <div class="flex items-center" style="padding-left: {depth * 0.75}rem">
      {#if node.children.length > 0}
        <button
          type="button"
          aria-label={collapsed.has(id) ? t('structure.expand') : t('structure.collapse')}
          onclick={() => toggle(id)}
          class="bg-transparent border-0 p-0.5 cursor-pointer text-[var(--color-text-muted)]"
        >
          <span class="{collapsed.has(id) ? 'i-mingcute:right-line' : 'i-mingcute:down-line'} w-3 h-3"></span>
        </button>
      {:else}
        <span class="w-4"></span>
      {/if}
      <button
        type="button"
        onclick={() => onselect(id)}
        class="flex-1 min-w-0 flex items-center gap-1.5 px-1.5 py-1 rounded border-0 text-left cursor-pointer
        {selectedId === id
          ? 'bg-[var(--color-surface-elevated)] text-[var(--color-text-primary)]'
          : 'bg-transparent text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]'}"
      >
        <span class="{entityIcon(catalog.chain(node.section.entity.entity_type))} w-3.5 h-3.5 shrink-0 text-[var(--color-text-muted)]"></span>
        <span class="truncate text-xs {depth === 0 ? 'font-serif font-medium' : 'font-sans'}">{node.section.entity.name}</span>
      </button>
    </div>
  {:else}
    <p class="m-2 text-xs text-[var(--color-text-muted)]">{t('structure.empty')}</p>
  {/each}
</div>
