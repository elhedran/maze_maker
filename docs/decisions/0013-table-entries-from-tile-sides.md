---
type: Decision Record
title: Table entries come from tile sides, as probabilities
description: The adjacency table's entries are worked out mainly from the tiles' sides, with probability 0 where sides don't pair up and slightly varied probabilities for what is allowed.
status: stable
decision_id: DR-0013
decision_status: accepted
decision_kind: architecture
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T15:05:00+01:00
supersedes: []
tags: [adjacency-table, data-model, tile-sides, probabilities, phase-1]
generated:
  by: facilitator/1
  at: 2026-10-31T15:10:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T15:05:00+01:00
  - by: human:ian
    at: 2026-10-31T15:14:00+01:00
  - by: human:ian
    at: 2026-10-31T15:17:00+01:00
---

# Context

[Q-0003](/questions/resolved/0003-where-table-entries-come-from.md) asked where the adjacency table's entries come from: worked out from the tiles' sides, written by hand pair by pair, learned from example maps, or a mix. It sets the table's data model and blocks the first demo (roadmap slice 1.1). Ian answered it in conversation with the main facilitator session on 2026-10-31.

# Decision

Ian (`human:ian`), verbatim:

> They will mainly come from the tiles sides, if it doesn't work or pair up the probability is 0. Paths are more likely to match paths, rooms other rooms. Rooms will need doors so that a room door is more likely to match a path but could match a room. For what is allowed, there should be slight variation in the probabilities.

The same, point by point, adding nothing:

- The table's entries come mainly from the tiles' sides.
- Where two facing sides don't work or don't pair up, the probability is 0.
- Paths are more likely to match paths, and rooms other rooms.
- Rooms need doors, so that a room door is more likely to match a path but could match a room.
- For what is allowed, there is slight variation in the probabilities.

Yew asked what else, besides the sides, sets entries. Ian (`human:ian`), verbatim, later on 2026-10-31:

> For Yews additional question to Q-0003 - probabilities are mainly "if it can work" and "should we see it often in that place" - and some randomness for to add flavour. I'm expecting a big table.

Ian's answers to the open details ([Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md), points 1 to 9), verbatim, later on 2026-10-31:

> rooms can only join at a door. We will add some game logic on not "doubling up" the doors in these scenarios at play time, not map generation time. A path can only lead into a room via a door - the open side of a room (i.e. not the wall) can only be to other open room. 3. is a good question. Lets allow for rooms we can't get into for now. We will revisit if and how we fix this later. for 4. its more that paths are more likely to be near other paths. Rooms should be rare. Although I guess the closed side of a room is equally likely to be another room or a path.
>
> 5: I want to see the variation in the table of probabilities. I plan on having in future different "zones" with different probabilities.

The same, point by point, adding nothing:

- Rooms join only at a door. Not "doubling up" doors is game logic at play time, not at map generation.
- A path leads into a room only through a door. A room's open side (not its wall) faces only another room's open side.
- Rooms we can't get into are allowed for now. If and how to fix that is revisited later.
- Paths are more likely to be near other paths. Rooms should be rare. The closed side of a room is equally likely to face another room or a path.
- The variation shows in the table of probabilities. In future, different "zones" will have different probabilities.

His answers on room walls, maze character and fallbacks are in [DR-0016](/decisions/0016-wren-designs-the-tile-set.md).

Ian's approval covers only what is above. Everything below is either open or the team's reading.

## Not decided here

- **The weight model for "should we see it often in that place":** (a) weights set by hand on every pair, which would partly supersede this record and needs table editing, or (b) a computed base plus a few exceptions for particular pairs in the tile set data, which Yew recommends. Ian's future "zones" are new input. (Yew, Wren) _Facilitator addition: since decided by the team as (b), in [DR-0022](/decisions/0022-weight-model.md)._
- **Guaranteeing rooms a door:** deferred by Ian. It is a global property of the maze, which the tile set session tracks in Q-0008.
- **The numbers:** the actual probabilities aren't set here. Whoever proposes them brings them back to Ian.
- **The team's reading of point 5:** the main facilitator session reads "I want to see the variation in the table" as noise in the table's numbers, which Yew says must be reproducible, stored or seeded ([DR-0007](/decisions/0007-wave-function-collapse.md) determinism follow-up). This reading is not part of Ian's approval.

# Alternatives considered

From [Q-0003](/questions/resolved/0003-where-table-entries-come-from.md):

- **Written by hand, pair by pair.** Full control, but well over a thousand entries even for the starter set, easy to get wrong, and it would need editing in the tool.
- **Learned from example maps.** Probabilities come for free, but example maps are needed before the first demo, and the table only knows pairings that appear in them.

# Consequences

- The table is computed from the tile set, not written by hand. Whether the tool can edit probabilities is a separate question for Ian ([Q-0010](/questions/resolved/0010-where-editing-goes.md)); [DR-0008](/decisions/0008-adjacency-tool-first.md) keeps Phase 1 to view and filter.
- The table holds probabilities, with 0 where sides don't pair up. Yew (`architect/1`) reads this as settling "weights vs allowed/forbidden" from [DR-0007](/decisions/0007-wave-function-collapse.md) in favour of weights, so the roadmap gate becomes "the weight model". That record still decides:
  - per-pair weights, per-tile weights, or both;
  - where base weights live;
  - whether weights are the same in both directions;
  - how the weights of several collapsed neighbours combine.

  Propagation only needs "probability above 0", so the last point can wait for slice 2.1.
- The tile set's sides need kinds that tell paths, rooms and doors apart. The tile set session defines them (DR-0015) and builds on this record. Ian's answers fit the door kind Wren and Yew proposed: rooms join only at a door, and a path enters a room only through a door.

# Concerns raised

No disagreement. On "slight variation", Yew and Wren both favour values set by hand in the tile set data over generated noise. Neither position is part of Ian's approval.

- **Wren (`game-designer/1`):** base weights from the sides, with small deliberate changes on top, such as making long straight runs rarer than turns. Pacing should come from choices someone made, and collapse is already random. Seeded noise adds biases nobody chose, which are hard to tune or explain. Variation from place to place would help discovery, but it is a separate idea.
- **Yew (`architect/1`):** set per tile, not per pair, since per pair is hand-writing the table again. Each entry is a base weight for the two facing side types times a per-tile weight, so the tool can explain every number (slices 1.2 and 1.4), and the table needs no seed. If Ian does mean jitter, seed it when the table is built and keep the seed in the tile set. Either way the variation must be reproducible, stored or seeded, which ties into the determinism follow-up in [DR-0007](/decisions/0007-wave-function-collapse.md). Don't write that weights come only from the sides: Ian said "mainly". After Ian's answer on "mainly", Yew notes that a weight per tile can't express "in that place", and recommends a computed base plus a few exceptions for particular pairs.

# AI guidance

- Work entries out from side types. Don't hand-write the table.
- Probability 0 means the pair can't happen.
- Don't settle anything under "Not decided here" in passing. Quote Ian's words where they are all there is.

# Links

- Question: [Q-0003](/questions/resolved/0003-where-table-entries-come-from.md)
- Open details: [Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md)
- Generation approach: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Roadmap: [roadmap](/roadmap.md)
