<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { getLocale, t, toggleLocale } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { Banner, Button, ConfirmDialog, OntaraLogo, SmokeBackground, TextField } from '@/shared/ui';
import { chooseFolder } from '../api/choose-folder';
import { updateUniverse } from '../api/project-api';
import { joinPath, projectFolderName } from '../state/project-path';
import { project } from '../state/project.svelte';

interface Props {
  /** Called once a project is open, so the application can load what it contains. */
  onopened: () => void | Promise<void>;
}

const { onopened }: Props = $props();

type ActionTab = 'create' | 'open';
let activeAction = $state<ActionTab>('create');

let name = $state('');
let summary = $state('');
let description = $state('');
let showExtraDetails = $state(false);

let location = $state<string | null>(null);
let busy = $state(false);
let error = $state<string | null>(null);
let migrating = $state<string | null>(null);

// Dark / Light Theme
let isDarkTheme = $state(true);
function toggleTheme() {
  isDarkTheme = !isDarkTheme;
  if (isDarkTheme) {
    document.documentElement.removeAttribute('data-theme');
  } else {
    document.documentElement.setAttribute('data-theme', 'light');
  }
}

const target = $derived(location ? joinPath(location, projectFolderName(name)) : null);
const canCreate = $derived(name.trim() !== '' && location !== null && !busy);

async function pickLocation() {
  const folder = await chooseFolder(t('welcome.chooseLocation'));
  if (folder) location = folder;
}

async function create() {
  if (!target) return;
  busy = true;
  error = null;
  try {
    await project.create(target, name.trim());
    // Apply premise and narrative description if provided
    if (summary.trim() !== '' || description.trim() !== '') {
      const updated = await updateUniverse(summary.trim(), description.trim());
      project.refresh(updated);
    }
    await onopened();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

async function open(path: string, allowMigration = false) {
  busy = true;
  error = null;
  try {
    await project.open(path, allowMigration);
    migrating = null;
    await onopened();
  } catch (failure) {
    const failed = toCommandError(failure);
    if (failed.code === 'migration_required') {
      migrating = path;
    } else {
      error = failed.message;
    }
  } finally {
    busy = false;
  }
}

async function pickAndOpen() {
  const folder = await chooseFolder(t('welcome.openTitle'));
  if (folder) await open(folder);
}

function forgetRecent(path: string) {
  project.recent = project.recent.filter((entry) => entry.path !== path);
  notices.show('info', t('welcome.forgotten'));
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Enter' && activeAction === 'create' && canCreate) {
    event.preventDefault();
    create();
  }
}
</script>

<svelte:window onkeydown={handleKeyDown} />

<main class="relative h-screen w-screen overflow-hidden bg-[var(--color-surface-base)] text-[var(--color-text-primary)] font-sans flex flex-col select-none">
  <!-- Living Atmospheric Smoke Background with Organic Fluid Motion -->
  <SmokeBackground />

  <!-- Outer Split Screen Layout (2 Balanced Sections with Full Space Utilization) -->
  <div class="relative z-10 flex-1 grid grid-cols-1 lg:grid-cols-12 overflow-hidden">
    <!-- =====================================================================
         LEFT SECTION (5 cols): Identidad Editorial, Manifiesto y Códices Recientes
         ===================================================================== -->
    <section class="lg:col-span-5 border-b lg:border-b-0 lg:border-r border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/80 backdrop-blur-md flex flex-col justify-between p-8 lg:p-10 overflow-y-auto">
      <div class="flex flex-col gap-6">
        <!-- Official Logo & Brand Lockup (Luna + Mundo acoplados, sin caja AI Slop) -->
        <div class="flex items-center gap-4">
          <OntaraLogo size="xl" animated />
          <div>
            <h1 class="text-3xl lg:text-4xl font-serif font-normal tracking-wide text-[var(--color-celestial-gold)] m-0 leading-none drop-shadow-[0_2px_12px_rgba(223,183,108,0.25)]">
              {t('common.appName')}
            </h1>
            <p class="text-[11px] font-mono tracking-wider uppercase mt-2 m-0 font-medium flex items-center gap-1.5">
              <span class="text-[#a855f7]">{t('welcome.brandStudio')}</span>
              <span class="text-[var(--color-text-muted)] opacity-60">×</span>
              <span class="text-[var(--color-crown-red)]">{t('welcome.brandEditorial')}</span>
            </p>
          </div>
        </div>

        <!-- Editorial Manifesto Quote -->
        <blockquote class="m-0 pl-3.5 border-l-2 border-[var(--color-crown-red)] py-1">
          <p class="text-xs font-serif italic text-[var(--color-text-secondary)] leading-relaxed m-0">
            "{t('welcome.manifesto')}"
          </p>
        </blockquote>

        <!-- Recent Codices / Universes Registry -->
        <div class="mt-2 flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="i-mingcute:archive-line w-4 h-4 text-[var(--color-meridian-blue)]"></span>
              <h2 class="text-xs font-mono uppercase tracking-wider text-[var(--color-text-muted)] font-medium m-0">
                {t('welcome.recentTitle')}
              </h2>
            </div>
            {#if project.recent.length > 0}
              <span class="text-[11px] font-mono text-[var(--color-text-muted)] px-1.5 py-0.2 rounded bg-[var(--color-surface-elevated)] border border-[var(--color-border-subtle)]">
                {project.recent.length}
              </span>
            {/if}
          </div>

          <div class="flex flex-col gap-2 max-h-[340px] overflow-y-auto pr-1">
            {#each project.recent as entry (entry.path)}
              <div
                class="group relative flex items-center justify-between p-3 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)]/70 hover:bg-[var(--color-surface-elevated)] hover:border-[var(--color-border-strong)] transition-all shadow-xs"
              >
                <button
                  type="button"
                  disabled={busy}
                  onclick={() => open(entry.path)}
                  class="flex-1 min-w-0 flex items-start gap-3 bg-transparent border-0 text-left p-0 cursor-pointer"
                >
                  <span class="i-mingcute:book-2-line w-4 h-4 text-[var(--color-crown-red)] shrink-0 mt-0.5"></span>
                  <div class="min-w-0 flex-1">
                    <span class="block text-sm font-serif font-medium text-[var(--color-text-primary)] group-hover:text-[var(--color-crown-red)] transition-colors truncate">
                      {entry.name}
                    </span>
                    <span class="block text-[11px] font-mono text-[var(--color-text-muted)] truncate mt-0.5" title={entry.path}>
                      {entry.path}
                    </span>
                  </div>
                </button>

                <div class="flex items-center gap-1 pl-2 shrink-0">
                  <button
                    type="button"
                    disabled={busy}
                    onclick={() => open(entry.path)}
                    title={t('welcome.resumeProject')}
                    class="p-1.5 rounded text-[var(--color-text-secondary)] hover:text-[var(--color-crown-red)] hover:bg-[var(--color-surface-hover)] bg-transparent border-0 cursor-pointer transition-colors"
                  >
                    <span class="i-mingcute:arrow-right-line w-4 h-4"></span>
                  </button>
                  <button
                    type="button"
                    aria-label={t('welcome.forget')}
                    title={t('welcome.forget')}
                    onclick={() => forgetRecent(entry.path)}
                    class="p-1.5 rounded text-[var(--color-text-muted)] hover:text-[var(--color-finding-error)] hover:bg-[var(--color-surface-hover)] bg-transparent border-0 cursor-pointer transition-colors opacity-60 group-hover:opacity-100"
                  >
                    <span class="i-mingcute:close-line w-3.5 h-3.5"></span>
                  </button>
                </div>
              </div>
            {:else}
              <div class="p-6 rounded-lg border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-surface-base)]/40 text-center flex flex-col items-center gap-2">
                <span class="i-mingcute:quill-pen-line w-6 h-6 text-[var(--color-text-muted)]"></span>
                <p class="text-xs font-serif text-[var(--color-text-secondary)] m-0 leading-relaxed max-w-xs">
                  {t('welcome.noRecent')}
                </p>
              </div>
            {/each}
          </div>
        </div>
      </div>

      <!-- Left Footer / Data Sovereignty & Preferences (Sin jerga SQLite) -->
      <div class="pt-6 border-t border-[var(--color-border-subtle)] flex items-center justify-between text-[11px] font-mono text-[var(--color-text-muted)]">
        <div class="flex items-center gap-1.5">
          <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-canon-established)]"></span>
          <span>{t('welcome.localBadge')}</span>
        </div>

        <div class="flex items-center gap-2">
          <!-- Language Toggle -->
          <button
            type="button"
            onclick={toggleLocale}
            title="{t('common.language')}: {getLocale().toUpperCase()}"
            class="flex items-center gap-1 px-2 py-1 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] hover:border-[var(--color-border-strong)] text-[var(--color-text-primary)] cursor-pointer"
          >
            <span class="i-mingcute:translate-line w-3 h-3 text-[var(--color-crown-red)]"></span>
            <span class="font-bold">{getLocale().toUpperCase()}</span>
          </button>

          <!-- Theme Toggle -->
          <button
            type="button"
            onclick={toggleTheme}
            title={isDarkTheme ? t('common.themeLight') : t('common.themeDark')}
            class="w-7 h-7 flex items-center justify-center rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] hover:border-[var(--color-border-strong)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] cursor-pointer"
          >
            {#if isDarkTheme}
              <span class="i-mingcute:sun-line w-3.5 h-3.5"></span>
            {:else}
              <span class="i-mingcute:moon-line w-3.5 h-3.5"></span>
            {/if}
          </button>
        </div>
      </div>
    </section>

    <!-- =====================================================================
         RIGHT SECTION (7 cols): Taller de Creación Completo y Apertura
         ===================================================================== -->
    <section class="lg:col-span-7 flex flex-col justify-between p-8 lg:p-12 overflow-y-auto">
      <div class="max-w-2xl mx-auto w-full flex flex-col gap-6">
        {#if error}
          <Banner kind="error">{error}</Banner>
        {/if}

        <!-- Mode Navigation Tabs -->
        <div class="flex items-center gap-2 border-b border-[var(--color-border-subtle)] pb-3">
          <button
            type="button"
            onclick={() => (activeAction = 'create')}
            class="flex items-center gap-2 pb-2 px-1 text-sm font-mono tracking-tight transition-all cursor-pointer bg-transparent border-0 relative
            {activeAction === 'create'
              ? 'text-[var(--color-crown-red)] font-semibold'
              : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'}"
          >
            <span class="i-mingcute:add-circle-line w-4 h-4"></span>
            <span>{t('welcome.createTab')}</span>
            {#if activeAction === 'create'}
              <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-[var(--color-crown-red)] rounded-full"></div>
            {/if}
          </button>

          <button
            type="button"
            onclick={() => (activeAction = 'open')}
            class="flex items-center gap-2 pb-2 px-1 text-sm font-mono tracking-tight transition-all cursor-pointer bg-transparent border-0 relative ml-4
            {activeAction === 'open'
              ? 'text-[var(--color-crown-red)] font-semibold'
              : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'}"
          >
            <span class="i-mingcute:folder-open-line w-4 h-4"></span>
            <span>{t('welcome.openTab')}</span>
            {#if activeAction === 'open'}
              <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-[var(--color-crown-red)] rounded-full"></div>
            {/if}
          </button>
        </div>

        <!-- Mode Content 1: Fundar Nuevo Universo (Con todas las funciones de proyecto) -->
        {#if activeAction === 'create'}
          <div class="p-6 lg:p-7 rounded-xl border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/90 backdrop-blur-md flex flex-col gap-4 shadow-md">
            <div>
              <h2 class="text-xl font-serif font-medium text-[var(--color-text-primary)] m-0">
                {t('welcome.createTitle')}
              </h2>
              <p class="text-xs text-[var(--color-text-secondary)] m-0 mt-1 font-sans">
                {t('welcome.subtitle')}
              </p>
            </div>

            <!-- Universe Name Field -->
            <TextField
              label={t('welcome.projectName')}
              bind:value={name}
              placeholder={t('welcome.projectNamePlaceholder')}
              focus
            />

            <!-- Toggle for Editorial Premise & World Description -->
            <div class="flex flex-col gap-3">
              <button
                type="button"
                onclick={() => (showExtraDetails = !showExtraDetails)}
                class="flex items-center gap-1.5 text-xs font-mono text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] bg-transparent border-0 p-0 cursor-pointer self-start"
              >
                <span class="{showExtraDetails ? 'i-mingcute:down-line' : 'i-mingcute:right-line'} w-3.5 h-3.5 text-[var(--color-crown-red)]"></span>
                <span>{t('welcome.optionalDetails')}</span>
              </button>

              {#if showExtraDetails}
                <div class="flex flex-col gap-3 pl-3 border-l border-[var(--color-border-subtle)] mt-1">
                  <TextField
                    label={t('welcome.premise')}
                    bind:value={summary}
                    placeholder={t('welcome.premisePlaceholder')}
                  />
                  <TextField
                    label={t('welcome.description')}
                    bind:value={description}
                    placeholder={t('welcome.descriptionPlaceholder')}
                    multiline
                    rows={3}
                  />
                </div>
              {/if}
            </div>

            <!-- Storage Location & Container Path Preview -->
            <div class="flex flex-col gap-2 pt-1 border-t border-[var(--color-border-subtle)]/60">
              <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] font-medium">
                {t('welcome.location')}
              </span>

              <div class="flex items-center gap-2.5">
                <Button icon="i-mingcute:folder-open-line" onclick={pickLocation}>
                  {t('welcome.browseFolder')}
                </Button>
                <div class="flex-1 min-w-0 p-2 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] text-xs font-mono text-[var(--color-text-secondary)] truncate">
                  {location ?? t('welcome.noLocation')}
                </div>
              </div>

              {#if target}
                <div class="mt-1 flex items-center gap-2.5 p-2.5 rounded bg-[var(--color-crown-red-subtle)] border border-[var(--color-crown-red-border)] text-xs font-mono text-[var(--color-text-primary)]">
                  <span class="i-mingcute:folder-download-line w-4 h-4 text-[var(--color-crown-red)] shrink-0"></span>
                  <div class="min-w-0 flex-1 truncate">
                    <span class="text-[var(--color-text-muted)] text-[10px] uppercase block">
                      {t('welcome.destinationPreview')}
                    </span>
                    <span class="font-medium text-[var(--color-crown-red)] truncate block">{target}</span>
                  </div>
                </div>
              {/if}
            </div>

            <!-- Create Action Trigger -->
            <div class="pt-2 flex items-center justify-between">
              <span class="text-[11px] font-mono text-[var(--color-text-muted)]">
                {t('welcome.pressEnter')}
              </span>
              <Button
                variant="primary"
                icon="i-mingcute:hammer-line"
                disabled={!canCreate}
                onclick={create}
              >
                {#if busy}
                  <span class="i-mingcute:loading-line animate-spin w-4 h-4"></span>
                {/if}
                <span>{t('welcome.createAction')}</span>
              </Button>
            </div>
          </div>

        <!-- Mode Content 2: Abrir Códice Existente (Sin repetición ni AI Slop Icon) -->
        {:else}
          <div class="p-8 lg:p-10 rounded-xl border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/90 backdrop-blur-md flex flex-col gap-6 shadow-md">
            <div>
              <h2 class="text-xl font-serif font-medium text-[var(--color-text-primary)] m-0">
                {t('welcome.openTitle')}
              </h2>
              <p class="text-xs text-[var(--color-text-secondary)] m-0 mt-2 leading-relaxed font-sans max-w-lg">
                {t('welcome.openHint')}
              </p>
            </div>

            <div class="p-6 rounded-lg border border-dashed border-[var(--color-border-strong)] bg-[var(--color-surface-base)]/50 flex flex-col items-center gap-3 text-center">
              <span class="i-mingcute:folder-open-line w-8 h-8 text-[var(--color-meridian-blue)] opacity-80"></span>
              <p class="text-xs font-mono text-[var(--color-text-muted)] m-0 max-w-sm">
                {t('welcome.openFolderHint')}
              </p>
              <div class="mt-2">
                <Button
                  icon="i-mingcute:folder-open-line"
                  disabled={busy}
                  onclick={pickAndOpen}
                  variant="primary"
                >
                  <span>{t('welcome.openAction')}</span>
                </Button>
              </div>
            </div>
          </div>
        {/if}

        <!-- Archival Pillars Feature Highlights -->
        <div class="pt-2 flex flex-col gap-3">
          <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] font-medium">
            {t('welcome.pillarsTitle')}
          </span>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div class="p-3.5 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/50 flex flex-col gap-1.5">
              <div class="flex items-center gap-2">
                <span class="i-mingcute:git-branch-line w-4 h-4 text-[var(--color-meridian-blue)]"></span>
                <span class="text-xs font-serif font-medium text-[var(--color-text-primary)] truncate">
                  {t('welcome.pillarGraph')}
                </span>
              </div>
              <p class="text-[11px] text-[var(--color-text-secondary)] font-sans m-0 leading-relaxed">
                {t('welcome.pillarGraphDesc')}
              </p>
            </div>

            <div class="p-3.5 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/50 flex flex-col gap-1.5">
              <div class="flex items-center gap-2">
                <span class="i-mingcute:history-line w-4 h-4 text-[var(--color-crown-red)]"></span>
                <span class="text-xs font-serif font-medium text-[var(--color-text-primary)] truncate">
                  {t('welcome.pillarChronology')}
                </span>
              </div>
              <p class="text-[11px] text-[var(--color-text-secondary)] font-sans m-0 leading-relaxed">
                {t('welcome.pillarChronologyDesc')}
              </p>
            </div>

            <div class="p-3.5 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/50 flex flex-col gap-1.5">
              <div class="flex items-center gap-2">
                <span class="i-mingcute:shield-shape-line w-4 h-4 text-[var(--color-canon-established)]"></span>
                <span class="text-xs font-serif font-medium text-[var(--color-text-primary)] truncate">
                  {t('welcome.pillarConsistency')}
                </span>
              </div>
              <p class="text-[11px] text-[var(--color-text-secondary)] font-sans m-0 leading-relaxed">
                {t('welcome.pillarConsistencyDesc')}
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</main>

{#if migrating}
  <ConfirmDialog
    title={t('welcome.migrationTitle')}
    confirmLabel={t('welcome.migrate')}
    {busy}
    onconfirm={() => migrating && open(migrating, true)}
    oncancel={() => (migrating = null)}
  >
    <p class="m-0 text-xs text-[var(--color-text-secondary)] leading-relaxed">
      {t('welcome.migrationBody')}
    </p>
  </ConfirmDialog>
{/if}
