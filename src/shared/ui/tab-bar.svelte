<script lang="ts">
export interface TabItem {
  key: string;
  label: string;
  icon?: string | undefined;
  /** A small count shown next to the label. */
  count?: number | undefined;
}

interface Props {
  tabs: TabItem[];
  active: string;
  onselect: (key: string) => void;
}

const { tabs, active, onselect }: Props = $props();
</script>

<div role="tablist" class="flex items-center gap-0.5 border-b border-[var(--color-border-subtle)]">
  {#each tabs as tab (tab.key)}
    <button
      type="button"
      role="tab"
      aria-selected={active === tab.key}
      onclick={() => onselect(tab.key)}
      class="flex items-center gap-1.5 px-3 py-2 -mb-px border-0 border-b-2 bg-transparent text-xs font-mono cursor-pointer transition-colors
      {active === tab.key
        ? 'border-b-[var(--color-crown-red)] text-[var(--color-text-primary)]'
        : 'border-b-transparent text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'}"
    >
      {#if tab.icon}<span class="{tab.icon} w-3.5 h-3.5"></span>{/if}
      <span>{tab.label}</span>
      {#if tab.count !== undefined && tab.count > 0}
        <span class="px-1 rounded bg-[var(--color-surface-elevated)] text-[10px] tabular-nums">{tab.count}</span>
      {/if}
    </button>
  {/each}
</div>
