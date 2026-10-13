<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { Banner, Button, Dialog, TextField } from '@/shared/ui';
import type { EditorialLevel } from '../api/catalog-api';
import { defineEntry, updateEntry } from '../state/catalog-actions';
import { catalog } from '../state/catalog.svelte';
import { isValidKey, keyFromLabel } from '../state/catalog-model';
import CatalogRow from './catalog-row.svelte';
import RemoveEntryDialog from './remove-entry-dialog.svelte';

let editing = $state<EditorialLevel | null>(null);
let creating = $state(false);
let removing = $state<EditorialLevel | null>(null);

let label = $state('');
let key = $state('');
let keyEdited = $state(false);
let depth = $state('');
let busy = $state(false);
let error = $state<string | null>(null);

const levels = $derived([...catalog.data.editorial_levels].sort((a, b) => a.depth - b.depth));

function begin(existing: EditorialLevel | null) {
  editing = existing;
  creating = existing === null;
  label = existing?.label ?? '';
  key = existing?.key ?? '';
  keyEdited = existing !== null;
  depth = String(existing?.depth ?? (levels.at(-1)?.depth ?? -1) + 1);
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
      catalog: 'editorial_level' as const,
      key,
      label: label.trim(),
      depth: Number.parseInt(depth, 10),
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
    <p class="m-0 text-xs text-[var(--color-text-secondary)] max-w-xl">{t('config.levelsHint')}</p>
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => begin(null)}>{t('config.newLevel')}</Button>
  </div>

  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each levels as entry (entry.key)}
      <CatalogRow
        title={entry.label}
        keyText={entry.key}
        icon="i-mingcute:book-2-line"
        indent={entry.depth}
        builtin={entry.builtin}
        onedit={() => begin(entry)}
        onremove={() => (removing = entry)}
      >
        {#snippet badges()}
          <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{t('config.depth', { depth: entry.depth })}</span>
        {/snippet}
      </CatalogRow>
    {/each}
  </div>
</div>

{#if creating || editing}
  <Dialog title={editing ? t('config.editLevel') : t('config.newLevel')} width="sm" onclose={close}>
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
        error={key !== '' && !isValidKey(key) ? t('config.keyInvalid') : null}
      />
      <TextField label={t('config.levelDepth')} bind:value={depth} type="number" mono hint={t('config.levelDepthHint')} />
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
    entry={{ catalog: 'editorial_level', key: removing.key }}
    name={removing.label}
    onclose={() => (removing = null)}
  />
{/if}
