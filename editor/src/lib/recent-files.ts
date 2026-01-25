/**
 * Recent files tracking with localStorage persistence.
 *
 * Stores the last MAX_RECENT files that have been opened,
 * sorted by most recently accessed first.
 */

const STORAGE_KEY = "speccade-recent-files";
const MAX_RECENT = 10;

export interface RecentFile {
  path: string;
  name: string;
  timestamp: number;
}

/**
 * Get the list of recently opened files.
 * Returns an empty array if no files have been stored or on error.
 */
export function getRecentFiles(): RecentFile[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : [];
  } catch {
    return [];
  }
}

/**
 * Add a file to the recent files list.
 * If the file already exists, it is moved to the top.
 * The list is capped at MAX_RECENT entries.
 */
export function addRecentFile(path: string): void {
  const name = path.split(/[/\\]/).pop() || path;
  const files = getRecentFiles().filter((f) => f.path !== path);
  files.unshift({ path, name, timestamp: Date.now() });
  if (files.length > MAX_RECENT) files.pop();
  localStorage.setItem(STORAGE_KEY, JSON.stringify(files));
}

/**
 * Clear all recent files from storage.
 */
export function clearRecentFiles(): void {
  localStorage.removeItem(STORAGE_KEY);
}
