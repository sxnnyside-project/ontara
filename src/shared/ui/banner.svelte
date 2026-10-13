<script lang="ts">
interface Props {
  kind?: 'error' | 'warning' | 'info' | undefined;
  children: import('svelte').Snippet;
}

const { kind = 'info', children }: Props = $props();

const look = {
  error: {
    icon: 'i-mingcute:close-circle-line',
    box: 'bg-[var(--color-finding-error-bg)] border-[var(--color-finding-error)]/30 text-[var(--color-finding-error)]',
  },
  warning: {
    icon: 'i-mingcute:alert-line',
    box: 'bg-[var(--color-finding-warning-bg)] border-[var(--color-finding-warning)]/30 text-[var(--color-finding-warning)]',
  },
  info: {
    icon: 'i-mingcute:information-line',
    box: 'bg-[var(--color-finding-info-bg)] border-[var(--color-finding-info)]/30 text-[var(--color-finding-info)]',
  },
} as const;
</script>

<div
  role={kind === 'error' ? 'alert' : 'status'}
  class="flex items-start gap-2 rounded border px-3 py-2 text-xs {look[kind].box}"
>
  <span class="{look[kind].icon} w-4 h-4 shrink-0 mt-0.5"></span>
  <div class="min-w-0 text-[var(--color-text-primary)]">{@render children()}</div>
</div>
