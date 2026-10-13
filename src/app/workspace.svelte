<script lang="ts">
import { LibraryView } from '@/features/asset';
import { AssistantSettingsDialog, ProposalsView, proposals } from '@/features/assistant';
import { catalog, ConfigurationView } from '@/features/catalog';
import { consistency, ConsistencyView } from '@/features/consistency';
import { StructureView } from '@/features/editorial';
import { EntityCreateDialog } from '@/features/entity';
import { GraphView } from '@/features/graph';
import { MatrixView } from '@/features/matrix';
import { history, HistoryDialog, changeText } from '@/features/history';
import { LinksPanel, navigation, type ViewKey } from '@/features/navigation';
import { project, ProjectDataDialog, ProjectSettingsDialog } from '@/features/project';
import { TimelineView } from '@/features/timeline';
import { toCommandError } from '@/shared/backend';
import { getLocale, t, toggleLocale } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Button } from '@/shared/ui';
import ActivityRail from './activity-rail.svelte';
import CommandPalette, { type Command } from './command-palette.svelte';
import EntityWorkspace from './entity-workspace.svelte';
import Navigator from './navigator.svelte';
import TopBar from './top-bar.svelte';
import UniverseHome from './universe-home.svelte';

interface Props {
  onclosed: () => void;
}

const { onclosed }: Props = $props();

/** Views where the navigator is part of how the author works. */
const WITH_NAVIGATOR: ViewKey[] = ['universe', 'graph', 'matrix', 'timeline', 'structure'];

const view = $derived(navigation.view);

let creating = $state(false);
let finding = $state(false);
let showingHistory = $state(false);
let settings = $state(false);
let dataDialog = $state(false);
let assistantSettings = $state(false);
let dark = $state(true);
let navigatorHidden = $state(false);
let linksHidden = $state(false);

const hasNavigator = $derived(WITH_NAVIGATOR.includes(view));
const navigatorVisible = $derived(!navigatorHidden && hasNavigator);
const hasLinks = $derived(view === 'universe' && navigation.current !== null);
const linksVisible = $derived(hasLinks && !linksHidden);

/** Goes to an entity's page, from wherever the link was followed. */
function open(id: string) {
  navigation.open(id, 'universe');
}

// The consistency badge stays current as the universe changes.
$effect(() => {
  void revision.value;
  consistency.refresh().catch(() => consistency.clear());
});

// Suggestions may arrive from a client on this computer, or be decided here.
$effect(() => {
  void revision.value;
  proposals.refresh().catch(() => proposals.clear());
});

// What can be undone follows every edit, including the ones the history itself makes.
$effect(() => {
  void revision.value;
  history.refresh().catch(() => history.clear());
});

function toggleTheme() {
  dark = !dark;
  if (dark) document.documentElement.removeAttribute('data-theme');
  else document.documentElement.setAttribute('data-theme', 'light');
}

async function closeProject() {
  try {
    await project.close();
    onclosed();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

const SHORTCUT_VIEWS: ViewKey[] = [
  'universe',
  'structure',
  'graph',
  'matrix',
  'timeline',
  'consistency',
  'library',
  'configuration',
  'proposals',
];

const MOD = typeof navigator !== 'undefined' && /mac/i.test(navigator.platform) ? '⌘' : 'Ctrl+';

function closePalette(then?: () => void) {
  finding = false;
  then?.();
}

const commands = $derived<Command[]>([
  ...SHORTCUT_VIEWS.map((target, index) => ({
    id: `view-${target}`,
    label: t('palette.goTo', { view: t(`rail.${target}` as 'rail.universe') }),
    shortcut: `${MOD}${index + 1}`,
    run: () => closePalette(() => navigation.show(target)),
  })),
  {
    id: 'new-entity',
    label: t('palette.newEntity'),
    shortcut: `${MOD}N`,
    keywords: 'create add crear añadir',
    run: () => closePalette(() => (creating = true)),
  },
  {
    id: 'undo',
    label: history.undoable[0]
      ? t('history.next', { what: changeText(history.undoable[0].label) })
      : t('palette.undo'),
    shortcut: `${MOD}Z`,
    run: () => closePalette(() => void history.undo()),
  },
  {
    id: 'redo',
    label: t('palette.redo'),
    shortcut: `${MOD}⇧Z`,
    run: () => closePalette(() => void history.redo()),
  },
  {
    id: 'history',
    label: t('palette.history'),
    run: () => closePalette(() => (showingHistory = true)),
  },
  {
    id: 'navigator',
    label: t('palette.toggleNavigator'),
    shortcut: `${MOD}B`,
    run: () => closePalette(() => (navigatorHidden = !navigatorHidden)),
  },
  {
    id: 'theme',
    label: t('palette.toggleTheme'),
    keywords: 'theme dark light tema oscuro claro',
    run: () => closePalette(toggleTheme),
  },
  {
    id: 'language',
    label: t('palette.toggleLanguage'),
    keywords: `language idioma ${getLocale()}`,
    run: () => closePalette(toggleLocale),
  },
  {
    id: 'settings',
    label: t('palette.settings'),
    run: () => closePalette(() => (settings = true)),
  },
  {
    id: 'assistant',
    label: t('assistant.settingsTitle'),
    keywords: 'ai ia mcp provider proveedor key clave',
    run: () => closePalette(() => (assistantSettings = true)),
  },
  {
    id: 'data',
    label: t('data.open'),
    keywords: 'backup export restore copia exportar restaurar',
    run: () => closePalette(() => (dataDialog = true)),
  },
  {
    id: 'close',
    label: t('palette.closeProject'),
    run: () => closePalette(() => void closeProject()),
  },
]);

/** Whether a key press belongs to a text field, where undo means undoing typing. */
function isEditing(event: KeyboardEvent): boolean {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return false;
  return target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
}

function onkeydown(event: KeyboardEvent) {
  const modifier = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  if (event.key === 'Escape' && view === 'universe' && navigation.current && !isEditing(event)) {
    // A dialog handles its own Escape; this one only closes the page behind it.
    if (!document.querySelector('dialog[open]')) navigation.close();
  } else if (modifier && key === 'z' && !isEditing(event)) {
    event.preventDefault();
    if (event.shiftKey) history.redo();
    else history.undo();
  } else if (modifier && key === 'y' && !isEditing(event)) {
    event.preventDefault();
    history.redo();
  } else if (modifier && key === 'k') {
    event.preventDefault();
    finding = true;
  } else if (modifier && key === 'n') {
    event.preventDefault();
    creating = true;
  } else if (modifier && key === 'b') {
    event.preventDefault();
    navigatorHidden = !navigatorHidden;
  } else if (modifier && /^[1-9]$/.test(key)) {
    event.preventDefault();
    const target = SHORTCUT_VIEWS[Number(key) - 1];
    if (target) navigation.show(target);
  } else if (event.altKey && event.key === 'ArrowLeft') {
    event.preventDefault();
    navigation.back();
  } else if (event.altKey && event.key === 'ArrowRight') {
    event.preventDefault();
    navigation.forward();
  }
}
</script>

<svelte:window {onkeydown} />

<div class="h-full flex flex-col">
  <TopBar
    {dark}
    onsearch={() => (finding = true)}
    onsettings={() => (settings = true)}
    onclose={closeProject}
    ontheme={toggleTheme}
  />

  <div class="flex-1 min-h-0 flex">
    <ActivityRail {view} errors={consistency.report?.counts.errors ?? 0} suggestions={proposals.pending} onselect={(next) => navigation.show(next)} />

    {#if navigatorVisible}
      <aside class="w-72 shrink-0 border-r border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/85 backdrop-blur-md min-h-0">
        <Navigator
          selectedId={navigation.current}
          onselect={(id) =>
            view === 'universe' && navigation.current === id ? navigation.close() : navigation.open(id)}
          oncreate={() => (creating = true)}
          onmanageStructure={() => navigation.show('structure')}
          oncollapse={() => (navigatorHidden = true)}
        />
      </aside>
    {:else if hasNavigator}
      <button
        type="button"
        title={t('navigator.show')}
        onclick={() => (navigatorHidden = false)}
        class="w-6 shrink-0 flex items-start justify-center pt-3 border-0 border-r border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/85 text-[var(--color-text-muted)] cursor-pointer hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface-hover)]"
      >
        <span class="i-mingcute:right-line w-4 h-4"></span>
      </button>
    {/if}

    <main class="flex-1 min-w-0 min-h-0 surface-ambient">
      {#if view === 'universe'}
        {#if navigation.current}
          {#key navigation.current}
            <EntityWorkspace id={navigation.current} onopen={open} onclose={() => navigation.close()}
              onassistant={() => (assistantSettings = true)}
            />
          {/key}
        {:else}
          <UniverseHome onopen={open} oncreate={() => (creating = true)} onshow={(next) => navigation.show(next)} />
        {/if}
      {:else if view === 'structure'}
        <StructureView
          selectedId={navigation.current}
          onselect={open}
          onentityremoved={(id) => navigation.forget(id)}
        />
      {:else if view === 'graph'}
        <GraphView onopen={open} />
      {:else if view === 'matrix'}
        <MatrixView onopen={open} />
      {:else if view === 'timeline'}
        <TimelineView onopen={open} />
      {:else if view === 'consistency'}
        <ConsistencyView onopen={open} />
      {:else if view === 'library'}
        <LibraryView onopen={open} />
      {:else if view === 'proposals'}
        <ProposalsView onopen={open} onsettings={() => (assistantSettings = true)} />
      {:else}
        <ConfigurationView />
      {/if}
    </main>

    {#if linksVisible && navigation.current}
      <aside class="w-72 shrink-0 border-l border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/85 backdrop-blur-md flex flex-col min-h-0">
        <div class="shrink-0 h-10 px-4 border-b border-[var(--color-border-subtle)] flex items-center gap-2">
          <span class="flex-1 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] font-medium">
            {t('links.title')}
          </span>
          <Button size="sm" variant="ghost" icon="i-mingcute:close-line" title={t('links.hide')} onclick={() => (linksHidden = true)} />
        </div>
        <div class="flex-1 overflow-y-auto p-3">
          {#key navigation.current}
            <LinksPanel entity={navigation.current} onopen={open} />
          {/key}
        </div>
      </aside>
    {:else if hasLinks}
      <button
        type="button"
        title={t('links.show')}
        onclick={() => (linksHidden = false)}
        class="w-6 shrink-0 flex items-start justify-center pt-3 border-0 border-l border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/85 text-[var(--color-text-muted)] cursor-pointer hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface-hover)]"
      >
        <span class="i-mingcute:left-line w-4 h-4"></span>
      </button>
    {/if}
  </div>

  <footer class="h-6 shrink-0 border-t border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/90 backdrop-blur-md px-3 flex items-center justify-between text-[10px] font-mono text-[var(--color-text-muted)]">
    <span class="flex items-center gap-1.5 min-w-0">
      <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-canon-established)]"></span>
      <span class="truncate">{t('status.localProject')} · {project.current?.path}</span>
    </span>
    <span class="flex items-center gap-3">
      {#if history.undoable[0]}
        <button
          type="button"
          onclick={() => (showingHistory = true)}
          class="border-0 bg-transparent p-0 font-mono text-[10px] text-[var(--color-text-muted)] cursor-pointer hover:text-[var(--color-text-primary)]"
        >
          {t('history.next', { what: changeText(history.undoable[0].label) })}
        </button>
      {/if}
      <span>{catalog.data.entity_types.length} {t('status.types')}</span>
    </span>
  </footer>
</div>

{#if creating}
  <EntityCreateDialog
    onclose={() => (creating = false)}
    oncreated={(entity) => {
      creating = false;
      open(entity.id);
    }}
  />
{/if}
{#if finding}
  <CommandPalette
    {commands}
    onclose={() => (finding = false)}
    onopen={(id) => {
      finding = false;
      open(id);
    }}
  />
{/if}
{#if showingHistory}
  <HistoryDialog onclose={() => (showingHistory = false)} />
{/if}
{#if settings}
  <ProjectSettingsDialog
    onclose={() => (settings = false)}
    ondata={() => {
      settings = false;
      dataDialog = true;
    }}
  />
{/if}
{#if assistantSettings}
  <AssistantSettingsDialog onclose={() => (assistantSettings = false)} />
{/if}
{#if dataDialog}
  <ProjectDataDialog
    onclose={() => (dataDialog = false)}
    onrestored={async (summary) => {
      dataDialog = false;
      project.refresh(summary);
      navigation.clear();
      await catalog.load();
      revision.bump();
    }}
  />
{/if}
