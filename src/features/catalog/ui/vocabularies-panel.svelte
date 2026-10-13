<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { Banner, Button, Dialog, SelectField, TextField } from '@/shared/ui';
import type { Vocabulary, VocabularyTerm } from '../api/catalog-api';
import { defineEntry, updateEntry } from '../state/catalog-actions';
import { catalog } from '../state/catalog.svelte';
import { isValidKey, keyFromLabel } from '../state/catalog-model';
import CatalogRow from './catalog-row.svelte';
import RemoveEntryDialog from './remove-entry-dialog.svelte';

const VOCABULARIES: readonly Vocabulary[] = [
  'trait_kind',
  'family_tie_kind',
  'partnership_kind',
  'containment_kind',
  'citizenship_status',
  'office_acquisition',
  'item_acquisition',
];

let vocabulary = $state<Vocabulary>('trait_kind');
let editing = $state<VocabularyTerm | null>(null);
let creating = $state(false);
let removing = $state<VocabularyTerm | null>(null);

let label = $state('');
let key = $state('');
let keyEdited = $state(false);
let description = $state('');
let busy = $state(false);
let error = $state<string | null>(null);

const terms = $derived(
  catalog.data.vocabulary_terms
    .filter((term) => term.vocabulary === vocabulary)
    .sort((a, b) => a.sort_order - b.sort_order),
);

function begin(existing: VocabularyTerm | null) {
  editing = existing;
  creating = existing === null;
  label = existing?.label ?? '';
  key = existing?.key ?? '';
  keyEdited = existing !== null;
  description = existing?.description ?? '';
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
      catalog: 'vocabulary_term' as const,
      vocabulary,
      key,
      label: label.trim(),
      description,
      sort_order: editing?.sort_order ?? terms.length,
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
  <div class="flex items-end justify-between gap-3">
    <div class="w-72">
      <SelectField
        label={t('config.vocabulary')}
        bind:value={vocabulary as string}
        options={VOCABULARIES.map((entry) => ({ value: entry, label: t(`vocabularies.${entry}`) }))}
      />
    </div>
  </div>

  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]">
    {#each terms as term (term.key)}
      <CatalogRow
        title={term.label}
        keyText={term.key}
        description={term.description}
        builtin={term.builtin}
        onedit={() => begin(term)}
        onremove={() => (removing = term)}
      />
    {/each}
  </div>
</div>

{#if creating || editing}
  <Dialog title={editing ? t('config.editTerm') : t('config.newTerm')} width="sm" onclose={close}>
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
    {/if}
    <TextField label={t('config.description')} bind:value={description} multiline rows={3} />
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
    entry={{ catalog: 'vocabulary_term', vocabulary: removing.vocabulary, key: removing.key }}
    name={removing.label}
    onclose={() => (removing = null)}
  />
{/if}
