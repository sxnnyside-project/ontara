<script lang="ts">
interface Props {
  value: string;
  /** Shown, dimmed, while the value is empty. */
  placeholder?: string | undefined;
  /** Accessible name of the control. */
  label: string;
  /** `title` reads as a heading; `text` as a line of body text. */
  look?: 'title' | 'text' | undefined;
  /** An empty value is refused when true. */
  required?: boolean | undefined;
  onsave: (value: string) => void | Promise<void>;
}

const { value, placeholder = '', label, look = 'text', required = false, onsave }: Props = $props();

let editing = $state(false);
let draft = $state('');
let control = $state<HTMLInputElement | undefined>();

function begin() {
  draft = value;
  editing = true;
}

$effect(() => {
  if (editing) {
    control?.focus();
    control?.select();
  }
});

async function commit() {
  if (!editing) return;
  editing = false;
  const next = draft.trim();
  if (next === value || (required && next === '')) return;
  await onsave(next);
}

function cancel() {
  editing = false;
}

const look_classes = {
  title: 'text-2xl font-serif font-medium',
  text: 'text-xs text-[var(--color-text-secondary)]',
} as const;
</script>

{#if editing}
  <input
    bind:this={control}
    bind:value={draft}
    aria-label={label}
    onblur={commit}
    onkeydown={(event) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        commit();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        cancel();
      }
    }}
    class="w-full bg-[var(--color-surface-base)] border border-[var(--color-border-focus)] rounded px-1.5 -mx-1.5 outline-none focus:border-[var(--color-border-focus)] select-text {look_classes[look]} text-[var(--color-text-primary)]"
  />
{:else}
  <button
    type="button"
    onclick={begin}
    title={label}
    class="group max-w-full text-left bg-transparent border-0 p-0 cursor-text rounded hover:bg-[var(--color-surface-hover)] -mx-1.5 px-1.5 {look_classes[look]}"
  >
    {#if value}
      <span class="text-[var(--color-text-primary)] {look === 'text' ? '!text-[var(--color-text-secondary)]' : ''}">{value}</span>
    {:else}
      <span class="text-[var(--color-text-muted)] italic">{placeholder}</span>
    {/if}
    <span class="i-mingcute:edit-2-line w-3 h-3 ml-1 align-middle opacity-0 group-hover:opacity-60 text-[var(--color-text-muted)] inline-block"></span>
  </button>
{/if}
