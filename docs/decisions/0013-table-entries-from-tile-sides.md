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

Ian's approval covers only what is above. Everything below is either open or the team's reading.

## Not decided here

These go back to Ian. The main facilitator session is putting them to him together; [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) tracks them.

- **"Slight variation":** fixed in the tile set data, generated when the table is built, or different from place to place or maze to maze.
- **Paths and rooms:** can a path meet a room side that has no door, at a low probability, or is that 0? (Wren)
- **"Rooms will need doors":** are door tiles available to rooms, or is every room guaranteed at least one door? A guarantee is a property of the whole room, which side probabilities alone can't deliver. (Wren, Yew)
- **"Paths are more likely to match paths, rooms other rooms":** pairs of sides (passage to passage, floor to floor), or kinds of tile (rooms tend to sit beside rooms)? Kinds of tile can't be expressed from sides as proposed so far, because a room's outer wall and a hallway's closed side are the same kind of side. (Wren, Yew)
- **"Could match a room":** another room's door only, or any room edge? Wren and Yew both read it as door to door. (Wren, Yew, and the main facilitator session)
- **"Mainly":** what else sets entries besides the sides? If it means exceptions for particular pairs, the tile set format has to carry them. (Yew)
- **The numbers:** the actual probabilities aren't set here. Whoever proposes them brings them back to Ian.

# Alternatives considered

From [Q-0003](/questions/resolved/0003-where-table-entries-come-from.md):

- **Written by hand, pair by pair.** Full control, but well over a thousand entries even for the starter set, easy to get wrong, and it would need editing in the tool.
- **Learned from example maps.** Probabilities come for free, but example maps are needed before the first demo, and the table only knows pairings that appear in them.

# Consequences

- The table is computed from the tile set, not written by hand. Phase 1 stays view and filter, with no editing in the tool ([DR-0008](/decisions/0008-adjacency-tool-first.md)).
- The table holds probabilities, with 0 where sides don't pair up. Yew (`architect/1`) reads this as settling "weights vs allowed/forbidden" from [DR-0007](/decisions/0007-wave-function-collapse.md) in favour of weights, so the roadmap gate becomes "the weight model". That record still decides:
  - per-pair weights, per-tile weights, or both;
  - where base weights live;
  - whether weights are the same in both directions;
  - how the weights of several collapsed neighbours combine.

  Propagation only needs "probability above 0", so the last point can wait for slice 2.1.
- The tile set's sides need kinds that tell paths, rooms and doors apart. The tile set session defines them and builds on this record. Wren and Yew propose that a doorway's outer side is its own kind, Door. It meets a passage (likely) or another room's door (less likely), and never a hedge or a room floor, which keeps rooms rectangular. That is their proposal, not part of Ian's approval.

# Concerns raised

No disagreement. On "slight variation", Yew and Wren both favour values set by hand in the tile set data over generated noise. Neither position is part of Ian's approval.

- **Wren (`game-designer/1`):** base weights from the sides, with small deliberate changes on top, such as making long straight runs rarer than turns. Pacing should come from choices someone made, and collapse is already random. Seeded noise adds biases nobody chose, which are hard to tune or explain. Variation from place to place would help discovery, but it is a separate idea.
- **Yew (`architect/1`):** set per tile, not per pair, since per pair is hand-writing the table again. Each entry is a base weight for the two facing side types times a per-tile weight, so the tool can explain every number (slices 1.2 and 1.4), and the table needs no seed. If Ian does mean jitter, seed it when the table is built and keep the seed in the tile set. Either way the variation must be reproducible, stored or seeded, which ties into the determinism follow-up in [DR-0007](/decisions/0007-wave-function-collapse.md). Don't write that weights come only from the sides: Ian said "mainly".

# AI guidance

- Work entries out from side types. Don't hand-write the table.
- Probability 0 means the pair can't happen.
- Don't settle anything under "Not decided here" in passing. Quote Ian's words where they are all there is.

# Links

- Question: [Q-0003](/questions/resolved/0003-where-table-entries-come-from.md)
- Open details: [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md)
- Generation approach: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Roadmap: [roadmap](/roadmap.md)
