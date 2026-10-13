<script lang="ts">
import { t } from '@/shared/i18n';

export type AppearanceKind = 'physical' | 'dialogue' | 'pov' | 'mention' | 'flashback' | 'absence';

interface Props {
  kind?: AppearanceKind | undefined;
  tooltip?: string | undefined;
  onclick?: (() => void) | undefined;
}

const { kind, tooltip, onclick }: Props = $props();

const config = $derived(
  kind
    ? {
        physical: {
          icon: 'i-mingcute:walk-line',
          label: t('appearance.physical'),
          textColor: 'text-[var(--color-meridian-blue)]',
          bgColor: 'bg-[var(--color-meridian-blue-bg)]',
          border: 'border-[var(--color-meridian-blue)]/20',
        },
        dialogue: {
          icon: 'i-mingcute:chat-3-line',
          label: t('appearance.dialogue'),
          textColor: 'text-[var(--color-canon-established)]',
          bgColor: 'bg-[var(--color-canon-established-bg)]',
          border: 'border-[var(--color-canon-established)]/20',
        },
        pov: {
          icon: 'i-mingcute:eye-line',
          label: t('appearance.pov'),
          textColor: 'text-[var(--color-crown-red)]',
          bgColor: 'bg-[var(--color-crown-red-subtle)]',
          border: 'border-[var(--color-crown-red-border)]',
        },
        mention: {
          icon: 'i-mingcute:quill-pen-line',
          label: t('appearance.mention'),
          textColor: 'text-[var(--color-canon-provisional)]',
          bgColor: 'bg-[var(--color-canon-provisional-bg)]',
          border: 'border-[var(--color-canon-provisional)]/20',
        },
        flashback: {
          icon: 'i-mingcute:history-line',
          label: t('appearance.flashback'),
          textColor: 'text-[var(--color-canon-speculative)]',
          bgColor: 'bg-[var(--color-canon-speculative-bg)]',
          border: 'border-[var(--color-canon-speculative)]/20',
        },
        absence: {
          icon: 'i-mingcute:close-line',
          label: t('appearance.absence'),
          textColor: 'text-[var(--color-text-muted)]',
          bgColor: 'bg-transparent',
          border: 'border-[var(--color-border-subtle)]',
        },
      }[kind]
    : null,
);
</script>

<button
  type="button"
  {onclick}
  title={tooltip ?? (config ? config.label : t('appearance.none'))}
  class="w-8 h-8 rounded flex items-center justify-center transition-colors border
  {config 
    ? `${config.bgColor} ${config.textColor} ${config.border} hover:brightness-125` 
    : 'border-transparent text-[var(--color-text-muted)]/20 hover:border-[var(--color-border-subtle)] hover:text-[var(--color-text-muted)]/60'}"
>
  {#if config}
    <span class="{config.icon} w-4 h-4"></span>
  {:else}
    <span class="w-1.5 h-1.5 rounded-full bg-[var(--color-border-strong)]/30"></span>
  {/if}
</button>
