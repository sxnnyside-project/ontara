<script lang="ts">
import { notices } from '@/shared/notice';

const icons = {
  error: 'i-mingcute:close-circle-line text-[var(--color-finding-error)]',
  success: 'i-mingcute:check-circle-line text-[var(--color-canon-established)]',
  info: 'i-mingcute:information-line text-[var(--color-finding-info)]',
} as const;
</script>

<div class="fixed bottom-10 right-4 z-50 flex flex-col gap-2 w-80 pointer-events-none">
  {#each notices.items as notice (notice.id)}
    <div
      role={notice.kind === 'error' ? 'alert' : 'status'}
      class="pointer-events-auto flex items-start gap-2 rounded border border-[var(--color-border-strong)] bg-[var(--color-surface-overlay)] px-3 py-2 text-xs shadow-[var(--shadow-md)]"
    >
      <span class="{icons[notice.kind]} w-4 h-4 shrink-0 mt-0.5"></span>
      <p class="m-0 flex-1 text-[var(--color-text-primary)]">{notice.message}</p>
      <button
        type="button"
        aria-label="Dismiss"
        onclick={() => notices.dismiss(notice.id)}
        class="bg-transparent border-0 p-0 cursor-pointer text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
      >
        <span class="i-mingcute:close-line w-3.5 h-3.5"></span>
      </button>
    </div>
  {/each}
</div>
