<script lang="ts">
import { t } from '@/shared/i18n';
import type {
  EntityRef,
  PropertyDefinition,
  PropertyGroup,
  PropertyValue,
  PropertyValueInput,
  PropertyValueOwner,
  TypeKey,
} from '@/shared/model';
import { EmptyState } from '@/shared/ui';
import PropertyValueRow from './property-value-row.svelte';

interface Props {
  owner: PropertyValueOwner;
  definitions: PropertyDefinition[];
  values: PropertyValue[];
  groups: PropertyGroup[];
  /** The entities a reference property may point at, for the type it names (or any). */
  loadCandidates: (entityType: TypeKey | null) => Promise<EntityRef[]>;
  onsave: (input: PropertyValueInput) => void | Promise<void>;
  onremove: (value: PropertyValue) => void | Promise<void>;
}

const { owner, definitions, values, groups, loadCandidates, onsave, onremove }: Props = $props();

interface Section {
  key: string;
  label: string;
  definitions: PropertyDefinition[];
}

const sections = $derived.by<Section[]>(() => {
  const labelOf = new Map(groups.map((group) => [group.key, group]));
  const buckets = new Map<string, PropertyDefinition[]>();
  for (const definition of definitions) {
    const key = definition.group ?? '';
    buckets.set(key, [...(buckets.get(key) ?? []), definition]);
  }
  return [...buckets.entries()]
    .map(([key, items]) => ({
      key,
      label: key === '' ? t('properties.ungrouped') : (labelOf.get(key)?.label ?? key),
      order: key === '' ? Number.MAX_SAFE_INTEGER : (labelOf.get(key)?.sort_order ?? 0),
      definitions: [...items].sort((a, b) => a.sort_order - b.sort_order),
    }))
    .sort((a, b) => a.order - b.order);
});

/** Candidates are loaded once per entity type and reused by every row. */
let candidateCache = $state<Record<string, EntityRef[]>>({});

function candidatesFor(definition: PropertyDefinition): EntityRef[] {
  const type = definition.data_type;
  if (type.type !== 'entity_reference') return [];
  const key = type.entity_type ?? '';
  if (!(key in candidateCache)) {
    candidateCache[key] = [];
    loadCandidates(type.entity_type).then((found) => {
      candidateCache[key] = found;
    });
  }
  return candidateCache[key] ?? [];
}

function valuesOf(definition: PropertyDefinition): PropertyValue[] {
  return values
    .filter((value) => value.definition === definition.id)
    .sort((a, b) => a.position - b.position);
}
</script>

{#if definitions.length === 0}
  <EmptyState
    icon="i-mingcute:tag-line"
    title={t('properties.noneTitle')}
    description={t('properties.noneHint')}
  />
{:else}
  <div class="flex flex-col gap-6">
    {#each sections as section (section.key)}
      <section class="flex flex-col gap-3">
        <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] pb-1 border-b border-[var(--color-border-subtle)]">
          {section.label}
        </h3>
        {#each section.definitions as definition (definition.id)}
          {@const current = valuesOf(definition)}
          <div class="flex flex-col gap-1.5">
            <div class="flex items-baseline gap-2">
              <span class="text-xs font-medium text-[var(--color-text-primary)]">{definition.label}</span>
              {#if definition.description}
                <span class="text-[11px] text-[var(--color-text-muted)] truncate">{definition.description}</span>
              {/if}
            </div>
            {#each current as value (value.id)}
              <PropertyValueRow
                {definition}
                {owner}
                {value}
                position={value.position}
                candidates={candidatesFor(definition)}
                {onsave}
                {onremove}
              />
            {/each}
            {#if current.length === 0 || definition.multiple}
              <PropertyValueRow
                {definition}
                {owner}
                value={null}
                position={current.length}
                candidates={candidatesFor(definition)}
                {onsave}
              />
            {/if}
          </div>
        {/each}
      </section>
    {/each}
  </div>
{/if}
