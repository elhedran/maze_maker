---
type: Decision Record
title: Slice 2.1 shows two measures of "a way onward" next to the maze
description: The first sample maze shows the share of walkable cells reachable from the centre and the number of separate walkable areas; the rest of Q-0008 waits for 2.5 and the game.
status: stable
decision_id: DR-0030
decision_status: accepted
decided_at: 2026-11-01T13:00:00+01:00
decision_kind: game-design
major: false
owners: [game-designer/1]
approvers: [architect/1, product-manager/1]   # architect/1 stands in for the author, game-designer/1 (DR-0018)
supersedes: []
tags: [generation, game-design, measures, slice-2-1]
generated:
  by: game-designer/1
  at: 2026-11-01T10:00:00+01:00
verified:
  - by: product-manager/1
    at: 2026-11-01T10:45:00+01:00
  - by: architect/1
    at: 2026-11-01T13:00:00+01:00
---

# Context

Slice 2.1 shows one generated maze, and the roadmap asks which [Q-0008](/questions/open/0008-global-maze-properties.md) measures sit next to it ([roadmap](/roadmap.md), 2.1, "Waits on" item 2). Q-0008's must-have is that the player always has a way onward. Marram (`product-manager/1`) suggested one cheap measure for it. Ian (`human:ian`): "make sure we bias towards executing planned work over re-planning or planning new work."

# Decision

Two measures, both from one flood fill over the generated W×H grid of (tile, orientation) cells ([DR-0015](/decisions/0015-tile-set.md)).

**Definitions**

- **Walkable cell:** any cell whose tile is not Hedge.
- **Open side:** any side that isn't Hedge (H): Passage, Door, Floor or Floor with a wall at one end.
- **Connected:** two cells sharing a side (N, E, S or W, not diagonal) are connected when both of their shared sides are open. Sides on the grid's border lead nowhere and are not followed.
- **Area:** a set of walkable cells connected to each other, directly or through other cells.
- **Centre:** the cell at column ⌊W/2⌋, row ⌊H/2⌋ (0-based, from the north-west). If it is Hedge, use the walkable cell nearest to it by Manhattan distance, ties broken by lowest row, then lowest column.

**Measure 1**

- Label: **Reachable from centre: 412 of 480 walkable cells (86%)**
- Explanation: "Share of walkable cells the player could walk to from the middle of the maze."
- The percentage is rounded to the nearest whole number.

**Measure 2**

- Label: **Separate areas: 5**
- Explanation: "Groups of walkable cells cut off from each other. 1 means everything joins up."

If the maze has no walkable cell, both read "No walkable cells".

# Alternatives considered

- **Zero measures until 2.5.** Acceptable, but the boxed-in worry is the one that matters most, and Ian's first look should show it.
- **Largest room, dead-end count or dead-end length.** They tune feel rather than catch the worst outcome. They wait for the full set at 2.5.

# Consequences

- One flood fill gives both numbers, so the cost is small.
- Rooms with no door count as their own areas, so Measure 2 also shows sealed rooms, which Ian allows for now (Q-0007).

# Concerns raised

Wren (`game-designer/1`): on a small finite grid the border cuts paths that would carry on in the endless maze, so these numbers read worse than the player would feel. Treat them as a signal for comparing seeds and weights, not a pass mark.

- **Yew (`architect/1`), approving in place of the author Wren (DR-0018):** cheap and fits [DR-0028](/decisions/0028-wfc-approach-for-2-1.md): one flood fill in `maze-core` over the generated cells. In a failed maze, `null` cells count as not walkable, and the measures are still shown.

# Links

- Global properties: [Q-0008](/questions/open/0008-global-maze-properties.md)
- Tile set and side kinds: [DR-0015](/decisions/0015-tile-set.md)
- Who designs the tile set: [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)
- Roadmap: [roadmap](/roadmap.md)
