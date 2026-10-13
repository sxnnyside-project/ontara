<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Button, CanonBadge, SelectField, TextField } from '@/shared/ui';
import { changeCanonState, type EntityDetail } from '../api/entity-api';

interface Props {
  detail: EntityDetail;
}

const { detail }: Props = $props();

let target = $state('');
let reason = $state('');
let busy = $state(false);

$effect(() => {
  target = detail.entity.canon_state;
});

const options = $derived(
  catalog.data.canon_states.map((state) => ({ value: state.key, label: state.label })),
);

async function change() {
  busy = true;
  try {
    await changeCanonState(
      { kind: 'entity', id: detail.entity.id },
      target,
      reason.trim() === '' ? null : reason.trim(),
    );
    reason = '';
    notices.success(t('canon.changed'));
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  } finally {
    busy = false;
  }
}

function dateOf(millis: number): string {
  return new Date(millis).toLocaleString();
}
</script>

<div class="flex flex-col gap-6 max-w-2xl">
  <section class="rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-4 flex flex-col gap-3">
    <div class="flex items-center gap-2">
      <span class="text-xs text-[var(--color-text-secondary)]">{t('canon.current')}</span>
      <CanonBadge state={detail.entity.canon_state} label={catalog.canonLabel(detail.entity.canon_state)} size="md" />
    </div>
    <SelectField label={t('canon.moveTo')} bind:value={target} {options} />
    <TextField label={t('canon.reason')} bind:value={reason} multiline rows={3} hint={t('canon.reasonHint')} />
    <div>
      <Button variant="primary" disabled={busy || target === detail.entity.canon_state} onclick={change}>
        {t('canon.change')}
      </Button>
    </div>
  </section>

  <section class="flex flex-col gap-2">
    <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('canon.history')}
    </h3>
    {#each [...detail.canon_history].reverse() as change (change.id)}
      <div class="rounded border border-[var(--color-border-subtle)] p-3 flex flex-col gap-1.5">
        <div class="flex items-center gap-2 flex-wrap">
          <CanonBadge state={change.from} label={catalog.canonLabel(change.from)} />
          <span class="i-mingcute:arrow-right-line w-3.5 h-3.5 text-[var(--color-text-muted)]"></span>
          <CanonBadge state={change.to} label={catalog.canonLabel(change.to)} />
          <span class="ml-auto text-[10px] font-mono text-[var(--color-text-muted)]">{dateOf(change.changed_at)}</span>
        </div>
        {#if change.reason}
          <p class="m-0 text-xs text-[var(--color-text-secondary)] select-text">{change.reason}</p>
        {/if}
      </div>
    {:else}
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('canon.noHistory')}</p>
    {/each}
  </section>
</div>
