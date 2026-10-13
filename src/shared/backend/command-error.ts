/**
 * Why a backend command failed. Mirrors `CommandError` in `src-tauri/src/workspace.rs`.
 *
 * `code` is stable and meant to be branched on; `message` is meant for people.
 */
export class CommandError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = 'CommandError';
    this.code = code;
  }
}

function isCommandErrorPayload(value: unknown): value is { code: string; message: string } {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value &&
    typeof value.code === 'string' &&
    typeof value.message === 'string'
  );
}

/** Normalizes whatever the IPC layer rejected with into a `CommandError`. */
export function toCommandError(error: unknown): CommandError {
  if (error instanceof CommandError) return error;
  if (isCommandErrorPayload(error)) return new CommandError(error.code, error.message);
  if (typeof error === 'string') return new CommandError('unknown', error);
  if (error instanceof Error) return new CommandError('unknown', error.message);
  return new CommandError('unknown', 'Something went wrong.');
}
