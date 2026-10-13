import { expect, mock, test } from 'bun:test';

const invoke = mock(async (command: string, _args?: Record<string, unknown>) => {
  if (command === 'fails') throw { code: 'rejected', message: 'Not allowed.' };
  return { ok: true };
});
mock.module('@tauri-apps/api/core', () => ({ invoke }));

const { call } = await import('@/shared/backend/call');
const { CommandError } = await import('@/shared/backend/command-error');

test('call forwards the command and its arguments', async () => {
  const result = await call('list_entities', { filter: {} });

  expect(invoke).toHaveBeenCalledWith('list_entities', { filter: {} });
  expect(result).toEqual({ ok: true });
});

test('call turns a rejection into a CommandError', async () => {
  const failure = await call('fails').catch((error: unknown) => error);

  expect(failure).toBeInstanceOf(CommandError);
  expect((failure as InstanceType<typeof CommandError>).code).toBe('rejected');
});
