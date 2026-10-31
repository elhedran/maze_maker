---
type: Decision Record
title: The weight model, a calculated base plus a few exceptions
description: Each table entry is a side-pair base times a weight per tile, with a few exceptions for particular pairs, all kept in the tile set data; side rules always win, and the tool shows where every number came from.
status: stable
decision_id: DR-0022
decision_status: accepted
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [architect/1, engineer-rowan/1]
decided_at: 2026-10-31T19:45:00+01:00
supersedes: []
tags: [adjacency-table, data-model, weights, probabilities, tile-set, phase-1]
generated:
  by: facilitator/1
  at: 2026-10-31T19:30:00+01:00
verified:
  - by: architect/1
    at: 2026-10-31T19:40:00+01:00
  - by: engineer-rowan/1
    at: 2026-10-31T19:45:00+01:00
---

# Context

[DR-0013](/decisions/0013-table-entries-from-tile-sides.md) says the adjacency table's entries come mainly from the tiles' sides, with 0 where sides don't pair up. Ian (`human:ian`) later said, verbatim:

> probabilities are mainly "if it can work" and "should we see it often in that place" - and some randomness for to add flavour. I'm expecting a big table.

And on randomness and zones:

> I want to see the variation in the table of probabilities. I plan on having in future different "zones" with different probabilities.

Point 10 of [Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md) asked how "should we see it often in that place" is expressed: (a) weights set by hand on every pair, or (b) a calculated base plus a few exceptions for particular pairs in the tile set data. Following [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md), it is the team's call: DR-0018 lists "probabilities, weights and the weight model" among the decisions that shouldn't have gone to Ian.

The tile set is [DR-0015](/decisions/0015-tile-set.md): 10 tiles, 29 (tile, orientation) pairs, five kinds of side. The table is about 30 by 30 now and millions of cells eventually ([Q-0006](/questions/resolved/0006-expected-table-size.md)). Editing in the tool waits for slice 2.3; until then Ian edits the tile set data and reloads it ([Q-0010](/questions/resolved/0010-where-editing-goes.md)).

# Decision

Yew (`architect/1`) decides option (b), as a two-way door. Rowan (`engineer-rowan/1`) approves, with the conditions below written in as part of the decision. Yew refined two of them and pinned exceptions as multipliers; Rowan agreed to those changes.

## The model

Each entry, for tile A beside tile B in a given direction:

> entry = side-pair base × weight of A × weight of B × exception multiplier (1 if no exception applies)

- **Side-pair base.** A number for each pair of side kinds that may meet (DR-0015). Pairs that may not meet are 0.
- **Weight per tile.** One number per tile, shared across its orientations.
- **Exceptions.** A few multipliers for particular pairs, for what a tile weight can't say: "in that place", such as long straights rarer than corners, or a dead end rarely beside a doorway.
- All of it lives in the tile set data. The table is calculated from it and never written by hand.

## Conditions (Rowan's, agreed by Yew)

1. **Side rules always win.** No tile weight, exception or zone can make a 0 entry non-zero. Tile set data that tries to is an error when it is loaded.
2. **Exceptions are keyed by tile type or tag on each side, plus the side kinds that meet.** Not by tile pair alone, and not by orientation or compass direction: the side kinds already tell apart a doorway beside a doorway along a wall from a doorway facing a doorway through the door. (Rowan proposed keying by direction; Yew changed it to side kinds, because exceptions ignore orientation.)
3. **Bad data is rejected with a clear error:** unknown tile or tag names, negative or non-finite numbers, and an exception that matches no allowed pair. Two exceptions matching the same pair is also an error for now, rather than a precedence rule.
4. **The same tile set data gives the same table**, checked by a test. If seeded noise is ever used, its seed is stored in the tile set data.
5. **The tool shows each entry's base, tile weights and the exceptions applied**, so a wrong number can be traced to its cause (roadmap slices 1.2 and 1.4).

## Defaults

- **Symmetric.** The entry for A beside B facing east equals the entry for B beside A facing west, and every exception applies both ways. Asymmetric entries aren't supported until review needs one. (Rowan)
- **Exceptions are multipliers**, never replacements, so a 0 stays 0 and condition 1 holds without extra code. (Yew)
- **Variation is set by hand** in the tile set data (Wren), not generated noise, until review shows a need.
- **Zones** come later, as extra sets of overrides for tile weights and exceptions over the same base. They never change the base or the side rules. Nothing is built for them until a zone is needed.

## Why it's a two-way door

Nothing is built on it yet: `maze-core` returns a version string, and there is no tile set format, table code, saved data or tool view (Rowan). Reversing it today means editing this record. Later, moving to (a) stays cheap, since (b) calculates a full table that can be exported once and edited by hand; going from (a) back to (b) would be the harder direction.

Reversing stops being cheap once any of these exists (Rowan): a tile set file with many hand-tuned exceptions and zones; saved data that depends on exact table values, such as seeds or sample mazes we promise to reproduce; or in-tool editing (slice 2.3) writing to this format. Before work like that builds on it, Yew checks again whether it's still a two-way door (DR-0018).

## Not decided here

- How the weights of several collapsed neighbours combine. Propagation only needs "above 0", so this can wait for slice 2.1 (DR-0013).
- The numbers themselves: the bases, tile weights and first exceptions. They are the team's to propose and tune by review.
- The file format of the tile set data, beyond what it must hold.

# Alternatives considered

- **(a) Weights set by hand on every pair.** Full control, but well over a thousand entries for the starter set and unworkable at millions of cells. It is the alternative DR-0013 rejected, and it needs editing in the tool, which waits for slice 2.3.
- **A weight per tile only, with no exceptions.** Covers "if it can work", "often" and "a big table", but can't express "in that place" (Yew).
- **Rules over tile types and tags as the whole model** (Wren's framing). Folded in: exceptions are keyed by tile type or tag, per condition 2.

# Consequences

- Slice 1.1 can define the tile set data and calculate the table from it.
- The tool's table view (1.2) and tracing (1.4) show where each number came from.
- Most tuning is a change to a tile weight or a few exceptions, which keeps editing the data by hand workable until 2.3.
- Invariants are checked when data loads, so a bad edit fails loudly instead of making a strange maze.

# Concerns raised

- **Marram (`product-manager/1`):** sees this as a one-way door, because it is the table's data model and the tile set format builds on it. Yew, who decides whether a technical decision is one-way (DR-0018), judged it two-way, and Rowan agreed. Yew can overrule Marram on architecture. Yew overrules Marram here; it was Yew's call. Rowan's answer to the concern: Marram is right that the format builds on this, so settle the format before slice 2.3; until then it is one file with one reader, no users and no saved data, which makes changing it a refactor. Ian also disagreed earlier with judging the weight model one-way (DR-0018).

# AI guidance

- Calculate the table from the tile set data. Never write table entries by hand.
- Never let a weight, exception or zone turn a 0 into a non-zero entry.
- Key exceptions by tile type or tag on each side plus the side kinds that meet, not by orientation or direction.
- Reject bad tile set data with a clear error; don't guess.
- Keep every entry explainable: base, tile weights, exceptions.

# Links

- Question: [Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md)
- Table entries: [DR-0013](/decisions/0013-table-entries-from-tile-sides.md)
- Tile set: [DR-0015](/decisions/0015-tile-set.md)
- One-way and two-way doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)
- Generation approach: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Roadmap: [roadmap](/roadmap.md)
