export type InspectBlobV1 = {
  v: 1;
  texture?: unknown;
  audio?: unknown;
  music?: unknown;
};

function isProbablyWindowsPath(p: string): boolean {
  return /^[A-Za-z]:\//.test(p) || p.startsWith("\\\\") || p.includes("\\");
}

export function normalizePathForStorage(path: string): string {
  let p = (path || "").trim();
  if (!p) return "editor.star";

  p = p.replace(/\\/g, "/");

  // Collapse repeated slashes, but preserve leading UNC double slash.
  const isUnc = p.startsWith("//");
  p = p.replace(/\/+/g, "/");
  if (isUnc && !p.startsWith("//")) p = `/${p}`;

  // Normalize Windows drive letter casing.
  if (/^[A-Za-z]:\//.test(p)) {
    p = p[0].toLowerCase() + p.slice(1);
  }

  if (isProbablyWindowsPath(path)) {
    p = p.toLowerCase();
  }

  return p;
}

export function loadJson<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

export function saveJson(key: string, value: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // ignore
  }
}

export function inspectKeyForFile(filePath: string): string {
  return `speccade:inspect:${normalizePathForStorage(filePath)}`;
}

export function loadInspectBlob(filePath: string): InspectBlobV1 {
  const key = inspectKeyForFile(filePath);
  const blob = loadJson<Partial<InspectBlobV1>>(key, {});
  if (blob && blob.v === 1) {
    return blob as InspectBlobV1;
  }
  return { v: 1 };
}

export function saveInspectSection<T>(
  filePath: string,
  section: "texture" | "audio" | "music",
  value: T
): void {
  const key = inspectKeyForFile(filePath);
  const blob = loadInspectBlob(filePath);
  const next: InspectBlobV1 = {
    ...blob,
    v: 1,
    [section]: value,
  };
  saveJson(key, next);
}

export function loadInspectSection<T>(
  filePath: string,
  section: "texture" | "audio" | "music",
  fallback: T
): T {
  const blob = loadInspectBlob(filePath);
  const value = blob[section];
  return (value as T) ?? fallback;
}
