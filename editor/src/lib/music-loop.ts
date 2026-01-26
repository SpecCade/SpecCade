export function clampOrderRange(
  range: { start: number; end: number },
  totalOrders: number
): { start: number; end: number } {
  const max = Math.max(0, Math.trunc(totalOrders) - 1);
  let start = Number.isFinite(range.start) ? Math.trunc(range.start) : 0;
  let end = Number.isFinite(range.end) ? Math.trunc(range.end) : start;
  start = Math.max(0, Math.min(max, start));
  end = Math.max(0, Math.min(max, end));
  if (end < start) end = start;
  return { start, end };
}
