<script lang="ts">
import { definitionLabel } from './property-labels';
import { t } from '@/shared/i18n';
import type {
  EntityRef,
  Instant,
  PropertyContent,
  PropertyDefinition,
  PropertyValue,
  PropertyValueInput,
  PropertyValueOwner,
} from '@/shared/model';
import { typeLabelOf } from '@/shared/model';
import { Button, CheckField, InstantField, SelectField, TextField } from '@/shared/ui';

interface Props {
  definition: PropertyDefinition;
  owner: PropertyValueOwner;
  /** The value being edited, or `null` for a value that does not exist yet. */
  value: PropertyValue | null;
  position: number;
  /** The entities a reference property may point at. */
  candidates: EntityRef[];
  onsave: (input: PropertyValueInput) => void | Promise<void>;
  onremove?: ((value: PropertyValue) => void | Promise<void>) | undefined;
}

const { definition, owner, value, position, candidates, onsave, onremove }: Props = $props();

const data = $derived(definition.data_type);

/** What the form starts with. Read once: the row keeps what the author typed. */
function initial() {
  const content = value?.content;
  return {
    text: content?.type === 'text' ? content.value : '',
    number: content?.type === 'integer' || content?.type === 'real' ? String(content.value) : '',
    flag: content?.type === 'boolean' ? content.value : false,
    choice: content?.type === 'choice' ? content.value : '',
    reference: content?.type === 'entity_reference' ? content.value : '',
    moment: content?.type === 'date' ? content.value : null,
    note: value?.note ?? '',
    validFrom: value?.valid_from ?? null,
    validUntil: value?.valid_until ?? null,
    isNew: value === null,
  };
}

const start = initial();
let text = $state(start.text);
let number = $state(start.number);
let flag = $state(start.flag);
let choice = $state(start.choice);
let reference = $state(start.reference);
let moment = $state<Instant | null>(start.moment);
let note = $state(start.note);
let validFrom = $state<Instant | null>(start.validFrom);
let validUntil = $state<Instant | null>(start.validUntil);
let showPeriod = $state(start.validFrom !== null || start.validUntil !== null);
let dirty = $state(start.isNew);
let busy = $state(false);

const content = $derived.by<PropertyContent | null>(() => {
  switch (data.type) {
    case 'text':
      return text.trim() === '' ? null : { type: 'text', value: text };
    case 'integer': {
      const parsed = Number.parseInt(number, 10);
      return number.trim() !== '' && Number.isFinite(parsed)
        ? { type: 'integer', value: parsed }
        : null;
    }
    case 'real': {
      const parsed = Number.parseFloat(number);
      return number.trim() !== '' && Number.isFinite(parsed)
        ? { type: 'real', value: parsed }
        : null;
    }
    case 'boolean':
      return { type: 'boolean', value: flag };
    case 'date':
      return moment ? { type: 'date', value: moment } : null;
    case 'choice':
      return choice ? { type: 'choice', value: choice } : null;
    case 'entity_reference':
      return reference ? { type: 'entity_reference', value: reference } : null;
  }
});

async function save() {
  if (!content) return;
  busy = true;
  try {
    await onsave({
      id: value?.id ?? null,
      owner,
      definition: definition.id,
      content,
      valid_from: validFrom,
      valid_until: validUntil,
      note: note.trim() === '' ? null : note,
      position: value?.position ?? position,
    });
    if (value === null) {
      text = '';
      number = '';
      choice = '';
      reference = '';
      moment = null;
      note = '';
      validFrom = null;
      validUntil = null;
    }
    dirty = value === null;
  } finally {
    busy = false;
  }
}

function touch() {
  dirty = true;
}
</script>

<div class="rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] p-2.5 flex flex-col gap-2">
  <div class="flex items-end gap-2">
    <div class="flex-1 min-w-0">
      {#if data.type === 'text'}
        <TextField label={t('properties.value')} bind:value={text} oninput={touch} />
      {:else if data.type === 'integer' || data.type === 'real'}
        <TextField label={t('properties.value')} bind:value={number} type="number" oninput={touch} mono />
      {:else if data.type === 'boolean'}
        <CheckField label={definitionLabel(definition)} bind:checked={flag} onchange={touch} />
      {:else if data.type === 'date'}
        <InstantField label={t('properties.value')} bind:value={moment} onchange={touch} />
      {:else if data.type === 'choice'}
        <SelectField
          label={t('properties.value')}
          bind:value={choice}
          options={data.options.map((option) => ({ value: option.key, label: option.label }))}
          placeholder={t('properties.choose')}
          onchange={touch}
        />
      {:else}
        <SelectField
          label={t('properties.value')}
          bind:value={reference}
          options={candidates.map((entity) => ({
            value: entity.id,
            label: `${entity.name} · ${typeLabelOf(entity)}`,
          }))}
          placeholder={t('properties.chooseEntity')}
          onchange={touch}
        />
      {/if}
    </div>
    <div class="flex items-center gap-1 pb-0.5">
      <Button
        size="sm"
        variant="ghost"
        icon="i-mingcute:history-line"
        title={t('properties.period')}
        onclick={() => (showPeriod = !showPeriod)}
      />
      {#if dirty}
        <Button
          size="sm"
          variant="primary"
          icon="i-mingcute:check-line"
          disabled={!content || busy}
          onclick={save}
        >
          {value === null ? t('common.add') : t('common.save')}
        </Button>
      {/if}
      {#if value && onremove}
        <Button
          size="sm"
          variant="ghost"
          icon="i-mingcute:delete-2-line"
          title={t('common.remove')}
          onclick={() => onremove(value)}
        />
      {/if}
    </div>
  </div>

  {#if showPeriod}
    <div class="grid gap-2 md:grid-cols-2">
      <InstantField label={t('properties.validFrom')} bind:value={validFrom} onchange={touch} />
      <InstantField label={t('properties.validUntil')} bind:value={validUntil} onchange={touch} />
    </div>
    <TextField label={t('properties.note')} bind:value={note} oninput={touch} />
  {/if}
</div>
