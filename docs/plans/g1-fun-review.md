---
type: Review
title: G.1 fun review of walking a grown world
description: Wren's game-design read of the built G.1 over Ash's scripted walks (seeds 1 to 5, 600 steps each), judged against DR-0044's demo checklist; verdict, what it teaches the 3D game, and what to change in the game and in the generator.
status: draft
owners: [game-designer/1]
tags: [review, game, slice-g-1, fun, tuning]
generated:
  by: game-designer/1
  at: 2026-11-01T12:00:00Z
---

# G.1 fun review

By Wren (game designer). I judged the walks against [DR-0044](/decisions/0044-walking-the-growing-world.md)'s checklist. They come from Ash's scripted walks (`g1-fun-walks.md` on Ash's fun-walk branch, which isn't in this bundle yet): seeds 1 to 5, 600 steps each, with the real generator under [DR-0036](/decisions/0036-growth-from-open-endpoints.md). The walker is a thorough explorer. It goes into every dead end and walks back by the shortest path, so the dead-end and walking-back figures are a worst case. It never bumps a hedge, so the bump feel is for Ian's hands at the demo.

## Verdict

**Good enough for Ian's demo.** Walking a grown world already works as play: there is always a way on, something is always in view, nothing you've seen ever changes, and steps never stall (p95 at most 1.1 ms, worst case 23 ms). Two things keep it from feeling like a hedge maze yet. Rooms are everywhere, and nothing marks the hedge. Neither blocks the demo. Both are tuning, listed below.

## The checklist

| # | Point | Figures (seeds 1–5) | Verdict |
|---|---|---|---|
| 1 | At least two ways out from the start | 2, 2, 3, 2, 2, all leading onward | **Pass** |
| 2 | A room in view within 30–60 steps, and enterable | first seen at step 0, first entered at step 1 or 2; 25–50 seen, 13–18 entered | **Passes as written, misses the intent.** Rooms are too common to feel like finds (see finding 1). |
| 3 | A choice in view on more than half the steps | a junction, doorway and room in view on 100% of steps; 2+ ways onward on 83–92% | **Pass, but too busy.** There is always a choice and never a quiet stretch. The rhythm is choose, choose, choose rather than walk, choose, walk. |
| 4 | Few dead ends longer than 3, none needing a long walk back | 1–9 of 12–34 per seed are longer than 3, longest 6–11; walking back is 19–28% of moves, median run 2, longest 14 | **Pass for G.1.** Most are 1–2 cells. The 6–11 cell ones are the ones that cost patience. |
| 5 | Never boxed in | 0 steps with nowhere to go; rule 6 held | **Pass** |
| 6 | No seen cell changes; how often cells appear inside the view | 0 changed; cells appear inside the view on 0–1% of steps, mostly at the rim, and on 2–7 steps per seed closer in (as near as 2 cells) | **Pass.** Cells appearing in the haze are rare and read as the unknown resolving. Watch the close-in cases. |
| 7 | "Just one more corner" | judged by eye from the snapshots | **Provisional yes.** Every view has a door, a junction or a room to pull you on. It's busier than cozy. Ian's hands will tell us more than the snapshots can. |

## The two findings

1. **Rooms aren't rare (my call: fine for the demo, tune it next).** About one room comes into view every 12–24 steps, and one is in view on every step. A room stops being a destination when there's always one beside you. That's the opposite of DR-0015's "rooms are rare" and DR-0044's "rooms feel like finds". It doesn't hurt the demo, because it reads as lively, not frustrating. It does go to the generator as request R1.
2. **No hedge is ever stored, so every gap is haze (my call: acceptable for G.1, with one game-side fix).** About 14–26% of the view is "not yet known" (median 23–44 of 169 cells). That works as a sign of what's still unexplored. But much of that haze can never become anything. An unknown pocket closed in on every side by stored cells' hedge sides has no open endpoint, so no patch can ever reach it (DR-0036 rules 2 and 4). It is hedge in all but name. Showing it as haze tells the player "maybe a path" about cells that never will be. The game should draw sealed pockets as hedge (G1 below), and haze should mean only unknown that is still open. Without that the world looks like corridors floating in fog rather than paths through a hedge. That fix is wanted for the demo if it's cheap, and the demo goes ahead without it if not. Damson owns the look of both.

## What this teaches the 3D game

Ian's intent (2026-11-01, quoted in DR-0044) is that this 2D game is a tech demo for a later 3D game, probably in Unity.

- **What carries over, because it is fun without art:** the way onward is always there, rooms can always be entered, a world you walk back through is the same, and dead ends are mostly short. A generator thread can also keep up: a step's growth is well under a frame at p95.
- **What doesn't carry over:** most of the rooms seen are never entered (about 50–70% per seed). They were glimpsed over the hedge. At ground level the player won't see those, so a 3D player meets far fewer rooms. Room rarity should be tuned for each way of seeing: what is too common from above may be about right at ground level.
- **The haze is a 2D-only idea.** In 3D the unknown has to be solid hedge wall, which makes storing sealed hedge (R3) matter more there.
- **Growth must stay ahead of the longest sight line, not a radius.** In 3D, a long straight corridor sees further than 6 cells. The close-in appearances in point 6 would be visible in 3D, so 2.6 should know about them now.

## What to change

### Game-side (Wren and Marram, two-way doors, this branch)

- **G1. Sealed unknown is drawn as hedge.** An unsettled region inside the view that doesn't reach the view's edge, and that no stored cell faces with a walkable side, is drawn as hedge. Other unsettled cells stay as haze. This is safe, because no patch can ever store a cell in such a region. It's a check the game makes when it draws the view, with no `maze-core` change. Damson decides how hedge and haze look side by side. It's wanted for the demo if cheap.
- **G2. Keep everything else in DR-0044 as it is:** the view radius of 6, no fog, the trail, and the movement rules. The figures give no reason to change them.

### Generator-side (requests for the 2.3 and 2.6 sessions; nothing changed in `maze-core` from this branch)

- **R1 (2.3, weights): make rooms rarer.** Aim for a room in view on about a third to a half of steps, not all of them, with one entered roughly every 60–100 steps of exploring. *Why:* rooms should feel like finds. At the current rate they are wallpaper. Lower the room-edge and doorway weights before touching room size, since the sizes (6–63 cells) feel right.
- **R2 (2.3, weights): more space between paths, and fewer junctions.** Corridors often run shoulder to shoulder, and a junction is in every view. *Why:* DR-0015's hedge maze is paths through hedge, walk, choose, walk. Now that hedge tiles are never stored (rule 4), hedge weight only shows as dropped cells, so the hedge-versus-corridor balance needs looking at again. Shift weight from T junctions toward straights and corners, aiming for a quiet stretch of 3–5 cells between choices.
- **R3 (2.6, needs a record from Yew): store sealed hedge.** When a patch leaves unsettled cells that can no longer be reached from any open endpoint, store them as the Hedge tile. *Why:* the world should know where its hedge is. The 3D game needs real walls there, not "unknown". It would also replace G1's check in the game. Rule 1 still holds, because hedge may face unsettled cells.
- **R4 (2.3, opening pressure): try `opening_pressure` at 0.5 and 0.25 on these same walks.** Aim for under 10% of dead ends longer than 3 cells (today it's 8–28%). *Why:* the 6–11 cell dead ends are the ones that cost patience, and DR-0036 rule 9 is the lever for them. I set the value once I've seen the walks.
- **R5 (2.6, low priority): keep growth out of the view.** Cells appear as close as 2 cells from the player on a few steps per 600. *Why:* it's harmless in 2D's haze, but visible in 3D. Clipping patch windows so they stay out of the view square (DR-0044's concern) is the first thing to try. Not before the demo.

## Links

- Rules judged: [DR-0044](/decisions/0044-walking-the-growing-world.md); growth: [DR-0036](/decisions/0036-growth-from-open-endpoints.md); the stored world: [DR-0035](/decisions/0035-the-world-is-stored.md)
- Tile intent: [DR-0015](/decisions/0015-tile-set.md); the earlier review: [2.1 maze review](/plans/2.1-maze-review.md)
- Walk figures: Ash's `docs/plans/g1-fun-walks.md` (on Ash's fun-walk branch, to land beside this review)
