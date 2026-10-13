<script lang="ts">
import { onMount } from 'svelte';
import { proposals } from '@/features/assistant';
import { catalog } from '@/features/catalog';
import { consistency } from '@/features/consistency';
import { history } from '@/features/history';
import { navigation } from '@/features/navigation';
import { project, WelcomeScreen } from '@/features/project';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Banner, ToastHost } from '@/shared/ui';
import Workspace from './workspace.svelte';

let failure = $state<string | null>(null);

/** Loads what the open project contains; the workspace shows once the catalog is in. */
async function load() {
  failure = null;
  try {
    navigation.clear();
    await catalog.load();
    revision.bump();
  } catch (error) {
    failure = toCommandError(error).message;
  }
}

async function closed() {
  catalog.clear();
  consistency.clear();
  history.clear();
  proposals.clear();
  navigation.clear();
}

onMount(async () => {
  try {
    await project.restore();
    if (project.current) await load();
  } catch (error) {
    failure = toCommandError(error).message;
  }
});
</script>

<div class="h-screen w-screen overflow-hidden bg-[var(--color-surface-base)] text-[var(--color-text-primary)] font-sans">
  {#if project.starting}
    <div class="h-full flex items-center justify-center gap-2 text-xs font-mono text-[var(--color-text-muted)]">
      <span class="i-mingcute:loading-line animate-spin w-4 h-4"></span>
      <span>{t('common.loading')}</span>
    </div>
  {:else if project.current && catalog.loaded}
    <Workspace onclosed={closed} />
  {:else}
    {#if failure}
      <div class="p-4"><Banner kind="error">{failure}</Banner></div>
    {/if}
    <WelcomeScreen onopened={load} />
  {/if}
  <ToastHost />
</div>
