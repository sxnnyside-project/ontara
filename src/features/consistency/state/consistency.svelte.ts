import { type Report, checkConsistency } from '../api/consistency-api';

/** The latest consistency report of the open project, shared by the view and the shell badge. */
class ConsistencyState {
  report = $state<Report | null>(null);
  loading = $state(false);

  async refresh(): Promise<void> {
    this.loading = true;
    try {
      this.report = await checkConsistency();
    } finally {
      this.loading = false;
    }
  }

  clear(): void {
    this.report = null;
  }
}

export const consistency = new ConsistencyState();
