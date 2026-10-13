<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t, tDynamic } from '@/shared/i18n';
import { describeMoment, type EntityRef, type Instant, type Temporal } from '@/shared/model';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Banner, Button, EmptyState, SectionHeading } from '@/shared/ui';
import {
  type Dossier,
  type DossierRecord,
  getDossier,
  getDossierSection,
  removeDossierRecord,
} from '../api/dossier-api';
import {
  blankRecord,
  cleanRecord,
  describeRecord,
  type DossierLabels,
  missingFields,
  sectionOf,
} from '../state/dossier-model';
import type { SectionDef } from '../state/dossier-schema';
import { saveDossierRecord } from '../api/dossier-api';
import FieldInput from './field-input.svelte';
import RecordDialog from './record-dialog.svelte';

interface Props {
  entity: string;
  /** The entity's type followed by its ancestors, nearest first. */
  typeChain: readonly string[];
}

const { entity, typeChain }: Props = $props();

let dossier = $state<Dossier | null>(null);
let offices = $state<{ value: string; label: string }[]>([]);
let failure = $state<string | null>(null);
let editing = $state<{ section: SectionDef; record: DossierRecord; existing: boolean } | null>(
  null,
);
/** The unsaved text of each profile form, by section. */
let drafts = $state<Record<string, DossierRecord>>({});
let saving = $state<string | null>(null);

const canonDefault = $derived(
  catalog.data.canon_states.find((state) => state.counts_as_canon)?.key ?? 'canon',
);

$effect(() => {
  void revision.value;
  let cancelled = false;
  getDossier(entity)
    .then((found) => {
      if (cancelled) return;
      dossier = found;
      failure = null;
      drafts = {};
    })
    .catch((error) => {
      if (!cancelled) failure = toCommandError(error).message;
    });
  getDossierSection('office')
    .then((found) => {
      if (cancelled) return;
      const records = found.sections[0]?.records ?? [];
      offices = records.map((record) => {
        const body = found.entities[String(record.body)]?.name ?? '';
        return {
          value: String(record.id),
          label: body === '' ? String(record.title) : `${record.title} · ${body}`,
        };
      });
    })
    .catch(() => {
      if (!cancelled) offices = [];
    });
  return () => {
    cancelled = true;
  };
});

const entities = $derived<Record<string, EntityRef>>(dossier?.entities ?? {});

const labels = $derived<DossierLabels>({
  entity: (id) => entities[id]?.name ?? '…',
  choice: (group, value) => tDynamic(`dossierChoices.${group}_${value}`, value),
  term: (vocabulary, key) =>
    catalog.data.vocabulary_terms.find((term) => term.vocabulary === vocabulary && term.key === key)
      ?.label ?? key,
  stance: (key) =>
    catalog.data.diplomatic_stances.find((stance) => stance.key === key)?.label ?? key,
  office: (id) => offices.find((office) => office.value === id)?.label ?? '…',
  period: (temporal) => periodText(temporal),
  moment: (instant: Instant) => describeMoment(instant),
  yes: t('dossier.yes'),
  no: t('dossier.no'),
});

function periodText(temporal: Temporal): string {
  switch (temporal.kind) {
    case 'unknown':
      return t('time.unknown');
    case 'unconstrained':
      return '';
    case 'at':
      return describeMoment(temporal.instant);
    case 'span':
      return `${temporal.start ? describeMoment(temporal.start) : '…'} → ${temporal.end ? describeMoment(temporal.end) : '…'}`;
  }
}

function begin(section: SectionDef, record?: DossierRecord) {
  editing = {
    section,
    record: record ?? blankRecord(section, entity, typeChain, canonDefault),
    existing: record !== undefined,
  };
}

async function remove(section: SectionDef, record: DossierRecord) {
  try {
    await removeDossierRecord(section.key, record);
    revision.bump();
  } catch (error) {
    notices.error(toCommandError(error).message);
  }
}

function profileDraft(section: SectionDef, stored: DossierRecord | undefined): DossierRecord {
  return drafts[section.key] ?? stored ?? blankRecord(section, entity, typeChain, canonDefault);
}

async function saveProfile(section: SectionDef, record: DossierRecord) {
  saving = section.key;
  try {
    await saveDossierRecord(section.key, cleanRecord(section, record));
    revision.bump();
  } catch (error) {
    notices.error(toCommandError(error).message);
  } finally {
    saving = null;
  }
}

const sections = $derived(
  (dossier?.sections ?? [])
    .map((entry) => ({ data: entry, def: sectionOf(entry.key) }))
    .filter(
      (entry): entry is { data: typeof entry.data; def: SectionDef } => entry.def !== undefined,
    ),
);
</script>

<div class="flex flex-col gap-8 max-w-5xl">
  {#if failure}<Banner kind="error">{failure}</Banner>{/if}

  {#if dossier && sections.length === 0}
    <EmptyState
      icon="i-mingcute:profile-line"
      title={t('dossier.empty')}
      description=""
    />
  {/if}

  {#each sections as { data, def } (def.key)}
    {@const title = tDynamic(`dossierSections.${def.key}`, def.key)}
    <section class="flex flex-col gap-3" aria-label={title}>
      <SectionHeading {title} count={def.shape === 'profile' ? undefined : data.records.length}>
        {#snippet actions()}
          {#if def.shape !== 'profile'}
            <Button size="sm" variant="ghost" icon="i-mingcute:add-line" onclick={() => begin(def)}>
              {t('dossier.add')}
            </Button>
          {/if}
        {/snippet}
      </SectionHeading>
      <p class="m-0 -mt-2 text-xs text-[var(--color-text-muted)]">
        {tDynamic(`dossierSections.${def.key}_hint`, '')}
      </p>

      {#if def.shape === 'profile'}
        {@const stored = data.records[0]}
        {@const draft = profileDraft(def, stored)}
        <div class="grid gap-3 md:grid-cols-2">
          {#each def.fields as field (field.name)}
            <FieldInput
              {field}
              {entities}
              {offices}
              bind:record={() => draft, (value) => (drafts = { ...drafts, [def.key]: value })}
            />
          {/each}
        </div>
        <div>
          <Button
            variant="primary"
            size="sm"
            disabled={saving === def.key ||
              drafts[def.key] === undefined ||
              missingFields(def, draft).length > 0}
            onclick={() => saveProfile(def, draft)}
          >
            {t('common.save')}
          </Button>
        </div>
      {:else if data.records.length === 0}
        <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('dossier.none')}</p>
      {:else}
        <ul class="m-0 p-0 list-none flex flex-col divide-y divide-[var(--color-border-subtle)] border-y border-[var(--color-border-subtle)]">
          {#each data.records as record}
            {@const line = describeRecord(def, record, labels)}
            <li class="flex items-center gap-3 py-2 min-h-10">
              <div class="flex-1 min-w-0">
                <div class="text-sm truncate">{line.title}</div>
                {#if line.detail}
                  <div class="text-xs text-[var(--color-text-muted)] truncate">{line.detail}</div>
                {/if}
              </div>
              <Button size="sm" variant="ghost" icon="i-mingcute:edit-line" title={t('dossier.edit')} onclick={() => begin(def, record)} />
              <Button size="sm" variant="ghost" icon="i-mingcute:delete-2-line" title={t('common.remove')} onclick={() => remove(def, record)} />
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/each}
</div>

{#if editing}
  <RecordDialog
    section={editing.section}
    record={editing.record}
    existing={editing.existing}
    {entities}
    {offices}
    onsaved={() => {
      editing = null;
      revision.bump();
    }}
    onclose={() => (editing = null)}
  />
{/if}
