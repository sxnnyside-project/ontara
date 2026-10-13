import { expect, test } from 'bun:test';
import { CommandError, toCommandError } from '@/shared/backend/command-error';

test('a payload from the backend becomes a CommandError with its code', () => {
  const error = toCommandError({ code: 'still_referenced', message: 'It is referenced.' });

  expect(error).toBeInstanceOf(CommandError);
  expect(error.code).toBe('still_referenced');
  expect(error.message).toBe('It is referenced.');
});

test('anything else is normalized to an unknown failure with a readable message', () => {
  expect(toCommandError('boom').code).toBe('unknown');
  expect(toCommandError('boom').message).toBe('boom');
  expect(toCommandError(new Error('bad')).message).toBe('bad');
  expect(toCommandError(42).message).toBe('Something went wrong.');
});

test('an existing CommandError is kept as it is', () => {
  const original = new CommandError('no_project', 'Open a project first.');

  expect(toCommandError(original)).toBe(original);
});
