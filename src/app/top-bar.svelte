<script lang="ts">
import { project } from '@/features/project';
import { getLocale, t, toggleLocale } from '@/shared/i18n';
import { Button, Kbd } from '@/shared/ui';

interface Props {
  dark: boolean;
  onsearch: () => void;
  onsettings: () => void;
  onclose: () => void;
  ontheme: () => void;
}

const { dark, onsearch, onsettings, onclose, ontheme }: Props = $props();
</script>

<header class="h-11 shrink-0 border-b border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)]/90 backdrop-blur-md px-3 flex items-center gap-3">
  <div class="flex items-center gap-2 min-w-0 w-[18rem]">
    <span class="i-mingcute:earth-2-line w-5 h-5 text-[var(--color-crown-red-text)] shrink-0"></span>
    <button
      type="button"
      onclick={onsettings}
      title={t('project.settingsTitle')}
      class="min-w-0 flex items-center gap-1.5 px-1.5 py-1 rounded border-0 bg-transparent cursor-pointer text-left hover:bg-[var(--color-surface-hover)]"
    >
      <span class="truncate text-sm font-serif font-medium text-[var(--color-text-primary)]">{project.current?.name}</span>
      <span class="i-mingcute:down-line w-3.5 h-3.5 shrink-0 text-[var(--color-text-muted)]"></span>
    </button>
  </div>

  <button
    type="button"
    onclick={onsearch}
    class="flex-1 max-w-xl mx-auto h-8 px-3 flex items-center gap-2 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] text-xs text-[var(--color-text-muted)] cursor-pointer hover:border-[var(--color-border-strong)]"
  >
    <span class="i-mingcute:search-line w-3.5 h-3.5"></span>
    <span class="flex-1 text-left truncate">{t('shell.searchPrompt')}</span>
    <Kbd>{t('common.searchShortcut')}</Kbd>
  </button>

  <div class="flex items-center gap-0.5 w-[18rem] justify-end">
    <Button variant="ghost" size="sm" title={t('common.language')} onclick={() => toggleLocale()}>
      {getLocale().toUpperCase()}
    </Button>
    <Button
      variant="ghost"
      size="sm"
      icon={dark ? 'i-mingcute:sun-line' : 'i-mingcute:moon-line'}
      title={dark ? t('common.themeLight') : t('common.themeDark')}
      onclick={ontheme}
    />
    <Button variant="ghost" size="sm" icon="i-mingcute:exit-line" title={t('shell.closeProject')} onclick={onclose} />
  </div>
</header>
