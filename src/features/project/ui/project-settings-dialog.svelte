<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { Banner, Button, Dialog, TextField } from '@/shared/ui';
import { createProjectBackup, renameProject, updateUniverse } from '../api/project-api';
import { project } from '../state/project.svelte';

interface Props {
  onclose: () => void;
  /** Opens the backups and export dialog. */
  ondata?: (() => void) | undefined;
}

const { onclose, ondata }: Props = $props();

const current = project.current;
let name = $state(current?.name ?? '');
let summary = $state(current?.universe.summary ?? '');
let description = $state(current?.universe.description ?? '');
let busy = $state(false);
let error = $state<string | null>(null);

async function save() {
  busy = true;
  error = null;
  try {
    let updated = await renameProject(name);
    updated = await updateUniverse(summary, description);
    project.refresh(updated);
    onclose();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

async function backup() {
  try {
    const path = await createProjectBackup();
    notices.success(t('project.backupDone', { path }));
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}
</script>

<Dialog title={t('project.settingsTitle')} {onclose}>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  <TextField label={t('project.name')} bind:value={name} />
  <TextField label={t('project.premise')} bind:value={summary} hint={t('project.premiseHint')} />
  <TextField label={t('project.description')} bind:value={description} multiline rows={6} />
  <div class="flex flex-col gap-1.5">
    <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('project.folder')}
    </span>
    <span class="text-xs font-mono text-[var(--color-text-secondary)] break-all select-text">
      {current?.path}
    </span>
    <div class="flex items-center gap-2">
      <Button size="sm" icon="i-mingcute:safe-flash-line" onclick={backup}>
        {t('project.backup')}
      </Button>
      {#if ondata}
        <Button size="sm" variant="ghost" onclick={ondata}>{t('data.open')}</Button>
      {/if}
    </div>
  </div>
  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy || name.trim() === ''} onclick={save}>
      {t('common.save')}
    </Button>
  {/snippet}
</Dialog>
