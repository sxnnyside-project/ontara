<script lang="ts">
import { t } from '@/shared/i18n';
import { Field } from '@/shared/ui';

interface Props {
  label: string;
  hint?: string | undefined;
  options: { key: string; label: string }[];
  value: string[];
  /** Keys that were already chosen and cannot be taken back. */
  locked?: string[] | undefined;
}

let { label, hint, options, value = $bindable(), locked = [] }: Props = $props();

function toggle(key: string, checked: boolean) {
  value = checked ? [...value, key] : value.filter((existing) => existing !== key);
}
</script>

<Field {label} {hint}>
  <div class="max-h-36 overflow-y-auto rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] p-2 grid grid-cols-2 gap-x-3 gap-y-1">
    {#each options as option (option.key)}
      <label class="flex items-center gap-1.5 text-xs cursor-pointer select-none">
        <input
          type="checkbox"
          checked={value.includes(option.key)}
          disabled={locked.includes(option.key)}
          onchange={(event) => toggle(option.key, event.currentTarget.checked)}
          class="accent-[var(--color-crown-red)]"
        />
        <span class="truncate">{option.label}</span>
      </label>
    {/each}
  </div>
  {#if value.length === 0}
    <p class="m-0 text-[11px] text-[var(--color-text-muted)]">{t('config.anyType')}</p>
  {/if}
</Field>
