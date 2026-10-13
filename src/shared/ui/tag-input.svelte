<script lang="ts">
import Field from './field.svelte';

interface Props {
  label: string;
  value: string[];
  placeholder?: string | undefined;
  onchange?: (() => void) | undefined;
}

let {
  label,
  value = $bindable(),
  placeholder = 'Type a tag and press Enter',
  onchange,
}: Props = $props();

const id = `field-${Math.random().toString(36).slice(2, 9)}`;
let draft = $state('');

function commit() {
  const tag = draft.trim();
  draft = '';
  if (tag && !value.includes(tag)) {
    value = [...value, tag];
    onchange?.();
  }
}

function remove(tag: string) {
  value = value.filter((existing) => existing !== tag);
  onchange?.();
}

function onkeydown(event: KeyboardEvent) {
  if (event.key === 'Enter' || event.key === ',') {
    event.preventDefault();
    commit();
  } else if (event.key === 'Backspace' && draft === '' && value.length > 0) {
    value = value.slice(0, -1);
    onchange?.();
  }
}
</script>

<Field {label} for={id}>
  <div
    class="flex flex-wrap items-center gap-1 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] p-1.5 focus-within:border-[var(--color-border-focus)]"
  >
    {#each value as tag}
      <span
        class="inline-flex items-center gap-1 rounded bg-[var(--color-surface-elevated)] border border-[var(--color-border-subtle)] pl-2 pr-1 py-0.5 text-[11px] font-mono text-[var(--color-text-secondary)]"
      >
        {tag}
        <button
          type="button"
          aria-label="Remove tag {tag}"
          onclick={() => remove(tag)}
          class="inline-flex bg-transparent border-0 p-0 cursor-pointer text-[var(--color-text-muted)] hover:text-[var(--color-finding-error)]"
        >
          <span class="i-mingcute:close-line w-3 h-3"></span>
        </button>
      </span>
    {/each}
    <input
      {id}
      bind:value={draft}
      {placeholder}
      {onkeydown}
      onblur={commit}
      class="min-w-24 flex-1 bg-transparent border-0 outline-none text-xs font-sans text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] select-text"
    />
  </div>
</Field>
