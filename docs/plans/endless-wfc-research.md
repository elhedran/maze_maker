---
type: Plan
title: Endless WFC, research for slice 2.6 and the game
description: Can WFC generate an endless maze around a moving player? Yes, with conditions. Recommends seam-first chunks, compares the alternatives, and sets 2.6's experiment, measures and definition of done.
status: draft
owners: [architect/1]
tags: [plan, research, wfc, infinite-world, chunks, determinism, persistence, phase-2, slice-2-6, game]
sources:
  - id: boris-mib
    resource: https://www.boristhebrave.com/2021/11/08/infinite-modifying-in-blocks/
    title: Infinite Modifying in Blocks (Boris the Brave, 2021)
  - id: marian42
    resource: https://marian42.de/article/wfc/
    title: Infinite procedurally generated city with the Wave Function Collapse algorithm (Marian Kleineberg, 2019)
  - id: nwfc
    resource: https://arxiv.org/abs/2308.07307
    title: Extend Wave Function Collapse to Large-Scale Content Generation (Nie et al., IEEE CoG 2023)
generated:
  by: architect/1
  at: 2026-11-01T15:30:00+01:00
---

# Endless WFC: research for 2.6 and the game

By Yew (architect). This is the research [DR-0007](/decisions/0007-wave-function-collapse.md) asks for ("Infinite WFC has known problems where newly generated areas meet old ones") and the [roadmap](/roadmap.md) gates slice 2.6 and the game on. The approach it recommends is proposed in [DR-0031](/decisions/0031-endless-generation-approach.md).

> **Superseded recommendation (2026-11-01).** Ian decided the world is stored and grows along the player's path ([DR-0035](/decisions/0035-the-world-is-stored.md)), so seam-first chunks gave way to growth from open endpoints ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)). The prior art, the six questions and the fallback proof ("Why the fallback can't fail") still hold, and DR-0036 rule 7 builds on them. The chunk design, the measures and the definition of done below are replaced by DR-0036's.

## The answer

**Yes, with conditions. WFC can generate this project's endless maze, and we don't need to replace it.** No one-way door goes to Ian.

The conditions:

1. **Chunks meet only across hedge or passage.** A chunk's border edges are fixed before either chunk is generated, and they may only be *closed* (hedge on both sides) or *open* (passage or door on both sides). Room floor never crosses a chunk border, so a room lies inside one chunk.
2. **The tile set keeps a complete fallback.** For every combination of hedge and passage on a cell's four sides, some tile fits, and every hedge-to-hedge and passage-to-passage pairing among those tiles has a table entry weight above 0. The current set has this: hedge, dead end, corner, straight, T and crossroads cover all 16 combinations. That is what [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)'s fallbacks are for, and it becomes a checked invariant, not a happy accident.
3. **"A way onward" is enforced per chunk**, by a check and, if the check fails, a pinned path. It is not left to weights.

Under these, every chunk is a pure function of (world seed, chunk coordinates, tile set, generator version). It comes out the same whatever order the player visits in, nothing generated needs saving, and a new chunk can never contradict an old one, because they share nothing but border labels that both respect.

The price is that rooms can't straddle chunk borders, and the borders may show if their statistics don't match the interior. Slice 2.6 measures both. If the seams turn out unacceptable, the fallback plan (layered blocks, below) reuses the same building block, so the work isn't lost.

## The recommended approach: seam-first chunks

The world is a grid of square chunks of S × S cells (S = 16 to start; a setting). Generation happens at three levels, each depending only on the levels above it:

1. **Seam labels.** Every cell edge on a chunk border gets a label, *closed* or *open*, from a hash of (world seed, border key, position along the border). One edge on each border, picked by a hash of (world seed, border key), is forced *open*: the border's **gate**. The border key is canonical: the coordinates of the chunk on the border's east or south side, plus the axis. So both chunks hash the same input and see the same labels and the same gate, whichever is generated first.
2. **The chunk.** The 2.1 WFC runs over the S × S cells. Before it starts, each border cell's candidates are cut to the tiles whose outward side matches its label: hedge for *closed*, passage or door for *open*. The seed is a hash of (world seed, chunk x, chunk y, attempt).
3. **Way onward.** After a chunk is generated, a flood fill (the one 2.1 already has for [DR-0030](/decisions/0030-measures-for-2-1.md)) checks that its four gates are connected inside the chunk. If not, the chunk is regenerated with the next attempt seed. After a few failures (R, a constant), a path joining the four gates (the **spine**) is chosen from the chunk's seed. Its edges are pinned *open* (not hedge on either side), and WFC fills around it.

Contradictions are handled by restarts with the full tile set, as in 2.1. If those fail, one run uses corridor and hedge tiles only, always with the spine pinned. Condition 2 means that last run can never contradict (see "Why the fallback can't fail"), and the spine means it needs no gate check. So a chunk is never left unfinished, and it always has a way onward.

**Attempt order** (Rowan's, fixed so seeds are deterministic). One counter runs across every attempt and goes into the seed:

1. gate tries 0 to R, full tile set, no spine, each with up to 20 contradiction restarts;
2. the spine with the full tile set, up to 20 restarts;
3. the fallback: corridor and hedge only, with the spine, one run.

The worst case is (R + 1) × 21 + 22 runs, about 100 at R = 3.

### Why the fallback can't fail

With only corridor and hedge tiles, a tile is fixed by its four side labels, each hedge or passage, and all 16 combinations exist. Edges are then independent of each other: what a cell's north side can be never depends on its east side. Propagation only narrows each edge's label set, and the two cells on an edge always narrow it to the same set. A set becomes empty only if two pins on one edge disagree. Seam labels and spine pins never disagree: a gate is *open* and the spine pins only *open*. So the run can't contradict. This is the "complete tile set" property Nie et al. use to make nested WFC infinite without backtracking[^nwfc]. Our set has it over the corridor-and-hedge subset, and only seams and fallbacks need it.

The guarantee holds only while the data keeps it, and it depends on **table entries**, not only tile weights. The solver's support is an entry weight above 0, and that weight is base × both tile weights × exception multiplier × noise. Setting a tile weight to 0 or removing the crossroads breaks the guarantee. So does a `passage_passage` or `hedge_hedge` base of 0, or a straight-to-straight multiplier of 0: a column pinned passage north and south can then hold only straights, which can't meet each other (Rowan). So `maze-core` checks, when the tile set loads and as an error, that for those 16 variants every hedge-to-hedge and passage-to-passage pairing in every direction has an entry weight above 0. It runs alongside DR-0028's basic check.

## The six questions

### 1. Generating in chunks

Prior art, briefly:

- **One growing wave** (Marian Kleineberg's infinite city[^marian42]). A sparse map, filled near the player, with backtracking through a stored history. He first tried chunks and dropped them: constraints spread "a few slots away", so a finished chunk could leave its neighbour with no solution. Backtracking can undo cells the player has seen, "in rare cases" the one they are standing in, and he called it unsuitable for commercial games. Order-dependent, and everything generated must be kept.
- **Modifying in blocks** (Paul Merrell's model synthesis, made infinite by Boris the Brave[^boris-mib]). Four layers of overlapping blocks, each offset by half a block. Each block depends only on the earlier-layer blocks it overlaps, so the result is order-independent and needs no storage. The cost is about four times the work: each final block needs 12 earlier blocks evaluated first. A block that fails keeps the earlier layer's tiles, which needs a fallback background layer to be robust. Nothing bigger than a block can form.
- **Nested WFC** (Nie et al.[^nwfc]). Fixed-size inner WFC runs inside an outer process that keeps their edges consistent. With a complete or sub-complete tile set it is infinite, deterministic and needs no backtracking.

Seam-first chunks are the nested-WFC idea, cut down to what this maze needs: the outer level is a hash, not a second WFC.

### 2. Where new areas meet old ones

The problem Marian Kleineberg hit is that a finished area can leave its neighbour with no solution. Seam-first chunks avoid it by never letting a constraint cross a border except as a label both sides agreed in advance. The guarantees, as they stand:

| Situation | Can it contradict? | Handled by |
|---|---|---|
| New chunk against an old chunk | No. They share only labels | Construction |
| Inside a chunk, full tile set | Yes, through rooms (rectangles, handed walls) | Restarts, then the fallback run |
| Fallback run, corridor and hedge only | No | Condition 2, checked at load |
| Spine pinned *open* | No. Corridors can satisfy any open edge | Condition 2 |

The options weighed and not chosen: backtracking into generated areas (breaks order independence, and can change what the player has seen); regenerating a border band when a seam fails (order-dependent, and the band's own seams can fail); fallback tiles alone with no seam rule (a room's floor reaching an old chunk's hedge has no tile that fits).

### 3. Determinism and order independence

Yes, and it costs almost nothing in this design: a chunk reads only hashes of the world seed and coordinates, never another chunk's tiles. What it does cost:

- **Seeds widen.** 2.1's `attempt_seed(seed: u32, attempt)` packs into 64 bits. Chunks need (world seed, chunk x, chunk y, attempt) mixed through the SplitMix64 finaliser that `noise.rs` already has, with attempts numbered in the fixed order above. Seam labels and gates use the same finaliser on the border's canonical key.
- **The same inputs, not just the same seed.** A chunk also depends on the tile set (weights, exceptions, noise seed) and the generator code. Change either and the world changes. That doesn't matter for the tool. It matters once the game saves a world (point 4).
- **Floating point.** Scores are `f64` products in a fixed order with no fast-math, which give the same results on any IEEE 754 machine Rust targets. A test pins one chunk's tiles, as `splitmix64_matches_reference` already pins the PRNG.

### 4. Persistence

**Generated chunks don't need to be stored for generation's sake.** Regenerating a chunk gives the same chunk, so the game keeps a cache of chunks near the player and drops the rest. What does need storing, when the game comes:

- the world seed;
- which tile set and generator version made the world, as a fingerprint, so a save can say "this world was made by a different table" instead of quietly changing;
- anything the player changes, if the game lets them change the world (a diff per chunk, not the chunk).

This removes "persistence" as a gate for 2.6: the roadmap says 2.6 waits on it, and it doesn't need to.

The one-way door is **later**, not now. Once a released game promises that a save reproduces its world, every change to weights or the generator either breaks old saves or needs versioned tile sets and generators kept side by side. Yew re-checks this before the game saves its first world. It may go to Ian then (see "Open points").

### 5. Global properties across chunks ([Q-0008](/questions/open/0008-global-maze-properties.md))

- **Must: always a way onward.** Every chunk has a gate on every side, gates are shared across borders, and point 3 of the approach makes sure each chunk joins its four gates. So the gates and the paths between them form one connected network with no end. If the player starts on it (spawn at a gate of the origin chunk), every cell they can reach is joined to it, so they can always go on. Pockets off the network still exist, but the player can only reach them from the network, and so can always walk back out. This is a guarantee, not a statistic.
- **Should: a doorway into every room.** Deferred by Ian (Q-0007). When it comes back, it is one more acceptance check per chunk, like the gate check. Rooms lie inside one chunk, so the check is local.
- **Tune: room size.** Rooms can't be bigger than one chunk, which caps the "rooms growing without end" worry at S × S. Below that cap, room size is a weights question.
- **Tune: short dead ends.** Measured per chunk with 2.5's measures, and tuned by weights. A dead end can cross a border, so the measure must follow it into the next chunk. That is cheap, since the neighbour can always be regenerated.

The pattern generalises: **a global property is enforced by (a) something the seam labels promise and (b) a check inside each chunk, with a pinned fallback.** Properties that can't be split this way, such as "a landmark every 100 cells", need a coarser level that places features by hash before chunks are generated. That level is also where [zones](/roadmap.md) fit: a zone per chunk from the world seed, with the zone's weights used inside the chunk ([DR-0022](/decisions/0022-weight-model.md)). Seams don't depend on the zone, so zones of different kinds still meet without contradiction.

### 6. Can WFC do endless mazes for this project?

**Yes, with the three conditions above.** Replacing WFC, the roadmap's only one-way door in this area, is not needed. If 2.6 shows that seams are visible or that rooms are too constrained, the next step is layered blocks, which is still WFC. No question goes to Ian.

## The options compared

| | One growing wave | Chunks constrained by existing neighbours, band repair | Layered modifying in blocks | **Seam-first chunks** |
|---|---|---|---|---|
| Order-independent | No | No | Yes | **Yes** |
| Must store what's generated | Yes, everything seen | Yes | No (cache optional) | **No** |
| New area can contradict old | Yes: backtrack, may change seen cells | Yes: repair a band, which can fail too | Falls back to the earlier layer | **No, by construction** |
| Rooms across chunk borders | Yes | Yes | Yes, up to a block | **No** |
| Work per final area | 1×, plus backtracking | 1× plus repairs | About 4×, 12 blocks per block | **1× plus rare restarts** |
| Code on top of 2.1 | Backtracking history, sparse map | Band repair, storage | Four layers, dependency tree, copy rules | **Masks, hashes, a fallback run** |
| Seams visible? | No | Rarely | Hard to see after four layers | **Possibly: 2.6 measures** |

Why seam-first: it is the only option that is order-independent, needs no storage, can't contradict at a seam, and adds little code to what 2.1 built. Its known weakness, visible seams, is a question 2.6 answers by looking. Layered blocks fixes seams at four times the cost and is the fallback plan. Both rest on the same building block: **WFC over a rectangle, with a seed and per-cell constraints.**

## What carries over from the 2.1 generator

From `crates/maze-core/src/wfc.rs` on `rowan/2.1-wfc` (6097188):

**Unchanged:** `Rules` (weights, pair factors, support, read from the table once); `SplitMix64`; FIFO propagation (`restrict`, `propagate`); fewest-candidates choice with a seeded tie-break; `collapse` and DR-0028's score; restarts with a derived seed, and the "always terminates" argument. `measures.rs`'s flood fill and connection rule carry over for the gate check.

**Changed:**

- **Initial constraints.** `Wave::new` starts every cell with every candidate. A chunk needs a per-cell starting mask (border labels, pinned spine edges, and corridor-and-hedge only for the fallback), ANDed with the zero-weight row. `run` already propagates every cell first and fails on an empty cell, so the masks spread with no other change (Rowan). This is the main change, and it is small.
- **Seeds.** `attempt_seed(u32, u32)` becomes a mix of (world seed, chunk x, chunk y, attempt). Seam labels and gates use the same hash with their own inputs.
- **Coordinates.** Chunk coordinates are `i64` and world cells are chunk × S + local. The 64 × 64 size cap still applies to a chunk.
- **Failure.** `failed: true` with a partial grid becomes the fallback run, so a chunk is always complete and its cells can be `Vec<usize>`. The result reports the attempt count, `fallback` and `spine` for the measures.
- **Past the edge.** DR-0028 rule 6 (unconstrained past the edge) still holds for 2.1's single maze. For a chunk, the border labels replace it.
- **Choice order.** Border cells start with fewer candidates, so fewest-candidates collapses them first. That's harmless and probably helpful, but it changes how a chunk "grows" in 2.4's stepping view.

Nothing in 2.1 needs to change before 2.6 starts. The 2.1 single-maze path can stay as it is.

## What 2.6 builds: the experiment

**Goal:** show that seam-first chunks give an endless, order-independent maze that meets itself without contradictions, and find out what it costs in seams, rooms and time.

**Build**

- `maze-core`: a chunk generator (border labels and gates from canonical border keys, starting masks, the gate check, the spine, the fallback run with the spine, the fixed attempt order), the load-time completeness check on table entries, and a "world" that hands out chunks by coordinates from a cache.
- Tool: a "Beyond the edge" view. Enter a world seed, see a window of chunks (5 × 5 to start), pan to generate more, and toggle a chunk-grid overlay (a debug view, in the spirit of Ian's debug mode). Each chunk shows restarts, fallback and spine on hover.

**Measures**, each with what counts as success. These are starting points; Wren and Marram can move them.

| # | Measure | Success |
|---|---|---|
| 1 | Order independence: one chunk generated alone, and after its neighbours in three shuffled orders; a border's labels derived from each of its two chunks | Identical tiles and labels, as tests |
| 2 | Way onward: over 1,000 chunks, the four gates are joined in every chunk; over a 9 × 9 window, every gate is reachable from the origin's gate | 100%, as a test |
| 3 | Fallback rate: share of chunks needing the corridor-and-hedge run | Under 1% over 1,000 chunks, or a tuning action recorded |
| 4 | Restarts per chunk, and how often the spine had to be pinned | Reported; Wren judges whether pinned spines look artificial |
| 5 | Seams: tile-kind shares (hedge, corridor, room) and open-edge density in the two cells either side of a border, against chunk interiors | Within 25% of the interior; and Wren and Damson can't reliably point out borders with the overlay off |
| 6 | Rooms: size distribution at S = 16 and S = 32 | Reported; Wren judges whether the cap is felt |
| 7 | Time per 16 × 16 chunk, release build: median, 99th percentile, and the worst case (all ~100 runs of the attempt order, forced in a test) | Median under 10 ms (Rowan estimates 1 to 3 ms); the 99th percentile and worst case reported. Chunks are generated ahead of the player and off the UI thread, so the worst case is a delay, not a frozen screen |

**Done when:** tests 1 and 2 pass; measures 3 to 7 have been taken and are written into the 2.6 records; Wren and Damson have reviewed seams and rooms; and Ian has panned a world in the tool, toggled the overlay, regenerated in a different order and seen the same world. If measure 5 or 6 fails review, the result is a recorded choice: tune the seam labels (a Markov chain along the border instead of independent coin flips), raise S, or move to layered blocks. It is not an open-ended rework.

## Open points

| Point | Who | One-way door? |
|---|---|---|
| How open-edge probability on seams is set (fixed, from 2.5's measured density, or a chain along the border) | Rowan or Ash with Wren, in 2.6 | No |
| Chunk size S | Engineers, Wren reviews | No |
| R, the gate-check retries before pinning a spine; how a spine is routed | Engineers | No |
| Rooms that must cross chunks, landmarks, zones: a coarser feature level | Yew, when a feature needs it | No |
| A completeness check as a tile-set invariant: it constrains Wren's future tile sets | Wren and Yew, in DR-0031 | No |
| Saved worlds that must reproduce across tile set or generator changes | Yew re-checks before the game saves a world | **Possibly, then.** Promising old saves still load would tie every later weight change to versioning. Not today |
| Replacing WFC | n/a | Not needed |

## Links

- Proposed record: [DR-0031](/decisions/0031-endless-generation-approach.md)
- WFC and its follow-ups: [DR-0007](/decisions/0007-wave-function-collapse.md)
- 2.1's WFC and measures: [DR-0028](/decisions/0028-wfc-approach-for-2-1.md), [DR-0030](/decisions/0030-measures-for-2-1.md)
- Tile set and fallbacks: [DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)
- Weights and zones: [DR-0022](/decisions/0022-weight-model.md)
- Grid: [DR-0014](/decisions/0014-square-grid.md); code layout: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- Global properties: [Q-0008](/questions/open/0008-global-maze-properties.md)
- Roadmap: [roadmap](/roadmap.md), slice 2.6 and "The game (outline)"

[^boris-mib]: Boris the Brave, [Infinite Modifying in Blocks](https://www.boristhebrave.com/2021/11/08/infinite-modifying-in-blocks/), 2021.
[^marian42]: Marian Kleineberg, [Infinite procedurally generated city with the Wave Function Collapse algorithm](https://marian42.de/article/wfc/), 2019.
[^nwfc]: Nie et al., [Extend Wave Function Collapse to Large-Scale Content Generation](https://arxiv.org/abs/2308.07307), IEEE Conference on Games 2023.
