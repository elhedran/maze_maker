---
type: Decision Record
title: How the generation view shows one sample maze
description: A Maze tab next to the Table tab under one shared tile set bar; a seed field with Generate and Random seed; the maze drawn two characters per cell; a hover readout, the restart note and Wren's measures beside it; Reload regenerates the shown seed.
status: draft
decision_id: DR-0029
decision_status: proposed
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]
supersedes: []
tags: [tool, generation-view, maze, phase-2, slice-2-1]
generated:
  by: ux-designer/1
  at: 2026-11-01T10:30:00+01:00
verified:
  - by: product-manager/1
    at: 2026-11-01T10:45:00+01:00
---

# Context

Slice 2.1 ([roadmap](/roadmap.md)) shows one generated maze. Ian, without help: types a seed, presses Generate, sees the maze; gets the same maze from the same seed and a different one from a new seed; edits `core.toml`, presses Reload, and sees the same seed's maze change; reads Wren's measures ([DR-0030](/decisions/0030-measures-for-2-1.md)). His question at this screen is "does this look like a hedge maze, and can you get around in it?" ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md)). Kept small on purpose. Ian: "There is such a thing as polishing too much before moving on."

# Decision

Damson (`ux-designer/1`), a two-way door: one view in the tool, nothing builds on it yet.

```
+-------------------------------------------------------------------------+
| Maze Maker — Adjacency Tool   maze-core 0.x                             |
| Tile set: data/tile-sets/core.toml  loaded 20:14:03  [Reload tile set]  |
| [ Table ] [#Maze#]                                                      |
+-------------------------------------------------------------------------+
| Seed [ 48213        ] [Generate]  [Random seed]                         |
|                                                                         |
| Seed 48213 · 24×16 · restarted 2 times    | Reachable from centre:      |
| +---------------------------------------+ | 412 of 480 walkable cells   |
| |░░░░┌───┐ ░░╔═══╗░░                    | | (86%)                       |
| |░░░░│   └───╢ · ║░░  ...               | | Share of walkable cells the |
| |░░░░╵ ░░░░░░╚═══╝░░      [ ]<- centre  | | player could walk to from   |
| |  ...                                  | | the middle of the maze.     |
| +---------------------------------------+ |                             |
| Corridor corner, open N and E · col 5, row 2   Separate areas: 5 ...    |
+-------------------------------------------------------------------------+
```

## What the build must do

1. **Tabs.** Two tabs under the header: **Table** and **Maze**. Table opens first (nothing changes for 1.x). Switching hides a view rather than destroying it, so each keeps its state (selection, direction, last maze, seed field).
2. **One shared tile set bar.** The "Tile set … loaded … [Reload tile set]" bar and the red error banner move out of `adjacency-table` into `maze-tool-app`, above the tabs, because both views show the same tile set. DR-0025 point 12 still holds in substance; only its position changes.
3. **Reload.** On success the table refetches as today, and if a maze is shown it **regenerates automatically with the shown seed**. The status line then reads "Seed 48213 · regenerated after reload". On bad data the banner shows Rust's error and the last good maze stays.
4. **Seed field.** A whole number (Rust's seed type). It opens filled with a random seed, so Generate works on the first press. **Enter in the field generates.** Anything not a whole number disables Generate and shows "A seed is a whole number." under the field.
5. **Random seed** fills the field with a new random seed and generates in one press.
6. **Status line above the maze** shows the seed of the maze on screen, not the field: "Seed 48213 · 24×16 · first try" or "· restarted 2 times". If the field is edited, the maze and its seed stay until Generate.
7. **Drawing.** DR-0017 glyphs, colours and tints, no frames, natural line height so lines join (rules 2, 4, 5). **Two characters per cell**, settling DR-0017's open point: the glyph, then a filler that carries its east side, so cells come out about square. Filler: `─` if the glyph's east arm is a single line (─ └ ┌ ├ ┴ ┬ ┼ ╶ ╟); `═` if double (═ ╔ ╚ ╧ ╤); `░` after hedge; otherwise a space. The tint class covers the whole 2-character cell, so its halves still land on the right side. Font 18px (DR-0017's floor is 16px). Grid size is the engineers' choice; about 24×16 fits.
8. **Hover.** A fixed readout line under the maze: "Corridor corner, open N and E · col 5, row 2" (DR-0017 rule 7 wording), blank when not hovering. No tooltip: it would cover the neighbours Ian is comparing. Each cell also carries the name as its `aria-label`.
9. **Centre mark.** The centre cell DR-0030 used (Rust returns it, since it moves off hedge) gets a 2px outline in the text colour, never red or magenta. The measures panel says "□ centre" beside it.
10. **Measures panel** to the right of the maze: per DR-0030, label and value in bold, the one-line explanation under it in muted text.
11. **States.**
    - Before the first Generate: a blank maze-sized box (blank means "not generated yet", DR-0017 rule 6) with "Press Generate to make a maze from this seed."
    - Generating: Generate shows "Generating…" and is disabled; the old maze stays, dimmed.
    - Gave up: the maze area is cleared and reads, in the problem red, "No maze for seed 48213: gave up after 10 restarts. Try another seed, or look in the table for tiles that can't meet." Measures are hidden. The tool stays usable.

# Alternatives considered

- **Maze beside the table on one screen.** Both are wide; neither would fit. Linking them is 2.2.
- **Reload without regenerating.** One more press in Ian's loop, and a maze from the old tile set left under a fresh "loaded" time would mislead.
- **One character per cell.** Simplest, but N–S corridors look twice as long as E–W ones (DR-0017).
- **Tooltip on hover**, as in the table. It hides the cells around the one hovered.

# Consequences

- The demo is: Maze tab, Generate; Generate again; Random seed; edit, Reload, look.
- `adjacency-table` loses its own Reload and banner; its tests for them move to the app.
- Not in 2.1: keyboard movement over cells, a fallback overlay (DR-0017 rule 8), marking unreachable cells, stepping, batches, links to the table.

# Concerns raised

- **Damson (`ux-designer/1`):** "restarted 2 times" means little without knowing the limit; if Ian asks, show "2 of 10".

# Links

- Roadmap, slice 2.1: [roadmap](/roadmap.md)
- Glyphs and colours: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
- Table view: [DR-0025](/decisions/0025-table-view-design.md)
- Measures: [DR-0030](/decisions/0030-measures-for-2-1.md)
