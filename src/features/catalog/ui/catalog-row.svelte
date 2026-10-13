<script lang="ts">
import type { Snippet } from 'svelte';
import { t } from '@/shared/i18n';
import { Button } from '@/shared/ui';

interface Props {
  title: string;
  /** Machine key, shown in monospace. */
  keyText?: string | undefined;
  description?: string | undefined;
  icon?: string | undefined;
  indent?: number | undefined;
  builtin?: boolean | undefined;
  badges?: Snippet | undefined;
  onedit?: (() => void) | undefined;
  onremove?: (() => void) | undefined;
}

const {
  title,
  keyText,
  description,
  icon,
  indent = 0,
  builtin = false,
  badges,
  onedit,
  onremove,
}: Props = $props();
</script>

<div
  class="flex items-center gap-3 px-3 py-2 border-b border-[var(--color-border-subtle)] last:border-b-0 hover:bg-[var(--color-surface-hover)]"
  style="padding-left: {0.75 + indent * 1.25}rem"
>
  {#if icon}<span class="{icon} w-4 h-4 shrink-0 text-[var(--color-text-muted)]"></span>{/if}
  <div class="flex-1 min-w-0">
    <div class="flex items-center gap-2 flex-wrap">
      <span class="text-sm text-[var(--color-text-primary)] truncate">{title}</span>
      {#if keyText}
        <span class="text-[10px] font-mono text-[var(--color-text-muted)]">{keyText}</span>
      {/if}
      {#if builtin}
        <span class="text-[10px] font-mono px-1 rounded bg-[var(--color-surface-elevated)] text-[var(--color-text-muted)]">
          {t('config.builtin')}
        </span>
      {/if}
      {@render badges?.()}
    </div>
    {#if description}
      <p class="m-0 text-[11px] text-[var(--color-text-muted)] truncate">{description}</p>
    {/if}
  </div>
  {#if onedit}
    <Button size="sm" variant="ghost" icon="i-mingcute:edit-line" title={t('common.edit')} onclick={onedit} />
  {/if}
  {#if onremove && !builtin}
    <Button size="sm" variant="ghost" icon="i-mingcute:delete-2-line" title={t('common.remove')} onclick={onremove} />
  {/if}
</div>
