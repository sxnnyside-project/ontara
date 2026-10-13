<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { Banner, Button, CheckField, Dialog, SelectField, TextField } from '@/shared/ui';
import type { RelationshipType } from '../api/catalog-api';
import { defineEntry, updateEntry } from '../state/catalog-actions';
import { catalog } from '../state/catalog.svelte';
import { isValidKey, keyFromLabel } from '../state/catalog-model';
import CatalogRow from './catalog-row.svelte';
import RemoveEntryDialog from './remove-entry-dialog.svelte';
import TypeChecklist from './type-checklist.svelte';

let editing = $state<RelationshipType | null>(null);
let creating = $state(false);
let removing = $state<RelationshipType | null>(null);

let label = $state('');
let key = $state('');
let keyEdited = $state(false);
let inverse = $state('');
let description = $state('');
let directionality = $state<'directed' | 'undirected'>('directed');
let acyclic = $state(false);
let sources = $state<string[]>([]);
let targets = $state<string[]>([]);
let busy = $state(false);
let error = $state<string | null>(null);

const typeOptions = $derived(
  catalog.data.entity_types.map((entry) => ({ key: entry.key, label: entry.label })),
);

function begin(existing: RelationshipType | null) {
  editing = existing;
  creating = existing === null;
  label = existing?.label ?? '';
  key = existing?.key ?? '';
  keyEdited = existing !== null;
  inverse = existing?.inverse_label ?? '';
  description = existing?.description ?? '';
  directionality = existing?.directionality ?? 'directed';
  acyclic = existing?.acyclic ?? false;
  sources = [...(existing?.allowed_sources ?? [])];
  targets = [...(existing?.allowed_targets ?? [])];
  error = null;
}

function close() {
  editing = null;
  creating = false;
}

async function save() {
  busy = true;
  error = null;
  try {
    const undirected = directionality === 'undirected';
    const record = {
      catalog: 'relationship_type' as const,
      key,
      label: label.trim(),
      inverse_label: undirected || inverse.trim() === '' ? null : inverse.trim(),
      description,
      directionality,
      acyclic: undirected ? false : acyclic,
      allowed_sources: sources,
      allowed_targets: targets,
      sort_order: editing?.sort_order ?? catalog.data.relationship_types.length,
      builtin: false,
    };
    if (editing) await updateEntry(record);
    else await defineEntry(record);
    close();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function endpointText(keys: string[]): string {
  return keys.length === 0
    ? t('config.anyType')
    : keys.map((entry) => catalog.typeLabel(entry)).join(', ');
}
</script>

<div class="flex flex-col gap-3">
  <div class="flex items-center justify-between">
    <p class="m-0 text-xs text-[var(--color-text-secondary)] max-w-xl">{t('config.relationshipTypesHint')}</p>
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => begin(null)}>{t('config.newRelationshipType')}</Button>
  </div>

  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each catalog.data.relationship_types as entry (entry.key)}
      <CatalogRow
        title={entry.inverse_label ? `${entry.label}  /  ${entry.inverse_label}` : entry.label}
        keyText={entry.key}
        description={`${endpointText(entry.allowed_sources)}  →  ${endpointText(entry.allowed_targets)}`}
        icon="i-mingcute:git-branch-line"
        builtin={entry.builtin}
        onedit={() => begin(entry)}
        onremove={() => (removing = entry)}
      >
        {#snippet badges()}
          {#if entry.directionality === 'undirected'}
            <span class="text-[10px] font-mono text-[var(--color-meridian-directed)]">{t('config.undirected')}</span>
          {/if}
          {#if entry.acyclic}
            <span class="text-[10px] font-mono text-[var(--color-canon-provisional)]">{t('config.hierarchy')}</span>
          {/if}
        {/snippet}
      </CatalogRow>
    {/each}
  </div>
</div>

{#if creating || editing}
  <Dialog title={editing ? t('config.editRelationshipType') : t('config.newRelationshipType')} width="lg" onclose={close}>
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
      <SelectField
        label={t('config.direction')}
        bind:value={directionality}
        options={[
          { value: 'directed', label: t('config.directed') },
          { value: 'undirected', label: t('config.undirected') },
        ]}
      />
    {/if}
    {#if directionality === 'directed'}
      <TextField label={t('config.inverseLabel')} bind:value={inverse} hint={t('config.inverseHint')} />
      {#if creating}
        <CheckField label={t('config.hierarchical')} bind:checked={acyclic} hint={t('config.hierarchicalHint')} />
      {/if}
    {/if}
    <TextField label={t('config.description')} bind:value={description} multiline rows={2} />
    <div class="grid gap-3 md:grid-cols-2">
      <TypeChecklist
        label={t('config.allowedSources')}
        hint={editing ? t('config.widenOnly') : undefined}
        options={typeOptions}
        bind:value={sources}
        locked={editing?.allowed_sources ?? []}
      />
      <TypeChecklist
        label={t('config.allowedTargets')}
        hint={editing ? t('config.widenOnly') : undefined}
        options={typeOptions}
        bind:value={targets}
        locked={editing?.allowed_targets ?? []}
      />
    </div>
    {#snippet actions()}
      <Button variant="ghost" onclick={close}>{t('common.cancel')}</Button>
      <Button
        variant="primary"
        disabled={busy || label.trim() === '' || !isValidKey(key)}
        onclick={save}
      >
        {t('common.save')}
      </Button>
    {/snippet}
  </Dialog>
{/if}

{#if removing}
  <RemoveEntryDialog
    entry={{ catalog: 'relationship_type', key: removing.key }}
    name={removing.label}
    onclose={() => (removing = null)}
  />
{/if}
