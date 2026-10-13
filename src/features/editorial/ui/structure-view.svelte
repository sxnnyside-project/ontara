<script lang="ts">
import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { Banner, Button, ConfirmDialog, EmptyState, entityIcon } from '@/shared/ui';
import {
  type EditorialImpact,
  getEditorialTree,
  removeSection,
  reorderSection,
  type Section,
  sectionImpact,
} from '../api/editorial-api';
import { buildTree, flatten, type SectionNode } from '../state/section-tree';
import SectionDialog from './section-dialog.svelte';

interface Props {
  selectedId: string | null;
  onselect: (id: string) => void;
  /** Called after a section was removed together with its entity. */
  onentityremoved: (id: string) => void;
}

const { selectedId, onselect, onentityremoved }: Props = $props();

let sections = $state<Section[]>([]);
let creating = $state<{ parent: Section | null } | null>(null);
let removing = $state<{ section: Section; impact: EditorialImpact } | null>(null);
let error = $state<string | null>(null);

$effect(() => {
  void revision.value;
  let cancelled = false;
  getEditorialTree()
    .then((found) => {
      if (!cancelled) sections = found;
    })
    .catch((failure) => {
      if (!cancelled) error = toCommandError(failure).message;
    });
  return () => {
    cancelled = true;
  };
});

const roots = $derived(buildTree(sections));
const rows = $derived(flatten(roots));

function siblingsOf(section: Section): SectionNode[] {
  const parent = rows.find((row) => row.node.section.entity.id === section.parent);
  return parent ? parent.node.children : roots;
}

async function move(section: Section, offset: number) {
  const siblings = siblingsOf(section);
  const index = siblings.findIndex((node) => node.section.entity.id === section.entity.id);
  const target = index + offset;
  if (index < 0 || target < 0 || target >= siblings.length) return;
  try {
    await reorderSection(section.entity.id, section.parent, target);
    revision.bump();
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

async function askToRemove(section: Section) {
  try {
    removing = { section, impact: await sectionImpact(section.entity.id) };
  } catch (failure) {
    notices.error(toCommandError(failure).message);
  }
}

async function remove(deleteEntity: boolean) {
  if (!removing) return;
  const { section } = removing;
  try {
    await removeSection(section.entity.id, deleteEntity);
    removing = null;
    if (deleteEntity) onentityremoved(section.entity.id);
    revision.bump();
  } catch (failure) {
    error = toCommandError(failure).message;
    removing = null;
  }
}

const hasChildren = $derived(removing ? removing.impact.descendants > 0 : false);
</script>

<div class="flex flex-col gap-4 p-6 max-w-4xl">
  <div class="flex items-center justify-between">
    <div>
      <h2 class="m-0 text-lg font-serif font-medium">{t('structure.title')}</h2>
      <p class="m-0 mt-0.5 text-xs text-[var(--color-text-secondary)]">{t('structure.subtitle')}</p>
    </div>
    <Button variant="primary" icon="i-mingcute:add-line" onclick={() => (creating = { parent: null })}>
      {t('structure.addTop')}
    </Button>
  </div>

  {#if error}<Banner kind="error">{error}</Banner>{/if}

  {#if rows.length === 0}
    <EmptyState icon="i-mingcute:book-2-line" title={t('structure.emptyTitle')} description={t('structure.emptyHint')} />
  {:else}
    <div class="rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-surface-panel)] divide-y divide-[var(--color-border-subtle)]">
      {#each rows as { node, depth } (node.section.entity.id)}
        {@const section = node.section}
        <div class="flex items-center gap-2 px-3 py-2 {selectedId === section.entity.id ? 'bg-[var(--color-surface-elevated)]' : ''}" style="padding-left: {0.75 + depth * 1.25}rem">
          <span class="{entityIcon(catalog.chain(section.entity.entity_type))} w-4 h-4 shrink-0 text-[var(--color-text-muted)]"></span>
          <button
            type="button"
            onclick={() => onselect(section.entity.id)}
            class="flex-1 min-w-0 text-left bg-transparent border-0 p-0 cursor-pointer"
          >
            <span class="block text-sm truncate {depth === 0 ? 'font-serif font-medium' : 'font-sans'} text-[var(--color-text-primary)]">{section.entity.name}</span>
            <span class="block text-[10px] font-mono text-[var(--color-text-muted)]">{catalog.levelLabel(section.level)}</span>
          </button>
          <Button size="sm" variant="ghost" icon="i-mingcute:up-line" title={t('structure.moveUp')} onclick={() => move(section, -1)} />
          <Button size="sm" variant="ghost" icon="i-mingcute:down-line" title={t('structure.moveDown')} onclick={() => move(section, 1)} />
          <Button size="sm" variant="ghost" icon="i-mingcute:add-line" title={t('structure.addInside')} onclick={() => (creating = { parent: section })} />
          <Button size="sm" variant="ghost" icon="i-mingcute:delete-2-line" title={t('common.remove')} onclick={() => askToRemove(section)} />
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if creating}
  <SectionDialog
    parent={creating.parent}
    onclose={() => (creating = null)}
    oncreated={(id) => {
      creating = null;
      onselect(id);
    }}
  />
{/if}

{#if removing}
  <ConfirmDialog
    title={t('structure.removeTitle', { name: removing.section.entity.name })}
    confirmLabel={t('structure.removeFromStructure')}
    danger
    busy={hasChildren}
    onconfirm={() => remove(false)}
    oncancel={() => (removing = null)}
  >
    {#if hasChildren}
      <Banner kind="warning">{t('structure.removeHasChildren', { count: removing.impact.descendants })}</Banner>
    {:else}
      <p class="m-0 text-xs text-[var(--color-text-secondary)]">{t('structure.removeBody')}</p>
      {#if removing.impact.appearances > 0 || removing.impact.context_uses > 0}
        <Banner kind="info">
          {t('structure.removeImpact', {
            appearances: removing.impact.appearances,
            contexts: removing.impact.context_uses,
          })}
        </Banner>
      {/if}
      <Button variant="danger" size="sm" onclick={() => remove(true)}>{t('structure.removeAndDelete')}</Button>
    {/if}
  </ConfirmDialog>
{/if}
