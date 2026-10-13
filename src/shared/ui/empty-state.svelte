<script lang="ts">
import type { Snippet } from 'svelte';

export interface EmptyStep {
  icon: string;
  title: string;
  text: string;
}

interface Props {
  icon: string;
  title: string;
  description?: string | undefined;
  /** What to do next, as short cards. */
  steps?: EmptyStep[] | undefined;
  /** The actions that get the author out of the empty state. */
  children?: Snippet | undefined;
}

const { icon, title, description, steps, children }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-5 text-center py-14 px-6 min-h-64">
  <span class="{icon} w-10 h-10 text-[var(--color-text-muted)]"></span>
  <div class="flex flex-col gap-1.5 max-w-md">
    <h3 class="m-0 text-lg font-serif font-medium">{title}</h3>
    {#if description}
      <p class="m-0 text-xs text-[var(--color-text-secondary)] leading-relaxed">{description}</p>
    {/if}
  </div>
  {#if children}
    <div class="flex items-center justify-center gap-2 flex-wrap">{@render children()}</div>
  {/if}
  {#if steps && steps.length > 0}
    <ol class="m-0 p-0 list-none grid gap-3 w-full max-w-3xl" style="grid-template-columns: repeat({steps.length}, minmax(0, 1fr))">
      {#each steps as step, index (index)}
        <li class="text-left rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-3.5 flex flex-col gap-1.5">
          <span class="flex items-center gap-2 text-[11px] font-mono text-[var(--color-text-muted)]">
            <span class="{step.icon} w-4 h-4 text-[var(--color-crown-red-text)]"></span>
            {index + 1}
          </span>
          <span class="text-sm font-medium">{step.title}</span>
          <span class="text-xs text-[var(--color-text-secondary)] leading-relaxed">{step.text}</span>
        </li>
      {/each}
    </ol>
  {/if}
</div>
