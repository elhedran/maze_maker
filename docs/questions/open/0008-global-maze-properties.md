---
type: Open Question
title: Which global properties must a maze have, and how are they enforced?
description: Local tile rules can't guarantee a way onward, doorways into rooms, room sizes or short dead ends; decide which matter and how to enforce them. Ian has steered enforcement - grow from open endpoints along the player's path, and bias towards an opening maze.
status: draft
question_id: Q-0008
question_status: open
waiting_on: [game-designer/1, architect/1]
raised_by: architect/1
raised_at: 2026-10-31T15:05:00+01:00
related:
  - /decisions/0007-wave-function-collapse.md
  - /decisions/0015-tile-set.md
  - /decisions/0028-wfc-approach-for-2-1.md
  - /decisions/archive/0031-endless-generation-approach.md
  - /plans/2.1-maze-review.md
  - /roadmap.md
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

# Ian's input on enforcement, 2026-11-01

Added by Marram (`product-manager/1`) at 2026-11-01T08:14:41Z.

Ian (`human:ian`), verbatim, given as steering for the product manager for future phases:

> Ok, on that not I have some future steering for the product manager for future phases around improving the mazes.  I'd like two changes - the first is we only generate based of "open" endpoints, endpoints connected to existing maze.  This basically means looking for where a path ends, a room door opens etc rather than just every space next to where the player has been, within the bounds of the player space radius.  The second change is to add some pressure to bias towards an opening maze.  that means that if a piece that goes down would only connect to already closed maze, that should get a reduced probability.

Then Ian (`human:ian`), verbatim, on [DR-0031](/decisions/archive/0031-endless-generation-approach.md):

> In regards to DR-0031 the world needs to be stored anyway - we want to know where the player has been, we will add loot, and hence we need to know what is looted, etc.  However given how this works, there should be quite an efficient in-memory data structure for this (sparse big table).

And Ian (`human:ian`), verbatim, on what he meant:

> and yes I mean growth follows the players path

**What it means for this question** (Marram):

- **Change 2 is enforcement pressure for the must-have, "a way onward".** It is the "weights" route in the Options above: a bias, not a check.
- **Change 1 is enforcement by construction.** If the maze only grows across open sides, everything generated is joined to where the player is. The pockets and separate areas Wren found in 2.1 (8–88% reachable, 5–19 separate areas; [review](/plans/2.1-maze-review.md)) can't form.

**The facilitator's reading, as given to Ian.** It is a proposal from `facilitator/1`, not Ian's words:

1. **Grow from open endpoints:** WFC only extends the maze beside an open side (a passage end, a door) of something already generated, rather than filling every cell in the player's radius. What's never reached stays solid.
2. **Opening pressure:** when a cell collapses, a candidate whose open sides all face already-settled cells, so that it adds no new open endpoint, gets a multiplier below 1. It would be one more factor in [DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 3's score, so 2.2's "why" trace would show it.
3. **Considered and not taken: grow each chunk from its gates.** The facilitator had proposed this so that the world stays order-independent and needn't be stored, as DR-0031 requires. Ian's follow-ups rule it out: growth follows the player's path, and the world is stored anyway. Ian's decision that the world is stored is being recorded as DR-0035 (drafted by the facilitator, pending Ian's approval of its wording). Yew is re-checking DR-0031 and is likely to supersede it (expected as DR-0036).

**Marram's notes for whoever picks it up.** These are proposals, not Ian's words:

- Keep the pressure above 0, so it biases and never rules a tile out. Then DR-0028 rule 1's support is untouched, and so is any fallback proof that rests on it.
- The pressure breaks DR-0028 rule 3's promise that, with one settled neighbour, the draw equals that neighbour's row in the 1.1 table. So the trace should show it as its own factor, and [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)'s "weight × factors equals the solver's score" should include it.
- With growth along the player's path, pressure lowers the chance that every open endpoint closes, but doesn't rule it out. The must-have still needs a guarantee, as DR-0031's gates gave one. Yew and Wren settle what replaces it.
- If growth only crosses open sides, every room that gets generated was entered through a doorway. So change 1 may also settle the deferred "should", a doorway into every room. Wren checks.

**Who settles the how, and when.** Yew (`architect/1`) settles how each change fits DR-0028 and the record that replaces DR-0031, and whether any part is a one-way door; if one is, it goes to Ian. Wren (`game-designer/1`) settles what it does for the player, and the tuning. Nothing is decided now. In the [roadmap](/roadmap.md) ("Steering 2026-11-01"):

- change 2 is picked up in slice 2.3; **revisit before 2.3's build starts**;
- change 1 is the core of 2.6's new shape; **revisit when Yew's record replacing DR-0031 is accepted**.

# Resolution

**Partly settled, 2026-11-01.** What 2.1 measures is settled by [DR-0030](/decisions/0030-measures-for-2-1.md): the share of walkable cells reachable from the centre, and the number of separate walkable areas. Still open: the full set of measures (slice 2.5) and how properties are enforced. For enforcement, Ian has steered two changes (above): opening pressure, picked up in 2.3, and growing from open endpoints along the player's path, picked up with 2.6's new shape. The rest of enforcement is still needed only before the game.

