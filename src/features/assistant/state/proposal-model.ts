import type { ChangeView, EntityTarget, Proposal } from '../api/proposal-api';

/** How many changes wait for a decision, across every proposal. */
export function pendingCount(proposals: readonly Proposal[]): number {
  return proposals.reduce(
    (total, proposal) =>
      total + proposal.changes.filter((change) => change.status === 'pending').length,
    0,
  );
}

/** The changes that can still be decided. */
export function pendingChanges(proposal: Proposal): ChangeView[] {
  return proposal.changes.filter((change) => change.status === 'pending');
}

/** The name of whatever a change points at. */
export function targetName(
  target: EntityTarget,
  entities: Proposal['entities'],
  missing: string,
): string {
  if (target.kind === 'created') return target.name;
  return entities[target.id]?.name ?? missing;
}

/**
 * Whether approving this set of changes leaves one that points at something created by a change
 * that is not in the set. The model would refuse it; saying so first is kinder.
 */
export function danglingPositions(proposal: Proposal, approved: ReadonlySet<number>): number[] {
  const created = new Set<string>();
  for (const change of proposal.changes) {
    if (change.change.op === 'create_entity' && approved.has(change.position)) {
      created.add(change.change.name);
    }
    // A creation that was applied earlier still counts.
    if (change.change.op === 'create_entity' && change.status === 'applied') {
      created.add(change.change.name);
    }
  }
  const missing: number[] = [];
  for (const change of proposal.changes) {
    if (!approved.has(change.position)) continue;
    const kinds: EntityTarget[] = [];
    if (change.change.op === 'add_relationship')
      kinds.push(change.change.source, change.change.target);
    if (change.change.op === 'add_note') kinds.push(change.change.entity);
    if (kinds.some((target) => target.kind === 'created' && !created.has(target.name))) {
      missing.push(change.position);
    }
  }
  return missing;
}
