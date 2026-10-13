<script lang="ts">
import { getLocale, t } from '@/shared/i18n';
import { Button, Dialog } from '@/shared/ui';
import { changeText, timeText } from '../state/history-model';
import { history } from '../state/history.svelte';

interface Props {
  onclose: () => void;
}

const { onclose }: Props = $props();

$effect(() => {
  history.refresh().catch(() => history.clear());
});
</script>

<Dialog title={t('history.title')} description={t('history.hint')} width="md" {onclose}>
  <div class="flex items-center gap-2">
    <Button
      size="sm"
      disabled={history.busy || history.undoable.length === 0}
      onclick={() => history.undo()}
    >
      {t('history.undo')}
    </Button>
    <Button
      size="sm"
      variant="ghost"
      disabled={history.busy || history.redoable.length === 0}
      onclick={() => history.redo()}
    >
      {t('history.redo')}
    </Button>
  </div>

  {#if history.redoable.length > 0}
    <section class="flex flex-col gap-1">
      <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
        {t('history.undone')}
      </h3>
      <ul class="m-0 p-0 list-none flex flex-col">
        {#each history.redoable as change}
          <li class="flex items-baseline gap-2 py-1 text-xs text-[var(--color-text-muted)]">
            <span class="flex-1 min-w-0 truncate">{changeText(change.label)}</span>
            <span class="font-mono text-[10px]">{timeText(change.at, getLocale())}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section class="flex flex-col gap-1">
    <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('history.recent')}
    </h3>
    {#if history.undoable.length === 0}
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('history.empty')}</p>
    {:else}
      <ul class="m-0 p-0 list-none flex flex-col max-h-72 overflow-y-auto">
        {#each history.undoable as change, index}
          <li class="flex items-baseline gap-2 py-1 text-xs {index === 0 ? '' : 'text-[var(--color-text-secondary)]'}">
            <span class="flex-1 min-w-0 truncate">{changeText(change.label)}</span>
            <span class="font-mono text-[10px] text-[var(--color-text-muted)]">{timeText(change.at, getLocale())}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</Dialog>
