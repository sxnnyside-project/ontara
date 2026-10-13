<script lang="ts">
import { t } from '@/shared/i18n';
import { Button, Segmented, ViewFrame } from '@/shared/ui';
import CanonStatesPanel from './canon-states-panel.svelte';
import EditorialLevelsPanel from './editorial-levels-panel.svelte';
import EntityTypesPanel from './entity-types-panel.svelte';
import PropertiesPanel from './properties-panel.svelte';
import RelationshipTypesPanel from './relationship-types-panel.svelte';
import VocabulariesPanel from './vocabularies-panel.svelte';

type Section =
  | 'entity_types'
  | 'relationship_types'
  | 'properties'
  | 'canon_states'
  | 'levels'
  | 'vocabularies';

let section = $state<Section>('entity_types');
let createSignal = $state(0);

const tabs = $derived([
  { key: 'entity_types', label: t('config.tabEntityTypes'), icon: 'i-mingcute:box-line' },
  {
    key: 'relationship_types',
    label: t('config.tabRelationshipTypes'),
    icon: 'i-mingcute:git-branch-line',
  },
  { key: 'properties', label: t('config.tabProperties'), icon: 'i-mingcute:tag-line' },
  { key: 'canon_states', label: t('config.tabCanonStates'), icon: 'i-mingcute:check-circle-line' },
  { key: 'levels', label: t('config.tabLevels'), icon: 'i-mingcute:book-2-line' },
  { key: 'vocabularies', label: t('config.tabVocabularies'), icon: 'i-mingcute:list-check-line' },
]);

const copy = $derived(
  {
    entity_types: { hint: t('config.entityTypesHint'), create: t('config.newType') },
    relationship_types: {
      hint: t('config.relationshipTypesHint'),
      create: t('config.newRelationshipType'),
    },
    properties: { hint: t('config.propertiesHint'), create: t('config.newProperty') },
    canon_states: { hint: t('config.canonStatesHint'), create: t('config.newCanonState') },
    levels: { hint: t('config.levelsHint'), create: t('config.newLevel') },
    vocabularies: { hint: t('config.vocabulariesHint'), create: t('config.newTerm') },
  }[section],
);
</script>

<ViewFrame
  icon="i-mingcute:settings-3-line"
  title={t('config.title')}
  description={copy.hint}
>
  {#snippet actions()}
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => (createSignal += 1)}>
      {copy.create}
    </Button>
  {/snippet}
  {#snippet toolbar()}
    <Segmented
      bind:value={section as string}
      label={t('config.title')}
      options={tabs.map((tab) => ({ value: tab.key, label: tab.label, icon: tab.icon }))}
    />
  {/snippet}

  <div>
    {#if section === 'entity_types'}
      <EntityTypesPanel {createSignal} />
    {:else if section === 'relationship_types'}
      <RelationshipTypesPanel {createSignal} />
    {:else if section === 'properties'}
      <PropertiesPanel {createSignal} />
    {:else if section === 'canon_states'}
      <CanonStatesPanel {createSignal} />
    {:else if section === 'levels'}
      <EditorialLevelsPanel {createSignal} />
    {:else}
      <VocabulariesPanel {createSignal} />
    {/if}
  </div>
</ViewFrame>
