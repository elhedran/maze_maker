// Copied from apps/tool/src/maze/maze-text.ts at 91db073 (DR-0043 point 5): the
// east filler only. A fix here is made in the tool's copy too.

// Glyphs whose east arm is a single or a double line (DR-0029 point 7).
const SINGLE_EAST = new Set([..."─└┌├┴┬┼╶╟"]);
const DOUBLE_EAST = new Set([..."═╔╚╧╤"]);

/**
 * The second character of a cell, carrying the glyph's east side so cells
 * come out about square and lines join across them (DR-0029 point 7).
 */
export function eastFiller(glyph: string): string {
  if (SINGLE_EAST.has(glyph)) return "─";
  if (DOUBLE_EAST.has(glyph)) return "═";
  if (glyph === "░") return "░";
  return " ";
}
