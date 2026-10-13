<script lang="ts">
import type { Snippet } from 'svelte';

interface Props {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | undefined;
  size?: 'sm' | 'md' | undefined;
  /** A MingCute icon class, such as `i-mingcute:add-line`. */
  icon?: string | undefined;
  type?: 'button' | 'submit' | undefined;
  disabled?: boolean | undefined;
  title?: string | undefined;
  onclick?: ((event: MouseEvent) => void) | undefined;
  children?: Snippet | undefined;
}

const {
  variant = 'secondary',
  size = 'md',
  icon,
  type = 'button',
  disabled = false,
  title,
  onclick,
  children,
}: Props = $props();

const palette = {
  primary:
    'bg-[var(--color-crown-red)] text-white border-transparent hover:bg-[var(--color-crown-red-hover)] active:bg-[var(--color-crown-red-active)]',
  secondary:
    'bg-[var(--color-surface-elevated)] text-[var(--color-text-primary)] border-[var(--color-border-subtle)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-hover)]',
  ghost:
    'bg-transparent text-[var(--color-text-secondary)] border-transparent hover:text-[var(--color-text-primary)] hover:bg-[var(--color-surface-hover)]',
  danger:
    'bg-transparent text-[var(--color-finding-error)] border-[var(--color-finding-error)]/40 hover:bg-[var(--color-finding-error-bg)]',
} as const;
</script>

<button
  {type}
  {title}
  {disabled}
  {onclick}
  class="inline-flex items-center justify-center gap-1.5 rounded border font-mono font-medium transition-colors cursor-pointer select-none disabled:opacity-40 disabled:cursor-not-allowed
  {size === 'sm' ? 'h-7 px-2 text-[11px]' : 'h-8 px-3 text-xs'} {palette[variant]}"
>
  {#if icon}<span class="{icon} w-3.5 h-3.5 shrink-0"></span>{/if}
  {@render children?.()}
</button>
