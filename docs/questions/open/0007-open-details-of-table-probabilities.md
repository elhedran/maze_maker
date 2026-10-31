---
type: Open Question
title: Open details of the table's probabilities and the tile set
description: Settle the points that DR-0013 (table entries) and DR-0016 (tile set) leave open, about doors, paths and rooms, variation, what else sets entries, room walls, maze character and fallbacks.
status: draft
question_id: Q-0007
question_status: open
waiting_on: [engineer-rowan/1, engineer-ash/1]
raised_by: facilitator/1
raised_at: 2026-10-31T15:10:00+01:00
related:
  - /decisions/0013-table-entries-from-tile-sides.md
  - /decisions/0016-wren-designs-the-tile-set.md
  - /questions/resolved/0003-where-table-entries-come-from.md
  - /questions/resolved/0004-where-the-tile-set-comes-from.md
  - /questions/resolved/0006-expected-table-size.md
  - /roadmap.md
tags: [adjacency-table, probabilities, tile-sides, tile-set, phase-1]
generated:
  by: facilitator/1
  at: 2026-10-31T15:10:00+01:00
---

# Question

Ian's answers to Q-0003 and Q-0004 are recorded in [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) and [DR-0016](/decisions/0016-wren-designs-the-tile-set.md). What do these parts of them mean? The numbers match the list the main facilitator session put to Ian. Points 5 and 10 came from Wren and Yew after his answer to point 6.

1. **Door meeting a room:** only another room's door, or any room edge? _Answered._
2. **Path meeting a room's open side:** allowed at a low weight, or only through a door? _Answered._
3. **"Rooms will need doors":** do rooms have door tiles available, or is every room guaranteed at least one door? _Answered: deferred._
4. **"Paths match paths, rooms match rooms":** by pairs of sides (what the tiles do now), or by kind of tile, which needs a new mechanism? _Answered._
5. **Randomness:** noise in the table's numbers, or just the random pick WFC already makes at each collapse? _Answered._
6. **Besides tile sides, what sets entries?** _Answered; see below._
7. **Room wall:** the ordinary, common piece of a room, or one of the rare "variations"? _Answered._
8. **Maze character:** paths through solid hedge, or a dense labyrinth with walls only between cells? _Answered: the team's call._
9. **Fallbacks:** rare but possible anywhere (a low tile weight), or a strict last resort used only when nothing else fits (a different mechanism, inside contradiction handling)? _Answered: the team's call._
10. **The weight model:** how is "should we see it often in that place" expressed? (a) weights set by hand on every pair, or (b) a computed base plus a few exceptions for particular pairs in the tile set data?

Ian has answered points 1 to 9 (see Context). Point 10 is still with him; the main facilitator session relays his answers, so don't ask him separately.

# Context

- Only Ian's own words in DR-0013 and DR-0016 are approved. These points need his reading before the weight model record and the tile set (DR-0015, from the tile set session) can rely on them.
- They block roadmap slice 1.1, through the weight model and the tile set.
- **Point 6 is answered.** Ian (`human:ian`), verbatim, 2026-10-31: "For Yews additional question to Q-0003 - probabilities are mainly "if it can work" and "should we see it often in that place" - and some randomness for to add flavour. I'm expecting a big table."

  So entries come from three things, in his words:
  - "if it can work";
  - "should we see it often in that place";
  - "some randomness ... to add flavour".

  "I'm expecting a big table" is partial input to [Q-0006](/questions/resolved/0006-expected-table-size.md), which stays open. Points 5 and 10 ask what his randomness and "in that place" mean.
- **Points 1 to 9 are answered.** Ian (`human:ian`), verbatim, 2026-10-31:

  > rooms can only join at a door. We will add some game logic on not "doubling up" the doors in these scenarios at play time, not map generation time. A path can only lead into a room via a door - the open side of a room (i.e. not the wall) can only be to other open room. 3. is a good question. Lets allow for rooms we can't get into for now. We will revisit if and how we fix this later. for 4. its more that paths are more likely to be near other paths. Rooms should be rare. Although I guess the closed side of a room is equally likely to be another room or a path.
  >
  > 5: I want to see the variation in the table of probabilities. I plan on having in future different "zones" with different probabilities.
  >
  > room walls are not rare - but go with what you think makes good for a nice maze here, the idea of the tool is to fix this by review, not perfect ahead of time. This answers 8 and 9 as well.

  Point by point, close to his words:
  1. Rooms join only at a door. Not "doubling up" doors is game logic at play time, not at map generation.
  2. A path leads into a room only through a door. A room's open side (not its wall) faces only another room's open side.
  3. No guarantee for now: rooms we can't get into are allowed. If and how to fix it is revisited later. This belongs with the tile set session's question on global maze properties (Q-0008).
  4. Paths are more likely near other paths. Rooms should be rare. The closed side of a room is equally likely to face another room or a path.
  5. Ian wants to see the variation in the table of probabilities. In future, different "zones" will have different probabilities. The main facilitator session reads this as noise in the table's numbers, which must be reproducible (Yew). Zones are new input for point 10.
  7, 8 and 9. Room walls are not rare. Beyond that, the team goes with what makes a nice maze, because the tool is for fixing the maze by review, not getting it perfect ahead of time. Wren's recommendations on 7 and 8 stand. The team decides 9. His approval covers "the team decides", not any particular answer, so those choices are approved as game design, by Wren and Marram.

  The main facilitator session reads "rooms should be rare" and "room walls are not rare" together: rooms are rare overall, and within a room the walls are the ordinary pieces.

# Options

1. **Door meeting a room.**
   - **Only another room's door.** Favoured by Wren (`game-designer/1`) and Yew (`architect/1`). A door meets a passage (likely) or another room's door (less likely), and never a hedge or a room floor, which keeps rooms rectangular.
   - **Any room edge.**
2. **Path meeting a room's open side.** Allowed at a low weight, or only through a door. No team recommendation.
3. **"Rooms will need doors."**
   - **Door tiles are available.** Side probabilities can express this.
   - **Every room is guaranteed a door.** That is a global property, which side probabilities alone can't deliver. Wren and Yew are opening a question on global properties.
4. **"Paths match paths, rooms match rooms."** By side pairs, or by tile kind, which needs a new mechanism. A room's outer wall and a corridor's closed side are both hedge, so tile kinds can't be told apart from sides today (Wren, Yew).
5. **Randomness** (Yew).
   - **Noise in the table's numbers.** It must be reproducible, stored or seeded, which ties into the determinism follow-up in [DR-0007](/decisions/0007-wave-function-collapse.md).
   - **Just WFC's random pick at each collapse.** This needs nothing new.

   Earlier, on "slight variation", Wren favoured fixed weights, tuned by hand and kept in the data.
6. **What else sets entries.** Wren and Yew proposed a weight per tile, shared across its orientations, and Yew wanted Ian to approve it explicitly. Ian's answer is under Context; see also point 10.
7. **Room wall.** Wren reads it as the ordinary, common piece of a room, not a rare variation, and says hedge isn't a variation at all.
8. **Maze character.** Wren recommends paths through solid hedge.
9. **Fallbacks.** Rare but possible anywhere, or a strict last resort. No team recommendation.
10. **The weight model.** Yew's model is: entry = side-pair base × a weight per tile. That covers "if it can work", "often" and "a big table". It can't express "in that place", meaning preferences between particular pairs of tiles: long straights rarer than corners, say, or a dead end rarely beside a doorway.
    - **(a) Weights set by hand on every pair.** This is the alternative DR-0013 rejected. Choosing it would partly supersede DR-0013, and it needs table editing, which [DR-0008](/decisions/0008-adjacency-tool-first.md) leaves out of Phase 1.
    - **(b) A computed base, plus a few exceptions for particular pairs in the tile set data.** Recommended by Yew (`architect/1`). He would change his mind if Ian expects to tune most pairs himself.
    - **Wren (`game-designer/1`):** "in that place" is where the maze's rhythm comes from: junctions spaced apart, doorways approached by a straight, rooms not back to back. Wren would write these as a few rules over tile types and tags.
    - Wren's and Yew's first reading, before they checked, leaned towards a weight for each pair held in the table.

# Recommendation

On point 10, Yew (`architect/1`) recommends (b): a computed base plus a few exceptions for particular pairs in the tile set data. Ian's plan for "zones" with different probabilities is new input to weigh against it.

# Resolution

_Open, now with the team._ Following Ian's direction on 2026-10-31 that the team makes two-way-door decisions itself ("too many details in the maze came to me and the intent of the tool was to be able to defer those. The whole team needs to learn about two way door decisions (they make) versus one way door decisions (I make)."), point 10 no longer goes to Ian.

- **Yew (`architect/1`):** two-way, because the table is calculated from the tile set data and nothing saved depends on it yet. Yew decides (b): a calculated base plus a few exceptions for particular pairs. The tool shows each final weight and where it came from. Zones come later as extra sets of weights over the same base. At millions of cells (Q-0006), setting every pair by hand isn't workable.
- **Marram (`product-manager/1`):** sees it as one-way, because it is the table's data model and the tile set format builds on it.
- Yew can overrule Marram on architecture ([DR-0002](/decisions/archive/0002-approval-and-escalation.md)). As an architecture decision it still needs one engineer's approval. Once an engineer agrees, it gets its own record and this question is resolved. If the engineer agrees with Marram that it's one-way, it goes back to Ian.
