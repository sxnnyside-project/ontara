<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  title: string;
  count?: number | undefined;
  actions?: Snippet | undefined;
}

const { title, count, actions }: Props = $props();
</script>

<div class="flex items-center gap-2 min-h-7">
  <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider font-medium text-[var(--color-text-muted)]">
    {title}
  </h3>
  {#if count !== undefined}
    <span class="px-1 rounded bg-[var(--color-surface-elevated)] text-[10px] font-mono tabular-nums text-[var(--color-text-secondary)]">{count}</span>
  {/if}
  <span class="flex-1"></span>
  {#if actions}{@render actions()}{/if}
</div>
