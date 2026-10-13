import { type Proposal, listProposals } from '../api/proposal-api';
import { pendingCount } from './proposal-model';

/** The suggestions waiting in the open project, shared by the view and the shell badge. */
class ProposalsState {
  items = $state<Proposal[]>([]);

  get pending(): number {
    return pendingCount(this.items);
  }

  async refresh(): Promise<void> {
    this.items = await listProposals();
  }

  clear(): void {
    this.items = [];
  }
}

export const proposals = new ProposalsState();
