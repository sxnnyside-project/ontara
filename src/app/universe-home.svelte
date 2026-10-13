<script lang="ts">
import { listAssets } from '@/features/asset';
import { catalog } from '@/features/catalog';
import { consistency } from '@/features/consistency';
import { getEditorialTree } from '@/features/editorial';
import { type EntitySummary, listEntities } from '@/features/entity';
import { type ViewKey } from '@/features/navigation';
import {
  dismissOnboarding,
  isOnboardingDismissed,
  loadExampleUniverse,
  onboardingSteps,
  project,
  type UniverseCounts,
  universeCounts,
} from '@/features/project';
import { toCommandError } from '@/shared/backend';
import { getLocale, t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { typeLabelOf } from '@/shared/model';
import { revision } from '@/shared/revision';
import { Button, EmptyState, SectionHeading, entityIcon } from '@/shared/ui';

interface Props {
  onopen: (id: string) => void;
  oncreate: () => void;
  onshow: (view: ViewKey) => void;
}

const { onopen, oncreate, onshow }: Props = $props();

let entities = $state<EntitySummary[]>([]);
let sections = $state(0);
let assets = $state(0);
let loaded = $state(false);
let counts = $state<UniverseCounts | null>(null);
let dismissed = $state(false);
let loadingExample = $state(false);

$effect(() => {
  void revision.value;
  Promise.all([listEntities({}), getEditorialTree(), listAssets(), universeCounts()])
    .then(([found, tree, files, totals]) => {
      entities = found;
      sections = tree.length;
      assets = files.length;
      counts = totals;
      dismissed = isOnboardingDismissed(
        typeof localStorage === 'undefined' ? undefined : localStorage,
        project.current?.id ?? '',
      );
      loaded = true;
    })
    .catch(() => {
      loaded = true;
    });
});

const roleOf = (key: string) => catalog.data.entity_types.find((entry) => entry.key === key)?.role;
const universe = $derived(
  entities.filter((entry) => ['generic', 'event'].includes(roleOf(entry.entity_type) ?? '')),
);
const events = $derived(entities.filter((entry) => roleOf(entry.entity_type) === 'event'));
const recent = $derived([...universe].sort((a, b) => b.updated_at - a.updated_at).slice(0, 6));
const report = $derived(consistency.report);
const steps = $derived(counts ? onboardingSteps(counts) : []);
const finished = $derived(steps.filter((step) => step.done).length);
const showSteps = $derived(!dismissed && steps.length > 0 && finished < steps.length);

const STEP_ACTIONS: Partial<Record<(typeof steps)[number]['key'], () => void>> = {
  entities: () => oncreate(),
  structure: () => onshow('structure'),
  appearances: () => onshow('matrix'),
  events: () => onshow('timeline'),
};

async function loadExample() {
  loadingExample = true;
  try {
    const loadedExample = await loadExampleUniverse(getLocale());
    notices.success(t('onboarding.exampleLoaded', { entities: loadedExample.entities }));
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  } finally {
    loadingExample = false;
  }
}

function hideSteps() {
  dismissOnboarding(
    typeof localStorage === 'undefined' ? undefined : localStorage,
    project.current?.id ?? '',
  );
  dismissed = true;
}
</script>

<div class="h-full overflow-y-auto">
  <div class="w-full px-8 py-8 flex flex-col gap-8">
    <header class="flex flex-col gap-1.5">
      <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">{t('shell.universe')}</span>
      <h1 class="m-0 text-3xl font-serif font-medium">{project.current?.name}</h1>
      {#if project.current?.universe.summary}
        <p class="m-0 text-sm text-[var(--color-text-secondary)]">{project.current.universe.summary}</p>
      {:else}
        <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('home.premiseEmpty')}</p>
      {/if}
    </header>

    {#if loaded && universe.length === 0}
      <EmptyState
        icon="i-mingcute:earth-2-line"
        title={t('home.firstTitle')}
        description={t('home.firstHint')}
        steps={[
          { icon: 'i-mingcute:box-line', title: t('home.stepEntities'), text: t('home.stepEntitiesText') },
          { icon: 'i-mingcute:git-branch-line', title: t('home.stepRelate'), text: t('home.stepRelateText') },
          { icon: 'i-mingcute:book-2-line', title: t('home.stepStructure'), text: t('home.stepStructureText') },
        ]}
      >
        <Button variant="primary" icon="i-mingcute:add-line" onclick={oncreate}>{t('navigator.newEntity')}</Button>
        <Button icon="i-mingcute:book-2-line" onclick={() => onshow('structure')}>{t('navigator.manageStructure')}</Button>
        <Button variant="ghost" disabled={loadingExample} onclick={loadExample}>{t('onboarding.loadExample')}</Button>
      </EmptyState>
    {:else if loaded}
      <section class="grid grid-cols-4 gap-3">
        {#each [
          { label: t('navigator.entities'), value: universe.length, icon: 'i-mingcute:box-line', view: 'universe' as ViewKey },
          { label: t('tabs.timeline'), value: events.length, icon: 'i-mingcute:history-line', view: 'timeline' as ViewKey },
          { label: t('shell.structure'), value: sections, icon: 'i-mingcute:book-2-line', view: 'structure' as ViewKey },
          { label: t('shell.library'), value: assets, icon: 'i-mingcute:pic-line', view: 'library' as ViewKey },
        ] as card (card.label)}
          <button
            type="button"
            onclick={() => onshow(card.view)}
            class="text-left rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] p-4 cursor-pointer hover:border-[var(--color-border-strong)] transition-colors"
          >
            <span class="flex items-center gap-1.5 text-[11px] font-mono text-[var(--color-text-muted)]">
              <span class="{card.icon} w-3.5 h-3.5"></span>{card.label}
            </span>
            <span class="block mt-1 text-2xl font-serif tabular-nums">{card.value}</span>
          </button>
        {/each}
      </section>

      <div class="grid grid-cols-[minmax(0,1fr)_22rem] gap-8">
        <section class="flex flex-col gap-2">
          <SectionHeading title={t('home.recent')} />
          <div class="flex flex-col gap-1">
            {#each recent as item (item.id)}
              <button
                type="button"
                onclick={() => onopen(item.id)}
                class="flex items-center gap-3 px-3 py-2 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] text-left cursor-pointer hover:border-[var(--color-border-strong)]"
              >
                <span class="{entityIcon(catalog.chain(item.entity_type))} w-4 h-4 text-[var(--color-text-muted)]"></span>
                <span class="flex-1 min-w-0">
                  <span class="block text-sm truncate">{item.name}</span>
                  <span class="block text-[11px] text-[var(--color-text-muted)] truncate">{item.summary || typeLabelOf(item)}</span>
                </span>
                <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{typeLabelOf(item)}</span>
              </button>
            {/each}
          </div>
        </section>

        <section class="flex flex-col gap-3">
          <SectionHeading title={t('tabs.consistency')} />
          <button
            type="button"
            onclick={() => onshow('consistency')}
            class="text-left rounded-lg border bg-[var(--color-surface-panel)] p-4 cursor-pointer hover:border-[var(--color-border-strong)]
            {report && report.counts.errors > 0 ? 'border-[var(--color-finding-error)]/40' : 'border-[var(--color-border-subtle)]'}"
          >
            <span class="flex items-center gap-2 text-sm">
              <span class="{report && report.counts.errors + report.counts.warnings > 0 ? 'i-mingcute:alert-line text-[var(--color-finding-warning)]' : 'i-mingcute:check-circle-line text-[var(--color-canon-established)]'} w-4 h-4"></span>
              {report && report.counts.errors + report.counts.warnings > 0
                ? t('home.consistencyFound', { errors: report.counts.errors, warnings: report.counts.warnings })
                : t('home.consistencyOk')}
            </span>
          </button>
          {#if showSteps}
            <SectionHeading
              title={t('onboarding.title')}
              count={undefined}
            >
              {#snippet actions()}
                <span class="text-[10px] font-mono text-[var(--color-text-muted)]">
                  {t('onboarding.progress', { done: finished, total: steps.length })}
                </span>
              {/snippet}
            </SectionHeading>
            <ol class="m-0 p-0 list-none flex flex-col gap-1.5">
              {#each steps as step (step.key)}
                {@const act = STEP_ACTIONS[step.key]}
                <li class="flex items-start gap-2">
                  <span
                    class="mt-0.5 w-4 h-4 shrink-0 {step.done ? 'i-mingcute:check-circle-line text-[var(--color-canon-established)]' : 'i-mingcute:add-circle-line text-[var(--color-text-muted)]'}"
                    role="img"
                    aria-label={step.done ? t('onboarding.done') : ''}
                  ></span>
                  <span class="flex-1 min-w-0 flex flex-col">
                    {#if act && !step.done}
                      <button
                        type="button"
                        onclick={act}
                        class="p-0 border-0 bg-transparent text-left text-xs cursor-pointer text-[var(--color-text-primary)] hover:text-[var(--color-crown-red-text)]"
                      >
                        {t(`onboarding.${step.key}` as 'onboarding.entities')}
                      </button>
                    {:else}
                      <span class="text-xs {step.done ? 'text-[var(--color-text-muted)] line-through' : ''}">
                        {t(`onboarding.${step.key}` as 'onboarding.entities')}
                      </span>
                    {/if}
                    {#if !step.done}
                      <span class="text-[11px] text-[var(--color-text-muted)]">
                        {t(`onboarding.${step.key}Text` as 'onboarding.entitiesText')}
                      </span>
                    {/if}
                  </span>
                </li>
              {/each}
            </ol>
            <div>
              <Button size="sm" variant="ghost" onclick={hideSteps}>{t('onboarding.dismiss')}</Button>
            </div>
          {/if}
          <Button variant="primary" icon="i-mingcute:add-line" onclick={oncreate}>{t('navigator.newEntity')}</Button>
          <p class="m-0 text-[11px] text-[var(--color-text-muted)]">{t('home.pickHint')}</p>
        </section>
      </div>
    {/if}
  </div>
</div>
