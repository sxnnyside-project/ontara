<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import type { EntityRef } from '@/shared/model';
import { revision } from '@/shared/revision';
import { Banner, Button, Dialog, SelectField, TextField } from '@/shared/ui';
import { type MatrixCell, saveAppearance, setAppearanceKind } from '../api/matrix-api';

interface Props {
  entity: EntityRef;
  context: EntityRef;
  /** The appearance being edited. */
  cell: MatrixCell;
  /** Entities that can tell the scene. */
  viewpoints: EntityRef[];
  onclose: () => void;
}

const { entity, context, cell, viewpoints, onclose }: Props = $props();

// The dialog edits the appearance it was opened for; later changes to the prop do not apply.
const initial = () => cell;
let kind = $state(initial().kind as string);
let viewpoint = $state(initial().viewpoint ?? '');
let busy = $state(false);
let error = $state<string | null>(null);

const kinds = $derived(
  catalog.data.appearance_kinds.map((entry) => ({ value: entry.key, label: entry.label })),
);

async function save() {
  busy = true;
  error = null;
  try {
    await saveAppearance({
      id: cell.appearance,
      entity: entity.id,
      context: context.id,
      kind,
      viewpoint: viewpoint === '' ? null : viewpoint,
      note: null,
    });
    revision.bump();
    onclose();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}

async function clear() {
  busy = true;
  try {
    await setAppearanceKind(entity.id, context.id, null);
    revision.bump();
    onclose();
  } catch (failure) {
    error = toCommandError(failure).message;
  } finally {
    busy = false;
  }
}
</script>

<Dialog
  title={t('matrixView.editTitle')}
  description={t('matrixView.editHint', { entity: entity.name, context: context.name })}
  {onclose}
>
  {#if error}<Banner kind="error">{error}</Banner>{/if}
  <SelectField label={t('matrixView.kind')} bind:value={kind} options={kinds} />
  <SelectField
    label={t('matrixView.viewpoint')}
    bind:value={viewpoint}
    options={viewpoints.map((other) => ({ value: other.id, label: other.name }))}
    placeholder={t('matrixView.noViewpoint')}
    hint={t('matrixView.viewpointHint')}
  />
  {#snippet actions()}
    <Button variant="danger" disabled={busy} onclick={clear}>{t('matrixView.clear')}</Button>
    <Button variant="ghost" onclick={onclose}>{t('common.cancel')}</Button>
    <Button variant="primary" disabled={busy} onclick={save}>{t('common.save')}</Button>
  {/snippet}
</Dialog>
