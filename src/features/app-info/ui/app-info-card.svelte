<script lang="ts">
import { getAppInfo } from '../api/get-app-info';

const info = getAppInfo();
</script>

{#await info}
  <div class="flex items-center gap-2 text-xs font-mono text-[var(--color-text-muted)]">
    <span class="i-mingcute:loading-line animate-spin w-4 h-4"></span>
    <p class="m-0">Loading…</p>
  </div>
{:then value}
  <div class="inline-flex items-center gap-2.5 px-3 py-1.5 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] text-xs font-mono">
    <span class="i-mingcute:shield-shape-line w-4 h-4 text-[var(--color-crown-red-text)]"></span>
    <p class="m-0"><strong class="font-semibold text-[var(--color-text-primary)]">{value.name}</strong> <span class="text-[var(--color-text-secondary)]">v{value.version}</span></p>
  </div>
{:catch error}
  <div class="flex items-center gap-2 text-xs font-mono text-[var(--color-finding-error)]" role="alert">
    <span class="i-mingcute:alert-line w-4 h-4"></span>
    <p class="m-0">Could not read application info: {String(error)}</p>
  </div>
{/await}
