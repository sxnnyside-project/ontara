<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { getLocale, t, tDynamic } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { Banner, Button, ConfirmDialog, Dialog, SectionHeading, Segmented } from '@/shared/ui';
import { type BackupSummary, listBackups, restoreProjectBackup } from '../api/backup-api';
import { type ExportFormat, type ExportWords, exportUniverse } from '../api/export-api';
import { createProjectBackup, type ProjectSummary } from '../api/project-api';

interface Props {
  /** Called after a backup was restored, when everything shown must be read again. */
  onrestored: (summary: ProjectSummary) => void;
  onclose: () => void;
}

const { onrestored, onclose }: Props = $props();

let backups = $state<BackupSummary[]>([]);
let failure = $state<string | null>(null);
let busy = $state(false);
let restoring = $state<BackupSummary | null>(null);
let format = $state<ExportFormat>('markdown');
let written = $state<string | null>(null);

async function load() {
  try {
    backups = await listBackups();
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

$effect(() => {
  void load();
});

function when(millis: number): string {
  return new Date(millis).toLocaleString(getLocale(), { dateStyle: 'medium', timeStyle: 'short' });
}

function size(bytes: number): string {
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function kindText(kind: string): string {
  const base = kind.startsWith('pre-migration') ? 'pre-migration' : kind;
  return tDynamic(`data.kind_${base}`, kind);
}

async function backupNow() {
  busy = true;
  try {
    await createProjectBackup();
    await load();
  } catch (error) {
    failure = toCommandError(error).message;
  } finally {
    busy = false;
  }
}

async function restore() {
  const chosen = restoring;
  if (!chosen) return;
  busy = true;
  try {
    const summary = await restoreProjectBackup(chosen.name);
    restoring = null;
    notices.success(t('data.restored'));
    onrestored(summary);
  } catch (error) {
    restoring = null;
    failure = toCommandError(error).message;
  } finally {
    busy = false;
  }
}

/** The words of the export, in the interface language and with the names the author chose. */
function words(): ExportWords {
  const data = catalog.data;
  return {
    contents: t('data.contents'),
    summary: t('data.summary'),
    description: t('data.description'),
    tags: t('data.tags'),
    aliases: t('data.aliases'),
    properties: t('data.properties'),
    relationships: t('data.relationships'),
    notes: t('data.notes'),
    structure: t('data.structure'),
    canon: t('data.canon'),
    types: Object.fromEntries(data.entity_types.map((entry) => [entry.key, entry.label])),
    relationship_types: Object.fromEntries(
      data.relationship_types.map((entry) => [
        entry.key,
        { label: entry.label, inverse: entry.inverse_label },
      ]),
    ),
    canon_states: Object.fromEntries(data.canon_states.map((entry) => [entry.key, entry.label])),
    property_names: Object.fromEntries(
      data.property_definitions.map((entry) => [entry.id, entry.label]),
    ),
  };
}

async function runExport() {
  busy = true;
  written = null;
  try {
    const file = await exportUniverse(format, words());
    written = t('data.exported', { entities: file.entities, path: file.path });
  } catch (error) {
    failure = toCommandError(error).message;
  } finally {
    busy = false;
  }
}

const formats = $derived([
  { value: 'markdown', label: t('data.formatMarkdown') },
  { value: 'json', label: t('data.formatJson') },
]);
</script>

<Dialog title={t('data.title')} width="lg" {onclose}>
  {#if failure}<Banner kind="error">{failure}</Banner>{/if}

  <section class="flex flex-col gap-3">
    <SectionHeading title={t('data.backups')} count={backups.length}>
      {#snippet actions()}
        <Button size="sm" disabled={busy} onclick={backupNow}>{t('data.backupNow')}</Button>
      {/snippet}
    </SectionHeading>
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('data.backupsHint')}</p>
    {#if backups.length === 0}
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('data.noBackups')}</p>
    {:else}
      <ul class="m-0 p-0 list-none flex flex-col max-h-56 overflow-y-auto divide-y divide-[var(--color-border-subtle)] border-y border-[var(--color-border-subtle)]">
        {#each backups as backup (backup.name)}
          <li class="flex items-center gap-3 py-1.5 text-xs">
            <span class="flex-1 min-w-0 flex flex-col">
              <span class="truncate">{when(backup.created_at)}</span>
              <span class="text-[11px] text-[var(--color-text-muted)]">{kindText(backup.kind)} · {size(backup.size_bytes)}</span>
            </span>
            <Button size="sm" variant="ghost" disabled={busy} onclick={() => (restoring = backup)}>
              {t('data.restore')}
            </Button>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="flex flex-col gap-3">
    <SectionHeading title={t('data.export')} />
    <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('data.exportHint')}</p>
    <Segmented options={formats} bind:value={format as string} label={t('data.export')} />
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">
      {format === 'markdown' ? t('data.formatMarkdownHint') : t('data.formatJsonHint')}
    </p>
    <div class="flex items-center gap-3">
      <Button variant="primary" size="sm" disabled={busy} onclick={runExport}>{t('data.exportNow')}</Button>
    </div>
    {#if written}
      <p class="m-0 text-xs font-mono text-[var(--color-text-secondary)] break-all select-text" role="status">{written}</p>
    {/if}
  </section>

  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('data.close')}</Button>
  {/snippet}
</Dialog>

{#if restoring}
  <ConfirmDialog
    title={t('data.restoreTitle')}
    confirmLabel={t('data.restore')}
    {busy}
    onconfirm={restore}
    oncancel={() => (restoring = null)}
  >
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">
      {t('data.restoreBody', { when: when(restoring.created_at) })}
    </p>
  </ConfirmDialog>
{/if}
