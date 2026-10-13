<script lang="ts">
import { navigation } from '@/features/navigation';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Banner, Button, EmptyState, FindingCard, InlineSelect, ViewFrame } from '@/shared/ui';
import type { Rule } from '../api/consistency-api';
import { consistency } from '../state/consistency.svelte';
import { explanationKey, explanationParams, filterFindings, titleKey } from '../state/finding-text';

interface Props {
  /** Follows an entity: shows its page. */
  onopen: (id: string) => void;
}

const { onopen }: Props = $props();

const RULES: Rule[] = [
  'participant_after_death',
  'participant_before_birth',
  'parent_not_older',
  'effect_before_cause',
  'presence_and_absence',
  'two_places_at_once',
  'canon_on_uncanon',
  'required_property_missing',
  'viewpoint_absent',
  'office_overfilled',
  'section_not_placed',
  'event_undated',
  'no_connections',
  'duplicate_name',
];

let severity = $state('');
let rule = $state('');
let failure = $state<string | null>(null);

$effect(() => {
  void revision.value;
  consistency
    .refresh()
    .then(() => {
      failure = null;
    })
    .catch((error) => {
      failure = toCommandError(error).message;
    });
});

const report = $derived(consistency.report);
const shown = $derived(report ? filterFindings(report.findings, severity, rule) : []);
</script>

<ViewFrame
  icon="i-mingcute:shield-shape-line"
  title={t('consistency.title')}
  description={t('consistency.subtitle')}
>
  {#snippet actions()}
    <Button icon="i-mingcute:refresh-2-line" disabled={consistency.loading} onclick={() => revision.bump()}>
      {t('consistency.rerun')}
    </Button>
  {/snippet}
  {#snippet toolbar()}
    {#if report}
      <span class="inline-flex items-center gap-1.5 text-xs font-mono text-[var(--color-finding-error)]">
        <span class="i-mingcute:close-circle-fill w-3.5 h-3.5"></span>{t('consistency.errors', { count: report.counts.errors })}
      </span>
      <span class="inline-flex items-center gap-1.5 text-xs font-mono text-[var(--color-finding-warning)]">
        <span class="i-mingcute:alert-fill w-3.5 h-3.5"></span>{t('consistency.warnings', { count: report.counts.warnings })}
      </span>
      <span class="inline-flex items-center gap-1.5 text-xs font-mono text-[var(--color-finding-info)]">
        <span class="i-mingcute:information-line w-3.5 h-3.5"></span>{t('consistency.infos', { count: report.counts.infos })}
      </span>
    {/if}
    <span class="w-px h-5 bg-[var(--color-border-subtle)]"></span>
    <InlineSelect
      label={t('consistency.severity')}
      bind:value={severity}
      options={[
        { value: 'error', label: t('consistency.severityError') },
        { value: 'warning', label: t('consistency.severityWarning') },
        { value: 'info', label: t('consistency.severityInfo') },
      ]}
      placeholder={t('consistency.all')}
    />
    <InlineSelect
      label={t('consistency.rule')}
      bind:value={rule}
      options={RULES.map((value) => ({ value, label: t(titleKey(value)) }))}
      placeholder={t('consistency.all')}
    />
    <span class="flex-1"></span>
    {#if report}
      <span class="text-[11px] font-mono text-[var(--color-text-muted)]">
        {t('consistency.examined', {
          entities: report.examined.entities,
          relationships: report.examined.relationships,
          appearances: report.examined.appearances,
          events: report.examined.events,
        })}
      </span>
    {/if}
  {/snippet}

  <div class="grid gap-3 items-start grid-cols-[repeat(auto-fill,minmax(26rem,1fr))]">
    {#if failure}<div class="col-span-full"><Banner kind="error">{failure}</Banner></div>{/if}
    {#if report && report.findings.length === 0}
      <div class="col-span-full"><EmptyState icon="i-mingcute:shield-shape-line" title={t('consistency.cleanTitle')} description={t('consistency.cleanHint')} /></div>
    {:else if report && shown.length === 0}
      <div class="col-span-full"><EmptyState icon="i-mingcute:filter-line" title={t('consistency.noMatchTitle')} description={t('consistency.noMatchHint')}>
        <Button onclick={() => { severity = ''; rule = ''; }}>{t('navigator.clearFilters')}</Button>
      </EmptyState></div>
    {:else}
      {#each shown as finding (finding.fingerprint)}
        <FindingCard
          severity={finding.severity}
          ruleCode={finding.code}
          title={t(titleKey(finding.rule))}
          description={t(explanationKey(finding.rule), explanationParams(finding))}
          affected={[finding.subject.name, ...finding.related.map((entity) => entity.name)]}
          onresolve={() => {
            navigation.open(finding.subject.id, 'universe');
            onopen(finding.subject.id);
          }}
        />
      {/each}
    {/if}
  </div>
</ViewFrame>
