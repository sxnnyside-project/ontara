<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { EntityRef } from '@/shared/model';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Button, ConfirmDialog, Dialog, EmptyState, SelectField, TextField } from '@/shared/ui';
import {
  attachAsset,
  type AssetView,
  type AttachmentView,
  detachAsset,
  importAsset,
  listAssets,
  listAttachments,
} from '../api/asset-api';
import { chooseFiles } from '../api/choose-file';
import { formatBytes } from '../state/media';
import AssetPreview from './asset-preview.svelte';

interface Props {
  entity: EntityRef;
  onopen: (id: string) => void;
}

const { entity, onopen }: Props = $props();

let attachments = $state<AttachmentView[]>([]);
let library = $state<AssetView[]>([]);
let choosing = $state(false);
let pick = $state('');
let purpose = $state('');
let detaching = $state<AttachmentView | null>(null);

$effect(() => {
  void revision.value;
  const id = entity.id;
  let cancelled = false;
  listAttachments(id)
    .then((found) => {
      if (!cancelled) attachments = found;
    })
    .catch((failure) => {
      if (!cancelled) notices.error(toCommandError(failure).message);
    });
  return () => {
    cancelled = true;
  };
});

async function openChooser() {
  try {
    library = await listAssets();
    pick = '';
    purpose = '';
    choosing = true;
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

const available = $derived(
  library.filter((asset) => !attachments.some((item) => item.asset.id === asset.entity.id)),
);

async function attach(assetId: string, why: string | null) {
  try {
    await attachAsset({ owner: entity.id, asset: assetId, purpose: why, note: null });
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

async function attachChosen() {
  if (!pick) return;
  await attach(pick, purpose.trim() === '' ? null : purpose.trim());
  choosing = false;
}

async function importAndAttach() {
  const paths = await chooseFiles(t('assets.chooseFiles'));
  try {
    for (const path of paths) {
      const imported = await importAsset(path, null);
      if (!attachments.some((item) => item.asset.id === imported.entity.id)) {
        await attachAsset({
          owner: entity.id,
          asset: imported.entity.id,
          purpose: null,
          note: null,
        });
      }
    }
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

async function detach() {
  if (!detaching) return;
  try {
    await detachAsset(detaching.attachment.id);
    detaching = null;
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}
</script>

<div class="flex flex-col gap-4 max-w-3xl">
  <div class="flex items-center gap-2">
    <Button icon="i-mingcute:upload-2-line" onclick={importAndAttach}>{t('assets.importAndAttach')}</Button>
    <Button icon="i-mingcute:attachment-line" onclick={openChooser}>{t('assets.attachExisting')}</Button>
  </div>

  {#if attachments.length === 0}
    <EmptyState icon="i-mingcute:attachment-line" title={t('assets.noAttachmentsTitle')} description={t('assets.noAttachmentsHint')} />
  {:else}
    <div class="grid gap-3 grid-cols-[repeat(auto-fill,minmax(10rem,1fr))]">
      {#each attachments as item (item.attachment.id)}
        <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] overflow-hidden flex flex-col">
          <button type="button" onclick={() => onopen(item.asset.id)} class="h-24 p-0 border-0 cursor-pointer bg-transparent">
            <AssetPreview
              entity={item.asset.id}
              mediaType={item.file?.media_type ?? null}
              name={item.asset.name}
              missing={item.missing}
            />
          </button>
          <div class="p-2 flex flex-col gap-0.5 flex-1">
            <span class="text-xs font-medium truncate">{item.asset.name}</span>
            <span class="text-[10px] font-mono text-[var(--color-text-muted)] truncate">
              {item.attachment.purpose ?? (item.file ? formatBytes(item.file.size_bytes) : '')}
            </span>
          </div>
          <div class="px-1 pb-1 flex justify-end">
            <Button size="sm" variant="ghost" icon="i-mingcute:close-line" title={t('assets.detach')} onclick={() => (detaching = item)} />
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if choosing}
  <Dialog title={t('assets.attachExisting')} width="sm" onclose={() => (choosing = false)}>
    {#if available.length === 0}
      <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('assets.nothingToAttach')}</p>
    {:else}
      <SelectField
        label={t('assets.asset')}
        bind:value={pick}
        options={available.map((asset) => ({ value: asset.entity.id, label: asset.entity.name }))}
        placeholder={t('assets.chooseAsset')}
      />
      <TextField label={t('assets.purpose')} bind:value={purpose} hint={t('assets.purposeHint')} />
    {/if}
    {#snippet actions()}
      <Button variant="ghost" onclick={() => (choosing = false)}>{t('common.cancel')}</Button>
      <Button variant="primary" disabled={!pick} onclick={attachChosen}>{t('assets.attach')}</Button>
    {/snippet}
  </Dialog>
{/if}

{#if detaching}
  <ConfirmDialog
    title={t('assets.detachTitle')}
    confirmLabel={t('assets.detach')}
    onconfirm={detach}
    oncancel={() => (detaching = null)}
  >
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('assets.detachBody')}</p>
  </ConfirmDialog>
{/if}
