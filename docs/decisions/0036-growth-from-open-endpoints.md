---
type: Decision Record
title: Endless generation by growing the stored world from open endpoints
description: The world grows in patches from open endpoints near the player. Each patch is a WFC run over a window, with stored cells pinned and the window's edge closed or open, and only cells joined to the stored world are kept. So stored cells meet the unknown only with hedge, passage or door, and are never changed. A check with a pinned spine keeps a way onward, and an opening-pressure factor joins the score. Supersedes DR-0031.
status: draft
decision_id: DR-0036
decision_status: proposed
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [engineer-rowan/1, engineer-ash/1]   # architect/1 is the author; two engineers stand in (DR-0018), as for DR-0031
supersedes: [/decisions/archive/0031-endless-generation-approach.md]
tags: [wfc, infinite-world, growth, frontier, determinism, global-properties, slice-2-6, game]
sources:
  - id: marian42
    resource: https://marian42.de/article/wfc/
    title: Infinite procedurally generated city with the Wave Function Collapse algorithm (Marian Kleineberg, 2019)
  - id: boris-mib
    resource: https://www.boristhebrave.com/2021/11/08/infinite-modifying-in-blocks/
    title: Infinite Modifying in Blocks (Boris the Brave, 2021)
generated:
  by: architect/1
  at: 2026-11-01T08:24:59Z
verified: []
---

# Context

Ian (`human:ian`), 2026-11-01, verbatim:

> Ok, on that not I have some future steering for the product manager for future phases around improving the mazes.  I'd like two changes - the first is we only generate based of "open" endpoints, endpoints connected to existing maze.  This basically means looking for where a path ends, a room door opens etc rather than just every space next to where the player has been, within the bounds of the player space radius.  The second change is to add some pressure to bias towards an opening maze.  that means that if a piece that goes down would only connect to already closed maze, that should get a reduced probability.

> In regards to DR-0031 the world needs to be stored anyway - we want to know where the player has been, we will add loot, and hence we need to know what is looted, etc.  However given how this works, there should be quite an efficient in-memory data structure for this (sparse big table).

> and yes I mean growth follows the players path

[DR-0035](/decisions/0035-the-world-is-stored.md) records that the world is stored and grows along the player's path. [DR-0031](/decisions/archive/0031-endless-generation-approach.md) (seam-first chunks) was designed for a world that is a pure function of coordinates. That premise is gone, so this record replaces it. The parts of DR-0031 that don't depend on that premise carry over, and so does what 2.6 step 1 has already built on `rowan/2.6-chunks` (starting masks, the completeness check).

# Decision

**Terms.** A **stored** cell is in the world's table (DR-0035); an **unsettled** cell isn't. A **walkable side** is any side but hedge. Two cells are **joined** when their facing sides are both walkable. An **open endpoint** is a stored cell's walkable side that faces an unsettled cell, and that unsettled cell is its **anchor**.

1. **The frontier rule.** A stored cell meets an unsettled cell only with hedge, passage or door, never floor. So a stored room is always complete, and every open endpoint is a passage end or a door. Rules 3 and 4 keep this rule, and a test checks it after every patch. Everything else rests on it, and old saves will too.
2. **When growth runs.** While some anchor lies within the player's radius r, grow a patch at the nearest one. Distance is Chebyshev, ties go in (y, x) order, and r is a setting that must be more than the player can see. Repeat until no anchor is within r. Each patch stores at least its anchor, so this ends. Only open endpoints grow, so a cell that is never reached stays unsettled and is drawn as solid hedge. A new world starts with one patch anchored at the origin, whose cell is masked to tiles with a walkable side, and the player spawns there.
3. **A patch** is one run of [DR-0028](/decisions/0028-wfc-approach-for-2-1.md)'s WFC over a P × P window. P is 16 to start (a setting). The window lies beyond the anchor in its endpoint's direction, centred on the anchor across that direction. Starting masks:
   - stored cells in the window are pinned to their tile. They count as settled, in support and in the score, and so do stored cells just outside the window;
   - an edge cell whose outside neighbour is unsettled may only show hedge, passage or door on that side;
   - every other cell starts with every candidate.

   *Yew's reading of "within the bounds of the player space radius":* r bounds where growth starts, so a window can reach up to P beyond r. If it should stop at r, clip the window there, and rooms near r get smaller.
4. **Keep what's joined.** After the run, store the new cells that are joined to the stored world through walkable sides (a flood fill, using `measures.rs`'s connection rule). Discard the rest; those cells stay unsettled. A kept cell faces a discarded one with hedge, because if they were joined, both would have been kept. The edge masks cover the window's edge. So rule 1 holds.
5. **Stored cells never change.** Contradictions are handled inside the patch only: restarts, then the fallback (rule 7). Nothing the player has seen is ever repaired. Rule 1 is what makes backtracking into the world unnecessary.
6. **A way onward** (Q-0008's must-have, as a guarantee). An open endpoint is **outward** when its anchor connects through unsettled cells to the unbounded unsettled region. After every patch, the world must have at least one outward endpoint. The stored world is one joined area with the player in it, so the player can never be boxed in.
   - **The check.** It runs after each attempt. The reference method floods unsettled cells inward from just outside the stored area's bounding box. A faster method is the engineers' call. A failed attempt goes on to the next one (rule 8).
   - **The spine.** A path of edges pinned open (passage or door on both sides). It runs from an outward endpoint in the patch, through the patch's unsettled cells, to an edge cell whose outside neighbour reaches the unbounded region without coming back into the window, and out across that edge. The patch seed chooses the route; how is the engineers' call.
   - **Why a spine always exists.** A patch stores only cells joined to its own anchors. So if a patch cut off the last outward endpoint, one of its anchors was outward before the patch, and that anchor's way out leaves the window for the last time somewhere.
7. **The fallback can't contradict.** [DR-0031](/decisions/archive/0031-endless-generation-approach.md) rules 5 and 6 carry over, applied to a patch instead of a chunk: restarts with the full set, then one run with corridor and hedge only and the spine pinned, which needs no check. The completeness check (checked at load, as an error) keeps DR-0031 rule 6 and adds one clause, because the fallback now meets stored tiles of every kind:
   - in every direction, every variant's hedge side has an entry weight above 0 with every fallback variant that shows hedge back;
   - every variant's passage or door side has an entry weight above 0 with every fallback variant that shows passage back.

   This covers the hedge sides of rooms and doorways' doors, and stored doors facing the fallback across the frontier (Rowan's door note). The check's messages cite **DR-0036 rule 7**.
8. **Attempt order and seeds.** DR-0031 rule 5's order, unchanged. One attempt counter runs across all of it:
   1. tries 0 to R with the full set and no spine, each with up to 20 contradiction restarts; the first run that completes and passes rule 6 is the patch;
   2. the spine with the full set, up to 20 restarts;
   3. the fallback with the spine, one run.

   A patch's seed mixes (world seed, anchor x, anchor y, attempt) through the SplitMix64 finaliser. Each anchor is used once.
9. **Opening pressure** (Ian's second change). DR-0028 rule 3's score gains one factor. A candidate gets it when it has at least one walkable side and all of its walkable sides face settled cells (stored, or already collapsed in this run). Its score is then multiplied by `opening_pressure`.
   - **What counts as new.** A side facing an unsettled cell outside the window, or past the edge of a fixed-size maze, counts as new. Hedge has no walkable side, so the factor never applies to it.
   - **The value.** An optional tile set key, with 0 < value ≤ 1. The loader rejects anything else, and an absent key means 1, so 2.1's seeds don't change. Wren sets the value.
   - **Why above 0.** It biases and never forbids, so support (DR-0028 rule 1) and rule 7's proof are untouched.
   - **The trace.** [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)'s trace shows it as its own factor.
10. **Reproducibility.** A patch is a pure function of the stored world before it, its anchor, the tile set and the code. So the world is a pure function of (world seed, tile set, generator version, walk), and the growth log (DR-0035) records the anchors in order. Tests and the tool drive a walk from a seeded random walk or a scripted path. Any one patch can be regenerated from the world as it was before it.

## What carries over from 2.6 step 1, and what's dropped

- **Kept:** the per-cell masks (`Wave::with_mask`, `Solver`); the completeness check in `TileSet::from_toml`, widened by rule 7's clause; `from_toml_allow_incomplete` for the deliberately incomplete test sets (Rowan's note, accepted); spine pinning, the fallback run and the attempt order; the pinned 2.1 seeds; the flood fill.
- **Dropped:** border labels, gates, canonical border keys, chunk seeds, the order-independence tests, and the gates-joined check, which becomes rule 6.

## 2.6's definition of done, technical part (Marram slices it)

- **Step 1, `maze-core`:**
  - rules 1 to 10, the world's table (DR-0035) and the growth log;
  - tests:
    - replay: the same seed and walk give the same world, and a patch regenerated from the world before it comes out the same;
    - over 100 seeded random walks of 1,000 steps: rule 1 holds after every patch; stored cells never change; every pair of neighbouring stored cells has an entry weight above 0; every stored cell is joined to the origin; there is an outward endpoint after every patch;
    - the spine and the fallback are each forced in a test;
    - rule 7's new clause fails loudly on a doorway-against-passage weight of 0;
    - with no `opening_pressure` key, 2.1's pinned seeds are unchanged;
  - clippy is clean.
- **Step 2, measures:**
  - per patch: the restart, spine and fallback rates;
  - the frontier (open and outward endpoints) over a walk, with pressure at 1 and at Wren's value;
  - room sizes at P = 16 and P = 32, and whether rooms avoid the edges of patches;
  - time per patch (median, 99th percentile and the forced worst case), and how the rule 6 check's time grows with the explored area;
  - memory per 1,000 stored cells.
- **Step 3, the view:** a world seed and a walker (a seeded random walk, or click to walk), with overlays for stored cells, the frontier and the last patch, and a way to replay the walk. Wren and Damson review it.
- **Done when:** the tests pass, the measures are written into the 2.6 records, Wren and Damson have reviewed, and Ian has walked a world in the tool and replayed the walk to see the same world.

## Why it's a two-way door

Nothing is built on it beyond what carries over. Unlike DR-0031, it stays a two-way door after the first save, because saves hold cells and not a recipe (DR-0035). Changing P, r, the check, the seeds or the pressure only changes new growth. The exception is rule 1: an old save's frontier must stay growable, so loosening rule 1 later needs a migration. Yew re-checks rule 1 before the game first saves a world. Reversing the whole record throws away the patch and keep code, and the solver, masks, spine and fallback stay.

# Alternatives considered

- **(a) Keep seam-first chunks, and store only player state and a chunk cache.** Order independence no longer buys anything once the world is stored. It fills whole chunks rather than growing from open endpoints, and it keeps a fixed grid that rooms can't cross. That isn't what Ian asked for.
- **(c) Chunks constrained by their stored neighbours** (DR-0031's rejected alternative, viable now). With rule 1 it would need no band repair. But it still fills every chunk the player comes near, and it fixes a grid of room seams. This record is (c) with the window placed at the endpoint instead of on a grid, and with only the joined cells kept.
- **One growing wave with backtracking** (Marian Kleineberg[^marian42]). This record takes his stored, order-dependent world but not his backtracking, which can change cells the player has seen. Rule 1 makes backtracking unnecessary.
- **Layered modifying in blocks** (Boris the Brave[^boris-mib]). Its strengths are order independence and needing no storage, and this project no longer needs either.

# Consequences

- **Wren's 2.1 finding** ([review](/plans/2.1-maze-review.md): 8–88% reachable, 5–19 separate areas, a sealed start) can't happen in a grown world. Everything stored is joined to the player's start, and sealed rooms and pockets are never stored. The risk moves to the frontier closing, which rule 6 rules out and rule 9 makes rarer. 2.1's tuning ("more hedge, more joined") may not carry over: here more hedge closes endpoints sooner. Step 2's frontier measure will show it. DR-0030's two measures don't apply to grown worlds.
- **Q-0008.** Its must-have is a guarantee (rule 6). For the deferred "should" (a doorway into every room): every stored room was reached, so it has a way in, as long as the game only draws stored cells. Wren checks. Dead ends at the frontier get the rule 9 factor. Room size is still a matter of weights.
- **Rooms** are up to P × P, with no fixed seams. A room never crosses the outer edge of the patch that made it, so rooms may avoid where patches end. Step 2 measures this, and a larger P is the first fix.
- **DR-0028 rule 3** gains a factor. Its promise that "one settled neighbour gives that neighbour's 1.1 row" holds only when the factor is 1. DR-0034's "weight × factors equals the score" includes the factor. [DR-0024](/decisions/0024-tile-set-file-format.md)'s format gains one optional key.
- **The roadmap's 2.6** (Marram): step 1's order-independence and gate tests give way to the tests above. The closing demo becomes walking a world and replaying the walk. The opening factor is built in 2.6 step 1 and tuned in 2.3.
- **The [research note](/plans/endless-wfc-research.md)**: its recommendation is superseded, and its prior art and fallback proof still hold. Yew adds a pointer.
- **Data that grows:** the table and the frontier grow with what has been explored. The reference rule 6 check is O(explored area) per patch. That is fine for 2.6's walks, but the game needs an incremental method (a two-way door for the engineers), sized by step 2's measure.
- 2.1's single-maze path is unchanged.

# Concerns raised

- **Yew (`architect/1`), the author:**
  1. Rooms may avoid the edges of patches. They are not on a grid, but 2.6 should look for it.
  2. Rule 6 is a global check, because a patch that closes a ring changes which endpoints are outward far from the patch. The reference flood is the slow part over long sessions.
  3. An unsettled cell drawn as solid may later become walkable when it is reached from another side. A player only sees this if they can see over hedges, which is for Wren and Damson.
  4. Reproducing a world now needs its walk, so build the growth log in step 1, not later.

# AI guidance

- Never change a stored cell. Repair happens only in the patch being generated.
- Never store a cell that shows anything but hedge, passage or door to an unsettled cell (rule 1). Keep only the joined cells (rule 4).
- Run the completeness check on table entries, with rule 7's clause. Don't weaken it to get a tile set to load. Raise it with Wren and Yew.
- Keep `opening_pressure` above 0.
- Seeds come from (world seed, anchor, attempt) through the SplitMix64 finaliser written in `maze-core`. Never use `std` hashers or `rand`. Keep the attempt order (rule 8).
- Never let `HashMap` order decide anything. Walk the table and the frontier in key order.

# Links

- The world is stored: [DR-0035](/decisions/0035-the-world-is-stored.md); superseded: [DR-0031](/decisions/archive/0031-endless-generation-approach.md)
- Research and prior art: [Endless WFC](/plans/endless-wfc-research.md)
- WFC: [DR-0007](/decisions/0007-wave-function-collapse.md), [DR-0028](/decisions/0028-wfc-approach-for-2-1.md); measures: [DR-0030](/decisions/0030-measures-for-2-1.md); trace: [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)
- Tile set and fallbacks: [DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md); file format: [DR-0024](/decisions/0024-tile-set-file-format.md)
- Global properties: [Q-0008](/questions/open/0008-global-maze-properties.md); 2.1 review: [2.1 maze review](/plans/2.1-maze-review.md)
- Roadmap: [roadmap](/roadmap.md), slice 2.6

[^marian42]: Marian Kleineberg, [Infinite procedurally generated city with the Wave Function Collapse algorithm](https://marian42.de/article/wfc/), 2019.
[^boris-mib]: Boris the Brave, [Infinite Modifying in Blocks](https://www.boristhebrave.com/2021/11/08/infinite-modifying-in-blocks/), 2021.
