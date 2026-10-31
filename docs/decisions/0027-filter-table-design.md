---
type: Decision Record
title: How filtering the table works
description: A filter bar above the grid with one group for the tile (rows) and one for its neighbour (columns), each filtering by kind, tile type, tag and the side on the shared edge; OR within a facet, AND across facets; a "showing N of 29" count, one Clear all, and filtering done in Rust through DR-0024's view request.
status: stable
decision_id: DR-0027
decision_status: accepted
decided_at: 2026-11-01T10:30:00+01:00
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [product-manager/1]
supersedes: []
tags: [tool, adjacency-table, table-view, filters, phase-1, slice-1-3]
generated:
  by: ux-designer/1
  at: 2026-11-01T10:00:00+01:00
verified:
  - by: product-manager/1
    at: 2026-11-01T10:30:00+01:00
---

# Context

Slice 1.3 ([roadmap](/roadmap.md)). Ian's demo task: show only pairs of a room and a corridor, and find the least likely one. Ian thinks in his own words, "room, corridor, corridor corner" (Q-0005), and in two halves: what the tile is, and what sits next to it. So the bar is split the same way the 1.1 grid already reads: tile (rows), then neighbour (columns). Tiles, kinds, tags and side kinds: [DR-0015](/decisions/0015-tile-set.md). The grid and panel: [DR-0025](/decisions/0025-table-view-design.md), whose point 13 puts filters in a bar above the grid and in the view request. The request already has `rowFilter`, `columnFilter` and `directions` ([DR-0024](/decisions/0024-tile-set-file-format.md)); this record uses them and adds one field.

# Decision

Damson (`ux-designer/1`); brief and drafting with the facilitator (`facilitator/1`). Numbers in the sketch are illustrative.

```
| Tile set bar and [Reload tile set], as DR-0025                                        |
| Neighbour to the:  [ North ] [#East#] [ South ] [ West ]          shade legend        |
+---------------------------------------------------------------------------------------+
| TILE (rows)      Kind [ Corridor ] [#Room#] [ Hedge ]   Type [ any            v ]     |
|                  Tag  [ junction ] [ fallback ] [ room edge ]                         |
|                  East side  [ Hedge ] [ Passage ] [ Door ] [ Floor ] [ Floor+wall ]   |
| NEIGHBOUR        Kind [#Corridor#] [ Room ] [ Hedge ]   Type [ any            v ]     |
| (columns)        Tag  [ junction ] [ fallback ] [ room edge ]                         |
|                  West side  [ Hedge ] [ Passage ] [ Door ] [ Floor ] [ Floor+wall ]   |
| Showing 13 of 29 rows, 15 of 29 columns.  Rows: Room. Columns: Corridor.  [Clear all] |
+------------------------------------------------------------+--------------------------+
| ROOM tile, then its CORRIDOR neighbour to the EAST (% of row) | DR-0025 panel          |
|            | Corridor │ ─ └ ┌ ┐ ...                       |                          |
| Room     · |                                               |                          |
```

1. **Two groups, tile and neighbour,** each with the same four facets. The groups are labelled "Tile (rows)" and "Neighbour (columns)", and the grid's corner sentence picks up the filter: "Room tile, then its Corridor neighbour to the EAST".
2. **Facets:**
   - **Kind:** toggles Corridor, Room, Hedge.
   - **Tile type:** a multi-select of DR-0015's ten names, grouped by kind. Tile type, not orientation: "Corridor corner" means all four.
   - **Tag:** toggles junction, fallback, room edge.
   - **Side on the shared edge:** toggles Hedge, Passage, Door, Floor, Floor+wall (any wall end, as DR-0024's exceptions match it). For the tile it is its side in the chosen direction ("East side"); for the neighbour, its side facing back ("West side"). The labels follow the switcher. This is how "rows that show a door to the east" is asked.
3. **Direction** is the 1.1 switcher, already above the grid; it is not repeated in the bar.
4. **How values combine:** within a facet, any (OR): Corridor + Room means either. Across facets, all (AND): Room + room edge means room tiles tagged room edge. Tile and neighbour are independent: the tile group picks rows, the neighbour group picks columns. An empty facet means any. A combination that matches nothing is allowed and shows the empty state (point 7).
5. **What's filtered out stays visible.** A line under the bar, always shown: "Showing 13 of 29 rows, 15 of 29 columns", then the active filters in words ("Rows: Room. Columns: Corridor, West side Passage."). With no filters: "Showing all 29 rows and 29 columns." Active toggles are drawn pressed (bold, filled grey; no new colours, per [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)). Kind dividers and labels stay for the kinds still shown.
6. **One action clears all:** [Clear all] on the count line resets both groups. It is disabled when nothing is filtered.
7. **Empty result:** the grid is replaced by "No tiles match these filters", the count line still reads "Showing 0 of 29 rows", and [Clear all] stays where it is.
8. **Selection:** if the selected cell is filtered out, the selection clears and the panel shows its "Select a cell" text. Filters are kept across a direction switch and a Reload; they are not saved between runs. Explore (DR-0026) is not filtered; "Explore this tile" still works from a filtered grid.

May slip if it slows the slice: the Tag and Side facets (Kind and Tile type are the must-haves for the demo, with the count line and Clear all), and the active filters in words (the counts stay).

**Rust does the filtering, through DR-0024's request.** Each of `rowFilter` and `columnFilter` already has `kinds`, `tileIds` (tile type ids) and `tags`, empty means any, and a tile must match every non-empty field, which is point 4 as written; any value in a field matches. This record adds one field to each:

- **`sides`**: side kinds, `hedge | passage | door | floor | floor_wall` (`floor_wall` matches any wall end). For `rowFilter`, a row matches if its side in the requested direction is one of them; for `columnFilter`, a column matches if its side facing back is. Because the match depends on direction, a request with a non-empty `sides` must name exactly one direction in `directions`, or Rust returns a `ToolError`.
- **From 1.3 the table asks for the switcher's direction only**, so `directions` holds one value. At 29 × 29 a request per switch is cheap. (Asking for all four when no side filter is set is the engineers' call.)
- **Counts:** the response already has `totalRows` and `totalColumns` after filtering, and `tiles` lists every pair, so "N of 29" needs no new field. Probabilities don't change under filtering: `rowTotal` stays over the whole set (DR-0024).
- **Empty result:** `totalRows` or `totalColumns` of 0, empty `rows`, `columns` and `entries`; not an error.

**The demo:** Tile Kind [Room], Neighbour Kind [Corridor]. The grid shows 13 room rows by 15 corridor columns. Look for the lightest non-blank shade in this direction, hover or click to read it, then switch N, S, W and compare; the panel says why it's low. For corridor-then-room, swap the two kinds. Five or six clicks plus scanning.

# Consequences

- Ian narrows the table in his own words, and always sees how much he's hiding.
- 1.5's paging uses the same `rows` and `columns` windows over the filtered set; nothing here changes that.
- Finding "the least likely" is still a scan across four directions; at 1.3's size that's fine.

# Concerns raised

- **Damson (`ux-designer/1`):** "pairs of a room and a corridor" includes both orders, and the shares differ by side (DR-0026's concern). The demo needs two passes, room-then-corridor and corridor-then-room, and four directions each. A "swap tile and neighbour" button or a "least likely shown" readout would make it one step; I've left both out to keep the slice small. Review will tell if scanning is enough.
- **Damson (`ux-designer/1`):** the `sides` field is an addition to DR-0024's request. It is additive and small, but it is Yew's record; Yew should confirm it, or move the side facet to a follow-up and ship 1.3 on DR-0024's fields as they stand.
  - **Yew (`architect/1`), 2026-11-01:** accepts `sides` as written. It only adds to DR-0024, and the one-direction rule fails loudly. DR-0024 stays as it is; this record is enough. For the engineers' review: the column check compares the neighbour's side in the opposite direction, so it needs a test that checks one pair both ways round.

# Why it's a two-way door

Every point is a two-way door: a bar in one view, one additive request field, and nothing saved depends on it. Changing it is frontend work and a small Rust filter change.

# Links

- Roadmap, slice 1.3: [roadmap](/roadmap.md)
- View request: [DR-0024](/decisions/0024-tile-set-file-format.md)
- Table view and panel: [DR-0025](/decisions/0025-table-view-design.md); explore view: [DR-0026](/decisions/0026-explore-one-tile-design.md)
- Tile set: [DR-0015](/decisions/0015-tile-set.md); glyphs and colours: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
