<script lang="ts">
import { t } from '@/shared/i18n';
import type { Instant, Temporal } from '@/shared/model';
import InstantField from './instant-field.svelte';
import SelectField from './select-field.svelte';

interface Props {
  label: string;
  value: Temporal;
  onchange?: (() => void) | undefined;
}

let { label, value = $bindable(), onchange }: Props = $props();

let kind = $state(value.kind);
let at = $state<Instant | null>(value.kind === 'at' ? value.instant : null);
let start = $state<Instant | null>(value.kind === 'span' ? value.start : null);
let end = $state<Instant | null>(value.kind === 'span' ? value.end : null);

function commit() {
  switch (kind) {
    case 'unknown':
      value = { kind: 'unknown' };
      break;
    case 'unconstrained':
      value = { kind: 'unconstrained' };
      break;
    case 'at':
      value = at ? { kind: 'at', instant: at } : { kind: 'unknown' };
      break;
    case 'span':
      value = start || end ? { kind: 'span', start, end } : { kind: 'unknown' };
      break;
  }
  onchange?.();
}

const options = $derived([
  { value: 'unconstrained', label: t('time.unconstrained') },
  { value: 'unknown', label: t('time.unknown') },
  { value: 'at', label: t('time.at') },
  { value: 'span', label: t('time.span') },
]);
</script>

<div class="flex flex-col gap-2">
  <SelectField
    {label}
    bind:value={kind as string}
    {options}
    onchange={() => commit()}
  />
  {#if kind === 'at'}
    <InstantField label={t('time.moment')} bind:value={at} onchange={commit} />
  {:else if kind === 'span'}
    <div class="grid gap-2 md:grid-cols-2">
      <InstantField label={t('time.from')} bind:value={start} onchange={commit} hint={t('time.openStart')} />
      <InstantField label={t('time.until')} bind:value={end} onchange={commit} hint={t('time.openEnd')} />
    </div>
  {/if}
</div>
