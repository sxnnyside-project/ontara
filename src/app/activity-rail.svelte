<script lang="ts">
import type { ViewKey } from '@/features/navigation';
import { t } from '@/shared/i18n';

interface Props {
  view: ViewKey;
  /** Errors the consistency checks found, shown as a badge. */
  errors: number;
  /** Suggestions waiting for a decision, shown as a badge. */
  suggestions?: number | undefined;
  onselect: (view: ViewKey) => void;
}

const { view, errors, suggestions = 0, onselect }: Props = $props();

interface Item {
  key: ViewKey;
  label: string;
  title: string;
  icon: string;
}

const main = $derived<Item[]>([
  {
    key: 'universe',
    label: t('rail.universe'),
    title: t('shell.universe'),
    icon: 'i-mingcute:earth-2-line',
  },
  {
    key: 'structure',
    label: t('rail.structure'),
    title: t('shell.structure'),
    icon: 'i-mingcute:book-2-line',
  },
  {
    key: 'graph',
    label: t('rail.graph'),
    title: t('tabs.graph'),
    icon: 'i-mingcute:git-branch-line',
  },
  { key: 'matrix', label: t('rail.matrix'), title: t('tabs.matrix'), icon: 'i-mingcute:grid-line' },
  {
    key: 'timeline',
    label: t('rail.timeline'),
    title: t('tabs.timeline'),
    icon: 'i-mingcute:history-line',
  },
  {
    key: 'consistency',
    label: t('rail.consistency'),
    title: t('tabs.consistency'),
    icon: 'i-mingcute:shield-shape-line',
  },
]);

const secondary = $derived<Item[]>([
  {
    key: 'proposals',
    label: t('rail.proposals'),
    title: t('proposals.title'),
    icon: 'i-mingcute:sparkles-line',
  },
  {
    key: 'library',
    label: t('rail.library'),
    title: t('shell.library'),
    icon: 'i-mingcute:pic-line',
  },
  {
    key: 'configuration',
    label: t('rail.configuration'),
    title: t('shell.configuration'),
    icon: 'i-mingcute:settings-3-line',
  },
]);
</script>

{#snippet button(item: Item)}
  <button
    type="button"
    title={item.title}
    aria-current={view === item.key ? 'page' : undefined}
    onclick={() => onselect(item.key)}
    class="relative w-full py-2 flex flex-col items-center gap-1 border-0 cursor-pointer transition-colors
    {view === item.key
      ? 'bg-[var(--color-surface-elevated)] text-[var(--color-text-primary)]'
      : 'bg-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface-hover)]'}"
  >
    {#if view === item.key}
      <span class="absolute left-0 top-1.5 bottom-1.5 w-0.5 rounded-r bg-[var(--color-crown-red)]"></span>
    {/if}
    <span class="{item.icon} w-5 h-5"></span>
    <span class="text-[10px] font-mono leading-none">{item.label}</span>
    {#if item.key === 'proposals' && suggestions > 0}
      <span class="absolute top-1 right-2 min-w-4 h-4 px-1 rounded-full bg-[var(--color-crown-red)] text-white text-[10px] font-mono leading-4 text-center tabular-nums">{suggestions}</span>
    {/if}
    {#if item.key === 'consistency' && errors > 0}
      <span class="absolute top-1 right-2 min-w-4 h-4 px-1 rounded-full bg-[var(--color-finding-error)] text-white text-[10px] font-mono leading-4 text-center tabular-nums">{errors}</span>
    {/if}
  </button>
{/snippet}

<nav class="w-[68px] shrink-0 flex flex-col justify-between border-r border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/90 backdrop-blur-md py-1" aria-label={t('shell.views')}>
  <div class="flex flex-col">
    {#each main as item (item.key)}{@render button(item)}{/each}
  </div>
  <div class="flex flex-col">
    {#each secondary as item (item.key)}{@render button(item)}{/each}
  </div>
</nav>
