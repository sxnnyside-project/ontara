<script lang="ts">
import { LibraryView } from '@/features/asset';
import { catalog, ConfigurationView } from '@/features/catalog';
import { EntityBrowser, EntityCreateDialog } from '@/features/entity';
import { SectionTree, StructureView } from '@/features/editorial';
import { LinksPanel, navigation, QuickFind } from '@/features/navigation';
import { project, ProjectSettingsDialog } from '@/features/project';
import { toCommandError } from '@/shared/backend';
import { getLocale, t, toggleLocale } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { Button, EmptyState } from '@/shared/ui';
import EntityWorkspace from './entity-workspace.svelte';

interface Props {
  onclosed: () => void;
}

const { onclosed }: Props = $props();

type Mode = 'universe' | 'structure' | 'library' | 'configuration';

let mode = $state<Mode>('universe');
let creating = $state(false);
let finding = $state(false);
let settings = $state(false);
let dark = $state(true);

const modes = $derived<{ key: Mode; label: string; icon: string }[]>([
  { key: 'universe', label: t('shell.universe'), icon: 'i-mingcute:box-line' },
  { key: 'structure', label: t('shell.structure'), icon: 'i-mingcute:book-2-line' },
  { key: 'library', label: t('shell.library'), icon: 'i-mingcute:pic-line' },
  { key: 'configuration', label: t('shell.configuration'), icon: 'i-mingcute:settings-3-line' },
]);

/** Goes to an entity's page, from wherever the link was followed. */
function open(id: string) {
  navigation.open(id);
  mode = 'universe';
}

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

function onkeydown(event: KeyboardEvent) {
  const modifier = event.metaKey || event.ctrlKey;
  if (modifier && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    finding = true;
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
  <header class="h-12 shrink-0 border-b border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] px-4 flex items-center gap-4">
    <div class="flex items-center gap-2 min-w-0">
      <span class="i-mingcute:earth-2-line w-5 h-5 text-[var(--color-crown-red)] shrink-0"></span>
      <h1 class="m-0 text-sm font-serif font-semibold tracking-tight">{t('common.appName')}</h1>
      <span class="text-[var(--color-border-strong)]">/</span>
      <button
        type="button"
        onclick={() => (settings = true)}
        title={t('project.settingsTitle')}
        class="flex items-center gap-1.5 px-2 py-0.5 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] text-xs font-serif cursor-pointer text-[var(--color-text-primary)] hover:border-[var(--color-border-strong)] max-w-56"
      >
        <span class="truncate">{project.current?.name}</span>
        <span class="i-mingcute:settings-3-line w-3 h-3 text-[var(--color-text-muted)]"></span>
      </button>
    </div>

    <nav class="flex items-center gap-1 bg-[var(--color-surface-elevated)] p-1 rounded-md border border-[var(--color-border-subtle)]">
      {#each modes as item (item.key)}
        <button
          type="button"
          onclick={() => (mode = item.key)}
          class="flex items-center gap-1.5 px-3 py-1 rounded text-xs font-mono transition-colors cursor-pointer border-0
          {mode === item.key
            ? 'bg-[var(--color-surface-panel)] text-[var(--color-crown-red)] font-semibold'
            : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] bg-transparent'}"
        >
          <span class="{item.icon} w-3.5 h-3.5"></span>
          <span>{item.label}</span>
        </button>
      {/each}
    </nav>

    <div class="ml-auto flex items-center gap-2">
      <Button
        variant="ghost"
        size="sm"
        icon="i-mingcute:arrow-left-line"
        title={t('shell.back')}
        disabled={!navigation.canGoBack}
        onclick={() => navigation.back()}
      />
      <Button
        variant="ghost"
        size="sm"
        icon="i-mingcute:arrow-right-line"
        title={t('shell.forward')}
        disabled={!navigation.canGoForward}
        onclick={() => navigation.forward()}
      />
      <button
        type="button"
        onclick={() => (finding = true)}
        class="flex items-center gap-1.5 px-2 py-1 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] text-[11px] font-mono text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] cursor-pointer"
      >
        <span class="i-mingcute:search-line w-3.5 h-3.5"></span>
        <span>{t('common.search')}</span>
        <kbd class="px-1 rounded bg-[var(--color-surface-base)] border border-[var(--color-border-subtle)] text-[10px]">{t('common.searchShortcut')}</kbd>
      </button>
      <Button
        variant="ghost"
        size="sm"
        title={t('common.language')}
        onclick={() => toggleLocale()}
      >
        {getLocale().toUpperCase()}
      </Button>
      <Button
        variant="ghost"
        size="sm"
        icon={dark ? 'i-mingcute:sun-line' : 'i-mingcute:moon-line'}
        title={dark ? t('common.themeLight') : t('common.themeDark')}
        onclick={toggleTheme}
      />
      <Button
        variant="ghost"
        size="sm"
        icon="i-mingcute:exit-line"
        title={t('shell.closeProject')}
        onclick={closeProject}
      />
    </div>
  </header>

  <div class="flex-1 min-h-0 flex">
    <aside class="w-64 shrink-0 border-r border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] flex flex-col py-2 gap-3 min-h-0">
      <div class="px-2">
        <div class="flex items-center gap-1.5 px-2 py-1 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
          <span class="i-mingcute:book-2-line w-3.5 h-3.5 text-[var(--color-crown-red)]"></span>
          <span>{t('sidebar.editorialStructure')}</span>
        </div>
        <div class="max-h-56 overflow-y-auto">
          <SectionTree selectedId={navigation.current} onselect={open} />
        </div>
      </div>
      <EntityBrowser selectedId={navigation.current} onselect={open} oncreate={() => (creating = true)} />
    </aside>

    <main class="flex-1 min-w-0 overflow-y-auto bg-[var(--color-surface-base)]">
      {#if mode === 'universe'}
        {#if navigation.current}
          {#key navigation.current}
            <EntityWorkspace id={navigation.current} onopen={open} />
          {/key}
        {:else}
          <EmptyState icon="i-mingcute:earth-2-line" title={t('shell.pickTitle')} description={t('shell.pickHint')}>
            <Button variant="primary" icon="i-mingcute:add-line" onclick={() => (creating = true)}>
              {t('sidebar.addEntity')}
            </Button>
          </EmptyState>
        {/if}
      {:else if mode === 'structure'}
        <StructureView
          selectedId={navigation.current}
          onselect={open}
          onentityremoved={(id) => navigation.forget(id)}
        />
      {:else if mode === 'library'}
        <LibraryView onopen={open} />
      {:else}
        <ConfigurationView />
      {/if}
    </main>

    {#if mode === 'universe' && navigation.current}
      <aside class="w-72 shrink-0 border-l border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] flex flex-col min-h-0">
        <div class="p-3 border-b border-[var(--color-border-subtle)] flex items-center gap-1.5">
          <span class="i-mingcute:link-2-line w-3.5 h-3.5 text-[var(--color-crown-red)]"></span>
          <span class="text-xs font-mono uppercase tracking-wider text-[var(--color-text-muted)] font-medium">
            {t('links.title')}
          </span>
        </div>
        <div class="flex-1 overflow-y-auto p-3">
          {#key navigation.current}
            <LinksPanel entity={navigation.current} onopen={open} />
          {/key}
        </div>
      </aside>
    {/if}
  </div>

  <footer class="h-7 shrink-0 border-t border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] px-4 flex items-center justify-between text-[11px] font-mono text-[var(--color-text-muted)]">
    <span class="flex items-center gap-1.5 min-w-0">
      <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-canon-established)]"></span>
      <span class="truncate">{t('status.localProject')} · {project.current?.path}</span>
    </span>
    <span>{catalog.data.entity_types.length} {t('status.types')}</span>
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
  <QuickFind
    onclose={() => (finding = false)}
    onselect={(id) => {
      finding = false;
      open(id);
    }}
  />
{/if}
{#if settings}
  <ProjectSettingsDialog onclose={() => (settings = false)} />
{/if}
