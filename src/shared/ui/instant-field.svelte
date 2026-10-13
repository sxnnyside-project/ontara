<script lang="ts">
import { t } from '@/shared/i18n';
import type { Instant } from '@/shared/model';
import Field from './field.svelte';

interface Props {
  label: string;
  value: Instant | null;
  hint?: string | undefined;
  onchange?: (() => void) | undefined;
}

let { label, value = $bindable(), hint, onchange }: Props = $props();

const id = `field-${Math.random().toString(36).slice(2, 9)}`;
let words = $state(value?.label ?? '');
let day = $state(value?.day === null || value?.day === undefined ? '' : String(value.day));

/** Rebuilds the moment from what the author typed; nothing typed means no moment. */
function commit() {
  const label = words.trim();
  const parsed = day.trim() === '' ? null : Number.parseInt(day, 10);
  const placed = parsed !== null && Number.isFinite(parsed) ? parsed : null;
  if (label === '' && placed === null) {
    value = null;
  } else {
    value = {
      day: placed,
      precision: value?.precision ?? 'day',
      earliest: placed === null ? null : (value?.earliest ?? null),
      latest: placed === null ? null : (value?.latest ?? null),
      calendar: value?.calendar ?? null,
      label: label === '' ? null : label,
    };
  }
  onchange?.();
}

const style =
  'h-8 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-2.5 text-xs text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] outline-none focus:border-[var(--color-border-focus)] select-text';
</script>

<Field {label} for={id} {hint}>
  <div class="flex items-center gap-2">
    <input
      {id}
      bind:value={words}
      oninput={commit}
      placeholder={t('time.words')}
      class="{style} flex-1 min-w-0 font-sans"
    />
    <div class="flex items-center gap-1">
      <span class="text-[11px] font-mono text-[var(--color-meridian-timeline)]">D+</span>
      <input
        bind:value={day}
        oninput={commit}
        inputmode="numeric"
        placeholder={t('time.day')}
        aria-label={t('time.day')}
        class="{style} w-20 font-mono tabular-nums"
      />
    </div>
  </div>
</Field>
