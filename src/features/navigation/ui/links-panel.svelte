<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t, tDynamic } from '@/shared/i18n';

import { revision } from '@/shared/revision';
import { entityIcon } from '@/shared/ui';
import { type Connection, getConnections } from '../api/navigation-api';
import { groupConnections } from '../state/connection-groups';

interface Props {
  entity: string;
  onopen: (id: string) => void;
}

const { entity, onopen }: Props = $props();

let connections = $state<Connection[]>([]);
let failure = $state<string | null>(null);

$effect(() => {
  void revision.value;
  const id = entity;
  let cancelled = false;
  getConnections(id)
    .then((found) => {
      if (cancelled) return;
      connections = found;
      failure = null;
    })
    .catch((error) => {
      if (!cancelled) failure = toCommandError(error).message;
    });
  return () => {
    cancelled = true;
  };
});

const groups = $derived(groupConnections(connections));

/** How the link reads from the entity being looked at. */
function describe(connection: Connection): string {
  switch (connection.via) {
    case 'relationship':
      return catalog.relationshipLabel(connection.relationship_type, connection.outgoing);
    case 'appears_in':
      return t('links.appearsIn');
    case 'hosts':
      return t('links.hosts');
    case 'participates':
      return t('links.participates', { role: connection.role });
    case 'has_participant':
      return t('links.hasParticipant', { role: connection.role });
    case 'takes_place_at':
      return t('links.takesPlaceAt');
    case 'hosts_event':
      return t('links.hostsEvent');
    case 'has_asset':
      return t('links.hasAsset');
    case 'attached_to':
      return t('links.attachedTo');
    case 'property_of':
      return t('links.propertyOf');
    case 'inside_section':
      return t('links.insideSection');
    case 'contains_section':
      return t('links.containsSection');
  }
}
</script>

<div class="flex flex-col gap-4">
  {#if failure}
    <p class="m-0 text-xs text-[var(--color-finding-error)]" role="alert">{failure}</p>
  {/if}
  {#each groups as { group, items } (group)}
    <section class="flex flex-col gap-1.5">
      <h3 class="m-0 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
        {tDynamic(`links.group_${group}`, group)} <span class="tabular-nums">({items.length})</span>
      </h3>
      {#each items as item}
        <button
          type="button"
          onclick={() => onopen(item.other.id)}
          class="w-full text-left rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] px-2.5 py-1.5 cursor-pointer hover:border-[var(--color-border-strong)] transition-colors"
        >
          <span class="block text-[10px] font-mono text-[var(--color-text-muted)] truncate">{describe(item)}</span>
          <span class="flex items-center gap-1.5 text-xs text-[var(--color-text-primary)]">
            <span class="{entityIcon(catalog.chain(item.other.entity_type))} w-3.5 h-3.5 shrink-0 text-[var(--color-text-muted)]"></span>
            <span class="truncate font-medium">{item.other.name}</span>
          </span>
        </button>
      {/each}
    </section>
  {:else}
    {#if !failure}
      <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('links.none')}</p>
    {/if}
  {/each}
</div>
