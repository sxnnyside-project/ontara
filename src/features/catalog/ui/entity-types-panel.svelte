<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import {
  Banner,
  Button,
  CheckField,
  Dialog,
  SelectField,
  TextField,
  entityIcon,
} from '@/shared/ui';
import type { EntityRole, EntityType } from '../api/catalog-api';
import { defineEntry, updateEntry } from '../state/catalog-actions';
import { catalog } from '../state/catalog.svelte';
import { isValidKey, keyFromLabel } from '../state/catalog-model';
import CatalogRow from './catalog-row.svelte';
import RemoveEntryDialog from './remove-entry-dialog.svelte';

let editing = $state<EntityType | null>(null);
let creating = $state(false);
let removing = $state<EntityType | null>(null);

let label = $state('');
let key = $state('');
let keyEdited = $state(false);
let description = $state('');
let parent = $state('');
let role = $state<EntityRole>('generic');
let isAbstract = $state(false);
let busy = $state(false);
let error = $state<string | null>(null);

const types = $derived(catalog.data.entity_types);
const parentType = $derived(types.find((entry) => entry.key === parent));

/** A type specializing a typed parent shares its role. */
const roleLocked = $derived(parentType !== undefined && parentType.role !== 'generic');

$effect(() => {
  if (parentType && parentType.role !== 'generic') role = parentType.role;
});

function begin(existing: EntityType | null) {
  editing = existing;
  creating = existing === null;
  label = existing?.label ?? '';
  key = existing?.key ?? '';
  keyEdited = existing !== null;
  description = existing?.description ?? '';
  parent = existing?.parent ?? '';
  role = existing?.role ?? 'generic';
  isAbstract = existing?.is_abstract ?? false;
  error = null;
}

function close() {
  editing = null;
  creating = false;
}

function onlabel() {
  if (!keyEdited) key = keyFromLabel(label);
}

async function save() {
  busy = true;
  error = null;
  try {
    if (editing) {
      await updateEntry({ catalog: 'entity_type', ...editing, label: label.trim(), description });
    } else {
      await defineEntry({
        catalog: 'entity_type',
        key,
        label: label.trim(),
        description,
        parent: parent || null,
        is_abstract: isAbstract,
        role,
        sort_order: types.length,
        builtin: false,
      });
    }
    close();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function depth(entry: EntityType): number {
  return catalog.chain(entry.key).length - 1;
}

const roleOptions = $derived(
  (['generic', 'event', 'editorial_unit', 'asset'] as const).map((value) => ({
    value,
    label: t(`roles.${value}`),
  })),
);

interface Props {
  /** Raised by the view's action each time the author asks for a new entry. */
  createSignal: number;
}

const { createSignal }: Props = $props();
let handled = 0;

$effect(() => {
  if (createSignal > handled) {
    handled = createSignal;
    begin(null);
  }
});
</script>


<div class="flex flex-col gap-3">
  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each types as entry (entry.key)}
      <CatalogRow
        title={entry.label}
        keyText={entry.key}
        description={entry.description}
        icon={entityIcon(catalog.chain(entry.key))}
        indent={depth(entry)}
        builtin={entry.builtin}
        onedit={() => begin(entry)}
        onremove={() => (removing = entry)}
      >
        {#snippet badges()}
          {#if entry.is_abstract}
            <span class="text-[10px] font-mono text-[var(--color-meridian-blue)]">{t('config.abstract')}</span>
          {/if}
          {#if entry.role !== 'generic'}
            <span class="text-[10px] font-mono text-[var(--color-canon-provisional)]">{t(`roles.${entry.role}`)}</span>
          {/if}
        {/snippet}
      </CatalogRow>
    {/each}
  </div>
</div>

{#if creating || editing}
  <Dialog
    title={editing ? t('config.editType') : t('config.newType')}
    description={editing?.builtin ? t('config.builtinEditHint') : undefined}
    onclose={close}
  >
    {#if error}<Banner kind="error">{error}</Banner>{/if}
    <TextField label={t('config.label')} bind:value={label} oninput={onlabel} focus />
    {#if creating}
      <TextField
        label={t('config.key')}
        bind:value={key}
        oninput={() => (keyEdited = true)}
        mono
        hint={t('config.keyHint')}
        error={key !== '' && !isValidKey(key) ? t('config.keyInvalid') : null}
      />
    {/if}
    <TextField label={t('config.description')} bind:value={description} multiline rows={3} />
    {#if creating}
      <SelectField
        label={t('config.parentType')}
        bind:value={parent}
        placeholder={t('config.noParent')}
        options={types.map((entry) => ({ value: entry.key, label: entry.label }))}
        hint={t('config.parentHint')}
      />
      <SelectField label={t('config.role')} bind:value={role} options={roleOptions} disabled={roleLocked} hint={t('config.roleHint')} />
      <CheckField label={t('config.abstractType')} bind:checked={isAbstract} hint={t('config.abstractHint')} />
    {/if}
    {#snippet actions()}
      <Button variant="ghost" onclick={close}>{t('common.cancel')}</Button>
      <Button
        variant="primary"
        disabled={busy || label.trim() === '' || (creating && !isValidKey(key))}
        onclick={save}
      >
        {t('common.save')}
      </Button>
    {/snippet}
  </Dialog>
{/if}

{#if removing}
  <RemoveEntryDialog
    entry={{ catalog: 'entity_type', key: removing.key }}
    name={removing.label}
    onclose={() => (removing = null)}
  />
{/if}
