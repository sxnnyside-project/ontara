<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Banner, Button, Dialog, SelectField, TextField } from '@/shared/ui';
import { createEntity, type Entity } from '../api/entity-api';

interface Props {
  initialType?: string | undefined;
  onclose: () => void;
  oncreated: (entity: Entity) => void;
}

const { initialType, onclose, oncreated }: Props = $props();

/** What can be made here: concrete types of the universe, not sections or files. */
const options = $derived(
  catalog.data.entity_types
    .filter((entry) => !entry.is_abstract && (entry.role === 'generic' || entry.role === 'event'))
    .map((entry) => ({ value: entry.key, label: entry.label })),
);

let entityType = $state('');

$effect(() => {
  if (entityType === '') entityType = initialType ?? 'character';
});
let name = $state('');
let summary = $state('');
let busy = $state(false);
let error = $state<string | null>(null);

async function create() {
  busy = true;
  error = null;
  try {
    const entity = await createEntity({ entity_type: entityType, name: name.trim(), summary });
    revision.bump();
    oncreated(entity);
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}
</script>

<Dialog title={t('entity.createTitle')} description={t('entity.createHint')} {onclose}>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  <SelectField label={t('entity.type')} bind:value={entityType} {options} />
  <TextField label={t('entity.name')} bind:value={name} focus />
  <TextField label={t('entity.summary')} bind:value={summary} hint={t('entity.summaryHint')} />
  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy || name.trim() === ''} onclick={create}>
      {t('entity.create')}
    </Button>
  {/snippet}
</Dialog>
