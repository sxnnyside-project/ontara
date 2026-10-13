<script lang="ts">
import { t } from '@/shared/i18n';

export type CanonState =
  | 'established'
  | 'provisional'
  | 'disputed'
  | 'deprecated'
  | 'rejected'
  | 'disestablished'
  | 'speculative';

interface Props {
  /** A canon state key. A project can define its own, which get a neutral look. */
  state: CanonState | (string & {});
  /** The project's own label for the state; a built-in state falls back to the translation. */
  label?: string | undefined;
  size?: 'sm' | 'md' | undefined;
  interactive?: boolean | undefined;
  onclick?: (() => void) | undefined;
}

const { state, label, size = 'sm', interactive = false, onclick }: Props = $props();

interface Look {
  label: string;
  icon: string;
  textColor: string;
  bgColor: string;
  borderColor: string;
}

const known = $derived<Record<CanonState, Look>>({
  established: {
    label: t('canon.established'),
    icon: 'i-mingcute:check-circle-line',
    textColor: 'text-[var(--color-canon-established)]',
    bgColor: 'bg-[var(--color-canon-established-bg)]',
    borderColor: 'border-[var(--color-canon-established)]/30',
  },
  provisional: {
    label: t('canon.provisional'),
    icon: 'i-mingcute:time-line',
    textColor: 'text-[var(--color-canon-provisional)]',
    bgColor: 'bg-[var(--color-canon-provisional-bg)]',
    borderColor: 'border-[var(--color-canon-provisional)]/30',
  },
  disputed: {
    label: t('canon.disputed'),
    icon: 'i-mingcute:alert-line',
    textColor: 'text-[var(--color-canon-disputed)]',
    bgColor: 'bg-[var(--color-canon-disputed-bg)]',
    borderColor: 'border-[var(--color-canon-disputed)]/30',
  },
  deprecated: {
    label: t('canon.deprecated'),
    icon: 'i-mingcute:archive-line',
    textColor: 'text-[var(--color-canon-deprecated)]',
    bgColor: 'bg-[var(--color-canon-deprecated-bg)]',
    borderColor: 'border-[var(--color-canon-deprecated)]/30',
  },
  rejected: {
    label: t('canon.rejected'),
    icon: 'i-mingcute:close-circle-line',
    textColor: 'text-[var(--color-canon-rejected)]',
    bgColor: 'bg-[var(--color-canon-rejected-bg)]',
    borderColor: 'border-[var(--color-canon-rejected)]/30',
  },
  disestablished: {
    label: t('states.disestablished'),
    icon: 'i-mingcute:forbid-circle-line',
    textColor: 'text-[var(--color-canon-deprecated)]',
    bgColor: 'bg-[var(--color-canon-deprecated-bg)]',
    borderColor: 'border-[var(--color-canon-deprecated)]/30',
  },
  speculative: {
    label: t('canon.speculative'),
    icon: 'i-mingcute:sparkles-line',
    textColor: 'text-[var(--color-canon-speculative)]',
    bgColor: 'bg-[var(--color-canon-speculative-bg)]',
    borderColor: 'border-[var(--color-canon-speculative)]/30',
  },
});

const neutral: Omit<Look, 'label'> = {
  icon: 'i-mingcute:flag-2-line',
  textColor: 'text-[var(--color-text-secondary)]',
  bgColor: 'bg-[var(--color-surface-elevated)]',
  borderColor: 'border-[var(--color-border-strong)]',
};

const config = $derived.by<Look>(() => {
  const look = known[state as CanonState];
  if (look) return { ...look, label: label ?? look.label };
  return { ...neutral, label: label ?? state };
});
</script>

{#if interactive}
  <button
    type="button"
    {onclick}
    class="inline-flex items-center gap-1.5 rounded border font-mono tracking-tight transition-all cursor-pointer select-none
    {size === 'sm' ? 'px-1.5 py-0.5 text-[11px]' : 'px-2 py-1 text-xs'}
    {config.bgColor} {config.textColor} {config.borderColor} hover:brightness-110 active:scale-95"
  >
    <span class="{config.icon} {size === 'sm' ? 'w-3 h-3' : 'w-3.5 h-3.5'} shrink-0"></span>
    <span class="font-medium">{config.label}</span>
  </button>
{:else}
  <span
    class="inline-flex items-center gap-1.5 rounded border font-mono tracking-tight select-none
    {size === 'sm' ? 'px-1.5 py-0.5 text-[11px]' : 'px-2 py-1 text-xs'}
    {config.bgColor} {config.textColor} {config.borderColor}"
  >
    <span class="{config.icon} {size === 'sm' ? 'w-3 h-3' : 'w-3.5 h-3.5'} shrink-0"></span>
    <span class="font-medium">{config.label}</span>
  </span>
{/if}
