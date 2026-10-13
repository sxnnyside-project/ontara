<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Banner, Button, SelectField, TextField } from '@/shared/ui';
import {
  listProviders,
  previewTask,
  type Provider,
  type RequestPreview,
  runTask,
  setPolicy,
  getPolicy,
  type TaskKind,
} from '../api/assistant-api';

interface Props {
  /** The entity the task is about; absent for tasks about the whole universe. */
  entity: string | null;
  /** Opens the assistant settings. */
  onsettings: () => void;
  /** Called once suggestions are ready to review. */
  onready: () => void;
}

const { entity, onsettings, onready }: Props = $props();

const KINDS: TaskKind[] = ['describe', 'relate', 'analyze', 'generate'];

let providers = $state<Provider[]>([]);
let provider = $state('');
let kind = $state<TaskKind>('describe');
let instruction = $state('');
let preview = $state<RequestPreview | null>(null);
let showText = $state(false);
let failure = $state<string | null>(null);
let busy = $state(false);
let loaded = $state(false);

$effect(() => {
  listProviders()
    .then((found) => {
      providers = found;
      if (provider === '' || !found.some((entry) => entry.id === provider)) {
        provider = found[0]?.id ?? '';
      }
      loaded = true;
    })
    .catch((error) => {
      failure = toCommandError(error).message;
      loaded = true;
    });
});

const needsEntity = $derived(kind !== 'generate');

// What would be sent is always shown fresh, from the same text the request is built from.
$effect(() => {
  void revision.value;
  if (provider === '' || (needsEntity && entity === null)) {
    preview = null;
    return;
  }
  let cancelled = false;
  previewTask({ kind, entity: needsEntity ? entity : null, instruction, provider })
    .then((found) => {
      if (!cancelled) {
        preview = found;
        failure = null;
      }
    })
    .catch((error) => {
      if (!cancelled) {
        preview = null;
        failure = toCommandError(error).message;
      }
    });
  return () => {
    cancelled = true;
  };
});

async function allowCloud() {
  try {
    const current = await getPolicy();
    await setPolicy({ ...current, cloud_allowed: true });
    revision.bump();
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

async function ask() {
  busy = true;
  failure = null;
  try {
    const outcome = await runTask({
      kind,
      entity: needsEntity ? entity : null,
      instruction,
      provider,
    });
    notices.success(t('assistant.done'));
    if (outcome.dropped > 0) {
      notices.show('info', t('assistant.dropped', { count: outcome.dropped }));
    }
    revision.bump();
    onready();
  } catch (error) {
    failure = toCommandError(error).message;
  } finally {
    busy = false;
  }
}
</script>

<section class="flex flex-col gap-4 max-w-2xl">
  {#if loaded && providers.length === 0}
    <div class="rounded-lg border border-dashed border-[var(--color-border-strong)] p-5 flex flex-col gap-3 items-start">
      <p class="m-0 text-sm text-[var(--color-text-secondary)]">{t('assistant.needProvider')}</p>
      <Button variant="primary" onclick={onsettings}>{t('assistant.configure')}</Button>
    </div>
  {:else}
    {#if failure}<Banner kind="error">{failure}</Banner>{/if}

    <SelectField
      label={t('assistant.task')}
      bind:value={kind as string}
      options={KINDS.map((value) => ({ value, label: t(`assistant.task_${value}` as 'assistant.task_describe') }))}
    />
    <TextField
      label={t('assistant.instruction')}
      bind:value={instruction}
      multiline
      rows={3}
      placeholder={t('assistant.instructionPlaceholder')}
    />
    <SelectField
      label={t('assistant.provider')}
      bind:value={provider}
      options={providers.map((entry) => ({
        value: entry.id,
        label: `${entry.name} · ${entry.model} · ${entry.is_local ? t('assistant.local') : t('assistant.cloud')}`,
      }))}
    />

    {#if preview}
      <div class="rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-3 flex flex-col gap-2">
        <div class="flex items-center gap-2">
          <span class="flex-1 text-xs text-[var(--color-text-secondary)]">
            {t('assistant.willSendHint', {
              characters: preview.characters,
              entities: preview.entities,
              destination: preview.destination,
              model: preview.model,
            })}
            {#if preview.truncated}{t('assistant.truncated')}{/if}
          </span>
          <Button size="sm" variant="ghost" onclick={() => (showText = !showText)}>
            {showText ? t('assistant.hide') : t('assistant.show')}
          </Button>
        </div>
        {#if showText}
          <pre class="m-0 max-h-72 overflow-auto text-[11px] font-mono whitespace-pre-wrap text-[var(--color-text-secondary)] select-text">{preview.text}</pre>
        {/if}
      </div>
      {#if preview.blocked}
        <Banner kind="warning">
          {t('assistant.blockedCloud')}
        </Banner>
        <div><Button onclick={allowCloud}>{t('assistant.allowCloud')}</Button></div>
      {/if}
    {:else if needsEntity && entity === null}
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('assistant.noEntity')}</p>
    {/if}

    <div class="flex items-center gap-2">
      <Button variant="primary" disabled={busy || preview === null || preview.blocked !== null} onclick={ask}>
        {busy ? t('assistant.asking') : t('assistant.ask')}
      </Button>
      <Button variant="ghost" onclick={onsettings}>{t('assistant.open')}</Button>
    </div>
  {/if}
</section>
