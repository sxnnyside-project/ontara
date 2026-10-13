import { invoke } from '@tauri-apps/api/core';
import { toCommandError } from './command-error';

/**
 * Invokes a backend command. Failures are always `CommandError`s.
 *
 * Argument names are camelCase here: Tauri maps them to the snake_case parameters of the Rust
 * command. Nested objects keep the field names the backend serializes.
 */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toCommandError(error);
  }
}
