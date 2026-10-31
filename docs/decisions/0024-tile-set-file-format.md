---
type: Decision Record
title: The tile set file is TOML in data/, reloaded by the tool on request
description: The tile set, side-pair bases, tile weights, exceptions and seeded noise live in one TOML file under data/tile-sets/, parsed and validated by maze-core, read and reloaded by the tool; the table view command takes filters and a row and column window and returns entries with probabilities and their explanations.
status: stable
decision_id: DR-0024
decision_status: accepted
decided_at: 2026-10-31T19:37:29+01:00
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [architect/1, engineer-ash/1]
supersedes: []
tags: [tile-set, data-format, toml, ipc, adjacency-table, noise, phase-1]
generated:
  by: architect/1
  at: 2026-10-31T19:35:00+01:00
verified:
  - by: engineer-ash/1
    at: 2026-10-31T19:37:29+01:00
---

**Revised 2026-10-31** by Yew (architect): seeded noise added for 1.1, and Ash's review gaps folded into the view command. Ash's approval is cleared, since the substance changed; his review is kept under Concerns raised.

# Context

Slice 1.1 builds the table in Rust from the tile set data. The roadmap's test: "changing a probability needs no code change"; until slice 2.3, Ian edits the data and the tool reloads it ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). The file must hold [DR-0015](/decisions/0015-tile-set.md)'s tiles and [DR-0022](/decisions/0022-weight-model.md)'s side-pair bases, tile weights and exceptions, and leave room for a stored seed. [DR-0012](/decisions/0012-code-layout-and-core-crate.md) put shared data in `data/` and forbids I/O in `maze-core`. DR-0015 left tile IDs and which way rotation 0 faces to this record.

# Decision

## Format and place

- **TOML**, one file per tile set: `data/tile-sets/core.toml` for DR-0015's core set. It is written by hand, so comments matter, which rules out JSON. TOML is more familiar to a non-Rust reader than RON.
- **`maze-core` parses and validates a `&str`** (`TileSet::from_toml(&str) -> Result<TileSet, Vec<LoadError>>`) and calculates the table. It never opens the file.
- **New dependencies in `maze-core`:** `serde` (derive) and `toml`. Both are already in `Cargo.lock` through Tauri, do no I/O when parsing a string, and need no OS features. This is the record DR-0012 asks for. Lockfile change: same branch as [DR-0023](/decisions/0023-rust-typescript-types-in-step.md)'s ts-rs.
- **Unknown keys are errors** (`deny_unknown_fields`), so a typo fails loudly rather than being ignored.

## What the file says

```toml
format = 1                 # bump on breaking changes
name = "core"

# Seeded variation on every allowed entry (optional; leave out or set amount = 0 for none).
[noise]
seed = 1234                # u32
amount = 0.15              # each allowed entry gets a factor in [1 - amount, 1 + amount]

# One base per pair of side kinds that may meet (DR-0015). All six are required;
# a base for a pair that may not meet is an error. Bases ignore the wall's end.
[bases]
hedge_hedge       = 1.0
passage_passage   = 1.0
passage_door      = 1.0
door_door         = 0.2
floor_floor       = 1.0
floor_wall_floor_wall = 1.0

[[tile]]
id = "corridor_corner"     # the tile type; snake_case, unique
name = "Corridor corner"   # shown in the tool
kind = "corridor"          # corridor | room | hedge
tags = []
sides = ["passage", "passage", "hedge", "hedge"]   # N, E, S, W at rotation 0
orientations = 4           # checked against the distinct rotations
weight = 1.0

[[tile]]
id = "room_doorway"
name = "Room doorway"
kind = "room"
tags = ["room_edge"]
sides = ["door", "floor_wall_n", "floor", "floor_wall_n"]
orientations = 4
weight = 0.3

[[exception]]
name = "Doorway rarely faces a dead end"
a = { tile = "room_doorway", side = "door" }        # tile = <id> or tag = <tag>
b = { tile = "corridor_dead_end", side = "passage" }
multiplier = 0.1
```

- **Side kinds** in `sides` are `hedge`, `passage`, `door`, `floor`, and `floor_wall_n|e|s|w` (DR-0015's F·wN etc.). Which kinds may meet is code in `maze-core`, not data: these are DR-0015's invariants, and they don't change with probabilities. Adding a side kind is a tile set change with its own record anyway.
- **Rotation:** rotation 0 is the sides as written, N, E, S, W. Orientation `k` is `k` clockwise quarter turns, which moves each side one direction on and turns a wall end with it (`floor_wall_n` becomes `floor_wall_e`). Only distinct rotations are kept, lowest `k` first, so orientation numbers match [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)'s glyph order. A (tile, orientation) is identified by `id` and `k`.
- **Table order** is file order of tiles, then `k`. Write the file in kind order with fallbacks last within their kind (DR-0017 rule 8).
- **Exceptions** match on tile `id` or `tag` on each side, plus the side kind each side presents, with `floor_wall` matching any wall end. They apply both ways (DR-0022). Validation is DR-0022 condition 3; all validation errors are reported together, each naming the tile or exception.
- **Noise** (optional `[noise]` table): each allowed entry gets a factor in `[1 - amount, 1 + amount]` from a stable hash of `seed` and the unordered pair of the two meeting sides, each as (tile `id`, orientation `k`, the side's direction). Using the direction rather than the side kind keeps "B east of A" and "B west of A" distinct. It is symmetric by construction, and a 0 stays 0 because the factor multiplies. `amount` must be in `[0, 1)`. The hash is written out in `maze-core` (no `std` hasher, which isn't promised stable, and no dependency) and pinned by a test against known values, so the same file gives the same table on any machine and any Rust version. `core.toml` ships with a seed set; Wren tunes `amount`.
- **Zones** later add sections; nothing for them now.

## Noise departs from DR-0022's default

[DR-0022](/decisions/0022-weight-model.md) defaults to "Variation is set by hand in the tile set data (Wren), not generated noise, until review shows a need", and its condition 4 provides for noise: "If seeded noise is ever used, its seed is stored in the tile set data." This record takes that provision for 1.1, as Yew's call on a two-way door. The reason: 1.1 is done when "every cell shows its probability, variation included", and Ian (`human:ian`) said, verbatim, "I want to see the variation in the table of probabilities" and "some randomness for to add flavour". Tile weights and a few exceptions vary rows and blocks, not cells, so hand-set variation alone wouldn't show what he asked for. The DR-0013 note that reading his point 5 as noise "is not part of Ian's approval" still holds: if Ian meant something else, `amount = 0` turns it off with no code change. The factor is its own line in every explanation, so it stays explainable and separable from what someone chose.

## Loading and reloading

- The tool reads `data/tile-sets/core.toml` from the repository (path built from `CARGO_MANIFEST_DIR` at compile time). Phase 1 runs with `tauri dev` only; bundling the file as a Tauri resource waits for the first release build.
- Loaded state (tile set, table, a `revision` counter) sits in Tauri managed state. Startup failure doesn't stop the app: the view command returns the load errors so Ian sees them.
- **`reload_tile_set`** re-reads and validates the file. On success it replaces the table and bumps `revision`; on failure it keeps the last good table and returns the errors. The tool has a Reload button. No file watcher in 1.1 (it would add a dependency); a later slice can add one.

## The view command (slice 1.1)

`table_view(request: TableViewRequest) -> Result<TableView, ToolError>`, types per DR-0023. In brief:

- **Request:** `rowFilter` and `columnFilter` (each: `kinds`, `tileIds`, `tags`; empty means any; a tile must match every non-empty field), `directions` (empty means all four), and a window `rows: {start, count}`, `columns: {start, count}` over the filtered rows and columns. 1.1 asks for everything. 1.3's filter UI and 1.5's paging use the same fields.
- **Response:** `revision`; `tileSetPath` and `loadedAt` (ISO 8601 text); `tiles` (every (tile, orientation) in table order: id, name, kind, tags, orientation, sides, weight); `totalRows`, `totalColumns` after filtering; `rows` and `columns` as indices into `tiles`; `exceptions` (name, multiplier); and `entries`, one per direction × row × column in the window.
- **Each entry:** `direction`, `row`, `column`; `facing` (the two side kinds that meet, wall ends included); `allowed`; `base`, `rowWeight`, `columnWeight`, `exceptions` (indices into `exceptions`); `noise` (the factor, 1 when there is no noise); `weight` (base × both tile weights × exceptions × noise); `rowTotal`; `probability`. A 0 has `allowed: false`, `facing` says which sides clash, and `mayMeet` lists the side kinds the row's side could meet, so the frontend never copies DR-0015's rules. If `rowTotal` is 0, `probability` is 0, never `NaN`.
- **`reload_tile_set() -> Result<ReloadResult, ToolError>`:** on success the new `revision`, `tileSetPath` and `loadedAt`; on failure `ToolError` with every load error as readable text, one per line item.

**Probability shown is normalised per (tile, direction) row**: `probability = weight / rowTotal`, the chance of that neighbour on that side given the tile. That is what Ian's demo question asks ("the probability of a corridor corner east of a straight corridor"), and every row sums to 1. The raw weight sits in the explanation as the product it came from. `rowTotal` is over the whole set, never the filtered columns, so filtering never changes a probability. Note for the explanation: weights are symmetric but probabilities aren't, because rows have different totals. How it is drawn is Damson's design ([DR-0025](/decisions/0025-table-view-design.md)); both numbers are in the response, so a change there costs no Rust.

Every entry carries its explanation inline. At 3,364 entries that is a few hundred KB, which is fine now; the paging and transport record ([roadmap](/roadmap.md) slice 1.5) may move explanations to a per-entry call.

# Alternatives considered

- **JSON:** no comments, noisy to edit by hand.
- **RON:** fits Rust enums well, but less familiar, and nothing here needs enum syntax.
- **Side compatibility as data:** more flexible, but lets data break DR-0015's invariants, which DR-0022 says must always win.
- **No noise in 1.1, variation by hand only** (DR-0022's default): misses "every cell shows its probability, variation included", since weights per tile can't vary cell by cell.
- **Noise keyed by the facing side kinds** (the facilitator's first sketch): many entries share the same pair of side kinds, so it would repeat the same factor across them.
- **Read the file on every request:** reload for free, but a half-saved edit would break the view mid-session. An explicit reload keeps the last good table.

# Consequences

- Changing a base, weight, exception or the noise seed or amount is an edit to `core.toml` and a click on Reload. No rebuild.
- Test fixtures (broken and tiny sets) are TOML strings beside the `maze-core` tests.
- Two-way door: one file, one reader, no saved data. It stops being cheap when slice 2.3 writes this format or sample mazes depend on exact values; Yew re-checks then (DR-0022).

# Concerns raised

- **Wren (`game-designer/1`), from [DR-0013](/decisions/0013-table-entries-from-tile-sides.md), carried here:** seeded noise "adds biases nobody chose, which are hard to tune or explain". Answered in part: the factor is shown on its own in every explanation, it is bounded by `amount`, and `amount = 0` removes it. Review of the table and of Phase 2's samples settles whether it stays.
- **Yew (architect):** I said in DR-0013 to prefer per-tile values over jitter. Ian's "variation in the table" wins over my preference; the factor is seeded, stored and shown, which were my conditions.

- **Ash (`engineer-ash/1`), on review:** approved. TOML parsed from a string in core, explicit reload keeping the last good table, and inline explanations at this size are the simple choices. Gaps against [DR-0025](/decisions/0025-table-view-design.md)'s "what the Rust view must carry", all additive fields for the build, not a new round: (1) `tiles` needs each tile's `weight` for header hover; (2) a 0 entry needs the side kinds the row's side may meet, for the clash line, or the frontend would have to copy DR-0015's rules, which this record keeps in `maze-core`; (3) the response needs the tile set path and load time for the top bar, and `reload_tile_set` needs a stated return (the errors on failure, as readable text); (4) the exception's `name` stands in for "the rule as written in the data". Edge case: if `rowTotal` is 0, `probability` must be 0, never `NaN`. Glyphs stay in the frontend, keyed by tile `id` and orientation, per DR-0017. Yew folded gaps (1) to (3) and the `NaN` edge case into the view command in the revision; (4) stands as written.
- **Ash (`engineer-ash/1`), on re-review of the revision:** approved. Noise as one bounded, seeded, displayed factor is simple and reversible (`amount = 0`). For the build: the hash must order the two sides canonically before hashing, or symmetry depends on which tile is the row; and the pinned test should include one pair checked both ways round. The view shows it as its own term, "× noise 1.07".

# AI guidance

- `maze-core` parses strings; the tool reads files. Never add file I/O to core.
- Never encode which side kinds may meet in the data file.
- Keep `rowTotal` over all neighbours, regardless of filters.
- Never let noise touch a 0, and never use `std`'s hasher or OS randomness for it.
- Identify tiles by `id` and orientation `k`, never by glyph.

# Links

- Tile set and side rules: [DR-0015](/decisions/0015-tile-set.md)
- Weight model: [DR-0022](/decisions/0022-weight-model.md)
- Glyph order: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
- Code layout, `data/`, no I/O in core: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md)
- Plan: [Phase 1](/plans/phase-1.md)
