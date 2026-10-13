import { expect, mock, test } from 'bun:test';

const invoke = mock(async (_command: string) => ({ name: 'Ontara', version: '0.1.0' }));
mock.module('@tauri-apps/api/core', () => ({ invoke }));

const { getAppInfo } = await import('@/features/app-info/api/get-app-info');

test('getAppInfo invokes the get_app_info command and returns its payload', async () => {
  const info = await getAppInfo();

  expect(invoke).toHaveBeenCalledWith('get_app_info');
  expect(info).toEqual({ name: 'Ontara', version: '0.1.0' });
});
