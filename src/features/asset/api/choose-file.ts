import { open } from '@tauri-apps/plugin-dialog';

/** Asks the author to pick files to bring into the project. Resolves to an empty list on cancel. */
export async function chooseFiles(title: string): Promise<string[]> {
  const picked = await open({ multiple: true, directory: false, title });
  if (picked === null) return [];
  return Array.isArray(picked) ? picked : [picked];
}
