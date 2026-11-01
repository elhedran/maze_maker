---
type: Decision Record
title: The start has two ways onward
description: Amends DR-0036 rules 2 and 8 for the origin patch only. The origin cell is masked to tiles with two or more walkable sides, and the origin patch's checked stage accepts a run only if two of the origin's sides lead through kept cells to an open endpoint, with up to 15 retries instead of R. If every try fails, the spine and fallback run as before, so it's a strong bias, not a guarantee. Deterministic and seeded, through the same attempt counter.
status: stable
decision_id: DR-0048
decision_status: proposed
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [engineer-rowan/1, engineer-ash/1]   # architect/1 is the author; Rowan approves as the engineer who builds it, Ash stands in for the architect (DR-0018), as for DR-0047
supersedes: []
tags: [infinite-world, growth, start, determinism, game, slice-2-6]
generated:
  by: architect/1
  at: 2026-11-01T13:10:00Z
verified: []
---

# Context

G.1's demo checklist ([DR-0044](/decisions/0044-walking-the-growing-world.md) item 1, on `claude/game-g1-walk`) wants at least two ways to go in view from the start. Over 40 seeds, today's `core.toml` gives 37 or 38 such starts, and Wren's tuned weights (fewer T junctions) give 25 to 35. Wren would rather fix the start in the generator than keep T weights high for its sake. [DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 2 masks the origin cell only to tiles with one walkable side or more. Rule 6 guarantees one way onward for the world, not two from the start.

# Decision

This applies only to the origin patch (`from: None`). Every other patch is unchanged.

1. **Mask.** The origin cell is masked to tiles with **two or more** walkable sides (it was one or more). Rule 7's fallback tiles include every mix of hedge and passage, so the fallback stage still can't contradict.
2. **Start check.** A side of the origin **leads onward** when the neighbour across it is kept and joined to the origin, and a flood from that neighbour over kept, joined cells, never entering the origin, reaches a kept cell with an open endpoint. In the origin patch, every open endpoint faces out of the window: a kept cell faces a discarded one with hedge (rule 4). So each way onward reaches at least 7 cells from the start, which is past G.1's view radius of 6. Two sides may lead onward through the same branch (a loop), and that counts as two ways.
3. **Acceptance.** In stage 1 of rule 8, an origin patch run is accepted only when rule 6 passes **and** at least two sides lead onward. A run that fails either check moves to the next try, exactly as a rule 6 failure does now. The origin patch gets `START_RETRIES = 15` tries after the first, instead of R. That's an engineers' setting.
4. **If every try fails,** the spine and fallback stages run as they do today, with no start check. So this is a strong bias, not a guarantee: at the worst measured rate (25/40 a try), 16 independent tries all fail about once in 4 × 10⁹. If that's ever not enough, the next step is pinning a second spine, and it would need its own record.
5. **Determinism.** Nothing new is random. The checks are pure functions of the run, and the tries use rule 8's single attempt counter and seeds. Same seed and walk, same world (rule 10). Seeds' worlds change, so `one_world_is_pinned` is re-pinned.

**Build.** Rowan builds it in `world.rs`. The code is the mask change at the origin, a `leads_onward` count beside the DR-0047 block, and the extra condition and try count in stage 1's loop.

**Tests.**
- Over 100 seeds with `core.toml`, and over 40 with Wren's tuned weights, every start has at least two sides that lead onward. If a seed doesn't, report it and don't loosen the test.
- A forced spine or fallback start (`first_stage`) still keeps every guarantee.

# Why it's a two-way door

It only changes how a new world's first patch is chosen. Stored worlds are untouched (DR-0035), so reversing it means deleting about 30 lines and re-pinning one test. `major: false`.

# Alternatives considered

- **Pin the origin to a junction or crossroads tile.** It gives two or more exits, but not two ways onward: an exit can dead-end within sight. The mask (point 1) takes the cheap part of this idea.
- **Keep the T weights high.** That works against Wren's R2 (fewer junctions) everywhere, just to fix one cell.
- **Measure in the view radius instead.** The view is the game's idea, not `maze-core`'s. An open endpoint at the window's edge is a core notion, and here it implies the view's rim.

# Concerns raised

- **Yew (`architect/1`), the author:** it's a bias, not a guarantee. G.1's checklist is judged over several seeds, so that's enough. A guarantee costs a two-spine stage, which isn't worth building yet.

# Links

- Amends [DR-0036](/decisions/0036-growth-from-open-endpoints.md) rules 2 and 8; next to [DR-0047](/decisions/0047-store-sealed-hedge.md) in `world.rs`
- Checklist: DR-0044 item 1 (on `claude/game-g1-walk`); Wren's review: `docs/plans/g1-fun-review.md` (same branch)
