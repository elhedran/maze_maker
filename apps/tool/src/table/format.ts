// Formatting for the table view (DR-0025). The numbers come from Rust; this
// only turns them into text and shade steps, never recalculates them.

import type { Direction, SideKind } from "./types.ts";

export const DIRECTIONS: Direction[] = ["north", "east", "south", "west"];

export const DIRECTION_LABEL: Record<Direction, string> = {
  north: "North",
  east: "East",
  south: "South",
  west: "West",
};

/** DR-0025 point 1: N, E, S and W switch direction. */
export const DIRECTION_KEYS: Record<string, Direction> = {
  n: "north",
  e: "east",
  s: "south",
  w: "west",
};

/** Share of row as DR-0025 point 5 shows it: one decimal, "<0.1", blank for 0. */
export function formatShare(probability: number): string {
  if (!(probability > 0)) return "";
  const percent = probability * 100;
  if (percent < 0.05) return "<0.1";
  return percent.toFixed(1);
}

/**
 * DR-0025 point 7: five fixed steps, so a shade means the same after a
 * reload. 0 is step 0 (no shade). Boundaries: above 0 to 5, 5 to 10,
 * 10 to 20, 20 to 40, 40 and up; a value on a boundary takes the upper step.
 */
export function shadeStep(probability: number): 0 | 1 | 2 | 3 | 4 | 5 {
  if (!(probability > 0)) return 0;
  const percent = probability * 100;
  if (percent < 5) return 1;
  if (percent < 10) return 2;
  if (percent < 20) return 3;
  if (percent < 40) return 4;
  return 5;
}

export const SHADE_LEGEND = ["0–5", "5–10", "10–20", "20–40", "40+"];

/** DR-0015's side names: "Passage", "Floor, wall at N end". */
export function sideName(side: SideKind): string {
  switch (side) {
    case "hedge":
      return "Hedge";
    case "passage":
      return "Passage";
    case "door":
      return "Door";
    case "floor":
      return "Floor";
    case "floor_wall_n":
      return "Floor, wall at N end";
    case "floor_wall_e":
      return "Floor, wall at E end";
    case "floor_wall_s":
      return "Floor, wall at S end";
    case "floor_wall_w":
      return "Floor, wall at W end";
  }
}

const lower = (side: SideKind) => sideName(side).toLowerCase();

/**
 * The clash line for a 0 (DR-0025 point 11):
 * "Passage meets Floor, wall at N end. A passage only meets a passage or a door."
 */
export function clashLine(facing: [SideKind, SideKind], mayMeet: SideKind[]): string {
  const [mine, theirs] = facing;
  const meets = `${sideName(mine)} meets ${sideName(theirs)}.`;
  if (mayMeet.length === 0) return `${meets} A ${lower(mine)} meets nothing.`;
  const options = mayMeet.map((s) => `a ${lower(s)}`).join(" or ");
  return `${meets} A ${lower(mine)} only meets ${options}.`;
}

/** Numbers in the explanation: up to two decimals, at least one ("1.0", "0.25", "1.07"). */
export function formatNumber(value: number): string {
  const fixed = value.toFixed(2);
  return fixed.endsWith("0") ? value.toFixed(1) : fixed;
}

/** "20:14:03" from an ISO 8601 time; the text as given if it doesn't parse. */
export function formatLoadTime(iso: string): string {
  const time = new Date(iso);
  if (Number.isNaN(time.getTime())) return iso;
  return time.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false });
}
