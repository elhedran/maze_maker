---
type: Open Question
title: Open details of the table's probabilities
description: Settle the points DR-0013 leaves open, about slight variation, paths meeting rooms, doors, and what "mainly" covers.
status: draft
question_id: Q-0007
question_status: open
waiting_on: [human:ian]
raised_by: facilitator/1
raised_at: 2026-10-31T15:10:00+01:00
related:
  - /decisions/0013-table-entries-from-tile-sides.md
  - /questions/resolved/0003-where-table-entries-come-from.md
  - /roadmap.md
tags: [adjacency-table, probabilities, tile-sides, phase-1]
generated:
  by: facilitator/1
  at: 2026-10-31T15:10:00+01:00
---

# Question

Ian's answer to Q-0003 is recorded in [DR-0013](/decisions/0013-table-entries-from-tile-sides.md). What do these parts of it mean?

1. "Slight variation in the probabilities": fixed in the tile set data, generated when the table is built, or different from place to place or maze to maze?
2. Can a path meet a room side that has no door, at a low probability, or is that 0?
3. "A room door ... could match a room": another room's door only, or any room edge? Can a door meet a door?
4. Entries come "mainly" from the tiles' sides. What else sets them?

The main facilitator session is putting all four to Ian together. This file tracks them; don't ask him separately.

# Context

- Only Ian's own words in DR-0013 are approved. These points need his reading before the weight model record and the tile set can rely on them.
- Points 2 and 3 shape the side types in the tile set (wall, path, room and door are proposed by Wren). Point 4 may mean the tile set format has to carry exceptions for particular pairs (Yew). Point 1 shapes the weight model and whether the table needs a seed.
- They block roadmap slice 1.1, through the weight model and the tile set.

# Options

**1. Slight variation**

- **Fixed, set by hand in the tile set data.** Favoured by Wren (`game-designer/1`) and Yew (`architect/1`). Pacing comes from choices someone made, the tool can explain every number, and the table needs no seed. Yew would set it per tile, not per pair, so nobody writes a thousand entries.
- **Generated when the table is built** (seeded noise on top of the side weights). Yew: if so, keep the seed in the tile set. Wren: it adds biases nobody chose, which are hard to tune or explain.
- **Different from place to place or maze to maze.** Wren: it would help discovery, but it's a separate idea from the table, and collapse already makes each maze different (Yew).

**2. Path meets a room side with no door**

- **Low probability.** It fits "more likely to match paths".
- **0.** It fits "rooms will need doors". Wren reads it this way.

**3. Door meets room**

- **Only another room's door.**
- **Any room edge.**
- Separately: **can a door meet a door?**

**4. "Mainly"**

- **Only the sides, for now.** "Mainly" leaves room for later changes.
- **Sides, plus exceptions for particular pairs.** The tile set format then has to carry them (Yew).
- **Sides, plus per-tile weights.** For example, crossroads and dead ends kept rare (see [Q-0004](/questions/open/0004-where-the-tile-set-comes-from.md)).

# Recommendation

On point 1, Wren and Yew recommend fixed values set by hand in the tile set data. No one has a recommendation on points 2 to 4. They are Ian's meaning to give.

# Resolution

_Open._
