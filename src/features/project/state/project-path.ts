/** The folder name for a project: what the author typed, made safe for any file system. */
export function projectFolderName(name: string): string {
  const cleaned = name
    .trim()
    .replace(/[\\/:*?"<>|]+/g, '-')
    .replace(/\s+/g, ' ')
    .replace(/^\.+/, '')
    .trim();
  return `${cleaned || 'Untitled'}.ontara`;
}

/** Joins a folder and a name with the separator the folder already uses. */
export function joinPath(folder: string, name: string): string {
  const separator = folder.includes('\\') && !folder.includes('/') ? '\\' : '/';
  return `${folder.replace(/[\\/]+$/, '')}${separator}${name}`;
}
