<script lang="ts">
import type { Mark } from '../state/edge-style';

interface Props {
  mark: Mark;
  x: number;
  y: number;
  color: string;
}

const { mark, x, y, color }: Props = $props();
</script>

{#if mark === 'circle'}
  <circle cx={x} cy={y} r="4" fill={color} />
{:else if mark === 'square'}
  <rect x={x - 3.5} y={y - 3.5} width="7" height="7" fill={color} />
{:else if mark === 'diamond'}
  <path d="M{x},{y - 5} L{x + 5},{y} L{x},{y + 5} L{x - 5},{y} Z" fill={color} />
{:else}
  <rect x={x - 1.5} y={y - 5} width="3" height="10" fill={color} />
{/if}
