import { invoke } from '@tauri-apps/api/core';

/** Mirrors `AppInfoDto` in `src-tauri/src/app_info/commands.rs`. */
export type AppInfo = {
  name: string;
  version: string;
};

export function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>('get_app_info');
}
