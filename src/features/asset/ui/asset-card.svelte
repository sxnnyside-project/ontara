<script lang="ts">
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { type AssetView, listAssets } from '../api/asset-api';
import { formatBytes } from '../state/media';
import AssetPreview from './asset-preview.svelte';

interface Props {
  entity: string;
}

const { entity }: Props = $props();

let asset = $state<AssetView | null>(null);

$effect(() => {
  void revision.value;
  const id = entity;
  let cancelled = false;
  listAssets()
    .then((found) => {
      if (!cancelled) asset = found.find((candidate) => candidate.entity.id === id) ?? null;
    })
    .catch(() => {
      if (!cancelled) asset = null;
    });
  return () => {
    cancelled = true;
  };
});
</script>

{#if asset}
  <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] overflow-hidden flex max-w-2xl">
    <div class="w-48 h-36 shrink-0 border-r border-[var(--color-border-subtle)]">
      <AssetPreview
        entity={asset.entity.id}
        mediaType={asset.file.media_type}
        name={asset.entity.name}
        missing={asset.missing}
      />
    </div>
    <dl class="m-0 p-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs content-start min-w-0">
      <dt class="text-[var(--color-text-muted)] font-mono">{t('assets.file')}</dt>
      <dd class="m-0 truncate font-mono select-text">{asset.file.relative_path}</dd>
      <dt class="text-[var(--color-text-muted)] font-mono">{t('assets.size')}</dt>
      <dd class="m-0 font-mono">{formatBytes(asset.file.size_bytes)}</dd>
      <dt class="text-[var(--color-text-muted)] font-mono">{t('assets.mediaType')}</dt>
      <dd class="m-0 font-mono">{asset.file.media_type ?? '—'}</dd>
      {#if asset.missing}
        <dd class="m-0 col-span-2 text-[var(--color-finding-warning)]">{t('assets.missing')}</dd>
      {/if}
    </dl>
  </div>
{/if}
