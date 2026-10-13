<script lang="ts">
import { t } from '@/shared/i18n';
import { TabBar } from '@/shared/ui';
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
</script>

<div class="flex flex-col gap-4 p-6 max-w-5xl">
  <div>
    <h2 class="m-0 text-lg font-serif font-medium">{t('config.title')}</h2>
    <p class="m-0 mt-0.5 text-xs text-[var(--color-text-secondary)]">{t('config.subtitle')}</p>
  </div>
  <TabBar {tabs} active={section} onselect={(key) => (section = key as Section)} />
  {#if section === 'entity_types'}
    <EntityTypesPanel />
  {:else if section === 'relationship_types'}
    <RelationshipTypesPanel />
  {:else if section === 'properties'}
    <PropertiesPanel />
  {:else if section === 'canon_states'}
    <CanonStatesPanel />
  {:else if section === 'levels'}
    <EditorialLevelsPanel />
  {:else}
    <VocabulariesPanel />
  {/if}
</div>
