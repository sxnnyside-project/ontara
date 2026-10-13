<script lang="ts">
import { t } from '@/shared/i18n';
import { readAsset } from '../api/asset-api';
import { isPreviewableImage } from '../state/media';

interface Props {
  entity: string;
  mediaType: string | null;
  name: string;
  missing?: boolean | undefined;
}

const { entity, mediaType, name, missing = false }: Props = $props();

let url = $state<string | null>(null);
let failed = $state(false);

$effect(() => {
  url = null;
  failed = false;
  if (missing || !isPreviewableImage(mediaType)) return;
  let cancelled = false;
  let created: string | null = null;
  readAsset(entity)
    .then((bytes) => {
      if (cancelled) return;
      created = URL.createObjectURL(new Blob([bytes], { type: mediaType ?? '' }));
      url = created;
    })
    .catch(() => {
      if (!cancelled) failed = true;
    });
  return () => {
    cancelled = true;
    if (created) URL.revokeObjectURL(created);
  };
});
</script>

<div class="w-full h-full flex items-center justify-center bg-[var(--color-surface-base)] text-[var(--color-text-muted)]">
  {#if url}
    <img src={url} alt={name} class="max-w-full max-h-full object-contain" />
  {:else if missing}
    <span class="i-mingcute:warning-line w-6 h-6 text-[var(--color-finding-warning)]" title={t('assets.missing')}></span>
  {:else if failed || !isPreviewableImage(mediaType)}
    <span class="i-mingcute:file-line w-6 h-6"></span>
  {:else}
    <span class="i-mingcute:loading-line animate-spin w-5 h-5"></span>
  {/if}
</div>
