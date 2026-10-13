<script lang="ts">
import type { Snippet } from 'svelte';
import { t } from '@/shared/i18n';
import Button from './button.svelte';
import Dialog from './dialog.svelte';

interface Props {
  title: string;
  confirmLabel: string;
  danger?: boolean | undefined;
  busy?: boolean | undefined;
  onconfirm: () => void;
  oncancel: () => void;
  children: Snippet;
}

const {
  title,
  confirmLabel,
  danger = false,
  busy = false,
  onconfirm,
  oncancel,
  children,
}: Props = $props();
</script>

<Dialog {title} width="sm" onclose={oncancel}>
  {@render children()}
  {#snippet actions()}
    <Button variant="ghost" onclick={oncancel}>{t('common.cancel')}</Button>
    <Button variant={danger ? 'danger' : 'primary'} disabled={busy} onclick={onconfirm}>
      {confirmLabel}
    </Button>
  {/snippet}
</Dialog>
