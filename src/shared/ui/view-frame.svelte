<script lang="ts">
import { t } from '@/shared/i18n';
import type { Snippet } from 'svelte';

interface Props {
  /** A MingCute icon class. */
  icon: string;
  title: string;
  /** One line that says what the view is for. */
  description?: string | undefined;
  /** The one primary action of the view, always at the right of the header. */
  actions?: Snippet | undefined;
  /** Filters and options, in a row under the header. What belongs to the focus goes at its end. */
  toolbar?: Snippet | undefined;
  /** A panel at the right, for the detail of what is selected. It is not shown without one. */
  aside?: Snippet | undefined;
  /** Whether the aside is open. Closing it is the author's choice and does not clear a selection. */
  asideOpen?: boolean | undefined;
  asideTitle?: string | undefined;
  /** Called when the author closes the aside. */
  onasideclose?: (() => void) | undefined;
  /** Width of the aside, in pixels. */
  asideWidth?: number | undefined;
  /** The content takes the whole area, without padding, for views that draw their own surface. */
  flush?: boolean | undefined;
  children: Snippet;
}

const {
  icon,
  title,
  description,
  actions,
  toolbar,
  aside,
  asideOpen = true,
  asideTitle,
  onasideclose,
  asideWidth = 340,
  flush = false,
  children,
}: Props = $props();
</script>

<div class="h-full min-h-0 w-full flex flex-col">
  <header class="shrink-0 h-14 px-5 flex items-center gap-3 border-b border-[var(--color-border-subtle)]">
    <span class="{icon} w-5 h-5 shrink-0 text-[var(--color-crown-red-text)]"></span>
    <div class="min-w-0 flex-1">
      <h2 class="m-0 text-base font-serif font-medium leading-tight truncate">{title}</h2>
      {#if description}
        <p class="m-0 text-[11px] text-[var(--color-text-secondary)] truncate">{description}</p>
      {/if}
    </div>
    {#if actions}
      <div class="flex items-center gap-2 shrink-0">{@render actions()}</div>
    {/if}
  </header>

  {#if toolbar}
    <div class="relative z-20 shrink-0 min-h-11 px-5 py-1.5 flex items-center gap-x-3 gap-y-1.5 flex-wrap border-b border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/70 backdrop-blur-sm">
      {@render toolbar()}
    </div>
  {/if}

  <div class="flex-1 min-h-0 flex">
    <div class="flex-1 min-w-0 min-h-0 overflow-auto {flush ? '' : 'p-5'}">
      {@render children()}
    </div>
    {#if aside && asideOpen}
      <aside
        class="shrink-0 min-h-0 flex flex-col border-l border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/85 backdrop-blur-md"
        style="width: {asideWidth}px"
      >
        {#if asideTitle || onasideclose}
          <div class="shrink-0 h-10 px-4 flex items-center gap-2 border-b border-[var(--color-border-subtle)]">
            <span class="flex-1 text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)] truncate">{asideTitle}</span>
            {#if onasideclose}
              <button
                type="button"
                onclick={onasideclose}
                aria-label={t('common.close')}
                class="w-6 h-6 flex items-center justify-center border-0 bg-transparent text-[var(--color-text-muted)] cursor-pointer hover:text-[var(--color-text-primary)]"
              >
                <span class="i-mingcute:close-line w-4 h-4"></span>
              </button>
            {/if}
          </div>
        {/if}
        <div class="flex-1 min-h-0 overflow-y-auto">{@render aside()}</div>
      </aside>
    {/if}
  </div>
</div>
