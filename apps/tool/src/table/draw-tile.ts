// Drawing one tile glyph with its kind colour and floor tint (DR-0017).
// Shared by the grid headers and the explanation panel.

import { css, html } from "lit";
import { tileLook } from "./tile-glyphs.ts";
import type { TileInfo } from "./types.ts";

/** DR-0017's colour tokens, light and dark, plus the table's grey ramp. */
export const themeStyles = css`
  :host {
    --page: #ffffff;
    --text: #1b1d21;
    --muted: #5d636d;
    --corridor-line: #1d5fa6;
    --room-line: #8a4100;
    --room-tint: #f2d0a6;
    --hedge-bg: #eef2ec;
    --hedge-glyph: #6f8a6a;
    --frame: #c8ccd2;
    --divider: #6d737d;
    --highlight: #e6e9ee;
    --selection: #c2188b;
    --problem: #b00020;
    --problem-bg: #fdecee;
    --shade-1: #f0f0f0;
    --shade-2: #dcdcdc;
    --shade-3: #bdbdbd;
    --shade-4: #7d7d7d;
    --shade-5: #4a4a4a;
    --shade-text-dark: #1b1d21;
    --shade-text-light: #ffffff;
    --mono: "Cascadia Mono", Consolas, monospace;
    color: var(--text);
    background: var(--page);
  }
  @media (prefers-color-scheme: dark) {
    :host {
      --page: #15171c;
      --text: #e4e6ea;
      --muted: #9aa0aa;
      --corridor-line: #7cb8f2;
      --room-line: #f2a65a;
      --room-tint: #4a3420;
      --hedge-bg: #1c241d;
      --hedge-glyph: #5a785a;
      --frame: #3a3f48;
      --divider: #8a909a;
      --highlight: #2a2e36;
      --selection: #f062c0;
      --problem: #ff8a95;
      --problem-bg: #3a1a1e;
      /* Darker steps stay darker on a dark page, so "more likely" reads the same way. */
      --shade-1: #22252b;
      --shade-2: #30343c;
      --shade-3: #4a505a;
      --shade-4: #8a919c;
      --shade-5: #c9ced6;
      --shade-text-dark: #e4e6ea;
      --shade-text-light: #15171c;
    }
  }
`;

export const glyphStyles = css`
  .glyph {
    display: inline-block;
    width: 1.6em;
    line-height: 1.6em;
    text-align: center;
    font-family: var(--mono);
    font-size: 20px;
    font-variant-ligatures: none;
    font-synthesis: none;
    border: 1px solid var(--frame);
    background-repeat: no-repeat;
  }
  .glyph.fallback {
    border-style: dashed;
  }
  .glyph.corridor {
    color: var(--corridor-line);
  }
  .glyph.room {
    color: var(--room-line);
  }
  .glyph.hedge {
    color: var(--hedge-glyph);
    background-color: var(--hedge-bg);
  }
  /* Floor tint on the floor side only (DR-0017 rule 4). */
  .glyph[class*="tint-"] {
    background-image: linear-gradient(var(--room-tint), var(--room-tint));
  }
  .tint-full { background-size: 100% 100%; }
  .tint-lower { background-size: 100% 50%; background-position: bottom; }
  .tint-upper { background-size: 100% 50%; background-position: top; }
  .tint-left { background-size: 50% 100%; background-position: left; }
  .tint-right { background-size: 50% 100%; background-position: right; }
  .tint-lower-left { background-size: 50% 50%; background-position: left bottom; }
  .tint-upper-left { background-size: 50% 50%; background-position: left top; }
  .tint-upper-right { background-size: 50% 50%; background-position: right top; }
  .tint-lower-right { background-size: 50% 50%; background-position: right bottom; }
`;

export function renderGlyph(tile: TileInfo, label: string) {
  const look = tileLook(tile.id, tile.orientation);
  const tint = look.tint === "none" ? "" : `tint-${look.tint}`;
  const fallback = tile.tags.includes("fallback") ? "fallback" : "";
  return html`<span class="glyph ${tile.kind} ${tint} ${fallback}" role="img" aria-label=${label}
    >${look.glyph}</span
  >`;
}
