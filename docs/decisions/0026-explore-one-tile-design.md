---
type: Decision Record
title: How exploring one tile works
description: One tile in the middle with a list of neighbours on each side, sorted by likelihood, facing sides on the shared edge, DR-0025's explanation panel, a step-to-centre action with a trail back, and one entry point that takes any tile and orientation.
status: stable
decision_id: DR-0026
decision_status: accepted
decided_at: 2026-10-31T20:15:00+01:00
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]
supersedes: []
tags: [tool, adjacency-table, explore-view, explanation, phase-1, slice-1-2]
generated:
  by: ux-designer/1
  at: 2026-10-31T20:00:00+01:00
verified:
  - by: product-manager/1
    at: 2026-10-31T20:15:00+01:00
---

# Context

Slice 1.2 ([roadmap](/roadmap.md)). Ian's demo tasks: name a corridor corner's most likely east neighbour; click from a corridor into a room and back out, through doorways. In the table Ian reads one cell; here he stands on a tile and asks "what's around me?", which is spatial, so the layout is a compass. Tiles and drawing: [DR-0015](/decisions/0015-tile-set.md), [DR-0017](/decisions/0017-tile-glyphs-and-colours.md). Numbers: [DR-0022](/decisions/0022-weight-model.md), shown as in [DR-0025](/decisions/0025-table-view-design.md).

# Decision

Damson (`ux-designer/1`). Numbers in the sketch are illustrative.

```
| [ Table ] [#Explore#]   tile set bar and [Reload tile set], as DR-0025      |               |
| [< Back]  Trail: ─ > └              Centre: [ └ Corridor corner, N+E  v ]   |               |
+-----------------------------------------------------------------------------+---------------+
|            NORTH of └  (% of └'s north neighbours)                          | DR-0025 panel |
|            │ Corridor straight, N-S   faces with Passage  31.0 [Go]         | for the       |
|            ┐ Corridor corner, S+W     faces with Passage  24.0 [Go]         | selected pair |
|            + 22 can't go here                Passage                        |               |
| WEST of └                       +-----------------------+  EAST of └        | └ then ┐ to   |
|  ░ Hedge        Hedge 48.0 [Go] | Hedge     └   Passage |  [┐ Corridor corner, S+W  Passage 15.0 [Go]]
|  ║ Room wall, E Hedge 20.0 [Go] +-----------------------+   ─ Corridor straight      Passage 12.0 [Go]
|  + 20 can't go here                      Hedge               ╢ Room doorway, door W   Door     6.2 [Go]
|            SOUTH of └  ...                                   + 21 can't go here |  [Explore ┐] |
```

1. **A second view.** Tabs "Table" and "Explore"; the tile set bar and Reload (DR-0025 point 12) are shared. Reload keeps centre, trail and selection; if the centre no longer exists, the view says so and opens the Centre picker.
2. **The centre** is the chosen (tile, orientation): DR-0017 glyph at 24px or more in a framed box, named in the Centre picker.
3. **Four arms** where the sides are: North above, East right, South below, West left, each headed "EAST of └ (% of └'s east neighbours)".
4. **Facing sides on the shared edge.** Each edge of the centre box shows the centre's side kind in DR-0015's words. Each neighbour row shows its side that faces back ("faces with Door").
5. **Neighbour rows:** glyph at 20px or more, name, facing side, share, [Go]. Share, shade steps, exception mark and tooltip are exactly the 1.1 cell's (DR-0025 points 5, 7, 8, 9). Sorted by share, highest first; ties keep the table's order.
6. **0s are hidden by default.** "+ 21 can't go here" at the foot of an arm opens that arm's 0s below a divider, blank as in the table and selectable.
7. **Select** a row by click: magenta outline, and DR-0025's panel, unchanged, explains the pair (centre as row tile, the arm's direction), including the clash line for a 0.
8. **Step:** [Go], double-click, or Enter on the selected row makes that neighbour the centre. The tile you came from is then selected in the opposite arm, marked "from", so the panel shows the edge you just crossed.
9. **Back:** [< Back] (and Alt+Left) returns to the previous centre and its selection. The trail shows past centres as glyphs; clicking one goes there and drops later ones. A long trail is cut at its oldest end with "...".
10. **Open from anywhere:** one entry point taking a (tile type, orientation) and, optionally, a direction and neighbour to select. It starts a new trail. Used by DR-0025's "Explore this tile" (its point 13: centre is the row tile, the pair stays selected), by the Centre picker (all 29 pairs in the table's order), and later by 2.2 from a spot in a maze.
11. **Back to the table:** the Table tab selects the cell for the centre and the selected neighbour in that direction.

May slip to a follow-up if they slow the slice: keyboard, the "from" mark, and point 11. Also, added on review by Marram (`product-manager/1`): clicking a past centre in the trail and cutting a long trail (point 9; [< Back] alone is the must-have), opening an arm's 0s (point 6; hiding them with the count stays), and Reload keeping centre, trail and selection (point 1).

The must-haves for the demo: the Explore tab, the Centre picker, four sorted arms with facing sides on the shared edge, shares and shade as in 1.1, select with DR-0025's panel, [Go], [< Back], and the one entry point (point 10).

**Same numbers, same source.** An arm is one row of the 1.1 table in one direction: the same entries, shares and explanations, from the same Rust table code; the frontend formats and never recalculates. Per arm Rust returns the centre's side kind and, per neighbour, entry, share, exception flag and facing side kind, sorted, with the 0s and their count. Whether that is the 1.1 view request for one row in four directions or its own command is Yew's call. Suggested test for "agrees with the 1.1 table": for all 29 pairs and four directions, each arm equals the table row's non-zero cells, share for share.

**The demo walk**, east all the way: ─ (Passage | Door) ╢ doorway, door W (Floor | Floor) · open room (Floor | Floor) ╟ doorway, door E (Door | Passage) ─. Four [Go]s out, Back or the trail home.

# Consequences

- Task one is a pick and a read; task two is four steps and a way home.
- 2.2 opens this view through point 10; showing how several neighbours combine is 2.2's addition.
- Arms are short now (about 8 non-zero neighbours per side); at full size (1.5) they may need a cap, which is 1.5's design.

# Concerns raised

- **Damson (`ux-designer/1`):** a pair's share differs by side (the entry is symmetric, row totals aren't), so after a step ┐ beside └ may read 15.0 one way and 9.4 the other, which can look like a bug. Point 8 shows the other side at once and the panel's "÷ row total" line explains it; review will tell if that's enough.

# Why it's a two-way door

One view in the tool, built on DR-0025's panel and the 1.1 view; changing it is frontend work and nothing saved depends on it.

# Links

- Roadmap, slice 1.2: [roadmap](/roadmap.md)
- Table view and panel: [DR-0025](/decisions/0025-table-view-design.md)
- Tile set: [DR-0015](/decisions/0015-tile-set.md); glyphs: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md); weight model: [DR-0022](/decisions/0022-weight-model.md)
