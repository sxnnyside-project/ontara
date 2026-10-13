<script lang="ts">
import { t } from '@/shared/i18n';
import { Button, InlineSelect } from '@/shared/ui';
import {
  dashArray,
  type EdgeStyle,
  type LegendEntry,
  MARKS,
  type Mark,
  type Pattern,
  PATTERNS,
} from '../state/edge-style';
import EdgeMark from './edge-mark.svelte';

interface Props {
  entries: LegendEntry[];
  styles: Map<string, EdgeStyle>;
  /** The words for a kind of link. */
  label: (key: string) => string;
  onchange: (key: string, change: Partial<EdgeStyle>) => void;
  onreset: () => void;
  /** Whether any style was changed from its default. */
  customized: boolean;
}

const { entries, styles, label, onchange, onreset, customized }: Props = $props();

let open = $state(true);
let editing = $state(false);
</script>

<div data-overlay class="absolute right-3 bottom-3 z-10 w-64 max-h-[60%] flex flex-col rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-surface-panel)]/95 backdrop-blur-md shadow-[var(--shadow-md)]">
  <div class="flex items-center gap-1 px-3 h-9 shrink-0">
    <button
      type="button"
      aria-expanded={open}
      onclick={() => (open = !open)}
      class="flex-1 flex items-center gap-1.5 border-0 bg-transparent p-0 text-left cursor-pointer text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
    >
      <span class="{open ? 'i-mingcute:down-line' : 'i-mingcute:right-line'} w-3.5 h-3.5"></span>
      {t('graphView.legend')}
      <span class="px-1 rounded bg-[var(--color-surface-elevated)] text-[10px] tabular-nums">{entries.length}</span>
    </button>
    {#if open}
      <Button size="sm" variant="ghost" onclick={() => (editing = !editing)}>
        {editing ? t('graphView.legendDone') : t('graphView.legendCustomize')}
      </Button>
    {/if}
  </div>

  {#if open}
    <ul class="m-0 p-0 px-3 pb-2 list-none overflow-y-auto flex flex-col gap-1.5">
      {#each entries as entry (entry.key)}
        {@const style = styles.get(entry.key)}
        {#if style}
          <li class="flex flex-col gap-1">
            <div class="flex items-center gap-2 text-xs">
              <svg width="34" height="14" viewBox="0 0 34 14" aria-hidden="true" class="shrink-0">
                <line x1="1" y1="7" x2="33" y2="7" stroke={style.color} stroke-width="2" stroke-dasharray={dashArray(style.pattern)} />
                <EdgeMark mark={style.mark} x={17} y={7} color={style.color} />
              </svg>
              <span class="flex-1 min-w-0 truncate">{label(entry.key)}</span>
              <span class="text-[10px] font-mono tabular-nums text-[var(--color-text-muted)]">{entry.count}</span>
            </div>
            {#if editing}
              <div class="flex items-center gap-1.5 pl-1">
                <input
                  type="color"
                  value={style.color}
                  aria-label={t('graphView.color')}
                  oninput={(event) => onchange(entry.key, { color: event.currentTarget.value })}
                  class="w-7 h-7 p-0 border border-[var(--color-border-subtle)] rounded bg-transparent cursor-pointer"
                />
                <InlineSelect
                  label=""
                  options={PATTERNS.map((value) => ({ value, label: t(`graphView.pattern_${value}` as 'graphView.pattern_solid') }))}
                  bind:value={() => style.pattern, (value) => onchange(entry.key, { pattern: value as Pattern })}
                />
                <InlineSelect
                  label=""
                  options={MARKS.map((value) => ({ value, label: t(`graphView.mark_${value}` as 'graphView.mark_circle') }))}
                  bind:value={() => style.mark, (value) => onchange(entry.key, { mark: value as Mark })}
                />
              </div>
            {/if}
          </li>
        {/if}
      {/each}
    </ul>
    {#if editing && customized}
      <div class="px-3 pb-2"><Button size="sm" variant="ghost" onclick={onreset}>{t('graphView.legendReset')}</Button></div>
    {/if}
  {/if}
</div>
