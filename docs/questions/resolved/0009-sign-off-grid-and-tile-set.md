---
type: Open Question
title: Square grid with four neighbours?
description: Approve DR-0014, a square grid with four neighbours and 90-degree rotations, which everything else is built on.
status: stable
question_id: Q-0009
question_status: resolved
waiting_on: [human:ian]
raised_by: facilitator/1
raised_at: 2026-10-31T15:16:00+01:00
resolved_by: human:ian
resolved_at: 2026-10-31T18:58:00+01:00
resolution_record: /decisions/0014-square-grid.md
related:
  - /decisions/0014-square-grid.md
  - /decisions/0015-tile-set.md
  - /decisions/0017-tile-glyphs-and-colours.md
tags: [grid, sign-off]
generated:
  by: facilitator/1
  at: 2026-10-31T15:16:00+01:00
---

# Question

Do you approve [DR-0014](/decisions/0014-square-grid.md): a square grid, four neighbours (N, E, S, W), no diagonals, and 90° rotations only?

# Context

- This is the one item from the tile-set work that Yew (`architect/1`) judges a one-way door. The side kinds, the tiles, the glyphs, the table, and soon the code and the game are all built on it. Changing it later means a new tile set and table.
- The tile set ([DR-0015](/decisions/0015-tile-set.md)) and how tiles are drawn ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md)) are two-way doors, tuned by review in the tool. The team approves those, and they don't need you.
- To see what the grid means in practice, look at the preview at `design/tile-set-preview.html` in the repository root.

# Options

- **Square, four neighbours, rotations only.** Favoured by Yew. Your tile words and the box-drawing glyphs are square-native, and four rotations keep the table small.
- **Hex, or diagonals.** More organic shapes, but a bigger table and no box-drawing glyphs. Nobody favours it.

# Recommendation

Square (`architect/1`). Wren (`game-designer/1`) needs no diagonal rules.

# Resolution

Ian approved DR-0014 on 2026-10-31, in conversation with the main facilitator session. His words, verbatim: "approve both, but neither decision should have been blocked on me" (both: DR-0014 and DR-0012).

His point: the grid was the team's call too. Decisions like it are made by the team, not brought to him.
