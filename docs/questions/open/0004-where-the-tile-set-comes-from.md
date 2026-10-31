---
type: Open Question
title: Where does the tile set come from?
description: Decide whether Wren designs the tile set from Ian's suggested vocabulary, or the team starts from a tile set from Ian's earlier work.
status: draft
question_id: Q-0004
question_status: open
waiting_on: [human:ian]
raised_by: product-manager/1
raised_at: 2026-10-31T15:05:00+01:00
related:
  - /roadmap.md
  - /decisions/0007-wave-function-collapse.md
  - /questions/open/0003-where-table-entries-come-from.md
tags: [tile-set, game-design, phase-1]
generated:
  by: product-manager/1
  at: 2026-10-31T15:05:00+01:00
---

# Question

Should Wren design the tile set, starting from the vocabulary you suggested? Or is there a tile set, or part of one, from your earlier work with WFC that we should start from?

# Context

- On 2026-10-31 you suggested tile types: "tunnel, turn, t intersection, open room, corner room, etc", with colour for room vs hallway ([roadmap](/roadmap.md)).
- Wren (`game-designer/1`) has turned that into a proposed starter set, pending this answer. It has six tiles on a square grid, and each side is a wall, a passage, or open to a room. That gives 19 rows:
  - tunnel (2 orientations), turn (4) and T-intersection (4), which are hallways;
  - open room (1), corner room (4) and doorway (4), which are rooms.

  The doorway is what connects rooms to hallways. Room vs hallway is a property of each tile.
- Full set v1 (roadmap slice 1.5) would add crossroads, dead end, plain room edge, corner doorway and some variants. Wren expects Phase 2 to change the set.
- [DR-0007](/decisions/0007-wave-function-collapse.md) mentions your past experience with this approach, which may have left material worth reusing.
- This question blocks slice 1.1 (the starter set) and slice 1.5 (the full set).
- It only asks where the tiles come from and who designs them. The file format is a separate record that Yew brings later.

# Options

- **1. Wren designs it from your vocabulary.** The starter set is as proposed above. After that, the tile set is a game-design matter: Wren and Marram approve it, and anything major comes to you.
  - Wren proposed it, and Marram favours it. It's ready now, and Wren owns the design of the tile set in Phase 1.
- **2. Start from your earlier tile set.** It builds on what you've already tested. You'd need to hand it over, and it may need converting.
- **3. A mix.** Wren's starter set for the first demo, with anything from your earlier work folded into full set v1 (slice 1.5).

# Recommendation

Marram (`product-manager/1`) recommends option 1, or option 3 if you have earlier material worth bringing in. The starter set gets the first demo out quickly, and lessons from your earlier work can shape full set v1.

# Resolution

_Open._
