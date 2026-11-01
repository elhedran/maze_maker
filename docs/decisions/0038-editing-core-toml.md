---
type: Decision Record
title: How the tool edits core.toml, records the edits and compares, with opening pressure in the single maze
description: The tool saves edited numbers into the tile set file with toml_edit, keeping comments and layout, after the new text passes the existing loader, so a refused edit leaves the file untouched and returns the loader's reasons; each saved number is appended to a sibling core.edits.toml; edit_numbers and compare_mazes commands with ts-rs bindings; opening pressure applies in generate_maze, where a side past the maze's edge counts as new; and DR-0034's trace gains a pressure factor applied last, so weight times factors times pressure is still the solver's score exactly. Also slice 2.3's technical plan.
status: stable
decision_id: DR-0038
decision_status: accepted
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [engineer-rowan/1]   # architect/1 is the author; one engineer approves (DR-0018)
supersedes: []
tags: [tool, editing, tile-set-file, compare, opening-pressure, wfc, trace, ipc, slice-2-3, plan]
generated:
  by: architect/1
  at: 2026-11-01T09:04:12Z
verified:
  - by: engineer-rowan/1
    at: 2026-11-01T09:08:34Z
    note: "Approves, Ash's two additions included. Rules 10 to 12 match what is built on rowan/2.3-pressure: Rules::new reads opening_pressure, Solver::new's override is gone, CollapseStep.pressure is parallel to scores, and the 200-seed test at 0.5 checks both the exact product and that 0.5 lands only where every walkable side faces a counted neighbour (an edge side counts as new). Copying the old value's decor is right: toml_edit gives a replaced value default decor. Notes, not blocking: adding toml_edit to the tool's manifest rewrites Cargo.lock, so this branch is the one lockfile writer under DR-0019 rule 5; floats are written with a '.0' so 1 stays a float; the pinned-seeds test strips any opening_pressure line, so Wren setting a value in core.toml doesn't break it; rule 10's turned-around test lives in tests/pressure.rs and seeds_pinned.rs, and the old one is deleted from tests/world.rs."
---

# Context

Slice 2.3 Tune and compare ([roadmap](/roadmap.md)): Ian changes a number in the tool, saves, and sees the same seed before and after, side by side. He also tries opening pressure ([DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 9) on the single maze. Editing was placed here by [Q-0010](/questions/resolved/0010-where-editing-goes.md). Damson's DR-0037 designs the views in parallel, so the commands here are generic: a number is named by a target, and compare returns both mazes, the changed cells and both sets of measures.

What exists:

- `TileSet::from_toml` parses and validates the file, including the completeness check (DR-0036 rule 7) and the `opening_pressure` range (0 < v ≤ 1, already refused at load). `ToolState::reload` reads the file and keeps the last good table on failure ([DR-0024](/decisions/0024-tile-set-file-format.md)).
- `wfc.rs` already scores DR-0036 rule 9's factor, but `Rules::new` pins it at 1, and only `Solver::new` (world growth) sets the tile set's value. In `generate`, cells past the maze's edge are never settled (`outside` is all `None`), so a side facing the edge already counts as new.
- DR-0034's trace promises `tileWeight × Π factor == score`, exactly. Once the pressure isn't 1 that breaks (Ash, on DR-0036).
- [DR-0022](/decisions/0022-weight-model.md) and [DR-0024](/decisions/0024-tile-set-file-format.md) asked Yew to re-check that the format is still a two-way door once 2.3 writes it. That re-check is in Consequences.

# Decision

## Writing the file

1. **`toml_edit`, in the app crate.** The tool crate adds `toml_edit = "0.25"`, a version already in `Cargo.lock` through Tauri's build tools. Editing code goes in `apps/tool/src-tauri/src/edit.rs`. `maze-core` stays as it is: it parses text and never touches files ([DR-0012](/decisions/0012-code-layout-and-core-crate.md)).
2. **Only the edited values change.** A save reads the file fresh from disk, parses it with `toml_edit::DocumentMut`, and replaces each target's value in place. It copies the old value's decor (the spacing and any trailing comment on that line) onto the new one, because `toml_edit` drops it when an item is replaced. Values are written as TOML floats, in Rust's shortest round-trip form, so a number written back reads as the same `f64`. Only `opening_pressure` can be missing. If it is, it is inserted as a root key after `name`.
3. **Validate first, then write, then reload.** In this order:
   1. **Stale checks:** the request's `revision` must be the loaded one, and the file on disk must be byte-for-byte the text that revision loaded. `Loaded` keeps that text. Either mismatch is `stale`, nothing is written, and the message says to press Reload. This stops a save from overwriting Ian's hand edits.
   2. **Targets:** each target must exist in the loaded set, or the request is `bad_request`.
   3. **Validate:** the edited text goes through `TileSet::from_toml`, the same function Reload uses. If it refuses, the result is a new error kind, `refused`, with the loader's messages, and the file isn't touched. Ranges and the completeness check are the loader's alone. The edit code adds no checks of its own.
   4. **Write:** the edited text goes to a temporary file beside `core.toml`, which is then renamed over it, so a crash never leaves half a file.
   5. **Record:** the edit is appended to the edit record (rules 4 to 6).
   6. **Reload:** `ToolState::reload` runs on the path as usual. So what's loaded is what's on disk, and the revision goes up by one.

   A failure to write is `load_failed`, and the old file and table are kept. A failure to append to the edit record doesn't undo the save. The result carries it as a warning message.

## The edit record

4. **A sibling file, `data/tile-sets/core.edits.toml`**: the tile set's file stem plus `.edits.toml`, in the same folder. It is append-only text, oldest first, with a two-line header comment written when the file is created:

   ```toml
   # Numbers changed in the tool in core.toml, oldest first (DR-0038).
   # To set one back, set it to its "from" value, in the tool or in core.toml.

   [[edit]]
   at = "2026-11-03T14:03:22Z"
   number = "tile.corridor_corner.weight"
   from = 1.2
   to = 0.8
   ```

   - **`number`** is a readable path:
     - `tile.<id>.weight`;
     - `bases.<pair>`;
     - `exception."<name>".multiplier`;
     - `opening_pressure`.
   - **`from`** is left out when the key wasn't in the file. That only happens for `opening_pressure`, where no key means 1.
   - **`core.toml` itself stays clean,** with no markers added. Ian finds his edits as changed values in `core.toml`, the record beside it, and git's diff of both.
   - **Git:** the record is tracked like `core.toml`. Whether to commit it is Ian's call.
5. **Only the tool's saves are recorded.** A hand edit followed by Reload isn't, because the tool can't know what Ian meant. The record says so in its header ("changed in the tool"). A future tile set picker skips `*.edits.toml`.
6. **Setting a number back** is an ordinary edit to the old value. The tool gets the old value from the save's result (`EditRecord.from`), and Ian finds it in the record. It is recorded like any other edit, so the record shows the round trip. Same numbers, same table, same maze ([DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 5), so the maze returns. Setting `opening_pressure` back to 1 writes `1.0`, which behaves exactly like having no key (rule 10). There is no undo stack and no revert command: the roadmap leaves undo beyond setting a number back out of 2.3.

## Compare

7. **The tool keeps the previous load.** `ToolState` gains `previous: Option<Loaded>` and `last_edits: Vec<EditRecord>`. Whenever a load succeeds with text *different* from the current text, the current load becomes `previous`. A save sets `last_edits` to that save's edits, and a Reload that changes the text clears them. A Reload of unchanged text keeps both, so pressing Reload again doesn't lose the "before". "Before and after the last save" is therefore "the previous load and the current one". This also works when Ian edits by hand and presses Reload, with no edits listed.
8. **`compare_mazes` generates both in Rust.** It runs the same generation and measures as `generate_maze`, for the request's seed and size, on `previous` and on the current load. It also works out which cells changed. A cell has changed when its (tile id, orientation) differs, or when one side has a tile and the other has `null`. Identity is used because indices are per revision. With no previous load, `before` is `null` and `changed` is empty. Size errors are `generate_maze`'s.
9. **The before maze carries its own tile list,** so the frontend can draw it even if a hand edit reordered tiles. Its `revision` is the previous one, so `trace_cell` on it is `stale`, as DR-0034 already behaves. "Why is this here?" works on the after maze.

## Opening pressure in the single maze

10. **`generate_maze` applies the tile set's `opening_pressure`.** `Rules::new` takes `set.opening_pressure`, and `Solver::new`'s override goes. A side past the maze's edge isn't settled, so it counts as new (DR-0036 rule 9) with no further code. The solver skips the factor when the pressure is exactly 1. So with no key, or with `1.0`, every score is bit-for-bit what it was, and 2.1's pinned seeds are unchanged. The `maze-core` test `opening_pressure_leaves_generate_alone` is turned around: with the key absent, the pinned seeds still match, and at 0.3 the maze changes. Values outside 0 < v ≤ 1 are already refused at load.

## DR-0034's trace gains a pressure factor

11. **The amended contract.** Pressure is applied last, after the pair factors, exactly as the solver multiplies: `score = (tileWeight × factor₁ × … × factorₙ) × pressure`, left to right. `pressure` is the tile set's `opening_pressure` when the candidate opens nothing new, and 1 otherwise. Multiplying by 1.0 is exact in IEEE 754, so the identity holds bit for bit in both cases. DR-0034's factor test becomes `fold(tileWeight, ×factor) × pressure == score`, as exact `f64` equality, over seeds 0 to 199 at 24 × 16 with `opening_pressure = 0.5`, plus the existing runs at 1.
12. **One definition of "opens nothing new".** The trace doesn't decide it again. `CollapseStep` gains `pressure: &'a [f64]`, parallel to `scores`, holding the factor the solver gave each candidate. `trace.rs` passes it through, as it already passes `score` through. The observer still only sees shared borrows (DR-0034 rule 5).
13. **How it is recorded:** this record states the amendment. DR-0034 gets a short dated "Amendments" section that points here, and it applies once this record is accepted.

## IPC contract (DR-0023 types)

- `edit_numbers(request: EditRequest) -> Result<EditResult, ToolError>`
- `compare_mazes(request: CompareRequest) -> Result<CompareResult, ToolError>`
- `ToolErrorKind` gains `refused`. `CandidateScore` gains `pressure`.

A save can hold several numbers. They are applied, validated, written and reloaded as one, and give one revision and one record entry per number. That serves a design that saves one number at a time, and one that saves several together.

The TypeScript the bindings will have, without doc comments, so Ash can stub against it now:

```ts
export type ToolErrorKind = "load_failed" | "not_loaded" | "bad_request" | "stale" | "refused";

export type NumberTarget =
  | { "kind": "tileWeight", tileId: string }
  | { "kind": "base", pair: string }
  | { "kind": "exception", index: number }
  | { "kind": "openingPressure" };
export type NumberEdit = { target: NumberTarget, value: number, };
export type EditRequest = { revision: number, edits: Array<NumberEdit>, };
export type EditRecord = { at: string, target: NumberTarget, number: string, from: number | null, to: number, };
export type EditResult = { reload: ReloadResult, edits: Array<EditRecord>, warnings: Array<string>, };

export type CompareRequest = { seed: number, width: number, height: number, };
export type BeforeMaze = { maze: GeneratedMaze, tiles: Array<TileView>, loadedAt: string, };
export type CompareResult = { before: BeforeMaze | null, after: GeneratedMaze, changed: Array<number>, edits: Array<EditRecord>, };

export type CandidateScore = { tile: number, tileWeight: number, factors: Array<PairFactor>, pressure: number, score: number, share: number, };
```

What the fields mean:

- **Targets:**
  - `tileId` is `TileView.id`.
  - `pair` is a `[bases]` key as written in the file, such as `passage_door`.
  - `index` is an index into `TableView.exceptions` at `revision`. Exceptions have no id, and the revision and disk checks make the index safe.
  - The exact TypeScript of the tagged union is whatever ts-rs gives for `#[serde(tag = "kind", rename_all = "camelCase")]`. The shape is what's fixed here.
- **`EditRecord`** is one line of the record. `number` is rule 4's path, `from` is `null` when the key was absent, and `at` is ISO 8601 UTC from the existing `iso8601_utc`.
- **`loadedAt`:** when the before maze's load happened, as `ReloadResult.loadedAt`, for DR-0037's "BEFORE · as loaded" label.
- **`changed`:** row-major cell indices, `y × width + x`, in ascending order.
- **`edits` in compare:** the tool's edits between before and after. It is empty after a hand edit, or when there is no before.
- **Measures:** both mazes carry DR-0030's measures in `GeneratedMaze.measures`, from the same code as `generate_maze`. Compare adds no measure code of its own, so 2.6 step 2's work on measures isn't touched.
- **`pressure`:** 1 or the set's `opening_pressure`, as rule 11 says. Whether the "why" band shows a factor of 1 is DR-0037's call.

**Added 2026-11-01, from Ash (`engineer-ash/1`), while building DR-0037; Yew agrees.** These are two additive fields, so the frontend works nothing out itself:

- **`TableView` gains `openingPressure: number`:** the loaded value, or 1 when the key is absent. Without it, the frontend can't show or prefill the pressure.
- **`EntryView` gains `basePair: string | null`:** `BasePair::key()` for the pair of side kinds, as written in `[bases]`, or `null` when the entry isn't allowed. It is the `pair` for a `base` target, so TypeScript doesn't copy `side.rs`'s pairing.

# Alternatives considered

- **Re-serialise the whole file with `toml`'s writer.** It loses every comment, which [DR-0024](/decisions/0024-tile-set-file-format.md) chose TOML to keep.
- **Regex or line edits on the raw text.** No dependency, but it would need its own TOML parser to find "the weight under the `[[tile]]` with this id", and it would break on the first inline table.
- **A trailing comment in `core.toml` on each edited line** (`weight = 0.8  # was 1.2`). It is easy to see, but it clutters the file Wren annotates, piles up over repeated edits, and lies after a hand edit. The sibling record has none of those problems.
- **Rely on git as the record.** It needs a commit per edit, and commits are the facilitator's. Git stays a second record, not the only one.
- **An undo stack or a revert command.** That is more than "set a number back", which the roadmap scopes 2.3 to.
- **Compare by re-indexing the before maze into the current tile order.** That fails when a hand edit changes the tile list, and it would mislabel `revision`. Sending the before tile list costs 29 small objects.
- **Recompute "opens nothing new" in the trace.** That gives two definitions that can drift apart. DR-0034 already passes the score through for the same reason.

# Consequences

- **What changes:** one dependency, one module, two commands, one error kind, one field on `CandidateScore`, one slice on `CollapseStep`, and one line in `Rules::new`.
- **2.1 and 2.2:** with no `opening_pressure` key, every 2.1 maze and 2.2 trace is unchanged, and the pinned seeds test guards it. DR-0036's consequence "2.1's single-maze path is unchanged" now holds only while the pressure is 1. Its rule 9 always said the single maze's edge counts as new.
- **2.6:** `wfc.rs` changes in two places, `Rules::new` and `collapse`'s observer call. Following the roadmap, on a clash with 2.6's sessions, whichever lands second adapts. Nothing here touches `world.rs`, `measures.rs` or the growth tests, apart from the one test turned around in rule 10.
- **Re-check of DR-0022 and DR-0024, now that the tool writes the format:** still two-way doors.
  - The writer knows four kinds of key.
  - A format change would mean a few lines in `edit.rs`, and a test pins each kind.
  - `toml_edit` keeps whatever Ian writes by hand, so tool edits and hand edits mix freely.
  - No saved data depends on exact values beyond the pinned-seed tests, which already existed.
- **Two-way door, `major: false`.** Nothing here is costly to reverse. Dropping in-tool editing means deleting a module, two commands and a sibling file. `core.toml` stays a valid hand-written file throughout. The trace field and `CollapseStep` slice are additive. No part ties down Ian's time or money, so nothing goes to Ian.

# Concerns raised

- **Yew (`architect/1`), the author:** the disk check refuses a save after any unreloaded hand edit, even one to an unrelated line. Merging would be friendlier but would hide surprises. Refusing with "press Reload" is the safe start, and loosening it later is a two-way door.
- **Yew:** setting `opening_pressure` back to 1 leaves `opening_pressure = 1.0` in the file, not the original absence. Behaviour is identical (rule 10). If Ian wants the line gone, he deletes it by hand.

# Build plan

**Done when** (the technical half; the slice's own "done when" is in the roadmap):

1. **Writing:** an edit to each target kind (tile weight, base, exception, opening pressure absent and present) changes exactly that value's text, and every other line of `core.toml` is byte-identical. A fixture with a trailing comment on the edited line keeps the comment. An edit then its set-back gives a `TileSet` equal to the original, and the same maze for a seed.
2. **Refusals:** `passage_passage = 0` (the completeness check, as in 2.6 step 1's tests) and `opening_pressure = 0` each return `refused` with the loader's messages. The file's bytes and the record are unchanged, and so is the loaded revision. A revision mismatch and a file changed on disk each return `stale`, and nothing is written. An unknown tile id, base key or exception index is `bad_request`.
3. **Record:** a save appends one `[[edit]]` per number with the right `number`, `from` and `to`, and creates the file with its header the first time. An absent `opening_pressure` gives no `from`.
4. **Compare:** for a seed, `changed` is exactly the cells whose identity differs, checked against two `generate_maze` calls. `before` is `null` with no previous load. A Reload of unchanged text keeps the before. Measures equal `generate_maze`'s for each side.
5. **Pressure:** with no key, the pinned 2.1 seeds are unchanged. At 0.5 over 200 seeds, `fold(tileWeight, ×factor) × pressure == score` exactly, and `pressure` is 0.5 only on candidates whose walkable sides all face counted neighbours. An edge cell's edge side counts as new.
6. **Checks:** the bindings are regenerated, and the staleness test, clippy, the build and Vitest are clean.

**Who** ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)). Rowan generates the bindings above first, so Ash's stub becomes the real import.

- **Rowan (`crates/maze-core`, `apps/tool/src-tauri`, bindings):** rules 10 to 12 in `wfc.rs` and `trace.rs`; `edit.rs`; `state.rs` (`previous`, `last_edits`, the loaded text); the two commands in `lib.rs`; the types in `ipc.rs`; the tests above.
- **Ash (`apps/tool/src`):** `editNumbers` and `compareMazes` in `ipc.ts` and `COMMANDS`; the views to DR-0037 against a stub and then the real commands; `refused` and `stale` handled as DR-0037 says; the "why" band showing `pressure`.

# Links

- Slice: [roadmap 2.3](/roadmap.md); editing placed in 2.3: [Q-0010](/questions/resolved/0010-where-editing-goes.md)
- Format and reload: [DR-0024](/decisions/0024-tile-set-file-format.md); weights: [DR-0022](/decisions/0022-weight-model.md); no I/O in core: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- Opening pressure and the completeness check: [DR-0036](/decisions/0036-growth-from-open-endpoints.md) rules 7 and 9
- WFC and determinism: [DR-0028](/decisions/0028-wfc-approach-for-2-1.md); the trace contract amended here: [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)
- Measures: [DR-0030](/decisions/0030-measures-for-2-1.md); IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md); stack: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)
- Two-way doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md); parallel work: [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)
