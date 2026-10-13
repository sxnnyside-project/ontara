<script lang="ts">
import { t } from '@/shared/i18n';
import type { EntityRef } from '@/shared/model';
import { typeLabelOf } from '@/shared/model';
import { SelectField, TextField } from '@/shared/ui';
import { type EntityFilter, listEntities } from '../api/entity-api';

interface Props {
  label: string;
  /** The chosen entity's identifier, or an empty string. */
  value: string;
  /** Narrows what can be chosen, such as events only. */
  filter?: EntityFilter | undefined;
  /** Entities that cannot be chosen. */
  exclude?: string[] | undefined;
  /** The entity already chosen, so it shows even when the search no longer finds it. */
  selected?: EntityRef | null | undefined;
  placeholder?: string | undefined;
  onchange?: (() => void) | undefined;
}

let {
  label,
  value = $bindable(),
  filter = {},
  exclude = [],
  selected = null,
  placeholder,
  onchange,
}: Props = $props();

let search = $state('');
let found = $state<EntityRef[]>([]);

$effect(() => {
  const text = search.trim();
  const narrowing = { ...filter, text: text === '' ? null : text };
  let cancelled = false;
  listEntities(narrowing)
    .then((entities) => {
      if (!cancelled) found = entities;
    })
    .catch(() => {
      if (!cancelled) found = [];
    });
  return () => {
    cancelled = true;
  };
});

const options = $derived.by(() => {
  const list = found.filter((entity) => !exclude.includes(entity.id));
  if (selected && !list.some((entity) => entity.id === selected.id)) list.unshift(selected);
  return list.map((entity) => ({
    value: entity.id,
    label: `${entity.name} · ${typeLabelOf(entity)}`,
  }));
});
</script>

<div class="flex flex-col gap-1.5">
  <TextField label={label} bind:value={search} type="search" placeholder={t('picker.search')} />
  <SelectField
    label={t('picker.choose')}
    bind:value
    {options}
    placeholder={placeholder ?? t('picker.none')}
    {onchange}
  />
</div>
