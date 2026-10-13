<script lang="ts">
import Field from './field.svelte';

export interface SelectOption {
  value: string;
  label: string;
  /** Options with the same group are shown together. */
  group?: string;
  disabled?: boolean;
}

interface Props {
  label: string;
  value: string;
  options: SelectOption[];
  hint?: string | undefined;
  error?: string | null | undefined;
  disabled?: boolean | undefined;
  /** Text of an extra first option that stands for "nothing chosen". */
  placeholder?: string | undefined;
  onchange?: (() => void) | undefined;
}

let {
  label,
  value = $bindable(),
  options,
  hint,
  error,
  disabled = false,
  placeholder,
  onchange,
}: Props = $props();

const id = `field-${Math.random().toString(36).slice(2, 9)}`;
const groups = $derived([...new Set(options.map((option) => option.group ?? ''))]);
</script>

<Field {label} for={id} {hint} {error}>
  <select
    {id}
    bind:value
    {disabled}
    {onchange}
    class="h-8 w-full rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-2 text-xs font-sans text-[var(--color-text-primary)] outline-none focus:border-[var(--color-border-focus)] disabled:opacity-50"
  >
    {#if placeholder !== undefined}
      <option value="">{placeholder}</option>
    {/if}
    {#each groups as group}
      {#if group}
        <optgroup label={group}>
          {#each options.filter((option) => (option.group ?? '') === group) as option}
            <option value={option.value} disabled={option.disabled}>{option.label}</option>
          {/each}
        </optgroup>
      {:else}
        {#each options.filter((option) => !option.group) as option}
          <option value={option.value} disabled={option.disabled}>{option.label}</option>
        {/each}
      {/if}
    {/each}
  </select>
</Field>
