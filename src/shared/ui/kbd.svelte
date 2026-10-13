<script lang="ts">
import type { Snippet } from 'svelte';

const { children }: { children: Snippet } = $props();
</script>

<kbd class="px-1 min-w-4 h-4 inline-flex items-center justify-center rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] text-[10px] font-mono text-[var(--color-text-muted)]">{@render children()}</kbd>
