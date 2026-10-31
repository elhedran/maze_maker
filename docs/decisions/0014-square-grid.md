---
type: Decision Record
title: Square grid with four neighbours
description: The maze is a square grid; each cell has four neighbours (north, east, south, west) and tiles turn in 90-degree steps.
status: stable
decision_id: DR-0014
decision_status: accepted
decision_kind: architecture
major: true
owners: [architect/1]
approvers: [architect/1, human:ian]
decided_at: 2026-10-31T19:00:00+01:00
supersedes: []
tags: [generation, wfc, grid, adjacency-table]
generated:
  by: facilitator/1
  at: 2026-10-31T15:05:00+01:00
verified:
  - by: engineer-ash/1
    at: 2026-10-31T16:30:00+01:00
  - by: architect/1
    at: 2026-10-31T19:00:00+01:00
  - by: human:ian
    at: 2026-10-31T18:58:00+01:00
---

# Context

[DR-0007](/decisions/0007-wave-function-collapse.md) leaves the grid shape open, as a follow-up that needs its own record. The tile set ([DR-0015](/decisions/0015-tile-set.md)) can't be defined without it, so this record goes to Ian alongside it.

Ian's tile vocabulary (tunnel, turn, T intersection, open room, corner room) and the Unicode box-drawing glyphs he suggested for the tool are both square-native.

# Decision

- The maze is a **square grid**.
- Each cell has **four neighbours**: north, east, south and west. There is no diagonal adjacency, so the adjacency table only ever relates a cell to those four.
- A tile's **orientations are 90-degree rotations**: at most four per tile. Rotations only, no mirror images.

This record doesn't settle how a side or direction is represented in the table, or which way rotation 0 faces in the data. Those stay with the DR-0007 follow-ups. Records may use "north up, rotating clockwise" as plain description.

# Alternatives considered

- **Hex grid.** Six neighbours give more organic layouts, but up to six orientations per tile make the table bigger, and the box-drawing glyphs and Ian's tile names don't fit it.
- **Square grid with diagonal adjacency** (eight neighbours). Allows diagonal constraints, but doubles the columns to check per cell. Wren (game designer) needs no diagonal rules.
- **Mirrored orientations as well as rotations.** Up to eight orientations per tile. No tile in the core set needs mirroring. A later tile that does (for example a corner doorway) can be added as two base tiles.

# Consequences

- At most four orientations per tile, which keeps the table small enough to read in the tool.
- The tool's glyphs can be box-drawing and block characters, one per (tile, orientation).
- Paths connect in four directions, as in traditional hedge mazes. How the player moves is a game-design matter for later.
- Changing the grid later means a new tile set and table, so this is hard to reverse.

# Concerns raised

None.

# AI guidance

- Use north, east, south and west for the four sides. Don't add diagonal neighbours or mirrored tiles without a superseding record.
- Don't treat "north up, clockwise" in the docs as a data convention. How the data numbers sides and rotations is still open.

# Links

- Generation approach: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Tile set that depends on this: [DR-0015](/decisions/0015-tile-set.md)
