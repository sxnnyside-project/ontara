import { call } from '@/shared/backend';
import type { EntityId, EntityRef, TypeKey } from '@/shared/model';

/** An entity a change refers to. Mirrors `EntityTarget` in `ontara-core`. */
export type EntityTarget = { kind: 'existing'; id: EntityId } | { kind: 'created'; name: string };

/** One thing an assistant suggests. Mirrors `ProposedChange`. */
export type ProposedChange =
  | {
      op: 'create_entity';
      entity_type: TypeKey;
      name: string;
      summary: string;
      description: string;
      tags: string[];
    }
  | {
      op: 'update_entity';
      entity: EntityId;
      summary: string | null;
      description: string | null;
      add_tags: string[];
    }
  | {
      op: 'add_relationship';
      relationship_type: TypeKey;
      source: EntityTarget;
      target: EntityTarget;
      qualifier: string | null;
    }
  | { op: 'add_note'; entity: EntityTarget; title: string; body: string };

export type ChangeStatus = 'pending' | 'applied' | 'rejected' | 'failed';

/** An entity as it is now. Mirrors `EntityBefore`. */
export interface EntityBefore {
  name: string;
  summary: string;
  description: string;
  tags: string[];
}

export interface ChangeView {
  position: number;
  change: ProposedChange;
  status: ChangeStatus;
  error: string | null;
  before: EntityBefore | null;
}

/** Mirrors `ProposalView` in `src-tauri/src/proposal.rs`. */
export interface Proposal {
  id: string;
  source: 'ai' | 'mcp';
  author: string;
  title: string;
  rationale: string;
  created_at: number;
  changes: ChangeView[];
  entities: Record<string, EntityRef>;
  settled: boolean;
}

export function listProposals(): Promise<Proposal[]> {
  return call('list_proposals');
}

export function decideProposal(id: string, approve: number[], reject: number[]): Promise<Proposal> {
  return call('decide_proposal', { id, approve, reject });
}

export function discardProposal(id: string): Promise<void> {
  return call('discard_proposal', { id });
}
