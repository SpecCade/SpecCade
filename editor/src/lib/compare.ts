export function clamp01(x: number): number {
  if (!Number.isFinite(x)) return 0;
  return Math.max(0, Math.min(1, x));
}

export function wipeToSplitX(wipe: number, widthPx: number): number {
  const w = Math.max(0, Math.trunc(widthPx));
  return Math.round(clamp01(wipe) * w);
}
