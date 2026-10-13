<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { EntityRef, PropertyValue, PropertyValueInput } from '@/shared/model';
import { typeLabelOf } from '@/shared/model';
import { PropertiesEditor, removePropertyValue, savePropertyValue } from '@/shared/properties';
import { revision } from '@/shared/revision';
import { Banner, Button, Dialog, SelectField, TemporalField, TextField } from '@/shared/ui';
import {
  getRelationship,
  type Lifecycle,
  type RelationshipDetail,
  type RelationshipOption,
  relationshipCandidates,
  relationshipOptions,
  saveRelationship,
} from '../api/relationship-api';

interface Props {
  /** The entity whose page the relationship is made from. */
  entity: EntityRef;
  /** An existing relationship to edit; absent when adding a new one. */
  relationshipId?: string | undefined;
  onclose: () => void;
}

const { entity, relationshipId, onclose }: Props = $props();

let options = $state<RelationshipOption[]>([]);
let optionIndex = $state('');
let candidates = $state<EntityRef[]>([]);
let search = $state('');
let targetId = $state('');
let qualifier = $state('');
let note = $state('');
let lifecycle = $state<Lifecycle>('active');
let period = $state<import('@/shared/model').Temporal>({ kind: 'unconstrained' });
let detail = $state<RelationshipDetail | null>(null);
let busy = $state(false);
let error = $state<string | null>(null);

const editing = $derived(relationshipId !== undefined);
const chosen = $derived(options[Number(optionIndex)]);

/**
 * What the author can pick. A symmetric type reads the same from both ends, and a label that
 * already appeared is the same choice, so each is offered once.
 */
const optionChoices = $derived.by(() => {
  const seen = new Set<string>();
  const choices: { value: string; label: string }[] = [];
  options.forEach((option, index) => {
    const type = catalog.data.relationship_types.find(
      (entry) => entry.key === option.relationship_type,
    );
    if (type?.directionality === 'undirected' && !option.outgoing) return;
    const label = catalog.relationshipLabel(option.relationship_type, option.outgoing);
    if (seen.has(label)) return;
    seen.add(label);
    choices.push({ value: String(index), label });
  });
  return choices.sort((a, b) => a.label.localeCompare(b.label));
});

$effect(() => {
  relationshipOptions(entity.id)
    .then((found) => {
      options = found;
      if (!editing && optionIndex === '') optionIndex = optionChoices[0]?.value ?? '';
    })
    .catch((failure) => {
      error = toCommandError(failure).message;
    });
});

$effect(() => {
  if (!chosen) return;
  const side = chosen.outgoing ? 'target' : 'source';
  let cancelled = false;
  relationshipCandidates(chosen.relationship_type, side, search.trim() || null)
    .then((found) => {
      if (!cancelled) candidates = found.filter((candidate) => candidate.id !== entity.id);
    })
    .catch(() => {
      if (!cancelled) candidates = [];
    });
  return () => {
    cancelled = true;
  };
});

$effect(() => {
  if (!relationshipId) return;
  getRelationship(relationshipId)
    .then((found) => {
      detail = found;
      const { relationship } = found;
      const outgoing = relationship.source === entity.id;
      const symmetric =
        catalog.data.relationship_types.find(
          (entry) => entry.key === relationship.relationship_type,
        )?.directionality === 'undirected';
      const index = options.findIndex(
        (option) =>
          option.relationship_type === relationship.relationship_type &&
          (symmetric ? option.outgoing : option.outgoing === outgoing),
      );
      optionIndex = index >= 0 ? String(index) : '';
      targetId = outgoing ? relationship.target : relationship.source;
      qualifier = relationship.qualifier ?? '';
      note = relationship.note ?? '';
      lifecycle = relationship.lifecycle;
      period = relationship.period;
    })
    .catch((failure) => {
      error = toCommandError(failure).message;
    });
});

async function save() {
  if (!chosen || !targetId) return;
  busy = true;
  error = null;
  try {
    const other = targetId;
    await saveRelationship({
      id: relationshipId ?? null,
      relationship_type: chosen.relationship_type,
      source: chosen.outgoing ? entity.id : other,
      target: chosen.outgoing ? other : entity.id,
      qualifier: qualifier.trim() === '' ? null : qualifier.trim(),
      context: detail?.relationship.context ?? null,
      lifecycle,
      period,
      note: note.trim() === '' ? null : note.trim(),
    });
    revision.bump();
    onclose();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

async function saveValue(input: PropertyValueInput) {
  await savePropertyValue(input);
  if (relationshipId) detail = await getRelationship(relationshipId);
  revision.bump();
}

async function removeValue(value: PropertyValue) {
  await removePropertyValue(value.id);
  if (relationshipId) detail = await getRelationship(relationshipId);
  revision.bump();
}
</script>

<Dialog
  title={editing ? t('relationship.editTitle') : t('relationship.addTitle')}
  description={t('relationship.from', { name: entity.name })}
  width="lg"
  {onclose}
>
  {#if error}<Banner kind="error">{error}</Banner>{/if}

  <SelectField
    label={t('relationship.type')}
    bind:value={optionIndex}
    options={optionChoices}
    placeholder={t('relationship.chooseType')}
    disabled={editing}
  />

  {#if chosen}
    {#if !editing}
      <TextField label={t('common.search')} bind:value={search} type="search" placeholder={t('relationship.searchTarget')} />
    {/if}
    <SelectField
      label={chosen.outgoing ? t('relationship.target') : t('relationship.source')}
      bind:value={targetId}
      options={candidates.map((candidate) => ({
        value: candidate.id,
        label: `${candidate.name} · ${typeLabelOf(candidate)}`,
      }))}
      placeholder={t('relationship.chooseEntity')}
      disabled={editing}
    />
    <TextField label={t('relationship.qualifier')} bind:value={qualifier} hint={t('relationship.qualifierHint')} />
    <SelectField
      label={t('relationship.lifecycle')}
      bind:value={lifecycle}
      options={[
        { value: 'active', label: t('relationship.active') },
        { value: 'historical', label: t('relationship.historical') },
      ]}
    />
    <TemporalField label={t('relationship.period')} bind:value={period} />
    <TextField label={t('relationship.note')} bind:value={note} multiline rows={3} />
  {/if}

  {#if detail && detail.definitions.length > 0}
    <div class="pt-3 border-t border-[var(--color-border-subtle)] flex flex-col gap-3">
      <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
        {t('relationship.properties')}
      </h3>
      <PropertiesEditor
        owner={{ kind: 'relationship', id: detail.relationship.id }}
        definitions={detail.definitions}
        values={detail.values}
        groups={catalog.data.property_groups}
        loadCandidates={async () => []}
        onsave={saveValue}
        onremove={removeValue}
      />
    </div>
  {/if}

  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy || !chosen || !targetId} onclick={save}>
      {t('common.save')}
    </Button>
  {/snippet}
</Dialog>
