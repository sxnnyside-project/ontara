<script lang="ts">
import { t } from '@/shared/i18n';

export type FindingSeverity = 'error' | 'warning' | 'info';

interface Props {
  severity: FindingSeverity;
  ruleCode: string;
  title: string;
  description: string;
  affected: string[];
  onresolve?: (() => void) | undefined;
}

const { severity, ruleCode, title, description, affected, onresolve }: Props = $props();

const config = $derived(
  {
    error: {
      icon: 'i-mingcute:close-circle-fill',
      badgeClass:
        'bg-[var(--color-finding-error-bg)] text-[var(--color-finding-error)] border-[var(--color-finding-error)]/30',
      borderClass: 'border-l-[var(--color-finding-error)]',
    },
    warning: {
      icon: 'i-mingcute:alert-fill',
      badgeClass:
        'bg-[var(--color-finding-warning-bg)] text-[var(--color-finding-warning)] border-[var(--color-finding-warning)]/30',
      borderClass: 'border-l-[var(--color-finding-warning)]',
    },
    info: {
      icon: 'i-mingcute:information-line',
      badgeClass:
        'bg-[var(--color-finding-info-bg)] text-[var(--color-finding-info)] border-[var(--color-finding-info)]/30',
      borderClass: 'border-l-[var(--color-finding-info)]',
    },
  }[severity],
);
</script>

<div
  class="p-3 rounded-r border border-l-3 bg-[var(--color-surface-panel)] border-[var(--color-border-subtle)] {config.borderClass} flex flex-col gap-2 transition-all hover:border-[var(--color-border-strong)]"
>
  <div class="flex items-center justify-between gap-2">
    <div class="flex items-center gap-2">
      <span class="{config.icon} w-4 h-4 shrink-0"></span>
      <span class="font-mono text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded border {config.badgeClass}">
        {ruleCode}
      </span>
      <h4 class="text-xs font-semibold text-[var(--color-text-primary)] m-0">{title}</h4>
    </div>
    {#if onresolve}
      <button
        type="button"
        onclick={onresolve}
        class="text-[11px] font-mono text-[var(--color-crown-red)] hover:underline flex items-center gap-1 cursor-pointer bg-transparent border-0 p-0"
      >
        <span>{t('common.inspect')}</span>
        <span class="i-mingcute:arrow-right-line w-3 h-3"></span>
      </button>
    {/if}
  </div>

  <p class="text-xs text-[var(--color-text-secondary)] m-0 leading-relaxed font-sans">
    {description}
  </p>

  {#if affected.length > 0}
    <div class="flex items-center gap-1.5 flex-wrap pt-1 border-t border-[var(--color-border-subtle)]/50">
      <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{t('consistency.affects')}:</span>
      {#each affected as item}
        <span class="inline-flex items-center gap-1 px-1.5 py-0.2 rounded bg-[var(--color-surface-elevated)] border border-[var(--color-border-subtle)] text-[10px] font-mono text-[var(--color-text-secondary)]">
          <span class="i-mingcute:link-2-line w-2.5 h-2.5 text-[var(--color-crown-red)]"></span>
          {item}
        </span>
      {/each}
    </div>
  {/if}
</div>
