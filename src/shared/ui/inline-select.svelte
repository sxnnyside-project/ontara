<script lang="ts">
import type { SelectOption } from './select-field.svelte';

interface Props {
  label: string;
  value: string;
  options: SelectOption[];
  /** Text of an extra first option that stands for "nothing chosen". */
  placeholder?: string | undefined;
  disabled?: boolean | undefined;
  onchange?: (() => void) | undefined;
}

let {
  label,
  value = $bindable(),
  options,
  placeholder,
  disabled = false,
  onchange,
}: Props = $props();
</script>

<label class="inline-flex max-w-full items-center gap-1.5 text-[11px] font-mono text-[var(--color-text-muted)]">
  <span class="shrink-0">{label}</span>
  <select
    bind:value
    {disabled}
    {onchange}
    class="h-7 min-w-0 max-w-52 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-1.5 text-xs font-sans text-[var(--color-text-primary)] outline-none focus:border-[var(--color-border-focus)] disabled:opacity-50 cursor-pointer"
  >
    {#if placeholder !== undefined}<option value="">{placeholder}</option>{/if}
    {#each options as option (option.value)}
      <option value={option.value} disabled={option.disabled}>{option.label}</option>
    {/each}
  </select>
</label>
