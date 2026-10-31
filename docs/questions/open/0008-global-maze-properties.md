---
type: Open Question
title: Which global properties must a maze have, and how are they enforced?
description: Local tile rules can't guarantee a way onward, doorways into rooms, room sizes or short dead ends; decide which matter and how to enforce them.
status: draft
question_id: Q-0008
question_status: open
waiting_on: [game-designer/1, architect/1]
raised_by: architect/1
raised_at: 2026-10-31T15:05:00+01:00
related:
  - /decisions/0007-wave-function-collapse.md
  - /decisions/0015-tile-set.md
tags: [generation, wfc, game-design]
generated:
  by: facilitator/1
  at: 2026-10-31T15:05:00+01:00
---

# Question

Which properties must every maze have that WFC's local rules can't guarantee, and how is each one enforced?

# Context

The tile set ([DR-0015](/decisions/0015-tile-set.md)) only constrains neighbouring edges. With its handed room edges and no inner-corner tiles, local rules give rectangular rooms of at least 2x2. They can't guarantee anything that isn't local: room size, a doorway into every room, short dead ends, or a maze where the player can always go on. Yew (`architect/1`) asked for this as its own question, and for it to stay out of the tile set: adding tiles to force room size or door count is how tables balloon. Wren (`game-designer/1`) agrees.

It doesn't block phase 1. Before slice 2.1 (the first sample maze), we need to choose which properties to measure. How they are enforced is needed only before the game.

# Options

What matters, from Wren (`game-designer/1`), in priority order:

1. **Must:** there is always a way onward; the player is never boxed into a finite pocket.
2. **Should:** any room the player can see has at least one doorway. **Deferred by Ian** on Q-0007 ("Lets allow for rooms we can't get into for now"). Revisit before the game.
3. **Tune:** rooms sized like places, not closets or fields. Nothing local stops a room growing without end.
4. **Tune:** dead ends stay short.

How to enforce them is open. Possible approaches include weights, checks with regeneration or repair after collapse, and constraints added to the generator. The choice touches contradiction handling and determinism, which are DR-0007 follow-ups.

# Recommendation

Wren (`game-designer/1`): phase 2 (slice 2.3) measures all four properties on sample mazes, and they are enforced before the game. Yew (`architect/1`) proposes how to enforce them. It touches contradiction handling and determinism, so it's likely major and goes to Ian.

# Parked

Parked on 2026-10-31. **Trigger: revisit before slice 2.1** (the first sample maze), when what to measure has to be chosen. Enforcement is still needed only before the game.

Ian (`human:ian`), 2026-10-31: "make sure we bias towards executing planned work over re-planning or planning new work." Q-0008 doesn't block Phase 1, so the facilitator stopped work on it. Wren and Yew were convened but stopped before either reached a position, so nothing new was settled.

# Resolution

**Partly settled, 2026-11-01.** What 2.1 measures is settled by [DR-0030](/decisions/0030-measures-for-2-1.md): the share of walkable cells reachable from the centre, and the number of separate walkable areas. Still open: the full set of measures (slice 2.5) and how properties are enforced (before the game).

