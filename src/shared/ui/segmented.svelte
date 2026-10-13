<script lang="ts">
export interface SegmentOption {
  value: string;
  label: string;
  /** A MingCute icon class. */
  icon?: string | undefined;
}

interface Props {
  options: SegmentOption[];
  value: string;
  label?: string | undefined;
  onchange?: ((value: string) => void) | undefined;
}

let { options, value = $bindable(), label, onchange }: Props = $props();
</script>

<div role="group" aria-label={label} class="inline-flex max-w-full p-0.5 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)]">
  {#each options as option (option.value)}
    <button
      type="button"
      aria-pressed={value === option.value}
      onclick={() => {
        value = option.value;
        onchange?.(option.value);
      }}
      class="inline-flex items-center gap-1 h-6 px-2 rounded-sm border-0 text-[11px] font-mono cursor-pointer transition-colors
      {value === option.value
        ? 'bg-[var(--color-surface-elevated)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
        : 'bg-transparent text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'}"
    >
      {#if option.icon}<span class="{option.icon} w-3 h-3"></span>{/if}
      {option.label}
    </button>
  {/each}
</div>
