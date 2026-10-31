---
type: Decision Record
title: Tile glyphs and colours in the tool
description: Each (tile, orientation) shows as one Unicode box-drawing or block character, with corridors in blue single lines, rooms in amber double-line walls on a floor tint, and hedge as texture.
status: stable
decision_id: DR-0017
decision_status: accepted
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]
decided_at: 2026-10-31T18:52:00+01:00
supersedes: []
tags: [tile-set, tool, glyphs, colour, accessibility]
sources:
  - id: fontsupport-2574
    resource: https://www.fileformat.info/info/unicode/char/2574/fontsupport.htm
    title: Font support for U+2574
  - id: fontsupport-2567
    resource: https://www.fileformat.info/info/unicode/char/2567/fontsupport.htm
    title: Font support for U+2567
  - id: cascadia-577
    resource: https://github.com/microsoft/cascadia-code/issues/577
    title: Cascadia Code issue 577, box-drawing line height
generated:
  by: facilitator/1
  at: 2026-10-31T15:15:00+01:00
verified:
  - by: ux-designer/1
    at: 2026-10-31T18:50:00+01:00
  - by: product-manager/1
    at: 2026-10-31T18:52:00+01:00
---

# Context

Ian's first demo is the tool exploring the adjacency table ([DR-0008](/decisions/0008-adjacency-tool-first.md)). He suggested "unicode characters to represent possible tile topology" and "color (room/hallway)". The tiles are set by [DR-0015](/decisions/0015-tile-set.md): 29 (tile, orientation) pairs in three kinds, Corridor, Room and Hedge, on the square grid of [DR-0014](/decisions/0014-square-grid.md).

Ian reads the glyphs in two places. As a table header he asks "which tile is this, and which way does it face?" In a map (phase 2) he asks "does this look like a hedge maze, and why does this pair fit?"

# Decision

## The drawing rule

**A pair fits when the drawing carries across the shared edge**: a line meets a line of the same style, floor tint meets floor tint, and a closed side meets a closed side.

- Corridors are **single lines** toward their open sides.
- Rooms are **double lines** along the room's wall, with the **floor tint** on the floor side of the cell. The tint is part of the drawing.
- A door is a **single line leaving a double wall**, the only glyph family that does this, so it reads as an opening through a wall.
- Hedge is a **texture**.

## Glyphs

One character per (tile, orientation):

| Tile type | Orientation | Glyph | Code point | Tint |
|---|---|---|---|---|
| Corridor straight | N–S, E–W | │ ─ | U+2502, U+2500 | none |
| Corridor corner | open N+E, E+S, S+W, W+N | └ ┌ ┐ ┘ | U+2514, U+250C, U+2510, U+2518 | none |
| Corridor T intersection | closed S, W, N, E | ┴ ├ ┬ ┤ | U+2534, U+251C, U+252C, U+2524 | none |
| Corridor crossroads | all | ┼ | U+253C | none |
| Corridor dead end | open N, E, S, W | ╵ ╶ ╷ ╴ | U+2575, U+2576, U+2577, U+2574 | none |
| Open room | | · | U+00B7 | full |
| Room wall | wall N, E, S, W | ═ ║ ═ ║ | U+2550, U+2551 | lower, left, upper, right half |
| Room corner | NE, SE, SW, NW | ╗ ╝ ╚ ╔ | U+2557, U+255D, U+255A, U+2554 | lower-left, upper-left, upper-right, lower-right quarter |
| Room doorway | door N, E, S, W | ╧ ╟ ╤ ╢ | U+2567, U+255F, U+2564, U+2562 | lower, left, upper, right half |
| Hedge | | ░ | U+2591 | hedge background |

## Colour

Corridors are blue and rooms are amber, a pair that holds up for protan, deutan and tritan colour vision. Colour is never the only cue: corridors are single lines, rooms are double lines with dots and tint, and hedge is texture, so the kinds still separate in greyscale.

| Token | Light | Dark |
|---|---|---|
| Page and corridor cell | #FFFFFF | #15171C |
| Corridor line | #1D5FA6 | #7CB8F2 |
| Room wall and dot | #8A4100 | #F2A65A |
| Room floor tint | #F2D0A6 | #4A3420 |
| Hedge background | #EEF2EC | #1C241D |
| Hedge glyph | #6F8A6A | #5A785A |
| Header cell frame | #C8CCD2 | #3A3F48 |

Red and magenta are kept for problems and selection; no tile uses them. Both themes are built from CSS variables and follow the Windows setting.

## Font

`"Cascadia Mono", Consolas, monospace`, regular weight, with `font-variant-ligatures: none; font-synthesis: none`. Not Cascadia Code, which has ligatures. Both ship with Windows 11. Consolas is confirmed for U+2574 and U+2567[^fontsupport-2574][^fontsupport-2567], and the preview checked both installed fonts in WebView2 and found every glyph above. The risk is per-character fallback: a missing glyph is drawn from another font with different metrics, and the joins break.

## Legibility rules

1. Table headers are at least 20px, and the explore view's centre tile 24px. Maps are at least 16px and never below 14px; below that, double lines merge and rooms stop reading as rooms. Check at 100% and 150% Windows scaling.
2. Map rows use the font's natural line height so vertical lines join. Every cell is a fixed box 1ch wide, with no letter-spacing.
3. Never rotate, mirror or stretch a glyph. A rotated column header would show a different orientation.
4. Room floor tint goes on the floor side only. Hedge cells get the hedge background. Corridor cells stay plain.
5. Header cells get a hairline frame, so you can see which edges an arm reaches (this makes dead ends readable). Maps have no frames.
6. No tile is blank. Blank is kept for "not generated yet" in phase 2.
7. Every tile shows its name on hover and keyboard focus, in the team's words, for example "Corridor corner, open N and E". The screen-reader label is the same.
8. Fallback tiles get a dashed header frame and sit last within their kind. An optional "show fallbacks" overlay on phase 2 maps is a candidate for the generation view, since where fallbacks land shows where the table struggles.

## Glyphs are presentation only

The glyphs are how the tool draws tiles. The game may draw them differently, and nothing in generation depends on them. Code, tests and logs identify tiles by tile type and orientation, never by glyph: ═ is two different tiles.

# Alternatives considered

- **An "openings" drawing for rooms**: heavy lines toward every walkable side (`┏┳┓┣╋┫`, door N `╈`). Every orientation gets a unique character and no tint is needed. But rooms draw as a lattice rather than as rooms, and a side with the wall at its north end looks the same as one with the wall at its south end, so forbidden pairs look joined (Damson).
- **Heavy dead-end stubs** (╹╺╻╸) instead of light ones. Easier to see, but they break the single-line rule for corridors. The preview shows both for Ian to compare.
- **Colour alone for room vs corridor.** Simplest, but fails for colour-blind readers and in greyscale.
- **Drawing tiles from their edge definitions** instead of characters. Never runs out and can never disagree with the table, but it isn't what Ian asked for in phase 1. It is the fallback when Unicode runs out (see Consequences).

# Consequences

- Room wall N and S share ═, and E and W share ║. Only the side the tint is on tells them apart. The tint has to render reliably, and the hover name backs it up.
- Unicode runs out as the tile set grows. The corner doorway (double E and S, single N) has no glyph, and inner corners and one-cell-wide rooms collide with existing glyphs. When a tile has no glyph, the tool draws tiles from their edge definitions in this same style. When to switch is Damson's and Marram's call.
- Monospace cells are about twice as tall as they are wide, so with one character per cell an N–S corridor looks twice as long as an E–W one. That's fine for headers. For phase 2 maps, 2 characters per cell is recommended, the second carrying the cell's east side; it is settled with the 2.1 generation-view design. The preview shows both.
- Cascadia's box-drawing heights have changed between versions[^cascadia-577]. Bundling a pinned Cascadia Mono (OFL) with the tool is recommended. That is a small technology choice for Yew and an engineer when the tool is built, and Ian's if they judge it major.

# Concerns raised

Damson (`ux-designer/1`):

- The shared ═ and ║ for opposite walls are the weakest point of this design. The openings drawing fixes it, but at a higher cost to how rooms read.

Wren (`game-designer/1`) raised, before the design: corridor glyphs naturally show where you can go, and room glyphs show walls, where you can't. The drawing rule above is Damson's answer: in both, the drawing carries across a shared edge exactly when the pair fits.

# AI guidance

- Use the glyph table here for the tool, and keep it in a presentation mapping keyed by (tile type, orientation), separate from generation data.
- Never identify a tile by its glyph in code, tests or logs.
- Don't use red or magenta for tiles.
- If a new tile has no unique glyph, don't reuse one. Raise it: the answer is drawing from edges.

# Links

- Tiles: [DR-0015](/decisions/0015-tile-set.md)
- Grid: [DR-0014](/decisions/0014-square-grid.md)
- Phase 1 scope: [DR-0008](/decisions/0008-adjacency-tool-first.md)
- Preview: `design/tile-set-preview.html` at the repository root, outside this bundle.

[^fontsupport-2574]: https://www.fileformat.info/info/unicode/char/2574/fontsupport.htm
[^fontsupport-2567]: https://www.fileformat.info/info/unicode/char/2567/fontsupport.htm
[^cascadia-577]: https://github.com/microsoft/cascadia-code/issues/577
