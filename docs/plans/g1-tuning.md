---
type: Review
title: G.1 tuning, rarer rooms, quieter corridors and short dead ends in the grown world
description: Wren's game-design call on tuning core.toml for the game (G.1 requests R1, R2 and R4), judged on 40 seeds of the walks example. Restates two targets the top-down view can't meet, picks variant v5, and asks for opening pressure 1 in the world and 0.5 in the single maze.
status: draft
owners: [game-designer/1]
approvers: [game-designer/1, product-manager/1]
tags: [review, game, slice-g-1, tuning, opening-pressure]
generated:
  by: game-designer/1
  at: 2026-11-01T12:40:00Z
verified:
  - by: product-manager/1
    at: 2026-11-01T12:45:30Z
    note: "Approves v5 in core.toml at opening pressure 0.5, and the restated targets. Rates (new room seen every 30-60, entered every 60-100) and ways onward (70-85%) still measure what R1 and R2 asked for, and the old in-view shares could not move. Before/after is honest, including the 0.5 quiet-run miss (2.9) and the start regression. Sequencing: v5 lands now, without waiting for DR-0048; DR-0048 must be in before Ian plays a build by hand. Wren approves as author. No decision record needed: DR-0022 leaves the numbers to review."
---

# G.1 tuning

By Wren (game designer). This answers requests R1, R2 and R4 from my G.1 fun review: rooms should feel like finds, there should be a walk between choices, and long dead ends should be rare. Ian asked us on 2026-11-01 to put the game first, so this tunes the tile set for the grown world the player walks. Tuning is a game-design two-way door ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)), approved by Wren and Marram. [DR-0022](/decisions/0022-weight-model.md) leaves the numbers to review, so no decision record is needed.

## How it was measured

Ash's `walks` example in `maze-core` replays G.1's scripted walks: a thorough explorer that goes into every dead end and walks back. Each run is 600 steps over the real growth rules ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)), with sealed hedge stored (DR-0047). The figures are pooled over **40 seeds**. Two earlier 5-seed rounds turned out to be noise: per-seed long-dead-end shares ran from 0% to 73%. Rowan checked that sealed hedge changes which world each seed grows but not the overall rates: over 100 seeds it was 24% long dead ends with sealing and 22% without.

## Targets, restated

Two of the G.1 targets measure the wrong thing:

- **"A room in view on a third to a half of steps"** is restated as **a new room first seen every 30-60 steps**. The 13 × 13 view sees over hedge, and a room stays in view for many steps once seen, longer still when the player walks back past it. Even rooms first seen every 50 steps are in view 88% of the time. Getting under 50% would take empty corridor mazes. The share also means nothing in 3D, where you only see a room down a line of sight.
- **"Choice in view below 100%"** is dropped, and **2+ ways onward on 70-85% of steps** replaces it. Any junction within 6 cells counts as "in view" whether or not it's on the player's way. Ways onward is the choice the player actually has.
- **Quiet stretch between choices: a quiet-run mean of 3-5.** The median is not a target. A run is the consecutive steps onto cells with 2 or fewer exits. Every step onto a room cell or a junction breaks it. Stepping into a one-cell nook and back makes a run of 1. Nooks beside junctions are the R4 fix, so the median sits at 2 by design. It is 2 in every variant, today's file included. The mean shows whether the long walks between choices are there.

Kept as they were: **a room entered every 60-100 steps** (R1) and **under 10% of dead ends longer than 3** (R4).

## Before and after (40 seeds × 600 steps)

| | core today, 0.5 | core today, 1 | **v5, 0.5** | **v5, 1** |
|---|---|---|---|---|
| Steps per new room seen (30-60) | 15 | 17 | 35 | 39 |
| Steps per room entered (60-100) | 49 | 45 | 62 | 66 |
| Mean room size | 17.0 | 17.1 | 15.5 | 16.3 |
| 2+ ways onward (70-85%) | 91% | 87% | 80% | 80% |
| Quiet run mean (3-5) / median | 2.1 / 2 | 2.3 / 2 | 2.9 / 2 | 3.2 / 2 |
| Dead ends longer than 3 (<10%) | 25% | 29% | 8% | 8% |
| Long dead ends per walk | 5.7 | 7.0 | 2.2 | 2.3 |
| All dead ends | 911 | 970 | 1046 | 1179 |
| Starts with 2+ ways onward (40/40) | 38/40 | 37/40 | 32/40 | 35/40 |
| Room in view (information only) | 100% | 100% | 97% | 96% |

At pressure 1, v5 meets every restated target. At 0.5 it misses only the quiet-run mean, by 0.1. Room sizes hold. There are a fifth more dead ends, but they are mostly one-cell nooks, and long ones fall from about 6 per walk to 2.

## Variants tried

All of them were run at pressure 0.5 and 1. The full tables are in the scratchpad rounds that Ash ran.

- **v1:** fewer doorways only. Rooms became rarer, but nothing else moved. Superseded at 5 seeds.
- **v2:** v1 plus T junctions halved, straights up and hedge up. Longer quiet runs, but long dead ends got worse. Without a rule for where dead ends go, longer corridors end in longer dead ends.
- **v3:** v2 plus dead ends as nooks: likelier beside a junction, rarer at the end of a run. This lever got long dead ends from 25% to 13%.
- **v4:** every lever pushed hard, with T at 0.2 and the room kind scaled down. Best quiet runs (mean 4.4), but long dead ends were still 16%, and rooms got sparse in some seeds (one entered in 300 steps).
- **v5:** v3 with stronger nook rules and doorways at 0.1. **The winner.**
- **v6:** v5 plus "junctions don't touch". No better on quiet runs (3.2 at 1), and worse starts (28/40). It wasn't worth an extra rule.
- **v7:** v6 plus hedge 3.0 and straights and corners at 1.3. Best quiet mean of the passing set (3.5 at 1) with fewer dead ends overall, but 11% long dead ends at 0.5. Close behind v5.
- **v8:** v7 with T back up to 0.45. No gain over v5.

## The winner: v5

Changes from today's `core.toml`:

- doorway 0.4 → 0.1;
- door-to-door 0.3 → 0.05;
- T junction 0.6 → 0.3;
- straight 1.0 → 1.2;
- hedge 1.5 → 2.5;
- dead end 0.05 → 0.1;
- three new exceptions:
  - a dead end beside a junction ×10;
  - a dead end after a straight ×0.15;
  - a dead end after a corner ×0.15.

Room walls, corners and floor are unchanged.

What the player feels:

- A room is something you come upon every minute or so of walking, not wallpaper.
- Corridors have a little more hedge around them.
- When a dead end appears, it is usually a nook you can see the end of from the fork. The long walk back to nowhere becomes rare.

v5 passes everything at pressure 1. At 0.5 it misses only the quiet-run mean, by 0.1 (2.9 against 3). It has the best starts of the passing variants. It is also the simplest, with one rule fewer than v6 to v8.

## Opening pressure: 1 for the world, 0.5 for the single maze

- **The world wants 1.** v5 at 1 meets the quiet-run target and has more good starts (35 against 32). Pressure exists to lift sealed-off areas in a bounded single maze ([2.3 review](/plans/2.3-tuning-review.md)). A grown world keeps only cells joined to what's stored, so it has no sealed-off areas to lift. Pressure 1 also leaves loops free to form, and loops give a wanderer a way round instead of back. The walks can't count loops yet.
- **The single maze keeps 0.5,** for the 2.3 reason: its worst seeds go from barely walkable to mostly walkable.
- **So the key needs splitting.** Yew decides how. Until then the file stays at 0.5. v5 at 0.5 still passes everything except the quiet-run mean (2.9), so the game is fine meanwhile.

## Open points

1. **Starts with 2+ ways onward (DR-0044 checklist item 1).** Tuning made this worse: 37-38/40 before, 32-35/40 with v5. Yew is settling a fix in the generator that guarantees the start. I accept v5 in the meantime. I'd want that fix in before Ian plays a build by hand, because the first few seconds are the player's first impression.
2. **The opening pressure key split** (above), for Yew.
3. **A better quiet measure:** the walk from one junction to the next along the player's first pass, leaving out walking back and nook visits. It would show the rhythm more directly than the mean. It's not needed now.
4. **v7 as a fallback** if the renders show v5 still has corridors running shoulder to shoulder.
5. **Renders by eye.** None of this has been looked at by eye yet. The next review of a build should check that rooms still read as clearings and that nooks don't make corridors look fussy.

## Final numbers, after DR-0048 and the world key

Added by the facilitator (`facilitator/1`), 2026-11-01, from Ash's run on `12ba473` (seeds 1-40 × 600 steps, `walks` example). The world now reads `world_opening_pressure` (DR-0036 rule 9 note); `core.toml` has no such key, so the world grows at 1 as Wren asked, and the single maze stays at 0.5. DR-0048 guarantees the start in practice. Both rows at world pressure 1; brackets are the min-max across seeds.

| | before (`846f3ad` core.toml) | after (v5, committed) | target |
|---|---|---|---|
| steps per new room seen | 17 | 39 | 30-60 |
| steps per room entered | 46 (30-120) | 64 (43-120) | 60-100 |
| 2+ ways onward | 86% | 82% | 70-85% |
| quiet run mean / median | 2.3 / 2 | 3.2 / 2 | mean 3-5 |
| dead ends longer than 3 | 29% (11-56%), 7.0 per walk | 7% (0-29%), 2.0 per walk | under 10% |
| starts with 2+ ways onward | 40/40 | 40/40 | 40/40 |
| boxed-in steps | 0 | 0 | 0 |

v5 meets every restated target. Open point 1 (starts) is closed by DR-0048, and open point 2 (the key split) by the DR-0036 note. Room in view (95%) and choice in view (100%) are kept for information only.

## Approval

Approved by Marram (product manager), 2026-11-01: v5 lands at 0.5 now, ahead of DR-0048; DR-0048 gates Ian's first hand-played build. Small fix for Wren: "v5 passes at both pressures" in the winner section should say it misses the quiet-run mean at 0.5 by 0.1.

## Links

- Requests: the G.1 fun review (R1, R2, R4), on the G.1 walk branch; [DR-0044](/decisions/0044-walking-the-growing-world.md)'s checklist.
- Rules: [DR-0036](/decisions/0036-growth-from-open-endpoints.md) (growth and rule 9's pressure), [DR-0022](/decisions/0022-weight-model.md) (weights and exceptions), [DR-0015](/decisions/0015-tile-set.md) and DR-0016 (tile intent).
- Earlier tuning: [2.3 tuning review](/plans/2.3-tuning-review.md).
