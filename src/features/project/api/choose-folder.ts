import { open } from '@tauri-apps/plugin-dialog';

/** Asks the author to pick a folder. Resolves to `null` when they cancel. */
export async function chooseFolder(title: string): Promise<string | null> {
  const picked = await open({ directory: true, multiple: false, title });
  return typeof picked === 'string' ? picked : null;
}
