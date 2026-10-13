<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  icon: string;
  title: string;
  description?: string | undefined;
  children?: Snippet | undefined;
}

const { icon, title, description, children }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-3 text-center py-12 px-6">
  <span class="{icon} w-9 h-9 text-[var(--color-text-muted)]"></span>
  <div class="flex flex-col gap-1 max-w-sm">
    <h3 class="m-0 text-base font-serif font-medium">{title}</h3>
    {#if description}
      <p class="m-0 text-xs text-[var(--color-text-secondary)] leading-relaxed">{description}</p>
    {/if}
  </div>
  {@render children?.()}
</div>
