<script lang="ts">
import Field from './field.svelte';

interface Props {
  label: string;
  value: string;
  placeholder?: string | undefined;
  hint?: string | undefined;
  error?: string | null | undefined;
  type?: 'text' | 'number' | 'search' | undefined;
  multiline?: boolean | undefined;
  rows?: number | undefined;
  disabled?: boolean | undefined;
  required?: boolean | undefined;
  mono?: boolean | undefined;
  /** Focus the control when it appears. */
  focus?: boolean | undefined;
  oninput?: (() => void) | undefined;
}

let {
  label,
  value = $bindable(),
  placeholder = '',
  hint,
  error,
  type = 'text',
  multiline = false,
  rows = 4,
  disabled = false,
  required = false,
  mono = false,
  focus = false,
  oninput,
}: Props = $props();

const id = `field-${Math.random().toString(36).slice(2, 9)}`;
let control = $state<HTMLInputElement | HTMLTextAreaElement | undefined>();

$effect(() => {
  if (focus) control?.focus();
});

const style =
  'w-full rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-2.5 text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] outline-none transition-colors focus:border-[var(--color-border-focus)] disabled:opacity-50 select-text';
</script>

<Field {label} for={id} {hint} {error}>
  {#if multiline}
    <textarea
      {id}
      bind:this={control}
      bind:value
      {rows}
      {placeholder}
      {disabled}
      {required}
      {oninput}
      class="{style} py-2 text-xs leading-relaxed resize-y {mono ? 'font-mono' : 'font-sans'}"
    ></textarea>
  {:else}
    <input
      {id}
      bind:this={control}
      bind:value
      {type}
      {placeholder}
      {disabled}
      {required}
      {oninput}
      class="{style} h-8 text-xs {mono ? 'font-mono' : 'font-sans'}"
    />
  {/if}
</Field>
