---
type: Decision Record
title: The world is stored, and grows along the player's path
description: Ian's decision that the endless world is kept rather than regenerated, because where the player has been, loot and what is looted all need it; it is held in an efficient sparse in-memory table, and growth follows the player's path. How it grows is DR-0036.
status: draft
decision_id: DR-0035
decision_status: proposed
decision_kind: architecture
major: true
owners: [human:ian]
approvers: [human:ian, engineer-rowan/1]   # Ian's decision; architect/1 drafted it, so one engineer approves (DR-0018)
supersedes: []
tags: [infinite-world, persistence, storage, determinism, game, slice-2-6]
generated:
  by: architect/1
  at: 2026-11-01T08:24:59Z
verified:
  - by: human:ian
    at: 2026-11-01T08:33:03Z
    note: "Approved in conversation, verbatim: \"I approve DR-0035\". No proposal struck."
---

# Context

[DR-0007](/decisions/0007-wave-function-collapse.md) left open whether "the world is kept when the player walks back". [DR-0031](/decisions/archive/0031-endless-generation-approach.md) rule 7 answered no: a chunk was a pure function of its coordinates, so nothing generated needed keeping. The [roadmap](/roadmap.md) lists "persistence, meaning whether the world is kept when the player walks back" as a gate for the game.

On 2026-11-01 Ian (`human:ian`) steered how mazes grow (quoted in [DR-0036](/decisions/0036-growth-from-open-endpoints.md)), and then said, verbatim:

> In regards to DR-0031 the world needs to be stored anyway - we want to know where the player has been, we will add loot, and hence we need to know what is looted, etc.  However given how this works, there should be quite an efficient in-memory data structure for this (sparse big table).

And, verbatim, when asked whether growth depends on where the player walks:

> and yes I mean growth follows the players path

# Decision

Ian's words above are the decision:

1. **The world is stored.** What is generated is kept, along with where the player has been and, later, loot and what is looted.
2. **Growth follows the player's path.** So the world depends on where the player walked and can't be regenerated from a seed alone. That is why it has to be stored.
3. **The store is an efficient in-memory sparse table.**

## Yew's proposals, beyond Ian's words

These are `architect/1`'s, not Ian's. Strike any of them.

- **The table's shape.** Fixed-size square pages (32 × 32 cells to start, an engineers' setting) keyed by `i64` page coordinates in a sorted map, so iteration order is always the same ([DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 5). Each page is a dense array. A million stored cells takes a few megabytes.
- **What a cell holds.** Its tile, as an index into the world's palette of (tile id, orientation) pairs. Not a table index: table order shifts when the tile set changes. Cells that haven't been generated hold nothing. Per-cell game state (visited, loot, looted) sits beside the tile and comes with the game's records.
- **Kept beside the table.** The frontier ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)) and the **growth log**: the growth steps in order. With the world seed, tile set and generator version, the log replays the world exactly. Tests, the tool and bug reports use it.
- **Where it lives.** In `maze-core`, with no UI, shared by the tool and the game ([DR-0012](/decisions/0012-code-layout-and-core-crate.md)). Game-only state can live in the game app, keyed the same way.
- **Saving to disk.** Not decided here; the game's own record decides it. My proposal: a save holds the pages, the palette, the world seed, the frontier and the player's state. The tile set fingerprint and generator version are kept for information only, because saved cells stay as saved and only new growth uses the current set.

## Why it's a one-way door

Today, reversing it costs little, because nothing is built on it yet. But the game, its saves and its loot are all meant to build on it. A world grown along the player's path can't be regenerated, so going back to a world that is a function of its coordinates later would mean rebuilding generation, the save format and everything keyed to stored cells. It is Ian's own call.

It also makes something cheaper. Once saves hold cells and not a recipe, changing weights or the generator no longer changes a saved world. DR-0031 said its rules would become a one-way door at the first saved world. Under this record, generation rules stay two-way. The one-way part moves to the save format, the palette's tile ids, and DR-0036's frontier rule (rule 1), because new code has to be able to grow from an old save's frontier.

## The persistence gate

This settles the roadmap's game gate, "persistence, meaning whether the world is kept when the player walks back": yes, it is kept. What's left is how the game writes saves to disk. That belongs in its own record when the game starts, and it isn't a gate on 2.6. Marram rewords the gate.

# Alternatives considered

- **Nothing stored** (DR-0031): a world that is a pure function of coordinates, plus a cache. Ian: the world "needs to be stored anyway". And growth along the player's path can't be regenerated.
- **Store only player state, and regenerate the maze from a seed.** That only works if growth doesn't depend on the path, which Ian ruled out.

# Consequences

- DR-0031 rule 7 ("No persistence for generation") is reversed. [DR-0036](/decisions/0036-growth-from-open-endpoints.md) supersedes DR-0031.
- Reproducibility changes from "same seed, same world" to "same seed and walk, same world" (DR-0036 rule 10).
- Memory grows in step with what the player explores. Memory per 1,000 stored cells is one of 2.6's measures.

# Concerns raised

- **Yew (`architect/1`), the author:** no concern about storing the world. It removes the one-way door DR-0031 would have hit at the first save. The cost is that reproducing a world now needs the walk, so the growth log should be built from the start, not added later.

# AI guidance

- Never change a stored cell to make generation work. Generation fits around the stored world (DR-0036 rule 5).
- Store tiles by tile id and orientation through the palette, never by table index (proposal above, if Ian keeps it).
- Iterate the table in key order. Never let `HashMap` order decide anything.

# Links

- How the world grows: [DR-0036](/decisions/0036-growth-from-open-endpoints.md); superseded: [DR-0031](/decisions/archive/0031-endless-generation-approach.md)
- WFC and its follow-ups: [DR-0007](/decisions/0007-wave-function-collapse.md); determinism: [DR-0028](/decisions/0028-wfc-approach-for-2-1.md)
- Code layout: [DR-0012](/decisions/0012-code-layout-and-core-crate.md); one-way doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)
- Global properties: [Q-0008](/questions/open/0008-global-maze-properties.md); roadmap: [roadmap](/roadmap.md), "The game (outline)"
