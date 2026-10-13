<script lang="ts">
import { t } from '@/shared/i18n';
import { type ViewKey, navigation } from '../state/navigation.svelte';

interface Props {
  /** Views to offer; the one being shown is left out. */
  views?: ViewKey[] | undefined;
}

const { views = ['universe', 'graph', 'matrix', 'timeline'] }: Props = $props();

const labels: Partial<Record<ViewKey, { label: () => string; icon: string }>> = {
  universe: { label: () => t('showIn.entity'), icon: 'i-mingcute:box-line' },
  graph: { label: () => t('tabs.graph'), icon: 'i-mingcute:git-branch-line' },
  matrix: { label: () => t('tabs.matrix'), icon: 'i-mingcute:grid-line' },
  timeline: { label: () => t('tabs.timeline'), icon: 'i-mingcute:history-line' },
  structure: { label: () => t('shell.structure'), icon: 'i-mingcute:book-2-line' },
};
</script>

{#if navigation.current}
  <div class="flex items-center gap-1.5 flex-wrap" role="group" aria-label={t('showIn.title')}>
    <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('showIn.title')}
    </span>
    {#each views.filter((view) => view !== navigation.view) as view (view)}
      {@const item = labels[view]}
      {#if item}
        <button
          type="button"
          onclick={() => navigation.show(view)}
          class="flex items-center gap-1 px-2 py-0.5 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] text-[11px] font-mono text-[var(--color-text-secondary)] cursor-pointer hover:border-[var(--color-border-strong)] hover:text-[var(--color-text-primary)] transition-colors"
        >
          <span class="{item.icon} w-3 h-3"></span>
          <span>{item.label()}</span>
        </button>
      {/if}
    {/each}
  </div>
{/if}
