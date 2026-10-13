<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Button, EmptyState, TextField } from '@/shared/ui';
import { type EntityDetail, type Note, removeNote, saveNote } from '../api/entity-api';

interface Props {
  detail: EntityDetail;
}

const { detail }: Props = $props();

let editing = $state<{ id: string | null; title: string; body: string } | null>(null);
let busy = $state(false);

function begin(note: Note | null) {
  editing = { id: note?.id ?? null, title: note?.title ?? '', body: note?.body ?? '' };
}

async function save() {
  if (!editing) return;
  busy = true;
  try {
    await saveNote({
      id: editing.id,
      entity: detail.entity.id,
      title: editing.title.trim(),
      body: editing.body,
    });
    editing = null;
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  } finally {
    busy = false;
  }
}

async function remove(note: Note) {
  try {
    await removeNote(note.id);
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}
</script>

<div class="flex flex-col gap-4 max-w-2xl">
  <div>
    <Button icon="i-mingcute:add-line" onclick={() => begin(null)}>{t('notes.add')}</Button>
  </div>

  {#if editing}
    <div class="rounded border border-[var(--color-border-strong)] bg-[var(--color-surface-panel)] p-4 flex flex-col gap-3">
      <TextField label={t('notes.title')} bind:value={editing.title} focus />
      <TextField label={t('notes.body')} bind:value={editing.body} multiline rows={8} hint={t('notes.markdown')} />
      <div class="flex gap-2">
        <Button variant="primary" disabled={busy || editing.title.trim() === ''} onclick={save}>
          {t('common.save')}
        </Button>
        <Button variant="ghost" onclick={() => (editing = null)}>{t('common.cancel')}</Button>
      </div>
    </div>
  {/if}

  {#each detail.notes as note (note.id)}
    <article class="rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-4 flex flex-col gap-2">
      <div class="flex items-center gap-2">
        <h3 class="m-0 flex-1 text-sm font-serif font-medium">{note.title}</h3>
        <Button size="sm" variant="ghost" icon="i-mingcute:edit-line" title={t('common.edit')} onclick={() => begin(note)} />
        <Button size="sm" variant="ghost" icon="i-mingcute:delete-2-line" title={t('common.remove')} onclick={() => remove(note)} />
      </div>
      <p class="m-0 text-xs text-[var(--color-text-secondary)] leading-relaxed whitespace-pre-wrap select-text">{note.body}</p>
    </article>
  {:else}
    {#if !editing}
      <EmptyState icon="i-mingcute:quill-pen-line" title={t('notes.noneTitle')} description={t('notes.noneHint')} />
    {/if}
  {/each}
</div>
