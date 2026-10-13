<script lang="ts">
import { AssetCard, AttachmentsPanel } from '@/features/asset';
import { catalog } from '@/features/catalog';
import {
  CanonPanel,
  EntityOverview,
  EntityRemoveDialog,
  getEntity,
  NotesPanel,
  PropertiesPanel,
  type EntityDetail,
} from '@/features/entity';
import { navigation } from '@/features/navigation';
import { RelationshipsPanel } from '@/features/relationship';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import { Banner, Button, CanonBadge, TabBar, entityIcon } from '@/shared/ui';

interface Props {
  id: string;
  onopen: (id: string) => void;
}

const { id, onopen }: Props = $props();

type Tab = 'overview' | 'properties' | 'relationships' | 'assets' | 'notes' | 'canon';

let detail = $state<EntityDetail | null>(null);
let failure = $state<string | null>(null);
let tab = $state<Tab>('overview');
let removing = $state(false);

$effect(() => {
  void revision.value;
  let cancelled = false;
  getEntity(id)
    .then((found) => {
      if (cancelled) return;
      detail = found;
      failure = null;
    })
    .catch((error) => {
      if (cancelled) return;
      const failed = toCommandError(error);
      if (failed.code === 'not_found') navigation.forget(id);
      failure = failed.message;
    });
  return () => {
    cancelled = true;
  };
});

const role = $derived(
  detail
    ? catalog.data.entity_types.find((entry) => entry.key === detail?.entity.entity_type)?.role
    : undefined,
);

const tabs = $derived([
  { key: 'overview', label: t('entity.tabOverview'), icon: 'i-mingcute:profile-line' },
  {
    key: 'properties',
    label: t('entity.tabProperties'),
    icon: 'i-mingcute:tag-line',
    count: detail?.values.length ?? 0,
  },
  { key: 'relationships', label: t('entity.tabRelationships'), icon: 'i-mingcute:git-branch-line' },
  { key: 'assets', label: t('entity.tabAssets'), icon: 'i-mingcute:attachment-line' },
  {
    key: 'notes',
    label: t('entity.tabNotes'),
    icon: 'i-mingcute:quill-pen-line',
    count: detail?.notes.length ?? 0,
  },
  { key: 'canon', label: t('entity.tabCanon'), icon: 'i-mingcute:check-circle-line' },
]);
</script>

{#if failure && !detail}
  <div class="p-6"><Banner kind="error">{failure}</Banner></div>
{:else if detail}
  {@const current = detail}
  <div class="flex flex-col min-h-full">
    <header class="px-6 pt-5 pb-0 flex flex-col gap-3 border-b border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] sticky top-0 z-10">
      <div class="flex items-start gap-3">
        <span class="{entityIcon(current.type_chain)} w-6 h-6 mt-1 text-[var(--color-crown-red)] shrink-0"></span>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
              {current.type_label}
            </span>
            <CanonBadge
              state={current.entity.canon_state}
              label={catalog.canonLabel(current.entity.canon_state)}
              interactive
              onclick={() => (tab = 'canon')}
            />
            {#if current.entity.archived}
              <span class="text-[10px] font-mono px-1 rounded bg-[var(--color-surface-elevated)] text-[var(--color-text-muted)]">{t('entity.archivedBadge')}</span>
            {/if}
          </div>
          <h2 class="m-0 text-2xl font-serif font-medium truncate">{current.entity.name}</h2>
          {#if current.entity.summary}
            <p class="m-0 mt-0.5 text-xs text-[var(--color-text-secondary)]">{current.entity.summary}</p>
          {/if}
        </div>
        <Button variant="danger" size="sm" icon="i-mingcute:delete-2-line" onclick={() => (removing = true)}>
          {t('common.remove')}
        </Button>
      </div>
      <TabBar {tabs} active={tab} onselect={(key) => (tab = key as Tab)} />
    </header>

    <div class="p-6 flex flex-col gap-5">
      {#if role === 'asset'}
        <AssetCard entity={current.entity.id} />
      {/if}
      {#if tab === 'overview'}
        <EntityOverview detail={current} />
      {:else if tab === 'properties'}
        <PropertiesPanel detail={current} />
      {:else if tab === 'relationships'}
        <RelationshipsPanel
          entity={{
            id: current.entity.id,
            name: current.entity.name,
            entity_type: current.entity.entity_type,
            type_label: current.type_label,
            canon_state: current.entity.canon_state,
            archived: current.entity.archived,
          }}
          {onopen}
        />
      {:else if tab === 'assets'}
        <AttachmentsPanel
          entity={{
            id: current.entity.id,
            name: current.entity.name,
            entity_type: current.entity.entity_type,
            type_label: current.type_label,
            canon_state: current.entity.canon_state,
            archived: current.entity.archived,
          }}
          {onopen}
        />
      {:else if tab === 'notes'}
        <NotesPanel detail={current} />
      {:else}
        <CanonPanel detail={current} />
      {/if}
    </div>
  </div>

  {#if removing}
    <EntityRemoveDialog
      id={current.entity.id}
      name={current.entity.name}
      onclose={() => (removing = false)}
      onremoved={() => {
        removing = false;
        navigation.forget(current.entity.id);
        revision.bump();
      }}
    />
  {/if}
{:else}
  <div class="p-6 flex items-center gap-2 text-xs font-mono text-[var(--color-text-muted)]">
    <span class="i-mingcute:loading-line animate-spin w-4 h-4"></span>
    <span>{t('common.loading')}</span>
  </div>
{/if}
