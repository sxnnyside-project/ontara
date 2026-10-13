<script lang="ts">
import { t } from '@/shared/i18n';
import CanonBadge, { type CanonState } from './canon-badge.svelte';

export type EntityType = 'character' | 'faction' | 'location' | 'event' | 'relic';

interface RelationPreview {
  typeLabel: string;
  targetName: string;
}

interface Props {
  name: string;
  type: EntityType;
  canonState: CanonState;
  subtitle?: string | undefined;
  selected?: boolean | undefined;
  relations?: RelationPreview[] | undefined;
  onclick?: (() => void) | undefined;
}

const {
  name,
  type,
  canonState,
  subtitle,
  selected = false,
  relations = [],
  onclick,
}: Props = $props();

const typeConfig = $derived(
  {
    character: {
      label: t('entityTypes.character'),
      icon: 'i-mingcute:user-3-line',
      color: 'text-[var(--color-meridian-blue)]',
    },
    faction: {
      label: t('entityTypes.faction'),
      icon: 'i-mingcute:group-line',
      color: 'text-[var(--color-canon-provisional)]',
    },
    location: {
      label: t('entityTypes.location'),
      icon: 'i-mingcute:map-pin-line',
      color: 'text-[var(--color-canon-established)]',
    },
    event: {
      label: t('entityTypes.event'),
      icon: 'i-mingcute:calendar-line',
      color: 'text-[var(--color-crown-red)]',
    },
    relic: {
      label: t('entityTypes.relic'),
      icon: 'i-mingcute:sparkles-line',
      color: 'text-[var(--color-canon-speculative)]',
    },
  }[type],
);
</script>

<div
  role="button"
  tabindex="0"
  onkeydown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') onclick?.();
  }}
  {onclick}
  class="w-64 rounded-lg border bg-[var(--color-surface-panel)] text-left transition-all cursor-pointer select-none p-3.5 flex flex-col gap-2.5 shadow-sm
  {selected
    ? 'border-[var(--color-crown-red)] ring-1 ring-[var(--color-crown-red)] shadow-md bg-[var(--color-surface-elevated)]'
    : 'border-[var(--color-border-subtle)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-elevated)]'}"
>
  <div class="flex items-start justify-between gap-2">
    <div class="flex items-center gap-1.5 min-w-0">
      <span class="{typeConfig.icon} w-4 h-4 {typeConfig.color} shrink-0"></span>
      <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] truncate">
        {typeConfig.label}
      </span>
    </div>
    <CanonBadge state={canonState} size="sm" />
  </div>

  <div>
    <h3 class="text-sm font-serif font-medium text-[var(--color-text-primary)] m-0 truncate">
      {name}
    </h3>
    {#if subtitle}
      <p class="text-xs text-[var(--color-text-secondary)] m-0 truncate mt-0.5">
        {subtitle}
      </p>
    {/if}
  </div>

  {#if relations.length > 0}
    <div class="pt-2 border-t border-[var(--color-border-subtle)] flex flex-col gap-1">
      {#each relations.slice(0, 2) as rel}
        <div class="flex items-center gap-1 text-[11px] font-mono text-[var(--color-text-muted)] truncate">
          <span class="i-mingcute:arrow-right-line w-3 h-3 text-[var(--color-crown-red)] shrink-0"></span>
          <span class="text-[var(--color-text-secondary)]">{rel.typeLabel}:</span>
          <span class="text-[var(--color-text-primary)] font-medium truncate">{rel.targetName}</span>
        </div>
      {/each}
      {#if relations.length > 2}
        <span class="text-[10px] font-mono text-[var(--color-text-muted)]">
          +{relations.length - 2} …
        </span>
      {/if}
    </div>
  {/if}
</div>
