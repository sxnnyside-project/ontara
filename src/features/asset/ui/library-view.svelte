<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Banner, Button, EmptyState } from '@/shared/ui';
import { type AssetView, importAsset, listAssets } from '../api/asset-api';
import { chooseFiles } from '../api/choose-file';
import { formatBytes } from '../state/media';
import AssetPreview from './asset-preview.svelte';

interface Props {
  onopen: (id: string) => void;
}

const { onopen }: Props = $props();

let assets = $state<AssetView[]>([]);
let busy = $state(false);
let error = $state<string | null>(null);

$effect(() => {
  void revision.value;
  let cancelled = false;
  listAssets()
    .then((found) => {
      if (!cancelled) assets = found;
    })
    .catch((failure) => {
      if (!cancelled) error = toCommandError(failure).message;
    });
  return () => {
    cancelled = true;
  };
});

const missingCount = $derived(assets.filter((asset) => asset.missing).length);

async function importFiles() {
  const paths = await chooseFiles(t('assets.chooseFiles'));
  if (paths.length === 0) return;
  busy = true;
  let added = 0;
  let repeated = 0;
  try {
    for (const path of paths) {
      const result = await importAsset(path, null);
      if (result.already_imported) repeated += 1;
      else added += 1;
    }
    notices.success(t('assets.imported', { added, repeated }));
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  } finally {
    busy = false;
  }
}
</script>

<div class="flex flex-col gap-4 p-6">
  <div class="flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-serif font-medium">{t('assets.title')}</h2>
      <p class="m-0 mt-0.5 text-xs text-[var(--color-text-secondary)]">{t('assets.subtitle')}</p>
    </div>
    <Button variant="primary" icon="i-mingcute:upload-2-line" disabled={busy} onclick={importFiles}>
      {t('assets.import')}
    </Button>
  </div>

  {#if error}<Banner kind="error">{error}</Banner>{/if}
  {#if missingCount > 0}
    <Banner kind="warning">{t('assets.missingBanner', { count: missingCount })}</Banner>
  {/if}

  {#if assets.length === 0}
    <EmptyState icon="i-mingcute:pic-line" title={t('assets.emptyTitle')} description={t('assets.emptyHint')} />
  {:else}
    <div class="grid gap-3 grid-cols-[repeat(auto-fill,minmax(11rem,1fr))]">
      {#each assets as asset (asset.entity.id)}
        <button
          type="button"
          onclick={() => onopen(asset.entity.id)}
          class="text-left rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] overflow-hidden cursor-pointer p-0 hover:border-[var(--color-border-strong)] transition-colors"
        >
          <div class="h-28">
            <AssetPreview
              entity={asset.entity.id}
              mediaType={asset.file.media_type}
              name={asset.entity.name}
              missing={asset.missing}
            />
          </div>
          <div class="p-2.5 flex flex-col gap-0.5">
            <span class="text-xs font-medium truncate text-[var(--color-text-primary)]">{asset.entity.name}</span>
            <span class="text-[10px] font-mono text-[var(--color-text-muted)] truncate">
              {formatBytes(asset.file.size_bytes)} · {t('assets.attachedCount', { count: asset.attached_to.length })}
            </span>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>
