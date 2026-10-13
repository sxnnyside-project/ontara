<script lang="ts">
import type { Snippet } from 'svelte';
import { onMount } from 'svelte';

interface Props {
  title: string;
  description?: string | undefined;
  width?: 'sm' | 'md' | 'lg' | undefined;
  /** Called when the dialog is dismissed with Escape. */
  onclose: () => void;
  children: Snippet;
  actions?: Snippet | undefined;
}

const { title, description, width = 'md', onclose, children, actions }: Props = $props();

let element = $state<HTMLDialogElement | undefined>();
const heading = `dialog-${Math.random().toString(36).slice(2, 9)}`;

/** Ctrl or Cmd with Enter does what the dialog's main action does, from wherever focus is. */
function onkeydown(event: KeyboardEvent) {
  if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return;
  const primary = element?.querySelector<HTMLButtonElement>(
    'footer button[data-variant="primary"]:not(:disabled)',
  );
  if (!primary) return;
  event.preventDefault();
  primary.click();
}

onMount(() => {
  element?.showModal();
});

const widths = { sm: 'w-[24rem]', md: 'w-[34rem]', lg: 'w-[46rem]' } as const;
</script>

<dialog
  bind:this={element}
  {onclose}
  {onkeydown}
  aria-labelledby={heading}
  class="m-auto p-0 rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-surface-panel)] text-[var(--color-text-primary)] shadow-[var(--shadow-lg)] backdrop:bg-black/60 max-w-[calc(100vw-2rem)] max-h-[calc(100vh-4rem)] {widths[width]}"
>
  <div class="flex flex-col max-h-[calc(100vh-4rem)]">
    <header class="px-5 pt-4 pb-3 border-b border-[var(--color-border-subtle)]">
      <h2 id={heading} class="m-0 text-base font-serif font-medium">{title}</h2>
      {#if description}
        <p class="m-0 mt-1 text-xs text-[var(--color-text-secondary)]">{description}</p>
      {/if}
    </header>
    <div class="px-5 py-4 overflow-y-auto flex flex-col gap-4">
      {@render children()}
    </div>
    {#if actions}
      <footer class="px-5 py-3 border-t border-[var(--color-border-subtle)] flex items-center justify-end gap-2">
        {@render actions()}
      </footer>
    {/if}
  </div>
</dialog>
