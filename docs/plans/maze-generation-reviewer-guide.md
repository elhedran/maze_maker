---
type: Guide
title: Reviewer guide to the maze generation code
description: Yew's guide for Ian's own review of maze-core's generation code. The pipeline on one page with file:line references, a 60 to 90 minute reading order, the invariants and where they are enforced and tested, where to look hardest (including known gaps and debts), a Unity and threading lens, how to run it on Windows, and questions a reviewer might ask.
status: stable
owners: [architect/1]
tags: [guide, review, maze-core, wfc, infinite-world, growth, determinism, unity, threading]
generated:
  by: architect/1
  at: 2026-11-01T12:24:51Z
---

# Reviewer guide: maze generation in `maze-core`

For Ian. It's written to be read on its own, with the code open beside it. All references are to `main` at `846f3ad`, and I checked each `file:line` against that commit. Paths are from the repository root. The code is `crates/maze-core/src/`, about 4,400 lines, and two dependencies, `serde` and `toml`.

Keep your own steer in mind as you read (2026-11-01, verbatim): *"hopefully have a maze generation algorithm in that can be attached to that unity game running in its own thread and working with the unity game."* The game (G.1, `apps/game/`) is now the priority, and tool work only comes from issues the game finds. Section 5 reads the code with that in mind.

Records that shape it: [DR-0007](/decisions/0007-wave-function-collapse.md) (WFC), [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) (entries from sides), [DR-0022](/decisions/0022-weight-model.md) (weights), [DR-0024](/decisions/0024-tile-set-file-format.md) (file format), [DR-0028](/decisions/0028-wfc-approach-for-2-1.md) (the solver), [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md) (trace and observer), [DR-0035](/decisions/0035-the-world-is-stored.md) (stored world, a one-way door), [DR-0036](/decisions/0036-growth-from-open-endpoints.md) (growth, with its dated notes), and [DR-0038](/decisions/0038-editing-core-toml.md) (opening pressure in the single maze).

## 1. The pipeline on one page

```
data/tile-sets/core.toml
        │  TileSet::from_toml (parse, validate, completeness)        tile_set.rs:391
        ▼
TileSet ── variants (tile × distinct rotation, table order)          tile_set.rs:154
        │  Table::calculate: side rules, base × weights × exception × noise   table.rs:49
        ▼
Table (4 × n × n entries; n = 29 for core)                           table.rs:42
        │  Rules::new: flatten into pair[], supported[], walkable[]  wfc.rs:171
        ▼
┌───────────── one WFC run (Wave) ────────────────┐
│ with_start → propagate → loop{choose → collapse  │  wfc.rs:253, 322, 345, 369, 449
│ → propagate}; Observer sees each collapse        │
└───────────────┬─────────────────────┬───────────┘
   2.1 single maze│                     │ world patch (no observer)
 generate_observed │                    │ Solver::run(start, seed)        wfc.rs:556
 (restarts ≤ 20)   │                    ▼
   wfc.rs:481      ▼      World::patch: window → masks → tries (rule 6 check)
            trace_cell     → spine → fallback → keep flood              world.rs:558
            trace.rs:133                │
                                        ▼
                    World::grow_at: store kept cells, push the Patch log entry   world.rs:524
                    World::grow_around(player): repeat at nearest anchor ≤ r     world.rs:483
                    World::replay(anchors): rebuild from the growth log          world.rs:370
```

**Stage by stage**

| Stage | File | Key types and functions |
|---|---|---|
| Side rules | `side.rs` | `SideKind` (`:56`, hedge, passage, door, floor, floor with a wall at one end); `base_pair` (`:125`) is the only place the side rules live. `may_meet` (`:111`) wraps it. |
| Tile set file | `tile_set.rs` | `TileSet` (`:154`); `from_toml` (`:391`) runs every check and then completeness; `from_toml_allow_incomplete` (`:405`) is for test fixtures only. Opening pressure is checked to be in (0, 1] at `:418-421`. Rotations come from `distinct_rotations` (`:367`). |
| Table | `table.rs` | `Entry` (`:22`) keeps every part of each number, for the tool's "why". `calculate` (`:49`): a disallowed pair weighs 0, and an allowed one weighs base × row weight × column weight × exception × noise (`:97-101`). |
| Noise | `noise.rs` | `pair_hash` (`:53`): FNV-1a over an ordered pair of encoded sides, then the SplitMix64 finaliser. The order makes it symmetric. `factor` (`:64`) falls in [1 − a, 1 + a). |
| Completeness | `completeness.rs` | `is_fallback` (`:32`): weight above 0, sides only hedge or passage. `check` (`:58`): all 16 hedge/passage mixes exist, and every live tile pairs above 0 with every fallback tile that faces it with hedge to hedge or with passage to passage or door. The proof is in the module doc (`:1-24`). |
| WFC core | `wfc.rs` | `SplitMix64` (`:37`); `Rules` (`:153`); `Wave` (`:213`); `restrict` (`:298`), `propagate` (`:322`, FIFO), `choose` (`:345`, fewest candidates with a seeded tie-break), `collapse` (`:369`), `run` (`:449`). |
| Single maze (2.1) | `wfc.rs` | `generate` (`:468`) is `generate_observed` (`:481`) with `&mut ()`. Restarts use `attempt_seed` (`:68`), up to `MAX_RESTARTS = 20` (`:25`). |
| Trace (2.2) | `trace.rs` | `trace_cell` (`:133`) replays through `generate_observed` with a `Watch` observer (`:204`, impl `:211`). `collapse_trace` (`:226`) splits each score into weight × pair factors × pressure. |
| Masked run | `wfc.rs` | `Start` (`:516`: `mask`, `settled`, `outside`); `Solver` (`:541`) reads `Rules` once per world; `Solver::run` (`:556`) is one attempt, with restarts left to the caller. |
| World store | `world.rs` | `Cells` (`:134`): a `BTreeMap` of 32 × 32 `u16` pages, holding palette indices. `store` asserts a cell is never overwritten (`:165`). The palette is (tile id, orientation), not a table index (`:307-312`). |
| Growth | `world.rs` | `World` (`:300`); `new` (`:352`); `nearest_anchor` (`:462`, Chebyshev, ties in (y, x) order); `grow_around` (`:483`); `grow_at` (`:524`); `window` (`:541`); `patch` (`:558`). |
| Patch internals | `world.rs` | Edge masks for rule 1 (`:574-589`); the origin cell is masked to tiles with a walkable side (`:591-595`); the seed (`:599-609`); checked tries (`:628-641`); spine pins (`:645-666`); spine stage (`:667-673`); fallback (`:675-689`); `keep` (`:694`); `has_outward` (`:722`); `spine` (`:744`); `pin_open` (`:892`). |
| Log and replay | `world.rs` | `Anchor` (`:107`), `Patch` (`:114`, the log entry, which also lists the cells it stored); `replay` (`:370`); `is_anchor` (`:1009`), so a replay that diverges can say so. |
| Measures | `measures.rs`, `world_measures.rs` | `connected` (`measures.rs:40`), the one joining rule used everywhere; `count_areas` (`:48`). World measures: `frontier` (`world_measures.rs:48`), `rooms` (`:151`), `walk` (`:300`), `explore` (`:325`). |

**The growth model in five lines** (DR-0036's terms). A *stored* cell is in the table, and an *unsettled* one isn't. An *open endpoint* is a stored cell's walkable side facing an unsettled cell, its *anchor*. Each patch is one masked WFC run over a P × P window (P = 16) placed beyond the anchor. Stored cells in and just outside the window are pinned and count in support and score. Then only the new cells joined to the anchor are kept. Window edges that face unsettled cells may show only hedge, passage or door, so a stored room is always closed (rule 1).

## 2. Reading order (about 85 minutes)

| # | Minutes | What | Why |
|---|---|---|---|
| 1 | 10 | [DR-0036](/decisions/0036-growth-from-open-endpoints.md) Decision, rules 1-10, and its two dated notes in Consequences | The spec the code is checked against. Rules 4, 6 and 7 carry the guarantees. |
| 2 | 5 | `lib.rs` (whole), `side.rs:107-136`, `table.rs:49-114` | Where the numbers come from. Short. |
| 3 | 5 | `completeness.rs:1-98` | The fallback's no-contradiction proof rests here, checked at load. |
| 4 | 20 | `wfc.rs:152-464`, then `:513-567` | The solver: `Rules`, `Wave::with_start`, `restrict`/`propagate`, `choose`, `collapse` (read it closely, `:369-445`), `run`, `Solver`. |
| 5 | 30 | `world.rs:1-35` (the module doc and Rowan's narrowing), `:134-188`, `:300-365`, `:440-539`, then `:541-808` | The store, then growth, then `patch`, `keep`, `has_outward`, `spine`. The core of the review. |
| 6 | 10 | `tests/world.rs:45-126` (`assert_guarantees`), `:167-255`; `tests/world_view.rs:152-193` | What is actually asserted after every patch, and the brute-force frontier cross-check. |
| 7 | 5 | Run the measures (section 6) | Times and rates on your own machine. |

Optional, for another 15 minutes: `trace.rs:133-275` (how the tool explains a draw) and `world_measures.rs` (what the step 2 report measures).

## 3. Invariants, and where they are enforced and tested

**Determinism and seeding**
- *PRNG:* SplitMix64, written out by hand (`wfc.rs:37-65`), pinned against the reference values (`wfc.rs:576`). `rand` and `std` hashers are never used.
- *2.1 seeds:* `attempt_seed(seed, k) = seed << 32 | k` (`wfc.rs:68`). This is fine because SplitMix64 mixes on output.
- *World seeds:* `hash(&[TAG_RUN, world seed, ax, ay, attempt])` (`world.rs:599-606`), where `hash` (`:196`) folds the words through SplitMix64 and is pinned at `:909`. The spine's direction shuffle has its own tag (`:772-777`). The attempt counter runs across every stage, as rule 8 says.
- *Fixed order everywhere:* row-major cells, candidates in table order, FIFO propagation, directions N, E, S, W, and `BTreeMap` for the store, so every walk over it is in key order. No `HashMap` decides anything (I grepped; there are none in `src/`).
- *Floating point:* scores use only `+` and `×`. There are no transcendentals in `src/` (I grepped for `ln`, `exp`, `powf`, `sqrt` and the trig functions). Rust doesn't contract to FMA by default, so results should be bit-identical across x86-64 and ARM. Pressure is skipped with an exact `!= 1.0` (`wfc.rs:401`), so 2.1's scores stay bit for bit.
- *Tests:* `tests/seeds_pinned.rs:78` and `:85` pin 2.1's seeds, with and without the pressure key. `tests/world.rs:305` pins one grown world (seed 2026, walk 400: 671 cells, 10 patches). `tests/world.rs:257` checks that the same seed and walk give the same world, walked, scripted or replayed. `tests/trace.rs:93-117` checks that an observer changes nothing, over 200 seeds plus the extreme sizes. `tests/masks.rs:37` checks that an open `Start` gives `generate`'s maze.

**Order-independence: no, by design.** Each patch is a pure function of the world before it, its anchor, the tile set and the code (`world.rs:555-558`). The world is a function of (seed, tile set, code, walk). Two walks give two worlds, which is why DR-0035 stores cells, not a recipe. The seed doesn't include a tile set hash or a generator version. So a replay against a changed tile set can diverge silently wherever the old anchors still happen to be anchors. `is_anchor` only catches it when an anchor goes missing. That is harmless for saves, which hold cells, but worth knowing for replay and debug.

**Rule 1, the frontier.** Enforced by the edge masks (`world.rs:574-589`) and by `keep` (`:694`): a kept cell faces a discarded one only with hedge. Tested after every patch at `tests/world.rs:62-71`.

**Rule 5, stored cells never change.** Enforced structurally: stored cells are pinned in the mask (`wfc.rs:272-277`), and `Cells::store` panics on an overwrite (`world.rs:165`). Tested at `tests/world.rs:50-53` and `world.rs:941`.

**Rule 4, everything is joined to the origin.** `keep` floods from the anchor through `connected` (`world.rs:694-717`), and the anchor itself is always kept (`:703`). Since every anchor faces a stored walkable side, and support forbids hedge meeting a walkable side, induction gives one joined area. Tested at `tests/world.rs:74-94`.

**Rule 6, a way onward.** Checked tries: `has_outward` after each completed run (`world.rs:634-638`) floods unsettled cells from the rim of the box around the window and the stored bounds (`:722-735`, `Area::around` `:244`). The spine stage *is not re-checked*. It relies on the proof in the module doc (`:25-34`): the spine exits the pocket at a cell that is free outside it (`:768-769`), and nothing kept lies outside the pocket. Tested after every patch at `tests/world.rs:95-121`, with forced spines and fallbacks at `:219` and `:229`, and the brute-force frontier at `tests/world_view.rs:152` and `:167`.

**Completeness at load.** `TileSet::from_toml` runs `completeness::check` (`tile_set.rs:391-399`), and `World::empty` runs it again (`world.rs:333-336`), so a fixture loaded with `allow_incomplete` can't build a world. Tested in `tests/completeness.rs` (13 tests, `:57-260`) and `tests/world.rs:346`.

**What restart and fallback guarantee.**
- *2.1:* up to 21 attempts, then `failed: true` with the last attempt's partial cells (`wfc.rs:500-507`). No guarantee.
- *World:* at most (R + 1) × 21 + 21 + 1 = 106 runs per patch (`world.rs:628-689`). The last is the fallback, which can't contradict given completeness. If it ever did, `patch` panics (`:683-688`). Under that proof, `grow_around` always ends: each patch stores at least its anchor, and only the (2r + 1)² cells near the player can be anchors.

## 4. Where to look hardest

My honest view, in order of what worries me most for the game and for Unity.

1. **The O(box) rule 6 check, now that the game is here.** `has_outward` floods the whole box around everything stored, after *every* completed attempt, with a `BTreeMap` lookup per cell (`world.rs:722-735`). `frontier()` does the same, then collects and sorts every stored cell (`:966-990`). Don't let anything call it per frame. Today's run on my sandbox: 15.7 ns a box cell, so 1 ms at about 64k box cells. An explorer walk of 480 steps reaches 219k box cells and 3.4 ms a check, against about 1.1 ms for a median patch. The box only ever grows, and quadratically on a diagonal walk. **Inconsistency to resolve (mine):** DR-0036's note puts the incremental check in "the first game slice that grows a world". The G.1 roadmap on `claude/game-g1-walk` lists it under "not in it ... unless G.1's walks show growth is slow". The second contradicts the recorded trigger. I'll reconcile them with a dated note before G.1 lands.
2. **No hedge is ever stored (Wren's G.1 fun review, a known gap).** `keep` floods only through walkable joins, so the Hedge tile (weight 1.5) is drawn and then always discarded. A pocket of unsettled cells that no stored cell faces with a walkable side can never be reached by any patch. It is hedge in all but name, yet the world doesn't know it. Wren's review (`docs/plans/g1-fun-review.md` on the G.1 branch) asks two things:
   - *G1:* the game draws sealed pockets as hedge, as a view-time check.
   - *R3:* a record from me to store them as the Hedge tile.

   For a 3D game, R3 matters more, because the unknown must be a real wall there. Architecturally, R3 and the incremental rule 6 check are *one* data structure: labelled regions of unsettled cells around the stored area. I'd design them together in one record rather than bolt on two floods.
3. **`World::patch`'s control flow** (`world.rs:558-690`). There are two closures over a shared mutable `attempt`, a labelled `continue 'tries` (a run that completes but fails rule 6 abandons that try's remaining restarts, which is correct for rule 8), and `keep` computed twice on success (`:611`, `:634`). It's cheap but easy to misread. Check it against rule 8's order yourself. It's the densest 130 lines in the crate.
4. **`collapse` and the observer** (`wfc.rs:369-445`). Points to check:
   - Outside settled neighbours enter `settled` through `self.outside` (`:380`), so cells just past the window count in the score (rule 3).
   - The pressure test (`:395-400`) counts a side as closed when *any* settled cell faces it. Support guarantees that a settled neighbour facing a walkable side is walkable too.
   - The draw falls back to the last positive candidate when rounding pushes the target past the sum (`:417-428`), and a non-finite total is a contradiction (`:410`).
   - The observer gets shared borrows only, after the state change (`:436`).
   - **Gap:** world runs pass `&mut ()` (`:559`), so there is no "why is this here?" for world cells. G.3's debug mode will need a `Solver::run_observed`, which is a small, additive change.
   - `choose` is a linear scan, so a run is O(cells²). That's fine at 256 cells, and it's part of why P = 32 costs four times as much.
5. **The hedge seen, then opening (my DR-0036 concern 3).** An unsettled cell behaves as solid. A discarded cell may later be stored as a path from another side. In 2D the game draws unsettled cells as haze, not hedge ([DR-0044](/decisions/0044-walking-the-growing-world.md) point 5, on the G.1 branch), which keeps the promise honest. **In 3D at ground level** the risk changes shape. r = 8 (Chebyshev) assumes the player can't see further than 8 cells. A first-person player can see down a straight corridor further than that, to an anchor that hasn't grown. Unity will need growth driven by sight lines, or fog, or corridors limited by the weights. Note that for Q-0014.
6. **The seed-1 stall.** With some seeds, the random walker never gets within r of an anchor after the origin patch, so nothing grows. That's the walker (`world.rs:498`), not DR-0036: every anchor is on the joined area, so a steering player always reaches one. But it means `tests/world.rs:209`'s "100 walks × 1,000 steps" exercises fewer patches than it sounds. The minimum-growth check and the frontier-seeking walker are parked in 2.6 step 2b.
7. **Rooms shy of patch edges.** The edge mask forbids floor towards unsettled cells, so a room never crosses the outer edge of the patch that made it. Measured: 24% room share in a window's outer ring, against 41-44% further in. P = 32 halves the edge-touching rooms without moving the ring effect, at four times the time. So P stays 16, and a larger P is the recorded answer if seams show. Rooms are at most 16 × 16.

**Known debts and deferred items, with their triggers**
- **Incremental rule 6 check.** Trigger: the first game slice that grows a world, which is G.1 (see item 1).
- **Store sealed hedge (R3).** No record yet. Trigger: Wren's request. Best done with the item above.
- **Pressure in the world is 0.5, not 1.** `core.toml:28` sets one key, and both the single maze and the world read it (`wfc.rs:203`). Wren's step 2 call (in `docs/log.md`) was "the world's opening pressure stays at 1". That was before 2.3 set the shared key. The world tests strip the key (`tests/world.rs:21-26`), so the 100-walk guarantee test runs at 1, with one 12 × 300 run at 0.3 (`:243`). The guarantees don't depend on the value (it is above 0), but the step 2 measures describe a world at pressure 1. Wren should confirm 0.5 is wanted for the world.
- **The growth log keeps each patch's stored cells.** That's 27 KB per 1,000 cells, against 3.5 KB for the table, and replay needs only the anchors. Trigger: saving worlds (G.2, DR-0046 on the G.1 branch).
- **Panics as API.** `grow_at` on a bad anchor (`:527-531`), `replay` on a divergent log, settings asserts (`:328-332`), and the fallback (`:683`). Fine in Rust, but a C ABI must catch them (section 5).
- **`random_walk` and `random_step` repeat the step logic** (`:498-519`, `:817-832`). They're kept in step by `tests/world_view.rs:85`. DR-0043 says to leave them.
- **`World::new` trusts that `table` came from `set`** (`world.rs:352`; `Solver::new` states it as a precondition, `wfc.rs:546`). Nothing checks it.

## 5. The Unity and threading lens

**Already holds (checked today, not taken on trust)**
- *No globals or interior mutability.* Grepping `src/` for `static`, `thread_local`, `RefCell`, `Cell<`, `Rc`, `Arc`, `Mutex`, `OnceLock`, `lazy` and `unsafe` finds only `&'static str` return types. There is no `std::fs` and no `tauri` (lib.rs doc, `:7-8`). The dependencies are `serde` and `toml` only.
- *Send and Sync.* `World`, `TileSet`, `Table` and `Solver` are `Send + Sync + 'static`. I compiled a scratch assertion against this commit; it isn't in the repo. The tool also proves `Send` already, since it holds a `World` inside `tauri::State<Mutex<ToolState>>`. DR-0043 (G.1 branch) plans a compile-time `World: Send` assertion in `play.rs`. **It isn't built yet:** `origin/claude/game-g1-walk` at `b5f2bbf` has no `play.rs` and no `apps/game/` code.
- *No I/O.* `from_toml` takes `&str`, so the host reads the file.
- *Deterministic across machines,* per section 3.
- *A single-writer object with self-contained deltas.* Each `Patch` in `log()` lists the cells it stored, so a generator thread that owns the `World` can send the log entries made since the last request as its deltas, with no shared state.

**Doesn't hold yet (Q-0014, open on the G.1 branch, "revisit before the 3D game starts")**
- *API shape for a C ABI.* `Pos` is an `(i64, i64)` tuple. Results are `Vec<LoadError>`, borrowed `&Patch`, `impl Iterator` and `String` palette ids. A C ABI wants an opaque handle, plain `#[repr(C)]` structs, caller-owned buffers, and error codes.
- *Panics.* They must not cross `extern "C"`. Wrap each entry point in `catch_unwind`, or turn the asserts into `Result`s first.
- *No budget or cancellation.* `grow_around` runs to completion. A typical patch takes about 1 ms. My estimated forced worst case is 355 ms (the measures' figure). Unity will want "grow up to N patches or T ms", returning whether more is pending. The deferred incremental check bounds the tail.
- *Read while growing.* `get` needs `&World`, so a renderer on another thread either takes a lock or keeps its own copy built from the deltas. The deltas make the copy cheap. Pick one in Q-0014.
- *Growth radius against sight* (item 5 above).

My recommendation for when Q-0014 opens: a thin `maze-ffi` crate (`cdylib`) over an unchanged `maze-core`, with one owned `World` per handle on a worker thread, a request queue of player positions in, and patch deltas out. Nothing in `maze-core` blocks that today.

## 6. How to run it (Windows, PowerShell, from the repository root)

```powershell
cargo test -p maze-core --release           # 116 tests; world.rs's walks take a few seconds
cargo clippy -p maze-core --all-targets -- -D warnings
cargo run --release -p maze-core --example measures            # the 2.6 step 2 report, ~30 s
cargo run --release -p maze-core --example measures -- --walks 20 --steps 80 --seed 7
cargo run --release -p maze-core --example tuning              # 2.3's pressure tables, Markdown
cargo test -p maze-core --test world -- --nocapture            # prints patch, spine and fallback counts
```

Debug-build tests are noticeably slower on the 100-walk test. Use `--release` for review.

**The tool** (`cd apps\tool; npm install; npm run tauri dev`, then maximise the window):
- **Maze tab** (2.1, 2.2, 2.3): **Generate** from a seed, click a cell, then **Why is this here?**. The Explore band shows weight × factors × pressure equal to the score. The opening-pressure number under the seed can be set to `1` to compare against 0.5.
- **World tab** (2.6): World seed **2026**, Walk seed **7**, **New world**, **Play**. Turn on **Patch debug (D)** to see each patch's anchor, window, attempts, spine and fallback. Turn off **Stored (S)** to see unsettled cells as solid. **Replay walk** rebuilds the world from the growth log and reports "same world as walked: yes". Some walk seeds never reach an anchor (item 6).

## 7. Questions a reviewer might ask

- **Why no backtracking into stored cells?** Rule 5. Repair happens only inside the patch, through restarts and then the fallback, and rule 1 makes it unnecessary. See DR-0036's alternatives.
- **Why is the fallback safe against arbitrary stored neighbours?** All 16 hedge/passage mixes exist, so each cell edge can settle on its own. Clause 2 makes every such meeting weigh above 0. Stored cells show only hedge, passage or door to unsettled cells. See `completeness.rs:1-24`.
- **What if windows overlap, or several stored cells face one anchor?** Stored cells in the window are pinned. `anchor_at` (`world.rs:447`) takes the first of N, E, S, W only to place the window. The others constrain the run through `settled`.
- **Can a patch store nothing?** No. `kept[a] = true` (`world.rs:703`). The anchor can't be the Hedge tile, because it faces a walkable side.
- **Is the spine's route deterministic?** Yes. It's a BFS through the anchor's pocket, with the direction order shuffled by a SplitMix64 seeded from (world seed, anchor) (`world.rs:771-787`).
- **What does a reload do to a live world?** Nothing. `World` owns a clone of its `TileSet` (`:338`), and cells store (tile id, orientation) through the palette, so table order can shift without harm.
- **Why `u16` pages of 32 × 32?** 2 KB a page, sorted keys for deterministic walks, and negative coordinates via `div_euclid` (`world.rs:144-149`, tested at `:917`). 3.5 KB per 1,000 stored cells at 58% page fill.
- **Why does `i64` become `i32` at IPC?** DR-0023 bans 64-bit integers in TypeScript. The conversion is checked (`i32::try_from`), and an overflow is an error no walk can reach.
- **How fast is it?** Median about 1.1 ms a patch at P = 16, and p99 about 2.5 ms (my run). Wren's G.1 walks: p95 at most 1.1 ms a step, worst 23 ms.
- **Is "outward" exact?** Yes, within the box. The rim of `Area::around` is all unsettled and lies outside everything stored, so reaching the rim is reaching the unbounded region. `tests/world_view.rs:152` cross-checks it against a brute-force flood.
- **What is a one-way door here?** Only DR-0035, the stored world. DR-0036 stays a two-way door after saves, except that loosening rule 1 later would need a migration of saved frontiers.
