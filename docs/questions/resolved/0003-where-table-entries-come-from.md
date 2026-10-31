---
type: Open Question
title: Where do the adjacency table's entries come from?
description: Choose how the table is produced, whether worked out from tile sides, written by hand, learned from example maps, or a mix.
status: stable
question_id: Q-0003
question_status: resolved
waiting_on: [human:ian]
raised_by: product-manager/1
raised_at: 2026-10-31T15:05:00+01:00
resolved_by: human:ian
resolved_at: 2026-10-31T15:05:00+01:00
resolution_record: /decisions/0013-table-entries-from-tile-sides.md
related:
  - /roadmap.md
  - /decisions/0007-wave-function-collapse.md
  - /decisions/0008-adjacency-tool-first.md
  - /questions/resolved/0004-where-the-tile-set-comes-from.md
  - /questions/open/0007-open-details-of-table-probabilities.md
tags: [adjacency-table, data-model, phase-1]
generated:
  by: product-manager/1
  at: 2026-10-31T15:05:00+01:00
---

# Question

How should the adjacency table's entries be produced? Should they be worked out from the tiles' sides, written by hand pair by pair, learned from example maps, or produced by a mix of these?

# Context

- The table says which tile, in which orientation, can sit next to which on each side, and possibly how likely each pairing is ([DR-0007](/decisions/0007-wave-function-collapse.md)). It has three axes: tile with orientation, side, and neighbour (Yew).
- Wren's proposed starter set has 19 rows. On a square grid with four sides (not yet decided), that's 19 × 4 × 19 = 1,444 entries before any tiles are added.
- Yew asks for this to be settled now, alongside the grid geometry record. The answer shapes the weights record and the tile set file format.
- It blocks roadmap slice 1.1, the first demo ([roadmap](/roadmap.md)).
- Phase 1 is view and filter only ([DR-0008](/decisions/0008-adjacency-tool-first.md)). If the table is written by hand, editing it in the tool becomes a scope question.
- It sets the data model and is hard to reverse, so it is major ([DR-0002](/decisions/0002-approval-and-escalation.md)).

# Options

- **1. Worked out from the tiles' sides.** Each side of a tile has a type. Wren's starter proposal uses wall, passage and room-open. Two tiles can be neighbours where their facing sides match, so the tool shows a table computed from the tiles.
  - For: little to write. The table is consistent by construction. The side types explain why an entry is allowed, which Wren values for the drawing.
  - Against: only "allowed or not" falls out. Likelihoods need another source (the weights record). Exceptions need special rules.
- **2. Written by hand, pair by pair.**
  - For: full control over every entry.
  - Against: well over a thousand entries even for the starter set, and easy to make mistakes such as one-way pairs. It would probably need editing in the tool, which is outside Phase 1.
- **3. Learned from example maps.** Draw some example mazes, count which tiles sit next to which, and turn the counts into likelihoods.
  - For: likelihoods come for free and reflect the mazes we want.
  - Against: the example maps, and a way to make them, are needed before the first demo. The table only knows pairings that appear in the examples.
- **4. A mix.** The sides decide what's allowed, and likelihoods come from elsewhere (set by hand per tile or per pair, or learned from examples), as the weights record decides.

Positions:

- **Yew (`architect/1`)** hasn't stated a preference between the options. Yew's point is the timing: decide this now, before the weights record and the tile set format.
- **Wren (`game-designer/1`)** proposed a starter set where each side has a type, and wants side types shown because they explain why an entry is allowed. That fits option 1 or 4. Separately, Wren will argue for weights in the weights record, because frequencies set a maze's pacing.

# Recommendation

Marram (`product-manager/1`) recommends option 1 for what's allowed, with likelihoods left to the weights record. That becomes option 4 if the weights record decides on weights. It's the quickest route to the first demo, it keeps Phase 1 view-only, and nobody has to check a thousand entries by hand. This is a delivery view, not an architecture one.

# Resolution

Ian answered on 2026-10-31, in conversation with the main facilitator session. His words, verbatim: "They will mainly come from the tiles sides, if it doesn't work or pair up the probability is 0. Paths are more likely to match paths, rooms other rooms. Rooms will need doors so that a room door is more likely to match a path but could match a room. For what is allowed, there should be slight variation in the probabilities."

Recorded as [DR-0013](/decisions/0013-table-entries-from-tile-sides.md). The points his answer leaves open are tracked in [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md).
