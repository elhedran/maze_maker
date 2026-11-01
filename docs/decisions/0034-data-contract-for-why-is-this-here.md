---
type: Decision Record
title: The data contract for "Why is this here?"
description: The explore view gets its own explore_tile command, built from the same entry code as table_view, with each arm sorted and its 0s split off in Rust; "why is this here" replays the seed through the one WFC code path with an optional observer on the collapse step, behind a trace_cell command that rejects a stale revision; it also stands as slice 2.2's technical plan.
status: stable
decision_id: DR-0034
decision_status: accepted
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [engineer-ash/1, engineer-rowan/1]   # Ash stands in for the architect, architect/1, who is the author (DR-0018)
supersedes: []
tags: [explore-view, wfc, trace, determinism, ipc, slice-2-2, plan]
generated:
  by: architect/1
  at: 2026-11-01T08:05:39Z
verified:
  - by: engineer-ash/1
    at: 2026-11-01T08:24:13Z
    note: "Stands in for the architect, who wrote the record (DR-0018). Approves; the frontend is built against these shapes. Notes, not blocking: explore_tile's response carries no tile list, so the frontend refetches one when the revision changes; a centre that no longer exists comes back as bad_request, so the adapter checks the tile list to tell it from other errors."
  - by: engineer-rowan/1
    at: 2026-11-01T08:25:28Z
    note: "Approves; built on rowan/2.2-cell-why with the bindings exactly as written, A1 to A4 included. Internal departures, not in the contract: core trace_cell takes (x, y) and returns its own TraceError; entry_view takes no set; per-cell tests record every cell in one pass through trace_cell's own builder, because replaying per cell is slow unoptimised. One wording fix: the JSON line's nulls."
---

# Context

Slice 2.2 ([roadmap](/roadmap.md)): click a cell of a generated maze, and the explore view ([DR-0026](/decisions/0026-explore-one-tile-design.md), folded in from 1.2) opens on it and shows how its neighbours combined. That leaves two technical questions. DR-0026 leaves it to Yew whether the arms reuse `table_view` or get a command of their own. And someone has to decide what data "why is this here" needs for one cell. Damson's DR-0033 designs the link in parallel, on the assumption written in rules 4 to 7. If it needs something different, this record changes before the build relies on it.

Slice 2.6 changes `crates/maze-core/src/wfc.rs` at the same time ([DR-0031](/decisions/archive/0031-endless-generation-approach.md) rules 3 and 6). The roadmap says 2.2's shape wins if the two clash there. Better that they don't clash, so 2.2's change is one hook (rule 5).

# Decision

## The explore view's arms

1. **A command of its own, `explore_tile`**, not the 1.1 request. DR-0026 asks Rust to sort each arm and split off its 0s. Doing that through `table_view` means adding a sort and a zero split to `TableViewRequest`, options only this view would use. **Same numbers, same source:** `table_view`'s per-entry mapping moves into one function in `view.rs`, `entry_view(set, table, d, row, column) -> EntryView`, and both commands call it. A neighbour is the whole `EntryView`, so DR-0025's panel explains it unchanged.
2. **The request names the centre by identity**, tile id and orientation, as DR-0026 point 10's entry point does, so a centre survives a Reload. An unknown pair is `bad_request`. The response carries `revision`, and its indices are table order at that revision, as `GeneratedMaze`'s are.
3. **An arm** covers one direction, and the arms come N, E, S, W. Each holds the centre's side kind on that edge, then `neighbours`, the entries with weight above 0, sorted by probability with the highest first and ties in table order (`f64::total_cmp`, then column), then `zeros`, the rest, in table order. The count DR-0026 shows is `zeros.length`, so there is no separate field.

## Why is this here

4. **Replay on demand, not recording.** `trace_cell` regenerates the maze from (seed, width, height) on the loaded tile set and watches one cell. The same seed and tile set give the same maze ([DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 5), so the cost is one generation: about 2 ms at 24 × 16 in release, and 21 attempts' worth at the very worst. Nothing is stored, nothing extra is sent with each maze, and `generate_maze` doesn't change.
5. **One code path with an optional observer.** In `wfc.rs`:

   ```rust
   /// One collapse as the solver made it; borrowed from its own working.
   pub struct CollapseStep<'a> {
       pub cell: usize,
       /// Neighbours already collapsed, N, E, S, W order: (direction from `cell`, variant).
       pub settled: &'a [(Direction, usize)],
       /// Candidates left after propagation, table order, with the score each was drawn by.
       pub scores: &'a [(usize, f64)],
       pub total: f64,
       pub chosen: usize,
   }
   pub trait Observer {
       fn attempt_started(&mut self, attempt: u32);
       fn collapsed(&mut self, step: &CollapseStep<'_>);
   }
   impl Observer for () { /* both no-ops */ }
   pub fn generate_observed(set: &TileSet, table: &Table, seed: u32, width: usize, height: usize,
       observer: &mut dyn Observer) -> Result<Generation, GenerateError>;
   ```

   `generate` becomes `generate_observed(.., &mut ())`, with the same signature as now. `Wave::collapse` calls `observer.collapsed` once, after the draw, passing borrows of what it has already computed. `Wave::run` passes the observer through. `generate_observed` calls `attempt_started(k)` as each attempt begins. The observer only ever gets shared borrows, never the wave or the PRNG, so the type system stops it from changing generation.
6. **The trace** goes in a new `maze-core` module, `trace.rs`, as `trace_cell(set, table, seed, width, height, cell) -> Result<.., GenerateError>`.
   - **Which attempt:** its observer keeps the target cell's collapse from the latest attempt only. So the trace comes from the attempt whose grid is shown: the one that succeeded, or a failed maze's last.
   - **Pair factors:** it reads each factor's parts from the `Entry` fields (base, exception multiplier, noise) with the same expression `Rules::new` uses. Tile weight × Π factors is then bit-for-bit the solver's score.
   - **Score and share:** `score` is the solver's own value, passed through, never recomputed. Shares are `score / total`, worked out in Rust.
   - **Order:** Rust sorts the candidates as arms are sorted, by share with the highest first, then table order. The draw itself walked table order.
   - **The four neighbours:** each direction is *counted* (collapsed before the draw), *later* (collapsed after it) or *edge*, worked out from `CollapseStep.settled` and the finished grid. `ruledOut` counts the variants that weren't possible at the draw.
   - **The panel:** the app adds to each pair factor its 1.1 table entry (the direction from the cell, the candidate as row, the neighbour as column), built by the shared `entry_view()`, so DR-0025's panel opens on it as it is.
7. **Errors, checked before any replay:**
   - no tile set loaded: the existing `not_loaded`;
   - a `revision` other than the loaded one: a new kind, `stale`. The maze came from a tile set that has been reloaded since, so it has to be regenerated;
   - a bad size, or a cell outside the maze: `bad_request`, with a readable message.

   A failed maze is not an error: the trace has `failed: true`, and a cell the last attempt left open has `collapse: null`. No request makes it panic.

## IPC contract (DR-0023 types)

- `explore_tile(request: ExploreRequest) -> Result<ExploreView, ToolError>`
- `trace_cell(request: TraceCellRequest) -> Result<CellTrace, ToolError>`
- `ToolErrorKind` gains `stale`.

The TypeScript the bindings will have, without doc comments, so Ash can stub against it now:

```ts
export type ToolErrorKind = "load_failed" | "not_loaded" | "bad_request" | "stale";

export type ExploreRequest = { tileId: string, orientation: number, };
export type ExploreView = { revision: number, centre: number, arms: [ArmView, ArmView, ArmView, ArmView], };
export type ArmView = { direction: Direction, centreSide: SideKind, neighbours: Array<EntryView>, zeros: Array<EntryView>, };

export type TraceCellRequest = { revision: number, seed: number, width: number, height: number, cell: CellPos, };
export type CellTrace = { revision: number, cell: CellPos, restarts: number, failed: boolean, steps: number, collapse: CollapseTrace | null, };
export type CollapseTrace = { step: number, neighbours: [NeighbourAtDraw, NeighbourAtDraw, NeighbourAtDraw, NeighbourAtDraw], candidates: Array<CandidateScore>, ruledOut: number, total: number, chosen: number, };
export type NeighbourAtDraw = { direction: Direction, state: NeighbourState, tile: number | null, };
export type NeighbourState = "counted" | "later" | "edge";
export type CandidateScore = { tile: number, tileWeight: number, factors: Array<PairFactor>, score: number, share: number, };
export type PairFactor = { direction: Direction, neighbour: number, base: number, exceptions: Array<number>, multiplier: number, noise: number, factor: number, entry: EntryView, };
```

What the fields mean:

- **Indices:** `centre`, `tile`, `neighbour` and `chosen` are indices into `TableView.tiles` at `revision`. `exceptions` are indices into `TableView.exceptions`, as in `EntryView`.
- **The attempt:** `restarts` and `failed` are what `generate_maze` returns for the same request. `steps` is how many collapses the shown attempt made: width × height when it succeeded.
- **When:** `step` is the order in which the cell collapsed in that attempt, starting at 1.
- **Neighbours:** `neighbours` always has four entries, in N, E, S, W order.
  - `counted`: already collapsed at the draw, with `tile` being its tile at that moment.
  - `later`: collapsed after the draw, with `tile` being its tile in the finished grid, or `null` if a failed maze left it open.
  - `edge`: past the edge, with `tile` `null`.

  Only counted neighbours have factors. Later ones shaped the cell only through propagation, by deciding which candidates were left (DR-0028 rule 3).
- **Candidates:** `candidates` are those left after propagation, sorted in Rust as in rule 6. `ruledOut` is how many variants weren't possible at the draw, zero-weight tiles included. Each candidate's `factors` follow the counted neighbours, in N, E, S, W order. `factor = base × multiplier × noise`, `multiplier` is the product of the exceptions' multipliers (1 when there are none), `score = tileWeight × Π factor` and `share = score / total`. A lone candidate has share 1. The drawn candidate is the one whose `tile` equals `chosen`.
- **Entry:** `entry` is the 1.1 table's `EntryView` for that pair, the same one `table_view` returns.

The frontend formats these numbers and never multiplies or divides them. When a revision differs from the one it holds, it refetches `table_view`, as it already does for mazes.

# Alternatives considered

- **Arms through `table_view`** (a one-row window in four directions). No new command, but the sort and the zero split end up in TypeScript, or in the 1.1 request as options for one view.
- **Naming the centre by index.** A shorter request, but a Reload can renumber the tiles under it. Identity is what DR-0026's entry point takes.
- **Recording every cell's trace in `generate_maze` and sending it.** Every Generate would pay for every candidate of every cell, hundreds of KB at 24 × 16, almost none of it ever read.
- **Caching the last maze's traces in tool state.** That state would need clearing on Reload and on every Generate, would hold one maze when 2.3 wants two side by side, and would tie the commands together.
- **A tracing copy of the solver, or `if tracing` branches inside it.** Two paths that can drift apart. The observer keeps it to one.

# Consequences

- 2.2's Rust is one hook in `wfc.rs`, one new core module, two commands and their types. `generate_maze` and every 2.1 seed's maze stay the same.
- 2.4 (watch it collapse) extends this observer, for propagation and contradictions, rather than adding a second instrumented path. 2.6's chunks can be traced the same way when their view needs it.
- Replay is only as good as determinism. DR-0028's same-seed tests and the identity test below guard it. Anything that makes generation depend on more than (seed, tile set, size) breaks `trace_cell` too.
- Two-way door, `major: false`: the IPC is tool-only and nothing persists. Reversing it means deleting two commands, a trait and a module.

# Concerns raised

- **Yew (`architect/1`), the author:** in practice `stale` only comes from a race, because Reload regenerates the shown seed ([DR-0029](/decisions/0029-generation-view-design.md)). A separate kind still lets the view regenerate or say so, instead of showing a trace of a different maze. The wording is Damson's, in DR-0033.
- **Reconciled with DR-0033 (Damson, `ux-designer/1`), through the facilitator:** four additions so the contract serves DR-0033's band.
  1. `settled` became `neighbours`: four entries, each counted, later or edge.
  2. Rust sorts the candidates.
  3. `ruledOut` gives the count. The ruled-out list may slip in DR-0033, so only the count is in the contract.
  4. `PairFactor.entry` carries the factor's `EntryView`, so DR-0025's panel opens on it unchanged.

  Yew agrees with all four. DR-0033's "drawn flag" is `tile === chosen`, so there is no separate field.

# Build plan

**Done when.** This is the technical half. The roadmap's 2.2 has no Done-when yet, and the slice-level one is Marram's to add.

1. **The explore view:** 1.2's own Done-when holds. Explore works for all 29 pairs in all four directions and agrees with the 1.1 table, checked by test.
2. **The trace:** any cell of a generated maze can be traced, and the identity, factor and agreement tests below pass.
3. **Errors:** `stale`, a cell outside the maze, a bad size, `not_loaded` and a failed maze each have a test, and none of them panics.
4. **Checks:** the bindings are regenerated, and the staleness test, clippy, the build and Vitest are all clean.
5. **Landing:** the DR-0033 view is built and Damson has reviewed it, the demo steps are in the README, and the slice is on `main` with a line in `/log.md`.

Rowan generates the bindings for the types above first, unchanged, so Ash's stub turns into the real import. After that they work in parallel ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)).

- **Rowan (`crates/maze-core`, `apps/tool/src-tauri`, `apps/tool/src/bindings/`):**
  - **`wfc.rs`, rule 5 only:**
    - What changes: `Wave::collapse` gains one parameter and one call after the draw, and the draw itself is unchanged. `Wave::run` passes the observer through. `generate`'s body moves into `generate_observed`, which adds the `attempt_started` call.
    - What stays: `Rules`, `Wave::new`, `restrict`, `propagate`, `choose`, `SplitMix64`, `attempt_seed`, `neighbour`, `Generation` and `GenerateError`. `Wave::new` is where DR-0031 rule 3's masks go, so it stays 2.6's to change.
    - If 2.6 lands first: its calls to `run` gain `&mut ()`.
  - **Rest of `maze-core`:** `trace.rs`, plus its exports in `lib.rs`.
  - **The app:**
    - `view.rs`: extract `entry_view`, add `explore`;
    - `maze.rs`: the trace mapping, with each factor's `entry` from `view::entry_view`;
    - `state.rs` and `lib.rs`: the two commands;
    - `ipc.rs`: the types, `Stale`, and their entries in the bindings list.
  - **Tests:**
    - **Identity:** for seeds 0 to 199 at 24 × 16, and a few seeds at 1 × 1 and 64 × 64, `generate` returns the same `Generation` as `generate_observed` with an observer that records every cell. The recorded pick for each cell equals `cells`, and every cell collapses exactly once, at steps 1 to width × height. Run the same check on DR-0028's always-contradicting fixture: the results are equal and failed, and only the last attempt is recorded.
    - **Factors:** for every cell of several seeds, `tileWeight × Π factor == score`, checked as exact f64 equality. Shares sum to 1 within 1e-9, candidates are in rule 6's order, the chosen tile is one of them, and `candidates.length + ruledOut` is the variant count. Each `counted` neighbour collapsed earlier and each `later` one collapsed later, both carrying the neighbour's final tile, and `edge` appears exactly where the cell is on the border. Each factor's `entry` equals `table_view`'s entry for that pair.
    - **Explore agrees with the table:** for all 29 pairs and four directions, `neighbours` plus `zeros` are exactly that row's `EntryView`s from `table_view`. `neighbours` all have weight above 0, in rule 3's order. `zeros` all have weight 0, in table order. `centreSide` is the centre's side.
    - **Trace matches the maze:** for a few seeds, every cell's `collapse.chosen` equals the cell `generate_maze` returned.
    - **Errors:** each of rule 7's cases. A failed maze's open cell gives `collapse: null`. An unknown explore pair is `bad_request`.
    - **JSON:** no NaN, so the only `null`s are `collapse` and `NeighbourAtDraw.tile` (corrected by the facilitator at Rowan's request, after A1).
- **Ash (`apps/tool/src`, not bindings):**
  - **IPC:** `ipc.ts` gets `exploreTile` and `traceCell`, and `COMMANDS` gets `explore_tile` and `trace_cell`.
  - **Views:**
    - the explore view to DR-0026, against a stub of the shapes above, then against the real command;
    - the link from the maze and the "why" display, to DR-0033;
    - `stale`, handled as DR-0033 says.
  - **Vitest:** arms render in the order given, with "+ N can't go here" from `zeros.length`. The trace display renders from a stub. The `stale` path is tested.

# Amendments

- **2026-11-01, [DR-0038](/decisions/0038-editing-core-toml-and-compare.md) (proposed; applies once it is accepted):** the trace gains opening pressure ([DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 9) as its own factor. `CandidateScore` gains `pressure`, and `CollapseStep` gains `pressure: &'a [f64]`, parallel to `scores`, which the trace passes through. Pressure is applied last, so the identity becomes `(tileWeight × Π factor) × pressure == score`, still exact, and the factor test runs at a pressure below 1 too. Nothing else in this contract changes.

# Links

- Explore view: [DR-0026](/decisions/0026-explore-one-tile-design.md); the explanation panel: [DR-0025](/decisions/0025-table-view-design.md)
- WFC, determinism and the scoring formula: [DR-0028](/decisions/0028-wfc-approach-for-2-1.md); the generation view: [DR-0029](/decisions/0029-generation-view-design.md)
- Chunks, which change `wfc.rs` in parallel: [DR-0031](/decisions/archive/0031-endless-generation-approach.md)
- IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md); the table view command: [DR-0024](/decisions/0024-tile-set-file-format.md)
- Weights: [DR-0022](/decisions/0022-weight-model.md); code layout, no I/O in core: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- Parallel work: [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md); two-way doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)
- Slice: [roadmap 2.2](/roadmap.md)
