---
type: Decision Record
title: The WFC approach for the first sample maze
description: Fewest-candidates cell choice with seeded tie-breaks, a weighted draw of tile weight times each settled neighbour's pair factor, restart with a derived seed on contradiction up to 20 times, a hand-rolled SplitMix64, an unconstrained edge, and a generate_maze command; it also stands as slice 2.1's plan.
status: stable
decision_id: DR-0028
decision_status: accepted
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [architect/1, engineer-rowan/1]
supersedes: []
tags: [wfc, generation, determinism, ipc, sample-maze, phase-2, plan]
generated:
  by: architect/1
  at: 2026-11-01T12:00:00+01:00
verified:
  - by: engineer-rowan/1
    at: 2026-11-01T13:00:00+01:00
  - by: engineer-ash/1
    at: 2026-11-01T00:22:09+01:00
    note: "Stands in for the architect (DR-0018: the author is a required approver). Approves rules 1-8. The IPC contract section still needs the final contract written in: GeneratedMaze gains centre (CellPos {x, y}), and Measure.value is a ready-to-show string, not a number."
---

# Context

Slice 2.1 ([roadmap](/roadmap.md)) generates one fixed-size maze in `maze-core` by WFC ([DR-0007](/decisions/0007-wave-function-collapse.md)) from the core tile set ([DR-0015](/decisions/0015-tile-set.md), [DR-0024](/decisions/0024-tile-set-file-format.md)) and shows it in the tool. [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) and [DR-0022](/decisions/0022-weight-model.md) left to 2.1 how several neighbours' weights combine. Ian (`human:ian`): "make sure we bias towards executing planned work over re-planning or planning new work." and "There is such a thing as polishing too much before moving on." So: the simplest WFC that meets the Done-when, nothing for 2.4 to 2.6 beyond not blocking them.

# Decision

All in a new `maze-core` module, `wfc.rs`, over the existing `TileSet` and `Table`. No new dependencies.

1. **Candidates and propagation.** Each cell holds a set of variant indices (table order). Candidate `c` in a cell survives only if, in each direction `d`, the neighbouring cell has some candidate `n` with `table.entry(d, c, n).weight > 0` (support means weight above 0, not just `allowed`, so a zero-weight tile can never be picked). Propagation is a FIFO queue of changed cells, run once over every cell at the start and after each collapse. Precompute, per direction and variant, the set of supported neighbours.
2. **Choosing a cell.** The uncollapsed cell with the **fewest candidates**. Ties are broken by the seeded PRNG: collect the tied cells in row-major order, pick one uniformly. Not Shannon entropy: fewer moving parts, and "fewest options" is easy to show when 2.4 steps through the collapse.
3. **Collapsing it: the combining formula.** For each candidate `c`, with settled (collapsed) neighbours `n_i` in directions `d_i` from the cell:

   > score(c) = weight of c × Π_i pair(c, d_i, n_i), where pair = side-pair base × exception multiplier × noise factor

   that is, `Entry::weight` with both tile weights divided out, read from the `Entry` fields (`base`, `exception`, `noise`) rather than by division, so a zero weight can't divide. Unsettled neighbours count only through propagation. Draw `c` with probability `score(c) / Σ score`, walking candidates in table order. The tile's own weight counts once however many neighbours are settled; each neighbour's weight is the same for every candidate, so it drops out. With exactly one settled neighbour, the draw equals that neighbour's row in the 1.1 table, so the table still means what the tool says it means. If `Σ score` is 0 (it shouldn't be, given rule 1), treat it as a contradiction; never divide into `NaN`.
4. **Contradictions.** A cell left with no candidates ends the attempt. Restart the whole maze from scratch with the next attempt's seed, up to **20 restarts** (a constant, `MAX_RESTARTS`). After that, return `failed: true` with the last attempt's partial grid. Each attempt collapses at most `width × height` cells and propagation only removes candidates, so it always terminates: no hang, no panic. The result carries `restarts` (0 for a clean run), and the tool says "restarted 2 times" or "failed after 20 restarts".
5. **Determinism.** A hand-rolled **SplitMix64** (a dozen lines; `noise.rs` already has its finaliser). Attempt `k` seeds its state with `(u64::from(seed) << 32) | k`. A weighted draw takes the top 53 bits as a fraction in [0, 1), times `Σ score`. Every loop runs in a fixed order (row-major cells, table-order candidates, FIFO propagation), and no `HashMap`/`HashSet` iteration decides anything. Same seed, same tile set file, same maze. Seeds are `u32` so they cross IPC as a plain number.
6. **Past the edge: unconstrained.** Cells on the border are only constrained by cells inside. The maze is a window on an endless world (DR-0007), so open sides may run off the edge; a hedge border would make 2.1's mazes look closed in a way 2.6's will not.
7. **First size: 24 × 16.** Fits one screen at about 40 px a cell. It is a setting in the tool (a constant the request sends), the engineers' two-way door; `maze-core` takes any size from 1 × 1 to 64 × 64 and the command rejects others with a readable error.
8. **The basic check** (Wren and Yew, from 1.5): a `maze-core` test loads `data/tile-sets/core.toml` with `include_str!` (compile time, so no I/O in core) and asserts `table.row_total(d, row) > 0` for every variant and direction: every side of every tile can meet something with weight above 0.

## IPC contract (DR-0023 types)

`generate_maze(request: GenerateRequest) -> Result<GeneratedMaze, ToolError>`, using the loaded tile set from state (a load failure returns the existing `LoadFailed`/`NotLoaded` errors; a bad size adds a `BadRequest` kind).

- **`GenerateRequest`:** `seed: u32`, `width: u32`, `height: u32`.
- **`GeneratedMaze`:** `revision` (the tile set revision used); `seed`, `width`, `height`; `cells: Array<number | null>`, row-major from the north-west, each an index into table order, the same index as `table_view`'s `tiles` at that `revision` (`null` only in a failed maze's unfilled cells); `centre: CellPos { x, y }`, the centre cell [DR-0030](/decisions/0030-measures-for-2-1.md) uses after its hedge fallback; `restarts: u32`; `failed: bool`; `measures: Measure[]`, each `{ name, value, explanation }` with `value` a ready-to-show string worded as DR-0030 words it, computed in `maze-core`. (Facilitator, 2026-11-01: brought in line with the contract agreed for the build, as Rowan and Ash asked in their approvals.)

The frontend draws cells from the `tiles` it already has, and refetches `table_view` if `revision` differs.

# Alternatives considered

- **Lowest Shannon entropy** for choosing a cell: the textbook choice, but extra code and harder to explain; review of sample mazes can bring it in if fewest-candidates looks wrong.
- **Product of the table's probabilities** from each neighbour: counts the candidate's tile weight once per neighbour, so common tiles snowball wherever many neighbours are settled.
- **Average of the neighbours' probabilities:** lets one neighbour that dislikes a tile be outvoted, which reads oddly against "should we see it often in that place".
- **Backtracking** instead of restarting: fewer wasted attempts, more code. Infinite generation (2.6) will need a local repair anyway; that is its decision.
- **The `rand` crate:** a new dependency whose value streams aren't promised stable across versions, which is the opposite of what a seed needs.
- **A hedge border:** see rule 6.

# Consequences

- 2.1 can start today: one core module, one command, one view.
- Weights shape the maze only through `score`; changing `core.toml`, Reload and regenerating changes the maze with no code change (the 2.1 demo).
- Two-way door, `major: false`: nothing persists and no seed is promised to reproduce across versions. It stops being cheap when 2.6 saves seeds or the world, since changing any rule above then changes saved mazes; Yew re-checks it then (DR-0018).

# Concerns raised

- **Yew (architect):** fewest-candidates with a uniform tie-break picks the first cell anywhere at random, and may grow the maze in visible fronts. Acceptable for a first sample; 2.4's stepping view will show it, and that review settles it.
- **Rowan (engineer), approving:** buildable as written. Two small notes, neither blocking. (1) The IPC section is behind the contract agreed with Ash for the build: `Measure.value` is a string worded as DR-0030 words it (e.g. "412 of 480 walkable cells (86%)"), not a number, and `GeneratedMaze` also carries `centre: { x, y }`, the centre cell DR-0030 uses after its hedge fallback. The section should be brought in line. (2) A candidate whose tile weight is 0 has no support anywhere, but on a 1 × 1 grid nothing removes it; the build drops zero-weight candidates at the start so rule 3's "Σ score is 0" case stays unreachable in practice.

# Build plan

Done when the roadmap's 2.1 Done-when holds. Rowan and Ash agree the `GeneratedMaze` binding first (Rowan generates it), then work in parallel ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)).

- **Rowan (`crates/maze-core`, `apps/tool/src-tauri`):** `wfc.rs` with SplitMix64, propagation, collapse, restarts, and `generate(set, table, seed, width, height) -> Generation`. Tests: same seed twice gives the same maze; several seeds give at least two different mazes; over 200 seeds every pair of neighbouring cells has `probability > 0` in the table; a tiny fixture that always contradicts returns `failed: true` with `restarts == 20` and no panic; the basic check (rule 8). Then the `generate_maze` command and ts-rs types (`GenerateRequest`, `GeneratedMaze`, `Measure`, the `BadRequest` error kind) with the bindings staleness test.
- **Ash (`apps/tool/src`):** the Lit generation view per Damson's 2.1 record: seed field, Generate, the maze drawn with DR-0017's glyphs from the existing tile drawing, hover showing the tile name, the restart or failure note, and the measures list (empty is fine). Vitest for the note text and for drawing from indices. Demo steps in the README.

# Links

- WFC: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Code layout, no I/O in core: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- Entries and weights: [DR-0013](/decisions/0013-table-entries-from-tile-sides.md), [DR-0022](/decisions/0022-weight-model.md)
- Grid: [DR-0014](/decisions/0014-square-grid.md)
- Tile set and side rules: [DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)
- IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md)
- File format, table, noise: [DR-0024](/decisions/0024-tile-set-file-format.md)
- Measures: [Q-0008](/questions/open/0008-global-maze-properties.md)
- Slice: [roadmap 2.1](/roadmap.md)
