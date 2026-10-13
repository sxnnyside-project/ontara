<script lang="ts">
import { catalog } from '@/features/catalog';
import { EntityPicker } from '@/features/entity';
import { t, tDynamic } from '@/shared/i18n';
import type { EntityRef, Instant, Temporal } from '@/shared/model';
import { CheckField, InstantField, SelectField, TemporalField, TextField } from '@/shared/ui';
import type { DossierRecord } from '../api/dossier-api';
import { CHOICES, type FieldDef } from '../state/dossier-schema';

interface Props {
  field: FieldDef;
  /** The record being edited; the field reads and writes its own name. */
  record: DossierRecord;
  /** Entities the record already mentions, so a chosen one shows before any search. */
  entities?: Record<string, EntityRef> | undefined;
  /** Offices to choose from, for the `office` kind. */
  offices?: { value: string; label: string }[] | undefined;
  /** The field cannot be changed. */
  locked?: boolean | undefined;
}

let { field, record = $bindable(), entities = {}, offices = [], locked = false }: Props = $props();

const label = $derived(tDynamic(`dossierFields.${field.name}`, field.name));
const text = $derived(
  record[field.name] === null || record[field.name] === undefined ? '' : String(record[field.name]),
);

function set(value: unknown) {
  record = { ...record, [field.name]: value };
}

const choices = $derived(
  field.group
    ? CHOICES[field.group].map((value) => ({
        value,
        label: tDynamic(`dossierChoices.${field.group}_${value}`, value),
      }))
    : [],
);
const terms = $derived(
  catalog.data.vocabulary_terms
    .filter((term) => term.vocabulary === field.vocabulary)
    .sort((a, b) => a.sort_order - b.sort_order)
    .map((term) => ({ value: term.key, label: term.label })),
);
const stances = $derived(
  catalog.data.diplomatic_stances.map((stance) => ({ value: stance.key, label: stance.label })),
);
const canonStates = $derived(
  catalog.data.canon_states.map((state) => ({ value: state.key, label: state.label })),
);
const flags = $derived([
  { value: 'true', label: t('dossier.yes') },
  { value: 'false', label: t('dossier.no') },
]);
const selectedEntity = $derived(
  typeof record[field.name] === 'string' ? (entities[record[field.name] as string] ?? null) : null,
);
const filter = $derived(
  field.classes && field.classes.length === 1 && field.classes[0]
    ? { entity_type: field.classes[0] }
    : {},
);
const placeholder = $derived(field.required ? t('dossier.pick') : t('dossier.notSaid'));
</script>

{#if field.kind === 'text'}
  <TextField
    {label}
    required={field.required}
    disabled={locked}
    bind:value={() => text, (value) => set(value)}
  />
{:else if field.kind === 'longtext'}
  <TextField
    {label}
    multiline
    rows={3}
    disabled={locked}
    bind:value={() => text, (value) => set(value)}
  />
{:else if field.kind === 'integer' || field.kind === 'number'}
  <TextField
    {label}
    type="number"
    disabled={locked}
    required={field.required}
    bind:value={() => text, (value) => set(value)}
  />
{:else if field.kind === 'flag'}
  {#if field.required}
    <CheckField
      {label}
      disabled={locked}
      bind:checked={() => record[field.name] === true, (value) => set(value)}
    />
  {:else}
    <SelectField
      {label}
      {placeholder}
      options={flags}
      disabled={locked}
      bind:value={
        () => (record[field.name] === null || record[field.name] === undefined ? '' : String(record[field.name])),
        (value) => set(value === '' ? null : value === 'true')
      }
    />
  {/if}
{:else if field.kind === 'choice'}
  <SelectField
    {label}
    {placeholder}
    options={choices}
    disabled={locked}
    bind:value={() => text, (value) => set(value === '' ? null : value)}
  />
{:else if field.kind === 'term'}
  <SelectField
    {label}
    {placeholder}
    options={terms}
    disabled={locked}
    bind:value={() => text, (value) => set(value === '' ? null : value)}
  />
{:else if field.kind === 'stance'}
  <SelectField
    {label}
    options={stances}
    disabled={locked}
    bind:value={() => text, (value) => set(value)}
  />
{:else if field.kind === 'canon'}
  <SelectField
    {label}
    options={canonStates}
    disabled={locked}
    bind:value={() => text, (value) => set(value)}
  />
{:else if field.kind === 'office'}
  <SelectField
    {label}
    options={offices}
    disabled={locked}
    bind:value={() => text, (value) => set(value)}
  />
{:else if field.kind === 'entity'}
  {#if locked}
    <TextField {label} disabled bind:value={() => selectedEntity?.name ?? text, () => {}} />
  {:else}
    <EntityPicker
      {label}
      {filter}
      selected={selectedEntity}
      placeholder={placeholder}
      bind:value={() => text, (value) => set(value === '' ? null : value)}
    />
  {/if}
{:else if field.kind === 'instant'}
  <InstantField
    {label}
    bind:value={() => (record[field.name] as Instant | null) ?? null, (value) => set(value)}
  />
{:else if field.kind === 'period'}
  <TemporalField
    {label}
    bind:value={
      () => (record[field.name] as Temporal | undefined) ?? { kind: 'unconstrained' },
      (value) => set(value)
    }
  />
{/if}
