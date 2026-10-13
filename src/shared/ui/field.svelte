<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  label: string;
  /** The `id` of the control inside, so the label is associated with it. */
  for?: string | undefined;
  hint?: string | undefined;
  error?: string | null | undefined;
  children: Snippet;
}

const { label, for: controlId, hint, error, children }: Props = $props();
</script>

<div class="flex flex-col gap-1">
  <label for={controlId} class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
    {label}
  </label>
  {@render children()}
  {#if error}
    <p class="m-0 text-[11px] font-mono text-[var(--color-finding-error)]" role="alert">{error}</p>
  {:else if hint}
    <p class="m-0 text-[11px] text-[var(--color-text-muted)]">{hint}</p>
  {/if}
</div>
