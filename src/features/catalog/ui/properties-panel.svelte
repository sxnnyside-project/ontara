<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { ChoiceOption, PropertyDataType, PropertyDefinition } from '@/shared/model';
import { Banner, Button, CheckField, Dialog, SelectField, TextField } from '@/shared/ui';
import { defineEntry, updateEntry } from '../state/catalog-actions';
import { catalog } from '../state/catalog.svelte';
import { isValidKey, keyFromLabel } from '../state/catalog-model';
import CatalogRow from './catalog-row.svelte';
import RemoveEntryDialog from './remove-entry-dialog.svelte';

/** `entity_type:<key>` or `relationship_type:<key>`: what the shown properties belong to. */
let ownerChoice = $state('entity_type:character');
let editing = $state<PropertyDefinition | null>(null);
let creating = $state(false);
let removing = $state<PropertyDefinition | null>(null);

let label = $state('');
let key = $state('');
let keyEdited = $state(false);
let description = $state('');
let group = $state('');
let dataType = $state<PropertyDataType['type']>('text');
let referenceType = $state('');
let optionsText = $state('');
let required = $state(false);
let multiple = $state(false);
let busy = $state(false);
let error = $state<string | null>(null);

let groupEditing = $state<{ key: string; label: string; sort_order: number } | null>(null);
let groupDraft = $state<{ key: string; label: string; sort_order: number } | null>(null);
let removingGroup = $state<{ key: string; label: string } | null>(null);

function beginGroup(existing: { key: string; label: string; sort_order: number } | null) {
  groupEditing = existing;
  groupDraft = existing
    ? { ...existing }
    : { key: '', label: '', sort_order: catalog.data.property_groups.length };
  error = null;
}

async function saveGroup() {
  if (!groupDraft) return;
  busy = true;
  error = null;
  try {
    const record = {
      catalog: 'property_group' as const,
      ...groupDraft,
      label: groupDraft.label.trim(),
    };
    if (groupEditing) await updateEntry(record);
    else await defineEntry(record);
    groupDraft = null;
    groupEditing = null;
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

const ownerOptions = $derived([
  ...catalog.data.entity_types.map((entry) => ({
    value: `entity_type:${entry.key}`,
    label: entry.label,
    group: t('config.ownerEntityTypes'),
  })),
  ...catalog.data.relationship_types.map((entry) => ({
    value: `relationship_type:${entry.key}`,
    label: entry.label,
    group: t('config.ownerRelationshipTypes'),
  })),
]);

const owner = $derived.by(() => {
  const [kind, ...rest] = ownerChoice.split(':');
  return { kind: kind as 'entity_type' | 'relationship_type', key: rest.join(':') };
});

/** The properties of the chosen type, with the ones it inherits listed first. */
const definitions = $derived.by(() => {
  const ownerKeys =
    owner.kind === 'entity_type' ? new Set(catalog.chain(owner.key)) : new Set([owner.key]);
  return catalog.data.property_definitions
    .filter(
      (definition) => definition.owner.kind === owner.kind && ownerKeys.has(definition.owner.key),
    )
    .sort((a, b) => a.sort_order - b.sort_order);
});

const isChoice = $derived(dataType === 'choice');

function optionsFromText(text: string, previous: ChoiceOption[]): ChoiceOption[] {
  const known = new Map(previous.map((option) => [option.label, option.key]));
  return text
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== '')
    .map((line) => ({ key: known.get(line) ?? keyFromLabel(line), label: line }));
}

function begin(existing: PropertyDefinition | null) {
  editing = existing;
  creating = existing === null;
  label = existing?.label ?? '';
  key = existing?.key ?? '';
  keyEdited = existing !== null;
  description = existing?.description ?? '';
  group = existing?.group ?? '';
  dataType = existing?.data_type.type ?? 'text';
  referenceType =
    existing?.data_type.type === 'entity_reference' ? (existing.data_type.entity_type ?? '') : '';
  optionsText =
    existing?.data_type.type === 'choice'
      ? existing.data_type.options.map((option) => option.label).join('\n')
      : '';
  required = existing?.required ?? false;
  multiple = existing?.multiple ?? false;
  error = null;
}

function close() {
  editing = null;
  creating = false;
}

function builtType(previous: PropertyDefinition | null): PropertyDataType {
  switch (dataType) {
    case 'choice': {
      const before = previous?.data_type.type === 'choice' ? previous.data_type.options : [];
      return { type: 'choice', options: optionsFromText(optionsText, before) };
    }
    case 'entity_reference':
      return { type: 'entity_reference', entity_type: referenceType || null };
    default:
      return { type: dataType } as PropertyDataType;
  }
}

async function save() {
  busy = true;
  error = null;
  try {
    const base = {
      catalog: 'property_definition' as const,
      key,
      label: label.trim(),
      description,
      group: group || null,
      required,
      multiple,
      builtin: false,
    };
    if (editing) {
      await updateEntry({
        ...base,
        id: editing.id,
        owner: editing.owner,
        key: editing.key,
        data_type: builtType(editing),
        required: editing.required,
        multiple: editing.multiple,
        sort_order: editing.sort_order,
      });
    } else {
      await defineEntry({
        ...base,
        id: crypto.randomUUID(),
        owner: { kind: owner.kind, key: owner.key },
        data_type: builtType(null),
        sort_order: definitions.length,
      });
    }
    close();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function typeLabel(data: PropertyDataType): string {
  return t(`dataTypes.${data.type}`);
}

const dataTypeOptions = $derived(
  (['text', 'integer', 'real', 'boolean', 'date', 'choice', 'entity_reference'] as const).map(
    (value) => ({ value, label: t(`dataTypes.${value}`) }),
  ),
);
</script>

<div class="flex flex-col gap-3">
  <p class="m-0 text-xs text-[var(--color-text-secondary)] max-w-xl">{t('config.propertiesHint')}</p>
  <div class="flex items-end justify-between gap-3">
    <div class="w-80">
      <SelectField label={t('config.propertiesOf')} bind:value={ownerChoice} options={ownerOptions} />
    </div>
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => begin(null)}>{t('config.newProperty')}</Button>
  </div>

  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each definitions as definition (definition.id)}
      <CatalogRow
        title={definition.label}
        keyText={definition.key}
        description={definition.description}
        builtin={definition.builtin}
        onedit={() => begin(definition)}
        onremove={() => (removing = definition)}
      >
        {#snippet badges()}
          <span class="text-[10px] font-mono text-[var(--color-meridian-blue)]">{typeLabel(definition.data_type)}</span>
          {#if definition.multiple}
            <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{t('config.multiple')}</span>
          {/if}
          {#if definition.owner.key !== owner.key}
            <span class="text-[10px] font-mono text-[var(--color-text-muted)]">
              {t('config.inheritedFrom', { type: catalog.typeLabel(definition.owner.key) })}
            </span>
          {/if}
        {/snippet}
      </CatalogRow>
    {:else}
      <p class="m-3 text-xs text-[var(--color-text-muted)]">{t('config.noProperties')}</p>
    {/each}
  </div>
</div>

<div class="mt-8 flex flex-col gap-2">
  <div class="flex items-center justify-between">
    <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('config.groups')}
    </h3>
    <Button size="sm" icon="i-mingcute:add-line" onclick={() => beginGroup(null)}>{t('config.newGroup')}</Button>
  </div>
  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each catalog.data.property_groups as entry (entry.key)}
      <CatalogRow
        title={entry.label}
        keyText={entry.key}
        onedit={() => beginGroup(entry)}
        onremove={() => (removingGroup = entry)}
      />
    {/each}
  </div>
</div>

{#if groupDraft}
  <Dialog title={groupEditing ? t('config.editGroup') : t('config.newGroup')} width="sm" onclose={() => (groupDraft = null)}>
    {#if error}<Banner kind="error">{error}</Banner>{/if}
    <TextField
      label={t('config.label')}
      bind:value={groupDraft.label}
      oninput={() => {
        if (groupDraft && !groupEditing) groupDraft.key = keyFromLabel(groupDraft.label);
      }}
      focus
    />
    {#snippet actions()}
      <Button variant="ghost" onclick={() => (groupDraft = null)}>{t('common.cancel')}</Button>
      <Button
        variant="primary"
        disabled={busy || groupDraft === null || groupDraft.label.trim() === '' || !isValidKey(groupDraft.key)}
        onclick={saveGroup}
      >
        {t('common.save')}
      </Button>
    {/snippet}
  </Dialog>
{/if}

{#if removingGroup}
  <RemoveEntryDialog
    entry={{ catalog: 'property_group', key: removingGroup.key }}
    name={removingGroup.label}
    onclose={() => (removingGroup = null)}
  />
{/if}

{#if creating || editing}
  <Dialog title={editing ? t('config.editProperty') : t('config.newProperty')} onclose={close}>
    {#if error}<Banner kind="error">{error}</Banner>{/if}
    <TextField
      label={t('config.label')}
      bind:value={label}
      oninput={() => {
        if (!keyEdited) key = keyFromLabel(label);
      }}
      focus
    />
    {#if creating}
      <TextField
        label={t('config.key')}
        bind:value={key}
        oninput={() => (keyEdited = true)}
        mono
        hint={t('config.keyHint')}
        error={key !== '' && !isValidKey(key) ? t('config.keyInvalid') : null}
      />
      <SelectField label={t('config.dataType')} bind:value={dataType as string} options={dataTypeOptions} />
    {/if}
    {#if dataType === 'entity_reference' && creating}
      <SelectField
        label={t('config.referenceType')}
        bind:value={referenceType}
        placeholder={t('config.anyType')}
        options={catalog.data.entity_types.map((entry) => ({ value: entry.key, label: entry.label }))}
      />
    {/if}
    {#if isChoice}
      <TextField
        label={t('config.options')}
        bind:value={optionsText}
        multiline
        rows={5}
        hint={editing ? t('config.optionsGrowOnly') : t('config.optionsHint')}
      />
    {/if}
    <TextField label={t('config.description')} bind:value={description} multiline rows={2} />
    <SelectField
      label={t('config.group')}
      bind:value={group}
      placeholder={t('config.noGroup')}
      options={catalog.data.property_groups.map((entry) => ({ value: entry.key, label: entry.label }))}
    />
    {#if creating}
      <CheckField label={t('config.multipleValues')} bind:checked={multiple} hint={t('config.multipleHint')} />
    {/if}
    {#snippet actions()}
      <Button variant="ghost" onclick={close}>{t('common.cancel')}</Button>
      <Button variant="primary" disabled={busy || label.trim() === '' || !isValidKey(key)} onclick={save}>
        {t('common.save')}
      </Button>
    {/snippet}
  </Dialog>
{/if}

{#if removing}
  <RemoveEntryDialog
    entry={{ catalog: 'property_definition', id: removing.id }}
    name={removing.label}
    onclose={() => (removing = null)}
  />
{/if}
