---
type: Decision Record
title: Wren designs the tile set, with crossroads and dead ends as rare fallbacks
description: Wren designs the tile set from Ian's vocabulary, and crossroads and dead ends are in it at very low probability, to fix issues where nothing else fits.
status: stable
decision_id: DR-0016
decision_status: accepted
decision_kind: game-design
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T15:09:00+01:00
supersedes: []
tags: [tile-set, game-design, fallback-tiles, phase-1]
generated:
  by: facilitator/1
  at: 2026-10-31T15:13:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T15:09:00+01:00
  - by: human:ian
    at: 2026-10-31T15:17:00+01:00
---

# Context

[Q-0004](/questions/resolved/0004-where-the-tile-set-comes-from.md) asked where the tile set comes from. The options were: Wren designs it from Ian's suggested vocabulary ("tunnel, turn, t intersection, open room, corner room, etc"), the team starts from a tile set from Ian's earlier WFC work, or a mix. Marram (`product-manager/1`) recommended Wren designing it, or a mix if Ian had earlier material worth bringing in. The question described full set v1 as adding "crossroads, dead end, plain room edge, corner doorway and some variants". Ian answered in conversation with the main facilitator session on 2026-10-31.

# Decision

Ian (`human:ian`), verbatim:

> we will want cross-roads and dead ends - but both should be very low probability - they are more there to fix issues where nothing else fits. Same with the other "variations". other than that happy to go with the recommendation

He offered no earlier material, so the recommendation he accepted is option 1 of Q-0004:

- Wren (`game-designer/1`) designs the tile set, starting from Ian's vocabulary.
- After that, the tile set is a game-design matter. Wren and Marram approve it, and anything major comes to Ian.

He also named crossroads and dead ends: both are in the set at very low probability, and they are there more to fix issues where nothing else fits.

Asked about room walls, maze character and fallbacks ([Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) points 7 to 9), Ian (`human:ian`) said, verbatim, later on 2026-10-31:

> room walls are not rare - but go with what you think makes good for a nice maze here, the idea of the tool is to fix this by review, not perfect ahead of time. This answers 8 and 9 as well.

The same, adding nothing:

- Room walls are not rare.
- Beyond that, the team goes with what makes a nice maze, for room walls, maze character and fallbacks. The tool is for fixing the maze by review, not getting it perfect ahead of time.

Ian's approval covers only what is above, and "the team decides" rather than any particular answer. The team's choices are game design, approved by Wren and Marram ([DR-0002](/decisions/archive/0002-approval-and-escalation.md)). The tiles themselves are listed in DR-0015, which links here.

## Left to the team

- **Which tiles "the other "variations"" cover,** beyond crossroads and dead ends. Room walls are not among them. Wren (`game-designer/1`) also says hedge isn't a variation.
- **Maze character:** Wren recommends paths through solid hedge.
- **How fallbacks work:** rare but possible anywhere (a low tile weight), or a strict last resort used only when nothing else fits (a different mechanism, inside contradiction handling from [DR-0007](/decisions/0007-wave-function-collapse.md)). No team recommendation yet.

# Alternatives considered

- **Start from a tile set from Ian's earlier work.** It would build on something already tested, but there was no material to hand over.
- **A mix.** Wren's starter set first, folding in earlier material for full set v1. There was nothing to fold in.

# Consequences

- [Q-0004](/questions/resolved/0004-where-the-tile-set-comes-from.md) is resolved.
- Wren and Marram approve the starter set and full set v1 as game-design records, unless something in them is major ([DR-0002](/decisions/archive/0002-approval-and-escalation.md)).
- Crossroads and dead ends are in the set as rare fallbacks, not ordinary tiles. The tile set session decides whether they join the starter set or wait for full set v1.

# Concerns raised

None.

# AI guidance

- Keep crossroads and dead ends rare. Don't treat them as ordinary tiles.
- Room walls are ordinary pieces, not rare ones.
- Which other tiles are "variations", maze character and how fallbacks work are the team's calls, recorded in the tile set's game-design records. They don't need Ian's approval unless something is major.

# Links

- Question: [Q-0004](/questions/resolved/0004-where-the-tile-set-comes-from.md)
- Open details: [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md)
- Table entries: [DR-0013](/decisions/0013-table-entries-from-tile-sides.md)
- Roadmap: [roadmap](/roadmap.md)
