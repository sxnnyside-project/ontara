<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { PropertiesEditor, removePropertyValue, savePropertyValue } from '@/shared/properties';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import type { EntityRef, PropertyValue, PropertyValueInput, TypeKey } from '@/shared/model';
import { type EntityDetail, listEntities } from '../api/entity-api';

interface Props {
  detail: EntityDetail;
}

const { detail }: Props = $props();

async function save(input: PropertyValueInput) {
  try {
    await savePropertyValue(input);
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
    throw failure;
  }
}

async function remove(value: PropertyValue) {
  try {
    await removePropertyValue(value.id);
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

async function candidates(entityType: TypeKey | null): Promise<EntityRef[]> {
  return listEntities({ entity_type: entityType });
}
</script>

<PropertiesEditor
  owner={{ kind: 'entity', id: detail.entity.id }}
  definitions={detail.definitions}
  values={detail.values}
  groups={catalog.data.property_groups}
  loadCandidates={candidates}
  onsave={save}
  onremove={remove}
/>
