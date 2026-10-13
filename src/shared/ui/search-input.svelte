<script lang="ts">
interface Props {
  value: string;
  placeholder?: string | undefined;
  label: string;
  /** Focus the control when it appears. */
  focus?: boolean | undefined;
}

let { value = $bindable(), placeholder = '', label, focus = false }: Props = $props();

let control = $state<HTMLInputElement | undefined>();

$effect(() => {
  if (focus) control?.focus();
});
</script>

<div class="relative">
  <span class="i-mingcute:search-line w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)] pointer-events-none"></span>
  <input
    bind:this={control}
    bind:value
    type="search"
    aria-label={label}
    {placeholder}
    class="h-8 w-full rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] pl-7 pr-2 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] outline-none focus:border-[var(--color-border-focus)] select-text"
  />
</div>
