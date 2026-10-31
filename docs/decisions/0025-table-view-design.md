---
type: Decision Record
title: How the table is laid out and how an entry explains itself
description: One 29 by 29 grid per direction with a direction switcher, rows grouped by kind, each cell a shade plus a row percentage, and a side panel that explains the selected entry from its facing sides, base, weights and exceptions.
status: stable
decision_id: DR-0025
decision_status: accepted
decided_at: 2026-10-31T19:33:30+01:00
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]
supersedes: []
tags: [tool, adjacency-table, table-view, explanation, phase-1, slice-1-1]
generated:
  by: ux-designer/1
  at: 2026-10-31T19:30:00+01:00
verified:
  - by: product-manager/1
    at: 2026-10-31T19:33:30+01:00
---

# Context

Slice 1.1 ([roadmap](/roadmap.md)) shows the whole table and lets any entry explain itself. Ian's demo tasks, without help: give the probability of a corridor corner east of a straight corridor and say why; find a 0, such as a corridor running into a room wall, and say which sides clash. He wants to "see the variation in the table of probabilities" (Q-0007), and his loop is: see a problem, look at the numbers, edit the data, reload ([Q-0010](/questions/resolved/0010-where-editing-goes.md)).

Ian's question at the table is always one sentence: "if this tile is here, what goes to its east, how likely, and why?" The layout reads in that sentence. Tiles and drawing: [DR-0015](/decisions/0015-tile-set.md), [DR-0017](/decisions/0017-tile-glyphs-and-colours.md). Numbers: entry = side-pair base × weight of A × weight of B × exception multiplier ([DR-0022](/decisions/0022-weight-model.md), the weight model). Rust returns a view; the frontend draws it ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)).

# Decision

Damson (`ux-designer/1`), a two-way door: it is one view in the tool and nothing else builds on it yet.

```
+-------------------------------------------------------------------------------------+
| Tile set: data/tiles.toml   loaded 20:14:03   [Reload tile set]                     |
| Neighbour to the:  [ North ] [#East#] [ South ] [ West ]    shade: 0-5 5-10 ... 40+ |
+---------------------------------------------------------+---------------------------+
| ROW tile, then its neighbour to the EAST (% of row)     | ─ then ┐ to its east      |
|            | Corridor              ¦ Room        ¦Hedge |                           |
|            | │  ─  └  ┌  ┐ ... ╴    ¦ · ═ ║ ... ¦ ░    | Corridor straight, E-W    |
| Corridor │ |    .  .  .  .         ¦ 7.9  ...    ¦30.1  |  then to its east         |
|          ─ |12.0   18.5 [15.0]     ¦             ¦      | Corridor corner, open S+W |
|          └ |  ...                  ¦             ¦      |                           |
| - - - - - -+- - - - - - - - - - - -¦- - - - - - ¦- - - | 15.0% of ─'s east         |
| Room     · |                       ¦ 22.4 ...    ¦      | neighbours                |
|          ═ |  ...                  ¦             ¦      |                           |
| - - - - - -+- - - - - - - - - - - -¦- - - - - - ¦- - - | Facing sides              |
| Hedge    ░ |  ...                  ¦             ¦      |  ─ east side: Passage     |
|            |                       ¦             ¦      |  ┐ west side: Passage     |
|            |                       ¦             ¦      | Where it comes from       |
|            |                       ¦             ¦      |  Passage-Passage base 1.0 |
|            |                       ¦             ¦      |  x Corridor straight  2.0 |
|            |                       ¦             ¦      |  x Corridor corner    3.0 |
|            |                       ¦             ¦      |  x exceptions: none       |
|            |                       ¦             ¦      |  = entry 6.0              |
|            |                       ¦             ¦      |  / row total 40.0 = 15.0% |
+---------------------------------------------------------+---------------------------+
```

## What the build must do

1. **One grid, one direction at a time.** A switcher (North, East, South, West) above the grid picks the direction; East is selected on open. Keys N, E, S and W switch too. The selected cell stays selected across a switch.
2. **Reading.** Rows are the tile; columns are the neighbour in the chosen direction. The corner of the grid says it in words: "Row tile, then its neighbour to the EAST", changing with the switcher.
3. **Grouped by kind.** Rows and columns run in the same order: Corridor, Room, Hedge. Within a kind, DR-0015's tile order, fallbacks last (DR-0017 rule 8), then orientations in DR-0017's order. Each kind has a label and a heavier divider, so room-to-corridor blocks stand out. The order comes from Rust.
4. **Headers** are DR-0017 glyphs at 20px or more, framed, never rotated. Hover or focus shows the name and weight: "Corridor corner, open N and E · weight 3.0".
5. **Cell number.** The share of its row: entry ÷ sum of the row's entries in this direction, as a percentage with one decimal and no % sign (the corner says "% of row"). Non-zero below 0.05 shows "<0.1", never "0.0". Each row adds up to 100.
6. **0 is blank**: no number, no shade, but still hoverable and selectable.
7. **Shade.** A neutral grey ramp in five fixed steps: above 0 to 5, 5 to 10, 10 to 20, 20 to 40, 40 and up, with a legend. Fixed steps, so a shade means the same before and after a reload. Text turns light on the two darkest steps. No blue, amber, red or magenta (DR-0017).
8. **Variation shows** three ways: the numbers, the shade steps, and a small corner mark on any cell where an exception applies (legend: "exception applied").
9. **Hover a cell:** its row and column headers light up, and a tooltip reads "─ Corridor straight, E–W, then ┐ Corridor corner, open S and W to its east: 15.0%".
10. **Select a cell** by click, or by arrow keys once the grid has focus. The cell gets a magenta outline and its row and column headers stay lit.
11. **The explanation panel**, on the right, for the selected cell:
    - the pair drawn side by side in the chosen direction, with both names;
    - the share, worded "15.0% of ─'s east neighbours";
    - **Facing sides:** each tile's side that touches, by DR-0015 name ("Passage", "Floor, wall at N end");
    - **Where it comes from:** base for that side pair, × weight of the row tile, × weight of the neighbour, × each exception with its value and the rule as written in the data ("exceptions: none" if none), = entry, ÷ row total = share.
    - **For a 0:** "Can't happen", the facing sides, and the clash in one line: "Passage meets Floor, wall at N end. A passage only meets a passage or a door."
    - With nothing selected: "Select a cell to see where its number comes from."
12. **Reload tile set** in the top bar, next to the file path and load time. On success the grid redraws and the selection and direction are kept. On bad data, the last good table stays on screen and a red banner shows Rust's error as given, until the next successful reload.
13. **Leave room for 1.2 and 1.3.** Filters (1.3) go in a bar above the grid and become part of the view request. The panel gets an "Explore this tile" action in 1.2. Neither is built here.

## What the Rust view must carry

- The direction; rows and columns in display order, each with its tile type, orientation, kind, tags, name and weight. Glyphs stay in the frontend mapping (DR-0017).
- Per cell: the entry, the share of the row, and whether exceptions apply. Per row: the total.
- Per entry, inline or through a separate explain call (Yew's call): the facing side kinds of both tiles, the base, both tile weights, each exception applied (value and its rule as written in the data), and, for a 0, the kinds the row's side may meet.
- The tile set file path and load time, and load errors as readable text.

If Yew decides the number shown is not a row share, points 5 and 11 change their wording and arithmetic; the layout holds.

# Alternatives considered

- **Four grids at once (N, E, S, W).** Every direction visible, but four 29-wide grids don't fit legibly at 20px headers, and the demo tasks ask about one direction at a time.
- **Raw entries in the cells.** Shows the model directly, but "6.0" doesn't answer "how likely?". The raw entry is in the panel.
- **Shade scaled to the largest cell on screen.** More contrast, but a shade would change meaning after every reload, which breaks Ian's edit-and-compare loop.
- **A mark on 0 cells (dot or hatch).** Easier to target, but 916 of 3,364 cells are non-zero; marking the rest drowns the pattern. Hover and selection still reach a blank cell.

# Consequences

- The demo tasks are two clicks each: East, then the cell, then read the panel.
- Wren's hand check of entries reads straight off the panel's arithmetic.
- At full size (1.5) one grid may stop being readable; that's 1.5's design, and the view request already allows ranges.

# Concerns raised

- **Damson (`ux-designer/1`):** row percentages hide that one row's tile is rarer than another's. The panel shows both tile weights, and header hover shows weight, which should be enough for 1.1. Review will tell.
- **Marram (`product-manager/1`), on review:** approved; both 1.1 demo tasks are a switch, a click and a read of the panel. Points for the build, not a new round: (1) the must-haves for the demo are the grid, share, blank 0s, shade, and the panel with facing sides, arithmetic and the clash line; keyboard shortcuts (N/E/S/W, arrows) and keeping the selection across reload can slip to a follow-up if they slow the slice. (2) The DR-0022 link should become bundle-relative now that DR-0022 is on `main`. (3) `generated.at` (20:10) is later than this approval; worth correcting when the record is committed. Both fixed by the facilitator before commit.

# AI guidance

- Rows are the tile, columns the neighbour in the chosen direction. Never swap them per direction.
- Keep shading steps fixed; don't rescale to the data.
- Explanations come from Rust; the frontend formats them and doesn't recalculate them.
- Name sides and tiles in DR-0015's words.

# Links

- Roadmap, slice 1.1: [roadmap](/roadmap.md)
- Table entries: [DR-0013](/decisions/0013-table-entries-from-tile-sides.md)
- Tile set and side kinds: [DR-0015](/decisions/0015-tile-set.md)
- Glyphs and colours: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
- Weight model: [DR-0022](/decisions/0022-weight-model.md)
- Where editing goes: [Q-0010](/questions/resolved/0010-where-editing-goes.md)
