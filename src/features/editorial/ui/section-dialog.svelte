<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Banner, Button, Dialog, SelectField, TextField } from '@/shared/ui';
import { addSection, type Section } from '../api/editorial-api';

interface Props {
  /** The section the new one goes inside, or `null` for the top of the work. */
  parent: Section | null;
  onclose: () => void;
  oncreated: (id: string) => void;
}

const { parent, onclose, oncreated }: Props = $props();

const parentDepth = $derived(
  parent
    ? (catalog.data.editorial_levels.find((level) => level.key === parent.level)?.depth ?? -1)
    : -1,
);

/** A section goes inside a shallower one, so only deeper levels are offered. */
const levels = $derived(
  catalog.data.editorial_levels
    .filter((level) => level.depth > parentDepth)
    .sort((a, b) => a.depth - b.depth),
);

/** The entity types that can stand for a section. */
const sectionTypes = $derived(
  catalog.data.entity_types.filter(
    (entry) => entry.role === 'editorial_unit' && !entry.is_abstract,
  ),
);

let level = $state('');
let entityType = $state('');
let name = $state('');
let busy = $state(false);
let error = $state<string | null>(null);

$effect(() => {
  if (!levels.some((candidate) => candidate.key === level)) level = levels[0]?.key ?? '';
});

// The type that carries the level's own key is the natural one; a project can pick another.
$effect(() => {
  const natural = sectionTypes.find((entry) => entry.key === level);
  if (natural) entityType = natural.key;
  else if (!sectionTypes.some((entry) => entry.key === entityType)) {
    entityType = sectionTypes[0]?.key ?? '';
  }
});

async function create() {
  busy = true;
  error = null;
  try {
    const created = await addSection({
      entity_type: entityType,
      level,
      name: name.trim(),
      parent: parent?.entity.id ?? null,
    });
    revision.bump();
    oncreated(created.id);
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}
</script>

<Dialog
  title={t('structure.addTitle')}
  description={parent ? t('structure.insideOf', { name: parent.entity.name }) : t('structure.topLevel')}
  {onclose}
>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  {#if levels.length === 0}
    <Banner kind="warning">{t('structure.noDeeperLevel')}</Banner>
  {:else}
    <SelectField
      label={t('structure.level')}
      bind:value={level}
      options={levels.map((entry) => ({ value: entry.key, label: entry.label }))}
    />
    <SelectField
      label={t('structure.entityType')}
      bind:value={entityType}
      options={sectionTypes.map((entry) => ({ value: entry.key, label: entry.label }))}
    />
    <TextField label={t('entity.name')} bind:value={name} focus />
  {/if}
  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy || name.trim() === '' || levels.length === 0} onclick={create}>
      {t('entity.create')}
    </Button>
  {/snippet}
</Dialog>
