<script lang="ts">
import { typeLabelOf } from '@/shared/model';
import { catalog } from '@/features/catalog';
import { listEntities, type EntitySummary } from '@/features/entity';
import { navigation, ShowIn } from '@/features/navigation';
import { project } from '@/features/project';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { revision } from '@/shared/revision';
import {
  Banner,
  Button,
  CanonBadge,
  CheckField,
  EmptyState,
  InlineSelect,
  ViewFrame,
  entityIcon,
} from '@/shared/ui';
import { type Graph, type GraphEdge, exploreGraph } from '../api/graph-api';
import {
  assignStyles,
  dashArray,
  type EdgeStyle,
  edgeKey,
  LABELS_UP_TO,
  LOCATION,
  legendOf,
  loadOverrides,
  PARTICIPATION,
  saveOverrides,
} from '../state/edge-style';
import { radialLayout } from '../state/graph-layout';
import EdgeLegend from './edge-legend.svelte';
import EdgeMark from './edge-mark.svelte';

interface Props {
  /** Follows an entity: shows its page. */
  onopen: (id: string) => void;
}

const { onopen }: Props = $props();

let depth = $state('1');
let includeEvents = $state(true);
let types = $state<string[]>([]);
let graph = $state<Graph | null>(null);
let failure = $state<string | null>(null);
let selected = $state<string | null>(null);
let asideOpen = $state(true);
let pan = $state({ x: 0, y: 0 });
let zoom = $state(1);
let dragging: { x: number; y: number } | null = null;
let suggestions = $state<EntitySummary[]>([]);
let filterOpen = $state(false);
let alwaysLabels = $state(false);
let hovered = $state<number | null>(null);
const store = typeof localStorage === 'undefined' ? undefined : localStorage;
const projectId = project.current?.id ?? '';
let overrides = $state<Record<string, Partial<EdgeStyle>>>(loadOverrides(store, projectId));

$effect(() => {
  if (center) return;
  void revision.value;
  listEntities({ role: 'generic' })
    .then((found) => {
      suggestions = found.slice(0, 8);
    })
    .catch(() => {
      suggestions = [];
    });
});

const center = $derived(navigation.current);

$effect(() => {
  void revision.value;
  const id = center;
  const query = {
    depth: Number(depth),
    relationship_types: [...types],
    include_events: includeEvents,
  };
  if (!id) {
    graph = null;
    return;
  }
  let cancelled = false;
  exploreGraph({ center: id, ...query })
    .then((found) => {
      if (cancelled) return;
      graph = found;
      failure = null;
      selected = id;
      asideOpen = true;
      pan = { x: 0, y: 0 };
    })
    .catch((error) => {
      if (!cancelled) failure = toCommandError(error).message;
    });
  return () => {
    cancelled = true;
  };
});

const positions = $derived(
  graph
    ? radialLayout(
        graph.nodes.map((node) => ({ id: node.id, distance: node.distance })),
        graph.edges,
      )
    : new Map(),
);

const relationshipTypes = $derived(
  catalog.data.relationship_types.map((entry) => ({ key: entry.key, label: entry.label })),
);

const styles = $derived(
  assignStyles(
    catalog.data.relationship_types.map((entry) => entry.key),
    overrides,
  ),
);
const legend = $derived(graph ? legendOf(graph.edges) : []);
const undirected = $derived(
  new Set(
    catalog.data.relationship_types
      .filter((entry) => entry.directionality === 'undirected')
      .map((entry) => entry.key),
  ),
);
const showLabels = $derived(alwaysLabels || (graph?.edges.length ?? 0) <= LABELS_UP_TO);

/** The words for a kind of link, as the legend and the line labels say them. */
function kindLabel(key: string): string {
  if (key === PARTICIPATION) return t('graphView.participation');
  if (key === LOCATION) return t('graphView.location');
  return catalog.relationshipLabel(key, true);
}

function restyle(key: string, change: Partial<EdgeStyle>) {
  overrides = { ...overrides, [key]: { ...overrides[key], ...change } };
  saveOverrides(store, projectId, overrides);
}

function resetStyles() {
  overrides = {};
  saveOverrides(store, projectId, overrides);
}

const chosen = $derived(graph?.nodes.find((node) => node.id === selected) ?? null);

function edgeLabel(edge: GraphEdge): string {
  switch (edge.via) {
    case 'relationship':
      return catalog.relationshipLabel(edge.relationship_type, true);
    case 'participation':
      return edge.role;
    case 'location':
      return t('graphView.location');
  }
}

const HALF_WIDTH = 88;
const HALF_HEIGHT = 26;

/** Where the line from `from` to `to` meets the border of the card centered on `to`. */
function edgeEnd(
  from: { x: number; y: number },
  to: { x: number; y: number },
): { x: number; y: number } {
  const dx = from.x - to.x;
  const dy = from.y - to.y;
  if (dx === 0 && dy === 0) return to;
  const scale = Math.min(
    dx === 0 ? Number.POSITIVE_INFINITY : HALF_WIDTH / Math.abs(dx),
    dy === 0 ? Number.POSITIVE_INFINITY : HALF_HEIGHT / Math.abs(dy),
  );
  return { x: to.x + dx * scale, y: to.y + dy * scale };
}

function touches(edge: GraphEdge): boolean {
  return selected !== null && (edge.source === selected || edge.target === selected);
}

function toggleType(key: string) {
  types = types.includes(key) ? types.filter((entry) => entry !== key) : [...types, key];
}

function onwheel(event: WheelEvent) {
  event.preventDefault();
  zoom = Math.min(2.5, Math.max(0.3, zoom * (event.deltaY < 0 ? 1.1 : 0.9)));
}

function ondown(event: PointerEvent) {
  // Controls laid over the canvas keep their own clicks: capturing the pointer would retarget them.
  if ((event.target as HTMLElement).closest('[data-overlay]')) return;
  dragging = { x: event.clientX - pan.x, y: event.clientY - pan.y };
  (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
}

function onmove(event: PointerEvent) {
  if (dragging) pan = { x: event.clientX - dragging.x, y: event.clientY - dragging.y };
}

function onup() {
  dragging = null;
}
</script>

<svelte:window
  onclick={(event) => {
    if (filterOpen && !(event.target as HTMLElement).closest('[data-filter]')) filterOpen = false;
  }}
/>

<ViewFrame
  icon="i-mingcute:git-branch-line"
  title={t('graphView.title')}
  description={center && chosen ? t('graphView.around', { name: graph?.nodes.find((node) => node.id === center)?.name ?? '' }) : t('graphView.subtitle')}
  asideWidth={288}
  asideOpen={chosen !== null && asideOpen}
  asideTitle={t('graphView.selected')}
  onasideclose={() => (asideOpen = false)}
  flush
>
  {#snippet toolbar()}
    <InlineSelect
      label={t('graphView.depth')}
      bind:value={depth}
      options={[1, 2, 3, 4].map((step) => ({ value: String(step), label: t('graphView.steps', { count: step }) }))}
    />
    <CheckField label={t('graphView.includeEvents')} bind:checked={includeEvents} />
    <div class="relative text-xs" data-filter>
      <button
        type="button"
        aria-expanded={filterOpen}
        onclick={() => (filterOpen = !filterOpen)}
        class="cursor-pointer text-[var(--color-text-secondary)] select-none flex items-center gap-1 h-7 px-2 rounded border border-[var(--color-border-subtle)] bg-[var(--color-surface-base)] hover:border-[var(--color-border-strong)]"
      >
        <span class="i-mingcute:filter-line w-3.5 h-3.5"></span>
        {types.length === 0 ? t('graphView.allTypes') : t('graphView.someTypes', { count: types.length })}
        <span class="{filterOpen ? 'i-mingcute:up-line' : 'i-mingcute:down-line'} w-3 h-3"></span>
      </button>
      {#if filterOpen}
        <div class="absolute left-0 z-30 mt-1 w-80 max-h-72 overflow-y-auto p-2 flex flex-wrap gap-1.5 rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-surface-panel)] shadow-[var(--shadow-md)]">
          {#each relationshipTypes as option (option.key)}
            {@const style = styles.get(option.key)}
            <button
              type="button"
              aria-pressed={types.includes(option.key)}
              onclick={() => toggleType(option.key)}
              class="flex items-center gap-1.5 px-2 py-0.5 rounded border text-[11px] font-mono cursor-pointer transition-colors
              {types.includes(option.key)
                ? 'border-[var(--color-crown-red)] bg-[var(--color-crown-red-subtle)] text-[var(--color-text-primary)]'
                : 'border-[var(--color-border-subtle)] bg-[var(--color-surface-elevated)] text-[var(--color-text-secondary)]'}"
            >
              <span class="w-2 h-2 rounded-full shrink-0" style="background: {style?.color}"></span>
              {option.label}
            </button>
          {/each}
          {#if types.length > 0}
            <Button size="sm" variant="ghost" onclick={() => (types = [])}>{t('graphView.clearTypes')}</Button>
          {/if}
        </div>
      {/if}
    </div>
    <CheckField label={t('graphView.alwaysLabels')} bind:checked={alwaysLabels} />
    <span class="flex-1"></span>
    <ShowIn views={['universe', 'matrix', 'timeline']} />
  {/snippet}

  {#snippet aside()}
    <div class="p-4 flex flex-col gap-3">
      {#if chosen}
        <div class="flex flex-col gap-1">
          <span class="text-[11px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">{typeLabelOf(chosen)}</span>
          <h3 class="m-0 text-base font-serif">{chosen.name}</h3>
          <CanonBadge state={chosen.canon_state} label={catalog.canonLabel(chosen.canon_state)} size="sm" />
        </div>
        <div class="flex flex-col gap-1.5">
          {#if chosen.id !== center}
            <Button size="sm" variant="primary" icon="i-mingcute:aiming-2-line" onclick={() => navigation.open(chosen.id)}>
              {t('graphView.centerHere')}
            </Button>
          {/if}
          <Button size="sm" variant="secondary" icon="i-mingcute:arrow-right-up-line" onclick={() => onopen(chosen.id)}>
            {t('graphView.openEntity')}
          </Button>
        </div>
        {#if graph}
          <p class="m-0 text-[11px] text-[var(--color-text-muted)]">
            {t('graphView.stats', { nodes: graph.nodes.length, edges: graph.edges.length })}
          </p>
        {/if}
      {:else}
        <p class="m-0 text-xs text-[var(--color-text-muted)]">{t('graphView.selectHint')}</p>
      {/if}
    </div>
  {/snippet}

  {#if !center}
    <EmptyState icon="i-mingcute:git-branch-line" title={t('graphView.pickTitle')} description={t('graphView.pickHint')}>
      {#each suggestions as item (item.id)}
        <Button icon={entityIcon(catalog.chain(item.entity_type))} onclick={() => navigation.open(item.id)}>{item.name}</Button>
      {/each}
    </EmptyState>
  {:else if failure && !graph}
    <div class="p-6"><Banner kind="error">{failure}</Banner></div>
  {:else if graph}
    {#if graph.truncated}
      <div class="absolute z-20 left-1/2 -translate-x-1/2 mt-3"><Banner kind="warning">{t('graphView.truncated', { limit: graph.limit })}</Banner></div>
    {/if}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="relative h-full overflow-hidden cursor-grab active:cursor-grabbing select-none"
      {onwheel}
      onpointerdown={ondown}
      onpointermove={onmove}
      onpointerup={onup}
    >
    {#if graph.nodes.length === 1}
      <div class="absolute z-10 left-1/2 bottom-16 -translate-x-1/2 text-center text-xs text-[var(--color-text-muted)] max-w-xs">
        {t('graphView.alone')}
      </div>
    {/if}
      <div
        class="absolute left-1/2 top-1/2"
        style="transform: translate({pan.x}px, {pan.y}px) scale({zoom}); transform-origin: 0 0;"
      >
        <svg class="absolute overflow-visible" width="1" height="1">
          <defs>
            {#each [...styles.entries()] as [key, style] (key)}
              <marker id="arrow-{key}" viewBox="0 0 10 10" refX="9" refY="5" markerUnits="userSpaceOnUse" markerWidth="11" markerHeight="11" orient="auto-start-reverse">
                <path d="M0,0 L10,5 L0,10 z" fill={style.color} />
              </marker>
            {/each}
          </defs>
          {#each graph.edges as edge, index (index)}
            {@const source = positions.get(edge.source)}
            {@const target = positions.get(edge.target)}
            {@const style = styles.get(edgeKey(edge))}
            {#if source && target && style}
              {@const from = edgeEnd(target, source)}
              {@const to = edgeEnd(source, target)}
              {@const active = touches(edge) || hovered === index}
              {@const middle = { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 }}
              {@const directed = edge.via !== 'relationship' || !undirected.has(edge.relationship_type)}
              <g opacity={selected !== null && !touches(edge) && hovered !== index ? 0.55 : 1}>
                <line
                  x1={from.x}
                  y1={from.y}
                  x2={to.x}
                  y2={to.y}
                  stroke={style.color}
                  stroke-width={active ? 2.6 : 1.6}
                  stroke-dasharray={dashArray(style.pattern)}
                  marker-end={directed ? `url(#arrow-${edgeKey(edge)})` : undefined}
                />
                <!-- A wider invisible line so a thin link can be pointed at. -->
                <line
                  x1={from.x}
                  y1={from.y}
                  x2={to.x}
                  y2={to.y}
                  stroke="transparent"
                  stroke-width="12"
                  role="presentation"
                  onpointerenter={() => (hovered = index)}
                  onpointerleave={() => (hovered = null)}
                  onpointerdown={(event) => event.stopPropagation()}
                >
                  <title>{kindLabel(edgeKey(edge))}{edge.via === 'participation' ? `: ${edge.role}` : ''}</title>
                </line>
                <EdgeMark mark={style.mark} x={middle.x} y={middle.y} color={style.color} />
                {#if showLabels || active}
                  <text
                    x={middle.x}
                    y={middle.y - 9}
                    text-anchor="middle"
                    font-size="10"
                    fill="var(--color-text-secondary)"
                    stroke="var(--color-surface-base)"
                    stroke-width="3"
                    paint-order="stroke"
                    font-family="var(--font-mono)"
                  >{edgeLabel(edge)}</text>
                {/if}
              </g>
            {/if}
          {/each}
        </svg>
        {#each graph.nodes as node (node.id)}
          {@const point = positions.get(node.id)}
          {#if point}
            <button
              type="button"
              onpointerdown={(event) => event.stopPropagation()}
              onclick={() => {
                selected = node.id;
                asideOpen = true;
              }}
              ondblclick={() => navigation.open(node.id)}
              class="absolute -translate-x-1/2 -translate-y-1/2 w-44 rounded-lg border px-2.5 py-2 text-left cursor-pointer transition-colors
              {selected === node.id
                ? 'border-[var(--color-crown-red)] ring-1 ring-[var(--color-crown-red)]'
                : 'border-[var(--color-border-subtle)] hover:border-[var(--color-border-strong)]'}
              {node.distance === 0 ? 'bg-[var(--color-surface-elevated)]' : 'bg-[var(--color-surface-panel)]'}"
              style="left: {point.x}px; top: {point.y}px;"
            >
              <span class="flex items-center gap-1.5 text-[10px] font-mono uppercase tracking-wider text-[var(--color-text-muted)]">
                <span class="{entityIcon(catalog.chain(node.entity_type))} w-3 h-3 shrink-0"></span>
                <span class="truncate">{typeLabelOf(node)}</span>
              </span>
              <span class="block text-sm font-serif truncate">{node.name}</span>
            </button>
          {/if}
        {/each}
      </div>
      <div data-overlay class="absolute bottom-3 left-3 flex items-center gap-0.5">
        <Button size="sm" variant="ghost" icon="i-mingcute:add-line" title={t('graphView.zoomIn')} onclick={() => (zoom = Math.min(2.5, zoom * 1.2))} />
        <Button size="sm" variant="ghost" icon="i-mingcute:minimize-line" title={t('graphView.zoomOut')} onclick={() => (zoom = Math.max(0.3, zoom / 1.2))} />
        <Button size="sm" variant="ghost" icon="i-mingcute:aiming-2-line" title={t('graphView.reset')} onclick={() => { pan = { x: 0, y: 0 }; zoom = 1; }} />
      </div>
      {#if legend.length > 0}
        <EdgeLegend
          entries={legend}
          {styles}
          label={kindLabel}
          onchange={restyle}
          onreset={resetStyles}
          customized={Object.keys(overrides).length > 0}
        />
      {/if}
    </div>
  {/if}
</ViewFrame>
