import { catalog } from '@/features/catalog';
import { toCommandError } from '@/shared/backend';
import { t } from '@/shared/i18n';
import { notices } from '@/shared/notice';
import { revision } from '@/shared/revision';
import { type Change, getHistory, redoChange, undoChange } from '../api/history-api';
import { changeText } from './history-model';

/** The edits of the open project that can be taken back or done again. */
class HistoryState {
  undoable = $state<Change[]>([]);
  redoable = $state<Change[]>([]);
  busy = $state(false);

  /** Reads the history again, as after the data changed. */
  async refresh(): Promise<void> {
    const state = await getHistory();
    this.undoable = state.undo;
    this.redoable = state.redo;
  }

  clear(): void {
    this.undoable = [];
    this.redoable = [];
  }

  /** Reverses the latest edit and says what it was. */
  async undo(): Promise<void> {
    await this.step('undo');
  }

  /** Applies again what was last undone. */
  async redo(): Promise<void> {
    await this.step('redo');
  }

  private async step(direction: 'undo' | 'redo'): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    try {
      const state = await (direction === 'undo' ? undoChange() : redoChange());
      this.undoable = state.undo;
      this.redoable = state.redo;
      if (state.applied) {
        // An undone catalog change must reach what the interface offers, not only the records.
        await catalog.load();
        revision.bump();
        notices.success(
          t(direction === 'undo' ? 'history.undoneWhat' : 'history.redoneWhat', {
            what: changeText(state.applied.label),
          }),
        );
      } else {
        notices.show(
          'info',
          t(direction === 'undo' ? 'history.nothingToUndo' : 'history.nothingToRedo'),
        );
      }
    } catch (failure) {
      notices.error(toCommandError(failure).message);
    } finally {
      this.busy = false;
    }
  }
}

export const history = new HistoryState();
