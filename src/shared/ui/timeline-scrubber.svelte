<script lang="ts">
import { t } from '@/shared/i18n';

export interface TimelineEvent {
  id: string;
  universeDay: number;
  title: string;
  era: string;
  canonState: 'established' | 'provisional' | 'disputed';
}

interface Props {
  events: TimelineEvent[];
  selectedId?: string | undefined;
  onselect?: ((event: TimelineEvent) => void) | undefined;
}

const { events, selectedId, onselect }: Props = $props();
</script>

<div class="w-full flex flex-col gap-2 p-3 bg-[var(--color-surface-panel)] border border-[var(--color-border-subtle)] rounded-lg">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="i-mingcute:history-line w-4 h-4 text-[var(--color-meridian-timeline)]"></span>
      <span class="text-xs font-mono font-medium text-[var(--color-text-primary)]">
        {t('timeline.title')}
      </span>
    </div>
    <span class="text-[11px] font-mono text-[var(--color-text-muted)]">
      {t('timeline.scale')}
    </span>
  </div>

  <!-- Timeline track -->
  <div class="relative py-4 flex items-center gap-4 overflow-x-auto">
    <!-- Connecting axis line -->
    <div class="absolute left-0 right-0 h-0.5 bg-[var(--color-border-strong)] top-1/2 -translate-y-1/2 z-0"></div>

    {#each events as ev}
      {@const isSelected = selectedId === ev.id}
      <button
        type="button"
        onclick={() => onselect?.(ev)}
        class="relative z-10 flex flex-col items-center gap-1 shrink-0 p-2 rounded transition-all cursor-pointer bg-transparent border-0
        {isSelected ? 'scale-105' : 'hover:scale-102 opacity-85 hover:opacity-100'}"
      >
        <!-- Marker dot -->
        <span
          class="w-4 h-4 rounded-full border-2 flex items-center justify-center transition-all
          {isSelected
            ? 'bg-[var(--color-crown-red)] border-white shadow-sm ring-2 ring-[var(--color-crown-red)]/40'
            : 'bg-[var(--color-surface-elevated)] border-[var(--color-meridian-blue)]'}"
        >
          <span class="w-1.5 h-1.5 rounded-full {isSelected ? 'bg-white' : 'bg-[var(--color-meridian-blue)]'}"></span>
        </span>

        <!-- Day tag -->
        <span
          class="font-mono text-[10px] tabular-nums font-semibold px-1 py-0.2 rounded border
          {isSelected
            ? 'bg-[var(--color-crown-red-subtle)] text-[var(--color-crown-red)] border-[var(--color-crown-red-border)]'
            : 'bg-[var(--color-surface-elevated)] text-[var(--color-text-secondary)] border-[var(--color-border-subtle)]'}"
        >
          {t('timeline.dayPrefix')}{ev.universeDay}
        </span>

        <!-- Event label -->
        <span class="text-[11px] font-sans font-medium text-[var(--color-text-primary)] max-w-28 truncate text-center">
          {ev.title}
        </span>
      </button>
    {/each}
  </div>
</div>
