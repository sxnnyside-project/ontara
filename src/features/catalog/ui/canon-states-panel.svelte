<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { Banner, Button, CanonBadge, CheckField, Dialog, TextField } from '@/shared/ui';
import type { CanonState } from '../api/catalog-api';
import { defineEntry, updateEntry } from '../state/catalog-actions';
import { catalog } from '../state/catalog.svelte';
import { isValidKey, keyFromLabel } from '../state/catalog-model';
import CatalogRow from './catalog-row.svelte';
import RemoveEntryDialog from './remove-entry-dialog.svelte';

let editing = $state<CanonState | null>(null);
let creating = $state(false);
let removing = $state<CanonState | null>(null);

let label = $state('');
let key = $state('');
let keyEdited = $state(false);
let description = $state('');
let countsAsCanon = $state(true);
let busy = $state(false);
let error = $state<string | null>(null);

function begin(existing: CanonState | null) {
  editing = existing;
  creating = existing === null;
  label = existing?.label ?? '';
  key = existing?.key ?? '';
  keyEdited = existing !== null;
  description = existing?.description ?? '';
  countsAsCanon = existing?.counts_as_canon ?? true;
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
    const record = {
      catalog: 'canon_state' as const,
      key,
      label: label.trim(),
      description,
      counts_as_canon: countsAsCanon,
      sort_order: editing?.sort_order ?? catalog.data.canon_states.length,
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
</script>

<div class="flex flex-col gap-3">
  <div class="flex items-center justify-between">
    <p class="m-0 text-xs text-[var(--color-text-secondary)] max-w-xl">{t('config.canonStatesHint')}</p>
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => begin(null)}>{t('config.newCanonState')}</Button>
  </div>

  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each catalog.data.canon_states as entry (entry.key)}
      <CatalogRow
        title={entry.label}
        keyText={entry.key}
        description={entry.description}
        builtin={entry.builtin}
        onedit={() => begin(entry)}
        onremove={() => (removing = entry)}
      >
        {#snippet badges()}
          <CanonBadge state={entry.key} label={entry.label} />
          {#if !entry.counts_as_canon}
            <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{t('config.notCanon')}</span>
          {/if}
        {/snippet}
      </CatalogRow>
    {/each}
  </div>
</div>

{#if creating || editing}
  <Dialog title={editing ? t('config.editCanonState') : t('config.newCanonState')} onclose={close}>
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
    {/if}
    <TextField label={t('config.description')} bind:value={description} multiline rows={3} />
    <CheckField label={t('config.countsAsCanon')} bind:checked={countsAsCanon} hint={t('config.countsAsCanonHint')} />
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
    entry={{ catalog: 'canon_state', key: removing.key }}
    name={removing.label}
    onclose={() => (removing = null)}
  />
{/if}
