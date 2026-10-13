<script lang="ts">
import { toCommandError } from '@/shared/backend';
import { t, tDynamic } from '@/shared/i18n';
import type { EntityRef } from '@/shared/model';
import { Banner, Button, Dialog } from '@/shared/ui';
import { type DossierRecord, saveDossierRecord } from '../api/dossier-api';
import { cleanRecord, isKeyField, missingFields } from '../state/dossier-model';
import type { SectionDef } from '../state/dossier-schema';
import FieldInput from './field-input.svelte';

interface Props {
  section: SectionDef;
  /** The record to start from: a blank one to add, a stored one to edit. */
  record: DossierRecord;
  /** Whether the record is already stored. */
  existing: boolean;
  entities: Record<string, EntityRef>;
  offices: { value: string; label: string }[];
  onsaved: () => void;
  onclose: () => void;
}

const props: Props = $props();
const { section, existing, entities, offices, onsaved, onclose } = $derived(props);

// The dialog edits a copy; the record it was opened with is only where it starts.
const start = () => props.record;
let record = $state<DossierRecord>(start());
let error = $state<string | null>(null);
let busy = $state(false);

const missing = $derived(missingFields(section, record));
const title = $derived(tDynamic(`dossierSections.${section.key}`, section.key));
const keyed = $derived(section.shape === 'links' && existing);

async function save() {
  busy = true;
  error = null;
  try {
    await saveDossierRecord(section.key, cleanRecord(section, record));
    onsaved();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}
</script>

<Dialog {title} description={existing ? t('dossier.editing') : t('dossier.new')} width="md" {onclose}>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  {#if keyed}
    <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('dossier.linkKeyHint')}</p>
  {/if}
  {#each section.fields as field (field.name)}
    <FieldInput
      {field}
      bind:record
      {entities}
      {offices}
      locked={keyed && isKeyField(section, field)}
    />
  {/each}

  {#snippet actions()}
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy || missing.length > 0} onclick={save}>
      {t('common.save')}
    </Button>
  {/snippet}
</Dialog>
