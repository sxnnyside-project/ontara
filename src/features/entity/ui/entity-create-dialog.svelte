<script lang="ts">
import { catalog, groupCreatableTypes } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Banner, Button, Dialog, Kbd, entityIcon } from '@/shared/ui';
import { createEntity, type Entity } from '../api/entity-api';

interface Props {
  /** The type to start with, such as an event when asked from the timeline. */
  initialType?: string | undefined;
  onclose: () => void;
  oncreated: (entity: Entity) => void;
}

const { initialType, onclose, oncreated }: Props = $props();

const LAST_TYPE_KEY = 'ontara.lastEntityType';

function rememberedType(): string | null {
  try {
    return localStorage.getItem(LAST_TYPE_KEY);
  } catch {
    return null;
  }
}

const groups = $derived(
  groupCreatableTypes(catalog.data.entity_types, ['generic', 'event'], t('entity.groupOther')),
);
const allTypes = $derived(groups.flatMap((group) => group.types));

let chosen = $state('');
let name = $state('');
let busy = $state(false);
let error = $state<string | null>(null);
let input = $state<HTMLInputElement | undefined>();

const entityType = $derived.by(() => {
  const keys = allTypes.map((entry) => entry.key);
  for (const wanted of [chosen, initialType, rememberedType(), 'character']) {
    if (wanted && keys.includes(wanted)) return wanted;
  }
  return keys[0] ?? '';
});

$effect(() => {
  input?.focus();
});

async function create(andOpen: boolean) {
  if (name.trim() === '' || entityType === '') return;
  busy = true;
  error = null;
  try {
    const entity = await createEntity({ entity_type: entityType, name: name.trim() });
    try {
      localStorage.setItem(LAST_TYPE_KEY, entityType);
    } catch {
      // Remembering the last type is a convenience only.
    }
    revision.bump();
    if (andOpen) {
      oncreated(entity);
    } else {
      name = '';
      input?.focus();
    }
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

function onkeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    event.preventDefault();
    create(!event.shiftKey);
  }
}
</script>

<Dialog title={t('entity.createTitle')} width="lg" {onclose}>
  {#if error}<Banner kind="error">{error}</Banner>{/if}

  <div class="flex flex-col gap-1.5">
    <label for="new-entity-name" class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('entity.createName')}
    </label>
    <input
      id="new-entity-name"
      bind:this={input}
      bind:value={name}
      {onkeydown}
      placeholder={t('entity.namePlaceholder')}
      class="h-11 w-full rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-3 text-lg font-serif text-[var(--color-text-primary)] placeholder:text-[var(--color-text-muted)] outline-none focus:border-[var(--color-border-focus)] select-text"
    />
  </div>

  <div class="flex flex-col gap-3">
    <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
      {t('entity.createWhat')}
    </span>
    {#each groups as group (group.root || group.label)}
      <div class="flex flex-col gap-1.5">
        <span class="text-[11px] text-[var(--color-text-muted)]">{group.label}</span>
        <div class="flex flex-wrap gap-1.5">
          {#each group.types as option (option.key)}
            <button
              type="button"
              aria-pressed={entityType === option.key}
              onclick={() => {
                chosen = option.key;
                input?.focus();
              }}
              class="inline-flex items-center gap-1.5 h-8 px-2.5 rounded border text-xs cursor-pointer transition-colors
              {entityType === option.key
                ? 'border-[var(--color-crown-red)] bg-[var(--color-crown-red-subtle)] text-[var(--color-text-primary)]'
                : 'border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] text-[var(--color-text-secondary)] hover:border-[var(--color-border-strong)] hover:text-[var(--color-text-primary)]'}"
            >
              <span class="{entityIcon(catalog.chain(option.key))} w-3.5 h-3.5"></span>
              {option.label}
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  {#snippet actions()}
    <span class="mr-auto inline-flex items-center gap-1 text-[11px] text-[var(--color-text-muted)]">
      <Kbd>Enter</Kbd> {t('entity.createAndOpen')} · <Kbd>Shift</Kbd>+<Kbd>Enter</Kbd> {t('entity.createAnother')}
    </span>
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy || name.trim() === ''} onclick={() => create(true)}>
      {t('entity.create')}
    </Button>
  {/snippet}
</Dialog>
