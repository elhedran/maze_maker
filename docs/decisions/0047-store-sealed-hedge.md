---
type: Decision Record
title: Store sealed regions as hedge
description: Amends DR-0036. After each patch, every bounded region of unsettled cells that no stored cell faces with a walkable side is stored as the Hedge tile, at once and for good. Nothing can ever grow there, so the world knows where its walls are, which the 3D game needs. It uses no seed, so a patch stays a pure function of the world before it; rule 4's "every stored cell is joined to the origin" test becomes "joined, or sealed hedge".
status: stable
decision_id: DR-0047
decision_status: proposed
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [engineer-rowan/1, engineer-ash/1]   # architect/1 is the author; Rowan approves as the engineer who builds it, Ash stands in for the architect (DR-0018), as for DR-0036 and DR-0041
supersedes: []
tags: [infinite-world, growth, frontier, hedge, determinism, game, slice-2-6]
generated:
  by: architect/1
  at: 2026-11-01T12:25:00Z
verified:
  - by: engineer-ash/1
    at: 2026-11-01T12:30:47Z
    note: "Approves, standing in for the architect. Checked against world.rs at 9e16692. Seeds are complete: a region is split off or loses its last facing endpoint only where the patch stores cells, so every newly sealed region touches a kept cell. Bounds hold: a bounded region lies inside the stored box, so sealing never grows `cells.bounds`. Seal-tile pairs are covered by rule 7's hedge clause, sealed against sealed included. Rule 6 is unaffected. For the build: share one visited set across seeds so a region that is not sealed is flooded once. Reusing the accepted attempt's `has_outward` rim flood as the bounded test avoids most of the second O(bounds) pass. Otherwise, floods from seeds on the outer frontier cover the whole outer unsettled area every patch."
---

# Context

[DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 4 stores only the new cells joined to a patch's anchor, so no all-hedge cell is ever stored. A cell nothing can reach stays unsettled forever, and the tool draws it as solid hedge. Wren's G.1 fun review (request R3, on `claude/game-g1-walk` in `docs/plans/g1-fun-review.md`) asks for those cells to be stored as Hedge. The world should know where its hedge is, and the 3D game (Ian's long-term target: `maze-core` driven from a Unity game on its own thread) needs real walls there, not "unknown". It also replaces the game-side check G1, which works out the same thing when the view is drawn.

# Decision

**Terms.** A **region** is a 4-connected set of unsettled cells, as large as it can be. It is **bounded** when it doesn't reach the unbounded unsettled region (DR-0036 rule 6's "outward" flood). A region is **sealed** when it is bounded and no stored cell shows a walkable side into it, which means no open endpoint has its anchor there. (This isn't rule 4's "pocket", which lies within one window.)

1. **Sealed is final.** Every cell around a bounded region is stored, and stored cells never change (rule 5). So a sealed region can never gain an anchor, and no patch can ever store anything else there.
2. **When.** As the last part of each patch (every stage, including the origin patch and the fallback), once the kept cells are known, every sealed region is stored whole as the **seal tile**. The patch records these cells in a new `Patch::sealed: Vec<Pos>`, in (y, x) order. `Patch::stored` keeps its meaning: the run's kept cells, in the window's row-major order.
3. **Where to look.** A region can only become sealed when the patch has just stored a cell next to it: either the cells that split it off, or the anchor that was its last open endpoint. So the search starts from the unsettled neighbours of the kept cells, walked in (y, x) order. Each search floods unsettled cells and stops as not sealed when it reaches the rim of `Area::around` the stored bounds and the window, or meets a walkable side facing into it. A bounded region lies inside the stored bounding box, so sealed cells may lie outside the window but never outside the bounds. The reference cost is O(bounds area) per patch, the same as rule 6's check. Reusing one flood for both (an unreached unsettled cell is bounded) is the engineers' call. The incremental method is a separate record (see Scope).
4. **The seal tile** is the first variant, in variant order, that is a fallback variant (`is_fallback`) with hedge on all four sides. Rule 7's completeness check already requires the all-hedge mix, so every set a `World` accepts has one, and no new load error is needed. Every neighbour of a sealed cell shows hedge towards it, so rule 7's clause gives every new stored pair an entry weight above 0.
5. **Rule 1** holds trivially: a sealed cell has no unsettled neighbour.
6. **Rule 6** is unaffected. A sealed region doesn't connect to the unbounded region and holds no anchor, so sealing changes no endpoint's outward flag. The check still runs on each attempt, before sealing, and needs no second run.
7. **Rule 2** is unaffected: sealing removes no anchor, and a sealed cell is never an anchor.
8. **Determinism.** Sealing uses no seed and no randomness. It is a pure function of the world plus the kept cells. `World::patch` computes `sealed` against that overlay without storing, so DR-0036 rule 10 still holds as written: a patch is a pure function of the stored world before it, its anchor, the tile set and the code, and the world is a pure function of (world seed, tile set, generator version, walk). DR-0036 promises the same world for the same seed **and walk**, not for any order of walking (order independence was dropped with DR-0031), and this record adds no such promise. Replay reproduces sealing because `grow_at` seals. The growth log records `sealed` for the view and for comparison only.
9. **Worlds change.** Later windows pin sealed cells instead of solving them, so the solver makes different draws and has fewer cells to contradict on. The pinned world (`one_world_is_pinned`) is re-pinned on purpose.

## Tests (amending DR-0036's step 1 list)

- **Invariant after every patch:** every bounded unsettled region has at least one stored cell showing a walkable side into it. That is, nothing sealed is left unstored.
- "Every stored cell is joined to the origin" becomes: **every stored cell is joined to the origin, or is in some patch's `sealed` and is the seal tile.**
- Every cell in `sealed` was unsettled before its patch. It isn't in `stored`, and it lies in a region that touches the patch's kept cells.
- The existing rule 1, never-changes, entry-weight-above-0 and outward checks stay as they are. They must still pass with sealed cells counted as stored.
- Replay, and a patch regenerated from the world before it, match `sealed` too.
- `world_view.rs`'s "discards and stored cells fill each window exactly" adds `sealed ∩ window`. `grow_with_discards` already returns only window cells still unsettled, so sealed cells drop out of the discards without a code change.
- A small scripted case: a patch that closes a ring around a few unsettled cells seals them.

## Boundaries

All of this lives in `world.rs`, as one contiguous block plus the new `Patch` field. `World` stays `Send`, seeded and deterministic, with no Tauri, file I/O or globals. Keep `BTreeMap` order, and never let `HashMap` order decide anything.

# Why it's a two-way door

Nothing is saved yet. Even after the first save, a sealed cell can never be reached, so reversing this means deleting the sealing block and re-pinning one test. Old saves would just hold extra hedge where newer code would leave unknown, which nobody can tell apart in play. It doesn't touch DR-0036 rule 1, the part of the save that is a one-way door. `major: false`.

# Alternatives considered

- **Game-side only (G1):** the game works out sealed regions each time it draws. It works, but every consumer (the tool, the game, a Unity bridge) has to repeat the flood. The question of where the walls are belongs in the world.
- **Store the run's own tiles for discarded cells:** they aren't joined to anything, and rooms nobody can enter are exactly what DR-0036 set out to avoid. Hedge is honest.
- **Seal only within the window:** cheaper, but it misses regions that run past the window's edge, and that breaks the invariant above.

# Consequences

- Stored counts go up, and so does memory per explored area. Anything that divides by stored cells shifts: 2.6 step 2's room share and memory per 1,000 stored cells, and Ash's G.1 walk figures. Compare like with like when re-measuring.
- **DR-0041's contract:** each patch delta needs the sealed cells, or the World tab draws them as unsettled. In the tool that looks the same (solid hedge), so this is a follow-up for whoever next touches the World tab: a `sealed` list on `PatchDelta`. The replay check already compares cells, and stays correct.
- Wren's G1 check in the game can go once this lands.
- **Scope: the incremental rule 6 check is not in this record (Yew, 2026-11-01).** The reviewer guide (section 4) suggested one record for both, since both work on regions of unsettled cells. I've kept them apart. Sealing is buildable now and stands alone. An incremental check needs its own design: the costly part is proving a region still reaches the open unknown, not finding regions next to the patch, and that has to be measured. It gets its own record from Yew on this generator-fixes track, parked until its trigger, and written after sealing because sealing changes which cells the check floods. That matches DR-0036's dated note on the G.1 branch: it is due before G.2's long saved sessions, or sooner if a measured step's worst case passes 125 ms (G.1's worst is about 23 ms). To keep it from becoming a second flood bolted on, rule 3's search is built as one function that returns, for each region next to the kept cells, its cells, whether it is bounded and whether something faces into it. The incremental check will extend that function rather than add another.

# Concerns raised

- **Yew (`architect/1`), the author:** the sealing flood is a second O(bounds) pass per patch until it is folded into rule 6's flood or made incremental. That's fine for now; the incremental-check record replaces it (see Scope).

# AI guidance

- Never seal a region that some stored cell faces with a walkable side, or one that reaches the unbounded region.
- Seal after the run's keep, never before, and never let sealing feed back into choosing the attempt.
- Use no randomness in sealing. Walk the seeds in (y, x) order.

# Links

- Amends [DR-0036](/decisions/0036-growth-from-open-endpoints.md) (rules 4 and 10, and the step 1 tests); the stored world: [DR-0035](/decisions/0035-the-world-is-stored.md)
- Completeness and fallback tiles: DR-0036 rule 7; tile set: [DR-0015](/decisions/0015-tile-set.md)
- World view contract: [DR-0041](/decisions/0041-data-contract-for-the-world-view.md); doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)
