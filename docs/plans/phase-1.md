---
type: Plan
title: Phase 1 plan, the table and the viewer
description: The steps for roadmap Phase 1, with slice 1.1 (first look at the table) in detail, the Rowan/Ash build split, and measurable done-when tests.
status: draft
owners: [architect/1]
tags: [plan, phase-1, adjacency-table, tool]
generated:
  by: architect/1
  at: 2026-10-31T19:36:00+01:00
---

# Phase 1 plan

By Yew (architect). The steps are the [roadmap](/roadmap.md)'s Phase 1 slices; this plan doesn't change them. Ian (`human:ian`), verbatim, 2026-10-31: "make sure we bias towards executing planned work over re-planning or planning new work." So only 1.1 is detailed; later slices get detail when they start.

Records that shape 1.1: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), [DR-0012](/decisions/0012-code-layout-and-core-crate.md), [DR-0013](/decisions/0013-table-entries-from-tile-sides.md), [DR-0014](/decisions/0014-square-grid.md), [DR-0015](/decisions/0015-tile-set.md), [DR-0017](/decisions/0017-tile-glyphs-and-colours.md), [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md), [DR-0020](/decisions/0020-vitest-for-frontend-tests.md), [DR-0022](/decisions/0022-weight-model.md), [DR-0023](/decisions/0023-rust-typescript-types-in-step.md), [DR-0024](/decisions/0024-tile-set-file-format.md).

## 1.1 First look at the table

**Status (2026-11-01):** built on branch `claude/slice-1-1-4a2272`, waiting to land on `main`. Points 1 to 6 and 8 are checked. Point 7 is checked at 125% scaling only (reload to repaint 28 ms, row headers 35px); 150% scaling hasn't been checked. Points 9 and 10 wait for `main`. The facilitator ran both demo tasks in the real Tauri window.

### Rowan: the Rust side

Rowan's branch holds `Cargo.lock` (serde, toml in `maze-core`; ts-rs in `maze-tool`).

1. **IPC types and bindings first.** `apps/tool/src-tauri/src/ipc.rs` with the request, response and error types from DR-0024, the generated files in `apps/tool/src/bindings/`, the staleness test, and a stub `table_view` returning a small hand-built view. Land this before the rest, so Ash builds on the real types.
2. **Tile set data.** `data/tile-sets/core.toml` with DR-0015's ten tiles, in kind order with fallbacks last. Start every base, weight and multiplier at 1.0 with no exceptions, and a `[noise]` table with a seed and `amount = 0.15`; Wren supplies the first numbers and exceptions as an edit to this file, which needs no code.
3. **Loader and validation in `maze-core`.** Parse, compute distinct orientations, check `orientations`, and reject everything in DR-0022 condition 3 and DR-0024, all errors at once.
4. **Table calculation with explanation in `maze-core`.** Side rules, base × weights × exception × noise (DR-0024's hand-written stable hash), `rowTotal` over all neighbours, probability.
5. **Commands in `src-tauri`.** `table_view` with filters and window; `reload_tile_set`; managed state with `revision`; load errors returned, not panicked.
6. **Rust tests**, listed under "done when".

### Ash: the Lit view

Ash adds no npm dependencies. Ash's branch holds `README.md` and writes the demo steps.

1. **`apps/tool/src/ipc.ts`**, the typed wrapper over the generated bindings (DR-0023).
2. **Glyph and colour mapping**, keyed by tile `id` and orientation `k`, with DR-0017's glyphs, tokens, font and hover names. An unknown tile shows `?` and its name.
3. **Table component.** Calls `table_view` for everything, shows probability per cell with 0 blank. The layout (directions, grouping by kind, shading, the explanation panel) follows Damson's table design, [DR-0025](/decisions/0025-table-view-design.md).
4. **Entry detail.** Selecting a cell shows the facing side kinds and the explanation: base, both tile weights, each exception by name and multiplier, the noise factor, weight, row total, probability. For a 0, which sides clash.
5. **Reload button and errors.** Calls `reload_tile_set`, refetches on a new `revision`, and lists load errors.
6. **Vitest tests** with `mockIPC`, rejecting with what Rust actually returns (DR-0020). Move per-test cleanup to `setupFiles` now that a second test file arrives.

Ash can start 2 and 3 against DR-0024's shape while Rowan's step 1 lands, then rebase onto the bindings.

### Done when

All of these, checked by the named person:

1. **Data, not code** (Rowan): `cargo test --workspace` includes tests that the core file loads to 29 (tile, orientation) pairs (15 corridor, 13 room, 1 hedge), and that exactly 916 entries are allowed, 229 per direction (DR-0015).
2. **Model** (Rowan): tests that A east of B and B west of A have equal weight; every row with a non-zero total sums to 1 within 1e-9; filtering never changes an entry's probability; noise is symmetric, stays within `[1 - amount, 1 + amount]`, never touches a 0, matches pinned known values, and changes when the seed changes; and every rejection in DR-0022 condition 3 has a fixture that fails with an error naming the culprit.
3. **Determinism** (Rowan): loading the same string twice gives identical serialized table views, byte for byte.
4. **Types in step** (Rowan): the bindings test passes, and fails if a field is renamed without regenerating.
5. **View** (Ash): `npm test` covers all 29 glyphs rendering, a 0 rendering blank, the detail panel for an allowed entry and a 0, and a reload error being shown. `npm run build` passes.
6. **No code change** (Ash, in the real app): edit a tile weight in `core.toml`, press Reload, and the changed probability shows without restarting. Break the file, press Reload, and the error shows while the last good table stays.
7. **Responsive and readable** (Ash, in the real app): the full table shows within 1 second of launch and of Reload; headers are at least 20px and every glyph joins cleanly at 100% and 150% Windows scaling (DR-0017 rules 1 and 2).
8. **Checked by hand** (Wren): with Wren's first numbers in the file, Wren works out at least five entries on paper, including one with an exception and one 0, noise factor included, and they match the tool.
9. **Demo** (Ian, without help, from `main`): gives the probability of a corridor corner east of a straight corridor and says why; finds a 0 and says which sides clash.
10. Landed on `main` with `README.md` demo steps and a line in the log.

## Later slices

- **1.2 Explore one tile:** a tile-centred view over the same `table_view` command, once Damson's exploring design lands.
- **1.3 Filter the table:** filter UI over the request's existing filter fields, once Damson's filtering design lands.
- **1.4 Table checks:** dropped by Marram.
- **1.5 Full tile set v1:** Wren's v1 file, the paging and transport record, and the speed and readability tests at full size.
