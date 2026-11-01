---
type: Decision Record
title: The data contract for the world view
description: Four world commands in a new src-tauri module, with world state in its own managed state that keeps the tile set the world was grown with; per-patch deltas (log entry, stored tiles, spine and discarded cells), the frontier with outward flags, and replay streamed patch by patch from the growth log with Rust's same-world check; positions as i32, so plain numbers; maze-core gains accessors only. It also stands as slice 2.6 step 3's technical plan.
status: draft
decision_id: DR-0041
decision_status: proposed
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [engineer-rowan/1, engineer-ash/1]   # architect/1 is the author; one engineer approves and the other may stand in for the architect (DR-0018), as for DR-0034
supersedes: []
tags: [world-view, ipc, growth, frontier, replay, determinism, slice-2-6, plan]
generated:
  by: architect/1
  at: 2026-11-01T15:10:00Z
verified:
  - by: engineer-rowan/1
    at: 2026-11-01T15:40:00Z
    note: "Approves as the approving engineer; building the Rust side against it. One concern, below, not blocking: replay must not panic when the replayed world diverges, so core gains a sixth additive accessor, is_anchor, and a divergent replay ends early with check.same false."
---

# Context

Slice 2.6 step 3 ([roadmap](/roadmap.md)) puts [DR-0036](/decisions/0036-growth-from-open-endpoints.md)'s grown world on screen, in the World tab Damson designs in [DR-0039](/decisions/0039-world-view-design.md). Step 1's engine is on `main` in `crates/maze-core/src/world.rs`. This record is the IPC between them, as [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md) was for 2.2.

What the view needs (DR-0039, "What the view needs from Rust", with Marram's four scope cuts applied): start a world from a seed; walk one step by key, or a seeded random walk a step at a time (Play, Pause, Step); get back each step's patch deltas, the frontier with outward flags, and the player; replay the growth log into a fresh world, patch by patch (Play, Pause, Step forward), ending in Rust's "same world as walked: yes/no"; and after a Reload, keep the world on the tile set it was grown with.

Ash's notes from his DR-0036 review: positions as `number`, not `bigint`; a public frontier accessor with outward flags; per-patch deltas, not the whole world each step; debug data for the patch (window, spine cells, discarded cells, attempts, flags); replay from the growth log.

A parallel session on 2.3 may touch `wfc.rs`, the tile set code and `apps/tool/src/`. So `maze-core` changes are additions to `world.rs` only, and the app's world code goes in new files.

# Decision

## Where the world lives

1. **Its own managed state.** A new module, `apps/tool/src-tauri/src/world.rs`, holds `WorldState` (plain Rust, tested like `ToolState`) and the four `#[tauri::command]`s. `lib.rs` adds `.manage(Mutex::new(WorldState::default()))` and registers the commands; `state.rs` doesn't change.
2. **The world keeps the tile set it was grown with** (DR-0039 rule 12; technically sound). `world_start` clones the loaded `TileSet`, `Table`, `revision` and `loadedAt` into the session, under a short lock of `ToolState` that is released before `WorldState` is locked. No command holds both locks. After that, no world command reads `ToolState`, so a Reload, good or bad, never touches the world or its replay. The cost is one cloned set and table per world, a few hundred KB.
3. **Tile indices in world data index the world's own tile list.** `world_start` returns `tiles`, the `TileView`s of the set it cloned, in variant order (table order at that revision). The view draws world cells from that list and never from `table_view`, so a Reload can't renumber them. This is the IPC view of DR-0035's palette: the palette is how cells are stored, and this list is how they are named over the wire.
4. **One world at a time, with an id.** Each `world_start` replaces the session and gets a new `worldId` (a `u32` counter). Every other request names it. A request for a world that isn't there (none started, or replaced since) is `stale`, with the message "That world has been replaced. Press New world." So a walk in flight when New world is pressed can't land on the new world. No new error kinds.

## Walking

5. **The app drives growth one patch at a time,** so each patch's delta, discarded cells included, is captured as it is made. To start: an empty world (`World::replay` with no anchors), then the origin patch, then `nearest_anchor` and grow until none is within r. That is what `World::new` and `grow_around` do, and a test holds them equal.
6. **A key step** moves the player to the neighbour in that direction if it is stored and joined. If it isn't, nothing moves, nothing grows and it isn't an error: `path` comes back empty, and the view writes "Hedge to the north" from the tile's side.
7. **A seeded walk** keeps its PRNG in the session. If the request's walk seed equals the session's, the stream carries on; otherwise a new one starts from the player's cell (DR-0039 rule 3). So Play at one step per call makes the same walk as one call of N steps, and the same walk as `maze-core`'s `random_walk` from the same cell and seed. Each step chooses with `World::random_step` (below), so the choice is defined once.
8. **Click to walk is cut** (Marram, DR-0039). If review brings it back, it is one more `Walk` variant, `to`, with the shortest path found in Rust over joined stored cells. The view doesn't learn adjacency rules.
9. **Limits:** a seeded walk takes 1 to 1,000 steps per call, and a world takes at most 1,000,000 steps in all. Anything else is `bad_request`. The commands are `#[tauri::command(async)]`, so a long walk doesn't block the window; nothing is awaited while a lock is held.

## Positions are `i32` over IPC

10. **Core keeps `Pos = (i64, i64)`; IPC types use `i32`,** which ts-rs writes as `number`. That meets Ash's note and keeps [DR-0023](/decisions/0023-rust-typescript-types-in-step.md)'s "no 64-bit integers in IPC types" whole, with no `#[ts(type)]` override to drift from the Rust type. **Why it's safe:** each step moves the player one cell, and growth reaches at most r + P past the player, so with rule 9's cap no stored cell, window or anchor lies more than 1,000,000 + 8 + 16 cells from the origin, far inside `i32`. The conversion is `i32::try_from`, mapped to `bad_request` rather than a panic, and never fires in practice. Requests widen `i32` to `i64` losslessly.

## Replay

11. **Streamed from the growth log, patch by patch.** `world_replay_start` builds a fresh empty world from the session's cloned set, table and seed, and keeps it beside the walked one. Each `world_replay_step` grows the next `patches` anchors from the walked world's log (clamped to what's left) through the same per-patch path as rule 5, so a replayed delta has the same shape as a walked one. The view asks for one per tick, or a large number to jump to the end. There is no step back (cut); a later step back is the view showing deltas 1 to k from its own list, with no contract change.
12. **The check is Rust's.** When the last patch is replayed, the response carries `check`: `same` is true when both worlds hold the same cells with the same tiles and the replayed log, discarded cells included, equals the walked one. `cells` is how many stored cells matched, and `firstDifference` is the first cell, in (y, x) order, where they differ, or `null`. Leaving replay ("Back to walk") is the view's; the walked world was never touched. The replay world is dropped on the next `world_replay_start` or `world_start`.

## What `maze-core` gains (additions to `world.rs` only)

13. ```rust
    /// An open endpoint (DR-0036 terms): stored `cell`'s walkable side facing `facing`,
    /// whose anchor is `step(cell, facing)`.
    pub struct Endpoint { pub cell: Pos, pub facing: Direction, pub outward: bool }

    impl World {
        /// Every open endpoint, sorted by (y, x) of `cell`, then N, E, S, W.
        /// `outward` by DR-0036 rule 6's reference flood, once over the stored bounds.
        pub fn frontier(&self) -> Vec<Endpoint>;
        /// The stored cells' bounding box, (min, max) inclusive.
        pub fn bounds(&self) -> Option<(Pos, Pos)>;
        /// Both cells stored and joined across `d`.
        pub fn joined(&self, from: Pos, d: Direction) -> bool;
        /// One step of `random_walk`'s choice from `at`, drawing from `rng` exactly as it does.
        pub fn random_step(&self, at: Pos, rng: &mut SplitMix64) -> Pos;
        /// `grow_at`, returning the patch's discarded cells: its window's cells still
        /// unsettled after it, in row-major order (DR-0036 rule 4).
        pub fn grow_with_discards(&mut self, anchor: Anchor) -> Vec<Pos>;
    }
    ```

    Nothing existing changes signature or behaviour. `random_walk` keeps its own body; a test holds `random_step` to it.

## IPC contract (DR-0023 types)

In a new file, `apps/tool/src-tauri/src/ipc/world.rs`, declared from `ipc.rs` with one `pub mod world; pub use world::*;`, and added to the bindings test's list. Bindings are one file per type, so they are all new files.

- `world_start(request: WorldStartRequest) -> Result<WorldStarted, ToolError>`
- `world_walk(request: WalkRequest) -> Result<WorldUpdate, ToolError>`
- `world_replay_start(request: WorldRef) -> Result<ReplayUpdate, ToolError>`
- `world_replay_step(request: ReplayStepRequest) -> Result<ReplayUpdate, ToolError>`

The TypeScript the bindings will have, without doc comments, so Ash can stub against it now:

```ts
export type WorldPos = { x: number, y: number, };
export type WorldRect = { x: number, y: number, width: number, height: number, };

export type WorldStartRequest = { seed: number, };
export type WorldRef = { worldId: number, };
export type WalkRequest = { worldId: number, walk: Walk, };
export type Walk = { "kind": "step", direction: Direction } | { "kind": "random", seed: number, steps: number };
export type ReplayStepRequest = { worldId: number, patches: number, };

export type WorldStarted = { worldId: number, seed: number, revision: number, loadedAt: string, tiles: Array<TileView>, update: WorldUpdate, };
export type WorldUpdate = { worldId: number, player: WorldPos, path: Array<WorldPos>, patches: Array<PatchDelta>, frontier: Array<Endpoint>, bounds: WorldRect, steps: number, patchCount: number, storedCount: number, };
export type PatchDelta = { index: number, step: number, anchor: WorldPos, from: Direction | null, window: WorldRect, attempts: number, spine: boolean, fallback: boolean, cells: Array<StoredCell>, spineCells: Array<WorldPos>, discarded: Array<WorldPos>, };
export type StoredCell = { x: number, y: number, tile: number, };
export type Endpoint = { cell: WorldPos, facing: Direction, anchor: WorldPos, outward: boolean, };
export type ReplayUpdate = { worldId: number, patches: Array<PatchDelta>, frontier: Array<Endpoint>, bounds: WorldRect | null, replayed: number, total: number, check: ReplayCheck | null, };
export type ReplayCheck = { same: boolean, cells: number, firstDifference: WorldPos | null, };
```

What the fields mean:

- **`tile`** indexes `WorldStarted.tiles`. `revision` and `loadedAt` say which load the world was grown with, for DR-0039's "grown with the tile set loaded 20:14".
- **`path`** is the cells this request walked, in order, after the starting cell: empty for `world_start` and for a blocked key step. `player` is where the walker stands after it. `steps` and `patchCount` are the world's totals so far.
- **`patches`** are this request's patches, in growth order. `index` is the patch's place in the growth log, from 0, so DR-0039's "patch 37" is `index + 1`. `step` is the walk step after which it grew, 0 for `world_start`'s. `from` is null only for the origin patch. `window` is P × P with its north-west corner at `x, y`. `cells` are the stored cells, in the window's row-major order. `spineCells` run from the anchor out, empty without a spine. `discarded` are window cells the run filled and rule 4 didn't keep.
- **`frontier`** is the whole frontier after the request's last patch, not a delta: it changes far from the patch when a ring closes (DR-0036 rule 6). With Play at one step per call it is nearly per patch. Several patches in one step share the end frontier; that is accepted.
- **`bounds`** is the stored cells' box, for panning. `ReplayUpdate.bounds` is null until the first replayed patch.
- **Replay:** `replayed` of `total` patches are done; `check` is null until `replayed == total`.
- **Spine and discarded cells stay in the contract** though DR-0039's drawing of them is cut. They cost one accessor and a few hundred cells a patch, the replay check uses the discards anyway, and the overlay can come back without a contract change.

**Errors**, checked before any growth; none panics:

- `world_start` with no tile set loaded: the existing `not_loaded`.
- an unknown or replaced `worldId`, or `world_replay_step` with no replay started: `stale`.
- seeded walk steps outside 1 to 1,000, the 1,000,000-step cap, `patches` of 0, or a position out of `i32`: `bad_request`, with a readable message.
- a set that fails the completeness check can't be loaded by the tool (`from_toml` refuses it), but if `World` refuses one anyway its messages come back as `bad_request`.

# Alternatives considered

- **Invalidate the world on Reload** (the facilitator's first guess). Simpler state, but DR-0039 wants the world kept, and Ian's 2.6 demo changes `core.toml` around a live world. Cloning the set makes keeping it cheap and removes `stale`-by-revision from every world command.
- **`i64` with `#[ts(type = "number")]`** (Ash's wording). Same wire shape, but it carves an exception into DR-0023 and lets the TypeScript type say `number` for a value Rust could send past 2^53. `i32` with a cap makes the bound true by construction.
- **The whole world in every response.** Simple for the view, but O(world) per step at 25 ms ticks. Deltas are O(patch).
- **Replay as one call returning every delta.** One round trip, but per-patch frontiers would cost O(patches × area) in one go, and the view animates one patch per tick anyway.
- **Driving growth with `grow_around` and diffing the log.** No core addition for discards, but a step's earlier patches lose their discards to later ones. `grow_with_discards` is five lines.
- **The world inside `ToolState`.** One lock, but it edits `state.rs`, which 2.3 also edits, and ties world lifetime to Reload.

# Consequences

- 2.6 step 3's Rust: five accessors added to `world.rs`, one new app module, one new IPC file, four commands. `wfc.rs`, the tile set code, `state.rs`, `view.rs`'s behaviour and every existing command stay as they are. `view.rs` may gain a `tile_views(set)` helper split out of `table_view`, with the same output.
- No new dependencies, so no lockfile change.
- The reference frontier flood is O(stored bounds) per world command. Fine for the tool's walks; DR-0036's incremental check, if step 2 calls for it, slots in behind `frontier()`.
- Replay proves "same seed and log, same world" in the tool. Anything that makes a patch depend on more than (world before it, anchor, tile set, code) fails the check on screen.
- **Two-way door,** `major: false`. The IPC is tool-only and nothing persists. Reversing it means deleting a module, an IPC file, four commands and five accessors.

# Concerns raised

- **Yew (`architect/1`), the author, departing from Ash's note on positions:** Ash asked for `i64` typed as `number`; this record uses `i32` with a step cap instead (rule 10). Same TypeScript, and DR-0023 stays whole. It is my call as architect, and a two-way door: switching to `i64` later changes no binding.
- **Yew, on the frontier per request:** several patches from one step share the end frontier. If Wren and Damson's review wants the frontier after every patch, `PatchDelta` gains a `frontier` field, at O(area) per patch.
- **Rowan (`engineer-rowan/1`), on a replay that diverges:** `World::grow_at` panics when its anchor isn't an anchor of the world before it. If determinism ever breaks, the replayed world can differ before the log ends, and replaying the next walked anchor would panic in the command instead of saying "no" on screen, which is the failure the check exists to show. So core gains a sixth accessor, `is_anchor(anchor) -> bool` (additive), and `world_replay_step` checks each anchor first. On a mismatch, replay ends there: the response carries `check` with `same: false` and the first differing cell, even though `replayed < total`. The view should treat a non-null `check` as the end of replay, not `replayed == total`. Also, `firstDifference` needs a sort: `World::cells()` walks pages in key order, which isn't (y, x) order across pages.
- **Ash (`engineer-ash/1`), on positions and the view's build:** `i32` with the 1,000,000-step cap is better than my `i64` override: the TypeScript type can't lie about its range. The cap is the tool's, not the game's; the game will need its own answer for unbounded walks, so don't carry this cap into the game's IPC without a fresh look. Two build notes, both two-way doors: the build plan puts the four command wrappers in `ipc.ts`, but I keep them in `apps/tool/src/world/world-ipc.ts` with their own `WORLD_COMMANDS`, because the parallel 2.3 session edits `ipc.ts`; fold them in once 2.3 lands if we want one list. And the view ends replay on a non-null `check`, per Rowan's concern, not on `replayed == total`.
- **Marram's scope cuts (DR-0039)** are taken: no click to walk, no replay step back, no drawn spine or discards. The contract still carries spine and discarded cells (above).

# Build plan

**Done when** (the technical half; the slice's demo is DR-0039's six tasks, less the click):

1. **Core, in `crates/maze-core/src/world.rs` tests:**
   - an empty world, the origin patch, then `nearest_anchor` and `grow_with_discards` until none is within r, equals `World::new` (cells and log), for 20 seeds;
   - `random_step` driven from the same seed gives the same path and world as `random_walk`, for 20 seeds of 500 steps;
   - `frontier()` equals a brute-force list (every stored cell's walkable side facing an unsettled cell), in its order, and each `outward` equals a separate flood from that anchor; after every patch of the 100-walk test, at least one endpoint is outward;
   - `bounds()` is the min and max of `cells()`; `joined` agrees with `measures::connected`;
   - each patch's discards are disjoint from every stored cell, and discards, its stored cells and the cells stored before it fill its window exactly.
2. **App, in `src-tauri/src/world.rs` tests:**
   - `world_start`: the deltas' cells rebuild the world's cells exactly; the first delta is anchored at (0, 0) with `from` null and step 0; the player is at (0, 0);
   - key steps: a blocked step moves nothing and grows nothing; a legal one moves one joined cell;
   - seeded walk: 200 calls of 1 step equal one call of 200, and equal `random_walk` from the same cell and seed; the same two seeds in a fresh world give identical updates;
   - replay: stepping one patch at a time, and in one jump, both end with `check.same` true and `cells` equal to `storedCount`, and the replayed deltas equal the concatenation of every walked update's deltas; a world tampered in the test gives `same: false` and the right `firstDifference`;
   - a Reload between start and walk, with a changed `core.toml`, leaves the world walking and replaying on its own set and tiles;
   - every error case above, and none panics;
   - JSON: the only nulls are `from`, `ReplayUpdate.bounds`, `check` and `firstDifference`.
3. **Checks:** the bindings are regenerated; the staleness test, clippy, the build and Vitest are clean.
4. **Landing:** the DR-0039 view is built against the real commands, Wren and Damson have reviewed it, the demo steps are in the README, and the slice is on `main` with a line in `/log.md`.

**Who does what** ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)), each in their own worktree under `.worktrees/`, which the facilitator creates from `main`. Neither changes a lockfile.

- **Rowan, `rowan/2.6-world-ipc`** (`crates/maze-core/src/world.rs`, `apps/tool/src-tauri/`, `apps/tool/src/bindings/`): first the IPC types and their bindings, exactly as above, pushed to Ash early; then the core additions, `WorldState` and the commands, and the tests in 1 and 2.
- **Ash, `ash/2.6-world-view`** (`apps/tool/src/`, new files where possible: `world-view.ts` and its test): `ipc.ts` gains `worldStart`, `worldWalk`, `worldReplayStart`, `worldReplayStep` and their `COMMANDS` entries; the view to DR-0039 with Marram's cuts, first against a stub of the shapes above, then the real bindings. Vitest: deltas draw in order one per tick; a blocked step says what blocked it; replay shows "patch k of n" and the same-world line from `check`; `stale` and `bad_request` go to the shared banner and keep the last good world.

# Links

- The view: [DR-0039](/decisions/0039-world-view-design.md)
- Growth and the frontier: [DR-0036](/decisions/0036-growth-from-open-endpoints.md); the stored world and its palette: [DR-0035](/decisions/0035-the-world-is-stored.md)
- The model for this record: [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)
- IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md); glyphs and the maze view: [DR-0029](/decisions/0029-generation-view-design.md), [DR-0033](/decisions/0033-why-is-this-here-design.md)
- Parallel work and the remote: [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md), [DR-0032](/decisions/0032-use-the-remote.md); two-way doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)
- Slice: [roadmap 2.6](/roadmap.md)
