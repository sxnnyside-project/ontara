<script lang="ts">
import { catalog } from '@/features/catalog';
import { excerptParts, searchEntities, type SearchResult } from '@/features/search';
import { t } from '@/shared/i18n';
import { typeLabelOf } from '@/shared/model';
import { Dialog, entityIcon } from '@/shared/ui';

/** Something the palette can run. */
export interface Command {
  id: string;
  label: string;
  /** The shortcut, written as the keyboard shows it. */
  shortcut?: string | undefined;
  /** Extra words that find it. */
  keywords?: string | undefined;
  run: () => void;
}

interface Props {
  commands: Command[];
  onopen: (entity: string) => void;
  onclose: () => void;
}

const { commands, onopen, onclose }: Props = $props();

type Row =
  | { kind: 'command'; id: string; command: Command }
  | { kind: 'entity'; id: string; result: SearchResult };

let text = $state('');
let found = $state<SearchResult[]>([]);
let active = $state(0);
/** The search for what is typed has answered, so an empty list means nothing matched. */
let settled = $state(true);

const commandMode = $derived(text.startsWith('>'));
const needle = $derived((commandMode ? text.slice(1) : text).trim().toLowerCase());

$effect(() => {
  if (commandMode || needle === '') {
    found = [];
    settled = true;
    return;
  }
  let cancelled = false;
  settled = false;
  // A short pause so typing does not search once per key.
  const timer = setTimeout(() => {
    searchEntities(needle, true)
      .then((results) => {
        if (cancelled) return;
        found = results;
        settled = true;
      })
      .catch(() => {
        if (cancelled) return;
        found = [];
        settled = true;
      });
  }, 90);
  return () => {
    cancelled = true;
    clearTimeout(timer);
  };
});

const matching = $derived(
  commands.filter((command) => {
    if (needle === '') return true;
    return `${command.label} ${command.keywords ?? ''}`.toLowerCase().includes(needle);
  }),
);

const rows = $derived.by<Row[]>(() => {
  const entityRows: Row[] = commandMode
    ? []
    : found.map((result) => ({ kind: 'entity', id: `e-${result.entity.id}`, result }));
  const commandRows: Row[] = (needle === '' || commandMode ? matching : matching.slice(0, 5)).map(
    (command) => ({ kind: 'command', id: `c-${command.id}`, command }),
  );
  return [...entityRows, ...commandRows];
});

$effect(() => {
  void rows.length;
  active = 0;
});

function run(row: Row | undefined) {
  if (!row) return;
  if (row.kind === 'entity') onopen(row.result.entity.id);
  else row.command.run();
}

function onkeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    active = Math.min(active + 1, rows.length - 1);
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    active = Math.max(active - 1, 0);
  } else if (event.key === 'Enter') {
    event.preventDefault();
    run(rows[active]);
  }
}

const listId = 'palette-results';
</script>

<Dialog title={t('palette.label')} {onclose}>
  <input
    bind:value={text}
    {onkeydown}
    placeholder={t('palette.placeholder')}
    aria-label={t('palette.label')}
    role="combobox"
    aria-expanded="true"
    aria-controls={listId}
    aria-activedescendant={rows[active]?.id ? `palette-${rows[active]?.id}` : undefined}
    class="h-9 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] px-3 text-sm outline-none focus:border-[var(--color-border-focus)] select-text"
  />
  <div id={listId} role="listbox" aria-label={t('palette.label')} class="flex flex-col gap-0.5 -mx-1 max-h-[22rem] overflow-y-auto">
    {#each rows as row, index (row.id)}
      {#if row.kind === 'entity'}
        {@const result = row.result}
        <div
          id="palette-{row.id}"
          role="option"
          aria-selected={index === active}
          tabindex="-1"
          onclick={() => run(row)}
          onkeydown={() => {}}
          class="flex items-start gap-2 px-2 py-1.5 rounded cursor-pointer
          {index === active ? 'bg-[var(--color-surface-elevated)]' : 'hover:bg-[var(--color-surface-hover)]'}"
        >
          <span class="{entityIcon(catalog.chain(result.entity.entity_type))} w-4 h-4 mt-0.5 shrink-0 text-[var(--color-text-muted)]"></span>
          <span class="flex-1 min-w-0 flex flex-col">
            <span class="text-xs truncate">{result.entity.name}</span>
            <span class="text-[11px] text-[var(--color-text-muted)] truncate">
              {#each excerptParts(result.excerpt) as part}
                {#if part.matched}<mark class="bg-transparent text-[var(--color-text-primary)] font-medium">{part.text}</mark>{:else}{part.text}{/if}
              {/each}
            </span>
          </span>
          <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{typeLabelOf(result.entity)}</span>
        </div>
      {:else}
        <div
          id="palette-{row.id}"
          role="option"
          aria-selected={index === active}
          tabindex="-1"
          onclick={() => run(row)}
          onkeydown={() => {}}
          class="flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer
          {index === active ? 'bg-[var(--color-surface-elevated)]' : 'hover:bg-[var(--color-surface-hover)]'}"
        >
          <span class="w-4 h-4 shrink-0 text-center text-[var(--color-text-muted)] font-mono text-xs" aria-hidden="true">›</span>
          <span class="flex-1 min-w-0 text-xs truncate">{row.command.label}</span>
          {#if row.command.shortcut}
            <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{row.command.shortcut}</span>
          {/if}
        </div>
      {/if}
    {:else}
      {#if settled}
        <p class="m-2 text-xs text-[var(--color-text-muted)]">{t('palette.none')}</p>
      {/if}
    {/each}
  </div>
  <p class="m-0 text-[10px] font-mono text-[var(--color-text-muted)]">{t('palette.hint')}</p>
</Dialog>
