<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Button, CheckField, SectionHeading, SelectField, TagInput, TextField } from '@/shared/ui';
import { type EntityDetail, updateEntity } from '../api/entity-api';
import { draftOf, editOf, NAME_KINDS, newAlternativeName, sameDraft } from '../state/entity-model';

interface Props {
  detail: EntityDetail;
}

const { detail }: Props = $props();

function initialDraft() {
  return draftOf(detail);
}

let draft = $state(initialDraft());
let saved = $state(initialDraft());
let busy = $state(false);
let error = $state<string | null>(null);

const dirty = $derived(!sameDraft(draft, saved));

// A different entity, or a save made elsewhere, replaces the form's baseline.
$effect(() => {
  const fresh = draftOf(detail);
  saved = fresh;
  draft = structuredClone(fresh);
});

async function save() {
  busy = true;
  error = null;
  try {
    await updateEntity(editOf(detail, draft));
    notices.success(t('entity.saved'));
    revision.bump();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function reset() {
  draft = structuredClone(saved);
  error = null;
}

function addName() {
  draft.names = [...draft.names, newAlternativeName()];
}

function removeName(index: number) {
  draft.names = draft.names.filter((_, position) => position !== index);
}
</script>

<form
  class="grid gap-6 grid-cols-[minmax(0,1fr)_18rem] items-start"
  onsubmit={(event) => {
    event.preventDefault();
    save();
  }}
>
  <div class="flex flex-col gap-3 min-w-0">
    <TextField label={t('entity.description')} bind:value={draft.description} multiline rows={16} placeholder={t('entity.descriptionPlaceholder')} />
    {#if error}
      <p class="m-0 text-xs font-mono text-[var(--color-finding-error)]" role="alert">{error}</p>
    {/if}
    <div class="flex items-center gap-2">
      <Button type="submit" variant="primary" icon="i-mingcute:check-line" disabled={!dirty || busy || draft.name.trim() === ''}>
        {t('common.save')}
      </Button>
      <Button variant="ghost" disabled={!dirty || busy} onclick={reset}>{t('common.discard')}</Button>
      {#if dirty}
        <span class="text-[11px] font-mono text-[var(--color-canon-provisional)]">{t('common.unsaved')}</span>
      {/if}
    </div>
  </div>

  <aside class="flex flex-col gap-5 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-4">
    <TagInput label={t('entity.tags')} bind:value={draft.tags} placeholder={t('entity.tagsPlaceholder')} />

    <section class="flex flex-col gap-2">
      <SectionHeading title={t('inspector.aliases')}>
        {#snippet actions()}
          <Button size="sm" variant="ghost" icon="i-mingcute:add-line" title={t('common.add')} onclick={addName} />
        {/snippet}
      </SectionHeading>
      {#each draft.names as name, index}
        <div class="flex flex-col gap-1.5 rounded border border-[var(--color-border-subtle)] p-2">
          <TextField label={t('entity.alternativeName')} bind:value={name.name} />
          <div class="flex items-end gap-1.5">
            <div class="flex-1">
              <SelectField
                label={t('entity.nameKind')}
                bind:value={name.kind}
                options={NAME_KINDS.map((kind) => ({ value: kind, label: t(`nameKinds.${kind}`) }))}
              />
            </div>
            <Button variant="ghost" icon="i-mingcute:delete-2-line" title={t('common.remove')} onclick={() => removeName(index)} />
          </div>
        </div>
      {:else}
        <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('entity.noAlternativeNames')}</p>
      {/each}
    </section>

    <details class="text-xs">
      <summary class="cursor-pointer select-none text-[var(--color-text-secondary)]">{t('entity.moreOptions')}</summary>
      <div class="mt-3 flex flex-col gap-3">
        <TextField label={t('entity.sortName')} bind:value={draft.sort_name} hint={t('entity.sortNameHint')} />
        <CheckField label={t('entity.archived')} bind:checked={draft.archived} hint={t('entity.archivedHint')} />
      </div>
    </details>
  </aside>
</form>
