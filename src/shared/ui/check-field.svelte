<script lang="ts">
interface Props {
  label: string;
  checked: boolean;
  hint?: string | undefined;
  disabled?: boolean | undefined;
  onchange?: (() => void) | undefined;
}

let { label, checked = $bindable(), hint, disabled = false, onchange }: Props = $props();
</script>

<label class="flex items-start gap-2 text-xs text-[var(--color-text-primary)] cursor-pointer select-none">
  <input
    type="checkbox"
    bind:checked
    {disabled}
    {onchange}
    class="mt-0.5 accent-[var(--color-crown-red)]"
  />
  <span class="flex flex-col">
    <span>{label}</span>
    {#if hint}<span class="text-[11px] text-[var(--color-text-muted)]">{hint}</span>{/if}
  </span>
</label>
