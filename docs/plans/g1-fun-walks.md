---
type: Report
title: G.1 fun walks, scripted walks for DR-0044's checklist
description: A seeded player walks grown worlds under DR-0044's rules for seeds 1, 2, 3, 4, 5, 600 steps each; counts for each checklist point, 13 by 13 snapshots and step times, for Wren to judge.
status: draft
owners: [engineer-ash/1]
tags: [report, game, slice-g-1, fun, measures]
generated:
  by: engineer-ash/1
  at: 2026-11-01T10:41:49Z
---

# G.1 fun walks

Made by `cargo run --release -p maze-game --example fun_walk -- --seeds 1,2,3,4,5 --steps 600 --every 60`. Counts are seeded and repeat exactly; times are this machine's, release build.

**The player.** Starts at the origin, steps only into stored, joined cells, and the world grows around it after every move (DR-0044 rules 1 to 3). It's a curious, thorough player: it takes an unvisited way when there is one (a seeded draw), otherwise it walks back by the shortest path to the nearest cell with an unvisited way or an open endpoint. So it walks into every dead end and every room cell, which makes the dead-end and walking-back counts a worst case. It never bumps a hedge, so blocked moves aren't measured here.

**Terms.** A *way* is a joined neighbour. *Ways onward in view*: of the player's ways, those that reach the view's rim or an open endpoint within the 13 by 13 view without coming back through the player. A *choice* is a junction (a corridor cell with 3 or 4 ways), a room doorway or any room cell in view. A *dead end* is a corridor cell with one walkable side, joined; its length is the cells walked from it to the first junction, room or doorway (dead ends that run into the unknown aren't counted). *Appearing* cells were unknown in the view the player stepped into and stored after the growth that step caused. Snapshots draw DR-0017's glyphs two characters per cell, `@` for the player and `::` for not yet known; the trail can't show in text, so each snapshot counts the trodden cells in view.

## Across the seeds

```text
seed  start ways  rooms seen/entered  1st room entered  2+ ways onward  dead ends >3  walked back  appeared (steps)  changed  step p95
1            2/2               36/17                 1             86%          4/18          22%                1%        0   0.02 ms
2            2/2               39/13                 1             86%          8/34          20%                1%        0   0.70 ms
3            3/3               38/15                 1             92%          4/27          19%                1%        0   1.12 ms
4            2/2               25/13                 2             83%          1/12          28%                0%        0   0.02 ms
5            2/2               50/18                 1             85%          9/32          20%                1%        0   0.71 ms
```

Start ways is onward/all. Walked back is the share of moves onto trodden cells. Appeared is the share of steps where any cell appeared inside the view.

**No hedge cell is ever stored.** A patch keeps only the cells joined to its anchor (DR-0036 rule 4), and a hedge tile has no walkable side, so it is never joined. Every gap between paths in these walks is a cell not yet known, drawn `::`, which is how DR-0045 point 7's haze will look on screen: the game will show no `░` at all.

## Seed 1

```text
600 steps asked, 600 moved, 0 with nowhere to go; 1864 cells stored in 40 patches; way onward at the end: yes

1. Start: 2 ways out of the start cell, 2 of them lead onward within the view.
2. Rooms: 36 seen, 17 entered. First seen at step 0, first entered at step 1.
   Seen, never entered: 35 cells, seen at step 594; 8 cells, seen at step 595; 12 cells, seen at step 539; 12 cells, seen at step 529; 9 cells, seen at step 579; 8 cells, seen at step 579; 34 cells, seen at step 534; 8 cells, seen at step 437; 12 cells, seen at step 533; 9 cells, seen at step 121; 14 cells, seen at step 38; 9 cells, seen at step 274; 6 cells, seen at step 244; 8 cells, seen at step 71; 20 cells, seen at step 174; 18 cells, seen at step 277; 18 cells, seen at step 244; 63 cells, seen at step 239; 28 cells, seen at step 251.
   Room sizes in cells: 35, 16, 8, 12, 12, 9, 8, 10, 6, 8, 34, 8, 46, 12, 9, 15, 10, 9, 14, 18, 14, 10, 12, 20, 9, 6, 15, 9, 6, 8, 20, 20, 18, 18, 63, 28.
3. 2+ ways onward in view on 86% of steps. Any junction, doorway or room in view on 100% (junction 100%, doorway 100%, room 100%).
   Ways onward in view, by steps: 0: 0%, 1: 14%, 2: 44%, 3+: 42%.
4. Dead ends seen: 18; lengths (cells to the junction) 1×7, 2×3, 3×4, 4, 5, 9×2; 4 longer than 3.
   Walking back over trodden cells: 45 runs, 131 steps (22% of moves); longest 13, median 2.
5. Boxed in: 0 steps with nowhere to go; the world's way onward check: holds.
6. Seen cells that changed: 0 (must be 0).
   Cells appearing inside the view: on 1% of steps (3 steps closer in than the rim); 141 cells, 141 walkable.
   By distance from the player (6 is the rim): 2: 7, 3: 21, 4: 34, 5: 37, 6: 42.
7. (Judged by eye from the snapshots below.) Not yet known in view: median 41 of 169 cells; hedge cells stored: 0.

Time: new world 3.24 ms; a step (move and grow) median 0.01 ms, p95 0.02 ms, max 12.83 ms; patches per step median 0, max 6.
```

### Snapshots

```text
Seed 1 · at the start · 0 steps · 1 trodden cells in view
+--------------------------+
|::::::::::╚═╤═╝ ├─┐ ::::::|
|────┬─────┬─┘ ::└─┤ ::::::|
|┌───┘ ::┌─┤ ╔═══╗ │ ::::::|
|│ ::::::└─┘ ║ · ╟─┘ ::::::|
|└─┐ ┌─┬─┐ ::║ · ╟─┐ ::::::|
|┌─┘ └─┤ └─┐ ║ · ╟─┘ ::::::|
|├───┐ │ ┌─┤ @═══╝ ::::::::|
|┴─┐ └─┘ └─┘ ::::::::::::::|
|::│ ::::::::::::::::::::::|
|::╵ ::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
+--------------------------+

Seed 1 · 3 west, 1 south of the start · 60 steps · 43 trodden cells in view
+--------------------------+
|┐ ║ ╟─────┬─────┬─┘ ::└─┤ |
|┤ ║ ║ ┌───┘ ::┌─┤ ╔═══╗ │ |
|╵ ║ ║ │ ::::::└─┘ ║ · ╟─┘ |
|::╚═╝ └─┐ ┌─┬─┐ ::║ · ╟─┐ |
|──┬─┐ ┌─┘ └─┤ └─┐ ║ · ╟─┘ |
|╗ └─┘ ├───┐ │ ┌─┤ ╚═══╝ ::|
|╟─────┴─┐ └─@ └─┘ ::::::┌─|
|║ ::┌─┐ │ ::::::::::::::│ |
|╟─┬─┘ │ ╵ ::::::::::┌─┬─┴─|
|╟─┘ ┌─┘ ::::::┌───┐ │ │ ┌─|
|║ ::├───┐ ::::├───┘ ├─┘ ├─|
|╝ ╔═╧═══╧═╗ ╔═╧═══╗ └─┐ │ |
|┐ ╚═╤═════╝ ║ · · ║ ::├─┴─|
+--------------------------+

Seed 1 · 14 west, 4 south of the start · 120 steps · 22 trodden cells in view
+--------------------------+
|┌─┬─┤ └───┤ └─┤ ┌─┘ ::::╚═|
|└─┘ │ ┌─┐ ├─┐ │ │ ┌─────┬─|
|╔═╗ ├─┘ │ │ └─┘ └─┘ ╔═╗ └─|
|║ ║ ├─┐ ├─┘ ┌─┐ ::┌─╢ ╟───|
|║ ╟─┴─┴─┤ ┌─┤ ├───┘ ║ ║ ::|
|║ ║ ┌─┬─┘ │ └─┘ ::┌─╢ ╟─┬─|
|║ ╟─┤ │ ::└─@ ┌───┴─╢ ╟─┘ |
|║ ║ └─┴─┐ ╔═╧═╧═══╗ ║ ║ ::|
|║ ║ ::┌─┘ ║ · · · ║ ╚═╝ ╔═|
|╢ ║ ┌─┘ ::╚═══════╝ ┌─┐ ╚═|
|║ ║ │ ┌─┬─┬─────┐ ::├─┴───|
|╢ ║ │ │ └─┘ ::┌─┴─┬─┘ ┌─┬─|
|╚═╝ └─┘ ::┌───┴─┬─┘ ::└─┴─|
+--------------------------+

Seed 1 · 6 west, 12 south of the start · 180 steps · 28 trodden cells in view
+--------------------------+
|· ║ ╚═╝ ╔═╧═══╧═╗ ╔═╧═══╗ |
|══╝ ┌─┐ ╚═╤═════╝ ║ · · ║ |
|┐ ::├─┴───┤ ::::::║ · · ╟─|
|┴─┬─┘ ┌─┬─┴───┬───╢ · · ╟─|
|┬─┘ ::└─┴───┐ ├─┐ ╚═╤═╤═╝ |
|┘ ╔═╗ ┌───┐ └─┘ │ ::│ ├───|
|──╢ ║ └─┐ └─@─┬─┴───┤ └─┐ |
|::║ ║ ╔═╧═════╧═╗ ::└─┬─┘ |
|::║ ║ ║ · · · · ║ ┌─┬─┘ ::|
|::╚═╝ ╚═════╤═══╝ │ ├─┬─┐ |
|::╔═══╗ ::┌─┘ ┌───┤ │ ├─┘ |
|::║ · ╟───┤ ┌─┘ ┌─┴─┘ └───|
|::╚═══╝ ╔═╧═╧═══╧═╗ ::::::|
+--------------------------+

Seed 1 · 4 east, 18 south of the start · 240 steps · 14 trodden cells in view
+--------------------------+
|┤ └─┐ └─┐ │ ║ · · · ║ ╚═══|
|└─┬─┘ ::└─┴─╢ · · · ║ ::::|
|┬─┘ ::╔═══╗ ╚═══════╝ ::::|
|├─┬─┐ ║ · ║ ::::::::::::::|
|│ ├─┘ ╚═╤═╝ ::::::::::::::|
|┘ └───┐ │ ::::::::::::::::|
|::::::└─┴─┬─@ ╷ ╔═══════╗ |
|::┌─┬─┐ ┌─┘ ├─┴─╢ · · · ║ |
|╶─┘ └─┴─┘ ┌─┴─┐ ║ · · · ║ |
|::::::::::│ ┌─┘ ╚═══════╝ |
|::::::::::└─┤ ::::::::::::|
|::::::::┌─┬─┤ ::::::::::::|
|::::::╶─┴─┴─┘ ::::::::::::|
+--------------------------+

Seed 1 · 20 south of the start · 300 steps · 58 trodden cells in view
+--------------------------+
|· · ║ ┌─┬─┘ ::╔═══╗ ╚═════|
|╤═══╝ │ ├─┬─┐ ║ · ║ ╷ ::::|
|┘ ┌───┤ │ ├─┘ ╚═╤═╝ └───┐ |
|┌─┘ ┌─┴─┘ └───┐ │ ::::::└─|
|╧═══╧═╗ ::::::└─┴─┬─┐ ╷ ╔═|
|══════╝ ::┌─┬─┐ ┌─┘ ├─┴─╢ |
|::::::::╶─┘ @─┴─┘ ┌─┴─┐ ║ |
|::::::::::::::::::│ ┌─┘ ╚═|
|::::::::::::::::::└─┤ ::┌─|
|::::::::::::::::┌─┬─┤ ::│ |
|::::::::::::::╶─┴─┴─┘ ::└─|
|::::::::::::::::::╔═╗ ::::|
|::::::╔═══╗ ::::::║ ╟─────|
+--------------------------+

Seed 1 · 10 west, 18 south of the start · 360 steps · 46 trodden cells in view
+--------------------------+
|║ · ║ └───╢ ║ └─┐ └───┬─┴─|
|╚═══╝ ::::║ ║ ╔═╧═════╧═╗ |
|::::::::::║ ║ ║ · · · · ║ |
|::::::::::╚═╝ ╚═════╤═══╝ |
|::::::::::╔═══╗ ::┌─┘ ┌───|
|::::::::::║ · ╟───┤ ┌─┘ ┌─|
|::::::::::╚═@═╝ ╔═╧═╧═══╧═|
|::::::::::::::::╚═════════|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
+--------------------------+

Seed 1 · 13 west, 11 south of the start · 420 steps · 71 trodden cells in view
+--------------------------+
|║ └─┴─┐ ╔═╧═╧═══╗ ║ ║ ::├─|
|║ ::┌─┘ ║ · · · ║ ╚═╝ ╔═╧═|
|║ ┌─┘ ::╚═══════╝ ┌─┐ ╚═╤═|
|║ │ ┌─┬─┬─────┐ ::├─┴───┤ |
|║ │ │ └─┘ ::┌─┴─┬─┘ ┌─┬─┴─|
|╝ └─┘ ::┌───┴─┬─┘ ::└─┴───|
|┐ ::::╔═╧═╗ @─┘ ╔═╗ ┌───┐ |
|┘ ::::║ · ║ └───╢ ║ └─┐ └─|
|::::::╚═══╝ ::::║ ║ ╔═╧═══|
|::::::::::::::::║ ║ ║ · · |
|::::::::::::::::╚═╝ ╚═════|
|::::::::::::::::╔═══╗ ::┌─|
|::::::::::::::::║ · ╟───┤ |
+--------------------------+

Seed 1 · 20 west, 6 south of the start · 480 steps · 60 trodden cells in view
+--------------------------+
|╗ ::╔═╧═══╗ ╔═╗ ├─┘ │ │ └─|
|║ ::╚═╤═╤═╝ ║ ║ ├─┐ ├─┘ ┌─|
|╟─┬───┴─┘ ::║ ╟─┴─┴─┤ ┌─┤ |
|╝ │ ╔═════╗ ║ ║ ┌─┬─┘ │ └─|
|::│ ║ · · ║ ║ ╟─┤ │ ::└─┐ |
|::└─╢ · · ║ ║ ║ └─┴─┐ ╔═╧═|
|::::║ · · ║ @ ║ ::┌─┘ ║ · |
|::::║ · · ╟─╢ ║ ┌─┘ ::╚═══|
|::::╚═════╝ ║ ║ │ ┌─┬─┬───|
|::::::::::┌─╢ ║ │ │ └─┘ ::|
|::::::::::│ ╚═╝ └─┘ ::┌───|
|::::::::::└─┬─┐ ::::╔═╧═╗ |
|::::::::::╶─┴─┘ ::::║ · ║ |
+--------------------------+

Seed 1 · 30 west, 6 north of the start · 540 steps · 19 trodden cells in view
+--------------------------+
|::┌───┐ ├─┘ └─┘ │ ├─┐ ┌─╢ |
|┌─┘ ::└─┴─┐ ::::└─┘ ├─┤ ╚═|
|└─┬───┐ ::├─┬─┬─┐ ::└─┘ ::|
|╗ ├─┬─┘ ┌─┘ └─┘ └─┬─┐ ::::|
|╟─┼─┤ ┌─┘ ┌───┐ ::├─┘ ┌───|
|║ └─┤ ├─┐ └─┬─┘ ┌─┤ ::│ ┌─|
|╟─┐ │ └─┴─┐ @ ::└─┘ ::├─┼─|
|╟─┘ └─┬───┤ ├─┐ ┌─┐ ::│ └─|
|╝ ┌───┤ ┌─┘ │ └─┤ │ ::╵ ::|
|::└─┬─┘ └─┐ │ ╔═╧═╧═══╗ ┌─|
|::╷ └─┐ ::└─┤ ╚═══╤═══╝ │ |
|::└─┐ └─┐ ┌─┤ ::┌─┘ ::┌─┘ |
|══╗ │ ::╵ └─┴─┐ │ ╔═╗ │ ╔═|
+--------------------------+

Seed 1 · 41 west, 5 south of the start · 600 steps · 17 trodden cells in view
+--------------------------+
|::::::::::::::::::::::::└─|
|::::::::::::::┌─╴ ::╔═══╗ |
|::::::::::::::└─┬─┬─╢ · ╟─|
|┌─┐ ╷ ::::::┌─┐ ├─┴─╢ · ╟─|
|│ │ ├───╴ ::╵ ├─┴─┐ ╚═╤═╝ |
|┘ │ └───┐ ╔═══╧═╗ ╵ ┌─┴─┐ |
|──┘ ┌─┐ │ ║ @ · ╟─┐ ├─┬─┴─|
|┬─┐ │ └─┴─╢ · · ╟─┴─┘ ├───|
|┤ │ ├─┐ ::╚═╤═╤═╝ ::::│ ::|
|│ │ ├─┘ ::::└─┴─────┬─┤ ::|
|┘ └─┤ ┌───┬───┬─────┘ │ ::|
|┌─┐ └─┴─┐ │ ┌─┴───┬───┘ ┌─|
|┴─┘ ::::└─┘ │ ::┌─┤ ::::│ |
+--------------------------+

```

## Seed 2

```text
600 steps asked, 600 moved, 0 with nowhere to go; 2703 cells stored in 57 patches; way onward at the end: yes

1. Start: 2 ways out of the start cell, 2 of them lead onward within the view.
2. Rooms: 39 seen, 13 entered. First seen at step 0, first entered at step 1.
   Seen, never entered: 21 cells, seen at step 436; 9 cells, seen at step 349; 15 cells, seen at step 184; 20 cells, seen at step 82; 22 cells, seen at step 184; 12 cells, seen at step 481; 10 cells, seen at step 185; 10 cells, seen at step 158; 20 cells, seen at step 63; 21 cells, seen at step 63; 10 cells, seen at step 115; 8 cells, seen at step 521; 12 cells, seen at step 500; 27 cells, seen at step 4; 6 cells, seen at step 147; 6 cells, seen at step 595; 10 cells, seen at step 537; 20 cells, seen at step 520; 45 cells, seen at step 136; 27 cells, seen at step 25; 14 cells, seen at step 567; 12 cells, seen at step 565; 10 cells, seen at step 573; 9 cells, seen at step 42; 6 cells, seen at step 591; 32 cells, seen at step 589.
   Room sizes in cells: 21, 9, 33, 15, 6, 20, 22, 12, 22, 10, 10, 8, 6, 14, 21, 24, 20, 21, 10, 14, 8, 12, 12, 27, 6, 6, 10, 15, 20, 35, 28, 45, 27, 14, 12, 10, 9, 6, 32.
3. 2+ ways onward in view on 86% of steps. Any junction, doorway or room in view on 100% (junction 100%, doorway 100%, room 100%).
   Ways onward in view, by steps: 0: 0%, 1: 14%, 2: 42%, 3+: 44%.
4. Dead ends seen: 34; lengths (cells to the junction) 1×11, 2×11, 3×4, 4×2, 5×3, 6, 7, 8; 8 longer than 3.
   Walking back over trodden cells: 42 runs, 119 steps (20% of moves); longest 13, median 2.
5. Boxed in: 0 steps with nowhere to go; the world's way onward check: holds.
6. Seen cells that changed: 0 (must be 0).
   Cells appearing inside the view: on 1% of steps (6 steps closer in than the rim); 127 cells, 127 walkable.
   By distance from the player (6 is the rim): 2: 4, 3: 12, 4: 18, 5: 38, 6: 55.
7. (Judged by eye from the snapshots below.) Not yet known in view: median 35 of 169 cells; hedge cells stored: 0.

Time: new world 3.32 ms; a step (move and grow) median 0.01 ms, p95 0.70 ms, max 11.90 ms; patches per step median 0, max 10.
```

### Snapshots

```text
Seed 2 · at the start · 0 steps · 1 trodden cells in view
+--------------------------+
|┬───┤ └─┴─┐ ┌─╢ ║ ::::::::|
|└─┬─┴─┐ ::└─┴─╢ ║ ::::::::|
|──┘ ┌─┘ ::╔═╗ ║ ║ ::::::::|
|::::└─┐ ::║ ║ ║ ║ ::╷ ::::|
|┌─────┤ ::║ ╟─╢ ╟───┘ ::::|
|┴─┐ ::├─┐ ║ ║ ║ ║ ::::::::|
|┐ │ ::└─┘ ╚═@ ╚═╝ ::::::::|
|└─┘ ::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
+--------------------------+

Seed 2 · 1 west, 5 north of the start · 60 steps · 44 trodden cells in view
+--------------------------+
|║ · ║ ║ · ║ ┌─┐ ::╚═══════|
|╢ · ║ ╚═╤═╝ │ ├─┬─┐ ::::╔═|
|╚═╤═╝ ::│ ┌─┘ ├─┘ ╵ ::::║ |
|──┤ ┌───┘ └─┐ ╵ ╔═╗ ::::║ |
|::└─┘ ┌─┬─┐ └───╢ ║ ::::║ |
|┌─┬───┤ └─┴─┐ ┌─╢ ║ ::::╚═|
|┤ └─┬─┴─┐ ::@─┴─╢ ║ ::::::|
|└───┘ ┌─┘ ::╔═╗ ║ ║ ::::::|
|::::::└─┐ ::║ ║ ║ ║ ::╷ ::|
|╗ ┌─────┤ ::║ ╟─╢ ╟───┘ ::|
|╟─┴─┐ ::├─┐ ║ ║ ║ ║ ::::::|
|╟─┐ │ ::└─┘ ╚═╝ ╚═╝ ::::::|
|║ └─┘ ::::::::::::::::::::|
+--------------------------+

Seed 2 · 9 west, 7 south of the start · 120 steps · 22 trodden cells in view
+--------------------------+
|::::::┌─┐ ::::║ ║ └─┘ ::::|
|┬─┬─┬─┴─┘ ┌───╢ ║ ::::::::|
|╧═╧═╧═╗ ::│ ::╚═╝ ::::::::|
|══╤═╤═╝ ::│ ::::::::::::::|
|──┘ └─┬─┬─┘ ::::::::::::::|
|┬───┐ │ └─┐ ::::::::::::::|
|╧═══╧═╧═══╧═@ ::::::::::::|
|╤═════╤═════╝ ::::::::::::|
|┤ ╷ ┌─┤ ╷ ::::::::::::::::|
|┴─┼─┤ └─┤ ::::::::::::::::|
|┐ ├─┘ ::│ ::::::::::::::::|
|┤ ├─┐ ┌─┴───┐ ::::::::::::|
|└─┴─┴─┤ ╔═╗ ├─────┐ ::::::|
+--------------------------+

Seed 2 · 14 west, 20 south of the start · 180 steps · 19 trodden cells in view
+--------------------------+
|::::::::::::::┌─┤ ║ ║ └─┐ |
|::::::::::::::└─┘ ║ ║ ::├─|
|::::::::::::╔═══╗ ║ ╟─┬─┤ |
|::::::::::┌─╢ · ║ ╚═╝ │ ├─|
|::::::::::└─╢ · ║ ::::└─┘ |
|::::::::::::║ · ╟─┐ ┌─┬─┬─|
|::::::::::::@ · ╟─┴─┤ └─┘ |
|::::::::::::║ · ╟─┬─┘ ::::|
|::::::::::::╚═══╝ ├───────|
|::::::::::::::::::├─┐ ┌─┐ |
|::::::::::::┌─┐ ┌─┤ ├─┤ │ |
|::::::::::::│ └─┘ │ │ └─┴─|
|::::::::::::│ ::┌─┤ ╵ ::::|
+--------------------------+

Seed 2 · 14 west, 34 south of the start · 240 steps · 23 trodden cells in view
+--------------------------+
|┴───╴ ╚═╤═════╝ │ ::└─┼─┘ |
|┐ ┌───┐ │ ::┌───┘ ╔═╗ └─┐ |
|┴─┘ ┌─┤ └───┤ ::::║ ╟─┐ ├─|
|╗ ┌─┤ └───┬─┘ ┌─┬─╢ ╟─┴─┤ |
|╝ ├─┘ ╔═╗ │ ┌─┴─┴─╢ ║ ╶─┘ |
|::└─┬─╢ ╟─┤ │ ╶─┐ ║ ║ ::::|
|────┘ ║ ╟─┴─@───┘ ║ ║ ::::|
|┌─┐ ::╚═╝ ┌─┘ ::::╚═╝ ::::|
|╵ └─┐ ::╔═╧═╗ ::::::::::::|
|────┤ ::╚═══╝ ::::::::::::|
|┬─┐ └─┐ ::::::::::::::::::|
|└─┴─┬─┴─┬─┐ ::::::::::::::|
|┐ ┌─┤ ::└─┘ ::::::::::::::|
+--------------------------+

Seed 2 · 9 west, 29 south of the start · 300 steps · 69 trodden cells in view
+--------------------------+
|┘ ::::::├─┐ ┌─┐ ├─┴─┘ ::├─|
|┐ ┌─┐ ┌─┤ ├─┤ │ │ ::::::│ |
|┤ │ └─┘ │ │ └─┴─┘ ┌─────┘ |
|┘ │ ::┌─┤ ╵ ::::╔═╧═╗ ::::|
|══╧═╗ ├─┘ ┌─┬─┐ ╚═══╝ ::::|
|════╝ │ ::└─┼─┘ ╷ ┌─┐ ::::|
|::┌───┘ ╔═╗ @─┐ │ ├─┘ ::::|
|──┤ ::::║ ╟─┐ ├─┴─┘ ::::::|
|┬─┘ ┌─┬─╢ ╟─┴─┤ ::::::::::|
|│ ┌─┴─┴─╢ ║ ╶─┘ ::::::::::|
|┤ │ ╶─┐ ║ ║ ::::::::::::::|
|┴─┼───┘ ║ ║ ::::::::::::::|
|┌─┘ ::::╚═╝ ::::::::::::::|
+--------------------------+

Seed 2 · 18 west, 30 south of the start · 360 steps · 62 trodden cells in view
+--------------------------+
|╚═╤═══╤═╝ ::::┌─┴─┐ ┌─┐ ┌─|
|┌─┴─┐ └───┐ ::│ ┌─┤ │ └─┘ |
|├─┐ └─┐ ╶─┘ ┌─┘ └─┘ │ ::┌─|
|┤ └─┐ ├─┬───┘ ╔═════╧═╗ ├─|
|┤ ::│ ├─┴───╴ ╚═╤═════╝ │ |
|│ ┌─┤ └─┐ ┌───┐ │ ::┌───┘ |
|┤ └─┘ ┌─┴─┘ @─┤ └───┤ ::::|
|├─┐ ╔═╧═╗ ┌─┤ └───┬─┘ ┌─┬─|
|└─┘ ╚═╤═╝ ├─┘ ╔═╗ │ ┌─┴─┴─|
|┐ ::::│ ::└─┬─╢ ╟─┤ │ ╶─┐ |
|└─┐ ╷ ├─────┘ ║ ╟─┴─┼───┘ |
|┐ │ └─┤ ┌─┐ ::╚═╝ ┌─┘ ::::|
|│ │ ┌─┘ ╵ └─┐ ::╔═╧═╗ ::::|
+--------------------------+

Seed 2 · 17 west, 39 south of the start · 420 steps · 69 trodden cells in view
+--------------------------+
|::::│ ::└─┬─╢ ╟─┤ │ ╶─┐ ║ |
|┐ ╷ ├─────┘ ║ ╟─┴─┼───┘ ║ |
|│ └─┤ ┌─┐ ::╚═╝ ┌─┘ ::::╚═|
|│ ┌─┘ ╵ └─┐ ::╔═╧═╗ ::::::|
|└─┴─┬─────┤ ::╚═══╝ ::::::|
|┌─┬─┴─┬─┐ └─┐ ::::::::::::|
|├─┘ ::└─┴─┬─@─┬─┐ ::::::::|
|┤ ::┌─┐ ┌─┤ ::└─┘ ::::::::|
|└─┬─┘ └─┴─┘ ┌─┐ ::::╔═══╗ |
|::└─┬─┬───┐ ├─┘ ╔═╗ ║ · ║ |
|══╗ └─┤ ::├─┘ ╷ ║ ║ ║ · ║ |
|· ╟─┬─┘ ┌─┴─┐ │ ║ ║ ╚═╤═╝ |
|══╝ ├───┤ ::└─┤ ║ ║ ╔═╧═══|
+--------------------------+

Seed 2 · 23 west, 45 south of the start · 480 steps · 41 trodden cells in view
+--------------------------+
|┘ └─┘ ├───┤ ├─┘ ::└─┴─┬─┴─|
|┐ ╔═══╧═╗ └─┤ ::┌─┐ ┌─┤ ::|
|┘ ║ · · ╟─┐ └─┬─┘ └─┴─┘ ┌─|
|::╚═══╤═╝ │ ::└─┬─┬───┐ ├─|
|::╔═══╧═══╧═══╗ └─┤ ::├─┘ |
|::║ · · · · · ╟─┬─┘ ┌─┴─┐ |
|::╚═══════╤═@═╝ ├───┤ ::└─|
|::::┌─┐ ╷ │ ::::└─┐ └─┬─┐ |
|┐ ::└─┴─┤ │ ┌─┬─┬─┴─┬─┘ │ |
|┴─┐ ┌─┐ └─┘ └─┤ └─┐ └─┬─┴─|
|::│ ├─┤ ::::┌─┘ ╷ │ ┌─┘ ::|
|::└─┤ ├─┬───┘ ┌─┴─┴─┘ ::::|
|::::└─┘ └─────┘ ::::::::::|
+--------------------------+

Seed 2 · 11 west, 41 south of the start · 540 steps · 52 trodden cells in view
+--------------------------+
|╚═╝ ┌─┘ ::╷ ╚═╝ ┌─┤ └─╢ ║ |
|::╔═╧═╗ ┌─┼─┬─┬─┤ ├─┐ ║ ║ |
|::╚═══╝ └─┘ └─┤ │ └─┘ ║ ╟─|
|┐ ::::::┌───┬─┘ ├─┐ ::╚═╝ |
|┴─┬─┐ ::└─┐ └─┐ │ ├───────|
|::└─┘ ::::└─╴ │ └─┘ ::╔═══|
|┌─┐ ::::╔═══@ └─┐ ╷ ::║ · |
|├─┘ ╔═╗ ║ · ║ ::└─┘ ::║ · |
|┘ ╷ ║ ║ ║ · ║ ┌───────╢ · |
|┐ │ ║ ║ ╚═╤═╝ │ ::::::╚═══|
|└─┤ ║ ║ ╔═╧═══╧═══╗ ::╔═══|
|┐ │ ║ ║ ╚═╤═══════╝ ┌─╢ · |
|│ └─╢ ╟─┐ ├─┐ ::::╷ └─╢ · |
+--------------------------+

Seed 2 · 1 west, 33 south of the start · 600 steps · 24 trodden cells in view
+--------------------------+
|╚═══╝ ::::::::::│ ┌─────┴─|
|╷ ┌─┐ ::::::::╷ ├─┤ ┌───┬─|
|│ ├─┘ ::::::::└─┘ └─┘ ::│ |
|┴─┘ ::┌─┬─┬─┬─┬─╴ ┌─────┤ |
|╔═══╗ └─┤ │ ╵ └───┤ ┌───┴─|
|╚═╤═╝ ::└─┴─┐ ::::└─┘ ::::|
|┐ └───┐ ::╔═@═════════╗ ::|
|┤ ╔═╗ └─┐ ║ · · · · · ║ ┌─|
|└─╢ ║ ┌─┴─╢ · · · · · ║ ├─|
|┐ ║ ║ └───╢ · · · · · ╟─┘ |
|┘ ║ ╟───┐ ╚═╤═╤═╤═╤═╤═╝ ::|
|::╚═╝ ╷ │ ::├─┴─┤ ├─┘ ┌─┐ |
|──────┘ └───┼───┘ │ ┌─┤ ├─|
+--------------------------+

```

## Seed 3

```text
600 steps asked, 600 moved, 0 with nowhere to go; 2756 cells stored in 60 patches; way onward at the end: yes

1. Start: 3 ways out of the start cell, 3 of them lead onward within the view.
2. Rooms: 38 seen, 15 entered. First seen at step 0, first entered at step 1.
   Seen, never entered: 32 cells, seen at step 562; 35 cells, seen at step 517; 36 cells, seen at step 342; 51 cells, seen at step 314; 12 cells, seen at step 293; 26 cells, seen at step 281; 6 cells, seen at step 282; 9 cells, seen at step 257; 15 cells, seen at step 316; 12 cells, seen at step 227; 30 cells, seen at step 492; 16 cells, seen at step 480; 15 cells, seen at step 393; 6 cells, seen at step 469; 27 cells, seen at step 204; 12 cells, seen at step 0; 62 cells, seen at step 68; 8 cells, seen at step 22; 10 cells, seen at step 2; 8 cells, seen at step 62; 28 cells, seen at step 8; 21 cells, seen at step 34; 6 cells, seen at step 63.
   Room sizes in cells: 32, 35, 36, 51, 9, 12, 26, 12, 47, 6, 12, 8, 18, 28, 9, 12, 15, 20, 12, 30, 16, 15, 6, 27, 9, 35, 9, 27, 12, 62, 8, 10, 8, 18, 30, 28, 21, 6.
3. 2+ ways onward in view on 92% of steps. Any junction, doorway or room in view on 100% (junction 100%, doorway 100%, room 100%).
   Ways onward in view, by steps: 0: 0%, 1: 8%, 2: 41%, 3+: 50%.
4. Dead ends seen: 27; lengths (cells to the junction) 1×9, 2×8, 3×6, 4×3, 10; 4 longer than 3.
   Walking back over trodden cells: 43 runs, 112 steps (19% of moves); longest 11, median 2.
5. Boxed in: 0 steps with nowhere to go; the world's way onward check: holds.
6. Seen cells that changed: 0 (must be 0).
   Cells appearing inside the view: on 1% of steps (2 steps closer in than the rim); 51 cells, 51 walkable.
   By distance from the player (6 is the rim): 3: 3, 4: 12, 5: 12, 6: 24.
7. (Judged by eye from the snapshots below.) Not yet known in view: median 30 of 169 cells; hedge cells stored: 0.

Time: new world 6.91 ms; a step (move and grow) median 0.01 ms, p95 1.12 ms, max 6.19 ms; patches per step median 0, max 5.
```

### Snapshots

```text
Seed 3 · at the start · 0 steps · 1 trodden cells in view
+--------------------------+
|::::::::::::╔═╗ ::::::::::|
|::::::::::::║ ║ ::::::::::|
|┐ ::┌─┐ ::┌─╢ ║ ::::::::::|
|┤ ┌─┼─┘ ::└─╢ ║ ::::::::::|
|┤ ├─┘ ::::┌─╢ ║ ::::::::::|
|├─┤ ╔═╗ ::└─╢ ║ ╔═══╗ ::::|
|│ ├─╢ ║ ::::@ ╟─╢ · ║ ::::|
|┴─┤ ║ ║ ::::║ ║ ║ · ║ ::::|
|┌─┘ ║ ║ ::::╚═╝ ╚═╤═╝ ::::|
|╧═╗ ║ ╟───┬─┐ ┌─┐ │ ::::::|
|· ║ ║ ║ ::│ └─┘ └─┘ ::::::|
|╤═╝ ║ ╟───┘ ::::::::::::::|
|├─┐ ║ ╟─┐ ┌───┐ ::::::::::|
+--------------------------+

Seed 3 · 8 south of the start · 60 steps · 31 trodden cells in view
+--------------------------+
|┌─┘ ║ ║ ::::╚═╝ ╚═╤═╝ ::::|
|╧═╗ ║ ╟───┬─┐ ┌─┐ │ ::::::|
|· ║ ║ ║ ::│ └─┘ └─┘ ::::::|
|╤═╝ ║ ╟───┘ ::::::::::::::|
|├─┐ ║ ╟─┐ ┌───┐ ::::::::::|
|│ │ ╚═╝ └─┘ ┌─┘ ::::::::::|
|╧═╧═══╗ ::┌─@ ::::::::::::|
|══════╝ ┌─┤ ┌─┬─┐ ╔═══╗ ::|
|::::::┌─┴─┘ ├─┘ │ ║ · ║ ::|
|::::::└─┬───┘ ┌─┘ ║ · ╟───|
|┌─┐ ::::└─┬─┐ ├───╢ · ║ ::|
|┴─┤ ::::┌─┤ └─┼───╢ · ║ ┌─|
|::└───┐ ├─┴───┘ ┌─╢ · ║ │ |
+--------------------------+

Seed 3 · 8 west, 8 south of the start · 120 steps · 52 trodden cells in view
+--------------------------+
|┘ ::::::::└───╴ ┌─┘ ║ ║ ::|
|::::::::::::╔═══╧═╗ ║ ╟───|
|::::::::::::║ · · ║ ║ ║ ::|
|::::::::::::╚═══╤═╝ ║ ╟───|
|::::::::::::┌─┐ ├─┐ ║ ╟─┐ |
|::::::::::::├─┘ │ │ ╚═╝ └─|
|╗ ::::::::┌─@ ╔═╧═╧═══╗ ::|
|║ ::::::::└─┐ ╚═══════╝ ┌─|
|║ ::::╔═══╗ │ ::::::::┌─┴─|
|╟─┬─┐ ║ · ║ │ ::::::::└─┬─|
|╟─┴─┘ ║ · ║ │ ::┌─┐ ::::└─|
|║ ::::║ · ╟─┴─┬─┴─┤ ::::┌─|
|╟─┬─┐ ║ · ╟─┬─┘ ::└───┐ ├─|
+--------------------------+

Seed 3 · 17 west, 15 south of the start · 180 steps · 37 trodden cells in view
+--------------------------+
|┐ ├─┤ ┌─┬─┬─┤ ║ · ║ ::::::|
|┴─┘ │ └─┤ └─┘ ║ · ║ ::::╔═|
|::┌─┘ ::├───┐ ║ · ╟─┬─┐ ║ |
|──┘ ::::│ ::│ ║ · ╟─┴─┘ ║ |
|══╗ ::┌─┤ ::╵ ║ · ║ ::::║ |
|· ╟─┐ ├─┘ ╔═╗ ║ · ╟─┬─┐ ║ |
|══╝ ├─┘ ┌─╢ @─╢ · ╟─┘ └─╢ |
|──┐ └─┐ │ ║ ║ ╚═══╝ ┌───╢ |
|::└─┐ └─┤ ╚═╝ ::╷ ╔═╧═╗ ║ |
|::┌─┘ ::├─┐ ┌───┤ ║ · ║ ╚═|
|──┤ ::┌─┘ └─┴─┬─┘ ╚═══╝ ::|
|┐ └───┴─┐ ::::│ ::::::::::|
|┘ ::::┌─┘ ::::└─┐ ::::┌───|
+--------------------------+

Seed 3 · 22 west, 14 south of the start · 240 steps · 33 trodden cells in view
+--------------------------+
|╝ ┌─┘ ::┌───┐ │ ::└───┐ ╔═|
|::└───┬─┴─┐ ├─┤ ┌─┬─┬─┤ ║ |
|┐ ╶─┐ └───┴─┘ │ └─┤ └─┘ ║ |
|└─┬─┘ ┌─┐ ::┌─┘ ::├───┐ ║ |
|──┘ ╶─┘ └───┘ ::::│ ::│ ║ |
|════╗ ::╔═══╗ ::┌─┤ ::╵ ║ |
|════╝ ┌─╢ · @─┐ ├─┘ ╔═╗ ║ |
|::::┌─┘ ╚═══╝ ├─┘ ┌─╢ ╟─╢ |
|┌───┘ ::┌───┐ └─┐ │ ║ ║ ╚═|
|│ ┌───┬─┘ ::└─┐ └─┤ ╚═╝ ::|
|┴─┴───┤ ::::┌─┘ ::├─┐ ┌───|
|┌─────┘ ┌───┤ ::┌─┘ └─┴─┬─|
|┘ ::::┌─┴─┐ └───┴─┐ ┌─┐ │ |
+--------------------------+

Seed 3 · 38 west, 16 south of the start · 300 steps · 25 trodden cells in view
+--------------------------+
|║ · · ╟─┤ ║ · ╟─┤ ╔═══╗ ├─|
|╚═╤═╤═╝ │ ╚═══╝ └─╢ · ╟─┘ |
|::├─┘ ┌─┴─┐ ::╷ ::║ · ╟─┐ |
|╔═╧═╗ └─┬─┘ ::│ ::╚═══╝ │ |
|╚═══╝ ::│ ::::│ ┌───┬───┤ |
|::┌─┐ ┌─┘ ::┌─┴─┘ ┌─┤ ::│ |
|::│ └─┘ ╔═══@═╗ ::└─┤ ┌─┤ |
|──┤ ::::╚═════╝ ::::│ ├─┘ |
|::└─┬─────┐ ╔═══╗ ┌─┘ └───|
|──┬─┘ ::┌─┤ ║ · ╟─┤ ::::┌─|
|══╧═══╗ ├─┘ ╚═══╝ │ ::┌─┤ |
|══════╝ └─┐ ┌───┐ ├─┐ ├─┤ |
|┐ ::┌───┐ └─┴───┤ │ ├─┤ ╵ |
+--------------------------+

Seed 3 · 42 west, 20 south of the start · 360 steps · 51 trodden cells in view
+--------------------------+
|┘ ╚═╤═╝ ╚═══╝ ::│ ::::│ ┌─|
|::::└─┐ ::┌─┐ ┌─┘ ::┌─┴─┘ |
|::┌─┐ │ ::│ └─┘ ╔═══╧═╗ ::|
|┐ │ └─┴───┤ ::::╚═════╝ ::|
|┘ └─┬─┐ ::└─┬─────┐ ╔═══╗ |
|┬───┘ ├───┬─┘ ::┌─┤ ║ · ╟─|
|│ ::╔═╧═══╧═@═╗ ├─┘ ╚═══╝ |
|┤ ::╚═════════╝ └─┐ ┌───┐ |
|┘ ::┌───┐ ::┌───┐ └─┴───┤ |
|────┼─┬─┘ ::└─┐ │ ::┌─┐ └─|
|┌───┤ └───┬───┴─┴─┐ └─┴─┐ |
|╧═══╧═╗ ╔═╧═══╗ ┌─┤ ::┌─┤ |
|· · · ╟─╢ · · ╟─┤ └───┘ └─|
+--------------------------+

Seed 3 · 32 west, 14 south of the start · 420 steps · 78 trodden cells in view
+--------------------------+
|· ║ ::┌─┤ ┌─┐ ╚═══╤═╝ ┌─┘ |
|· ╟─┬─┘ └─┘ │ ::┌─┤ ::└───|
|· ╟─┤ ╔═══╗ ├─┐ ╵ ├─┐ ╶─┐ |
|══╝ └─╢ · ╟─┘ └───┘ └─┬─┘ |
|::╷ ::║ · ╟─┐ ::::┌───┘ ╶─|
|::│ ::╚═══╝ │ ╔═══╧═════╗ |
|::│ ┌───┬───@ ╚═════════╝ |
|┌─┴─┘ ┌─┤ ::│ ┌─┬─┐ ::::┌─|
|╧═╗ ::└─┤ ┌─┤ │ │ │ ┌───┘ |
|══╝ ::::│ ├─┘ └─┴─┤ │ ┌───|
|╔═══╗ ┌─┘ └─────┐ ├─┴─┴───|
|║ · ╟─┤ ::::┌─┐ └─┘ ┌─────|
|╚═══╝ │ ::┌─┤ │ ::┌─┘ ::::|
+--------------------------+

Seed 3 · 29 west, 9 north of the start · 480 steps · 11 trodden cells in view
+--------------------------+
|::::::::::╚═╤═╤═╝ ::├─┬─┐ |
|════╗ ::::::└─┴─┐ ::└─┘ │ |
|· · ╟─┐ ::┌─┐ ┌─┴─┐ ::┌─┘ |
|· · ║ └─┬─┴─┘ └─┐ └─┐ │ ::|
|╤═╤═╝ ::│ ::┌─┐ │ ::│ └─┐ |
|┘ ├─────┘ ┌─┘ └─┘ ┌─┴───┴─|
|──┴───────┴─@ ::╔═╧═╗ ::::|
|┐ ::::╶─┐ ::└─┐ ╚═╤═╝ ::::|
|┴───┬───┘ ::::└─┐ └─┬───┬─|
|::┌─┘ ::┌─┐ ::┌─┤ ::└───┘ |
|::└─┐ ::│ └─┬─┘ ├─┐ ┌─┐ ::|
|──┐ └─┐ └─┐ │ ╷ ╵ │ │ ├───|
|::│ ::└─┐ │ │ ├─┐ └─┤ │ ┌─|
+--------------------------+

Seed 3 · 40 west, 10 north of the start · 540 steps · 49 trodden cells in view
+--------------------------+
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::╔═════|
|::::::┌─┬───┐ ::╔═╗ ║ · · |
|::::::├─┴───┤ ::║ ╟─╢ · · |
|::::::├───┬─┴─┬─╢ ║ ╚═╤═╤═|
|┌─┐ ╔═╧═══╧═@ │ ║ ║ ┌─┘ ├─|
|┤ │ ║ · · · ║ │ ║ ║ └───┴─|
|└─┘ ╚═════╤═╝ │ ╚═╝ ┌─┐ ::|
|┬─┐ ╔═════╧═══╧═══╗ └─┴───|
|└─┴─╢ · · · · · · ╟─┐ ::┌─|
|::::║ · · · · · · ╟─┘ ::└─|
|::::╚═════╤═══╤═╤═╝ ┌───┐ |
+--------------------------+

Seed 3 · 40 west of the start · 600 steps · 38 trodden cells in view
+--------------------------+
|└─┴─╢ · · · · · · ╟─┐ ::┌─|
|::::║ · · · · · · ╟─┘ ::└─|
|::::╚═════╤═══╤═╤═╝ ┌───┐ |
|::::::┌─┬─┴─┐ │ │ ┌─┘ ::│ |
|══╗ ::├─┴─┬─┴─┴─┤ │ ┌─┐ │ |
|· ╟───┘ ╔═╧═╗ ┌─┤ ├─┤ └─┴─|
|══╝ ::┌─╢ · @ └─┴─┘ │ ::┌─|
|::::::├─╢ · ╟───┐ ╷ │ ::├─|
|::::::│ ╚═══╝ ::│ │ └─┐ ├─|
|::::┌─┤ ::::::┌─┴─┘ ┌─┤ └─|
|::┌─┘ │ ::::::└─┐ ┌─┘ └─┬─|
|┌─┤ ┌─┘ ::::::::└─┴─┐ ╷ │ |
|└─┘ └─┐ ::::::╔═══╗ │ └─┘ |
+--------------------------+

```

## Seed 4

```text
600 steps asked, 600 moved, 0 with nowhere to go; 1514 cells stored in 32 patches; way onward at the end: yes

1. Start: 2 ways out of the start cell, 2 of them lead onward within the view.
2. Rooms: 25 seen, 13 entered. First seen at step 0, first entered at step 2.
   Seen, never entered: 18 cells, seen at step 0; 24 cells, seen at step 566; 6 cells, seen at step 544; 10 cells, seen at step 132; 26 cells, seen at step 132; 6 cells, seen at step 498; 12 cells, seen at step 133; 6 cells, seen at step 311; 10 cells, seen at step 332; 16 cells, seen at step 337; 8 cells, seen at step 344; 26 cells, seen at step 351.
   Room sizes in cells: 18, 33, 24, 9, 8, 16, 10, 6, 24, 12, 6, 15, 10, 26, 15, 6, 15, 12, 6, 6, 10, 16, 8, 24, 26.
3. 2+ ways onward in view on 83% of steps. Any junction, doorway or room in view on 100% (junction 100%, doorway 100%, room 100%).
   Ways onward in view, by steps: 0: 0%, 1: 17%, 2: 43%, 3+: 40%.
4. Dead ends seen: 12; lengths (cells to the junction) 1×4, 2×6, 3, 6; 1 longer than 3.
   Walking back over trodden cells: 47 runs, 169 steps (28% of moves); longest 14, median 2.
5. Boxed in: 0 steps with nowhere to go; the world's way onward check: holds.
6. Seen cells that changed: 0 (must be 0).
   Cells appearing inside the view: on 0% of steps (3 steps closer in than the rim); 121 cells, 121 walkable.
   By distance from the player (6 is the rim): 2: 5, 3: 16, 4: 32, 5: 33, 6: 35.
7. (Judged by eye from the snapshots below.) Not yet known in view: median 44 of 169 cells; hedge cells stored: 0.

Time: new world 14.86 ms; a step (move and grow) median 0.01 ms, p95 0.02 ms, max 8.45 ms; patches per step median 0, max 6.
```

### Snapshots

```text
Seed 4 · at the start · 0 steps · 1 trodden cells in view
+--------------------------+
|::::::├─┘ └─┐ ╔═══╗ ::::::|
|::::::├─┐ ┌─┴─╢ · ║ ::::::|
|::┌─┬─┘ ├─┤ ::║ · ║ ::::::|
|┌─┘ └───┴─┘ ::║ · ║ ::┌─┬─|
|╧═══════════╗ ║ · ║ ::└─┘ |
|╤═╤═╤═╤═╤═╤═╝ ║ · ╟───┬───|
|┘ │ ├─┘ └─┤ @─╢ · ║ ┌─┤ ┌─|
|──┘ │ ::::│ └─╢ · ╟─┤ └─┴─|
|┐ ┌─┴─┐ ┌─┘ ::║ · ║ └───┬─|
|┘ │ ::└─┤ ::::║ · ║ ┌───┴─|
|::├─┬─┬─┤ ::::╚═╤═╝ │ ┌─┐ |
|┌─┘ ├─┤ │ ::::::│ ::├─┘ ├─|
|┘ ::└─┴─┘ ::::::└───┘ ┌─┤ |
+--------------------------+

Seed 4 · 4 east of the start · 60 steps · 57 trodden cells in view
+--------------------------+
|┘ └─┐ ╔═══╗ ::::::::╔═══╗ |
|┐ ┌─┴─╢ · ║ ::::::::║ · ╟─|
|├─┤ ::║ · ║ ::::::┌─╢ · ╟─|
|┴─┘ ::║ · ║ ::┌─┬─┘ ╚═╤═╝ |
|════╗ ║ · ║ ::└─┘ ::╔═╧═╗ |
|╤═╤═╝ ║ · ╟───┬─────╢ · ║ |
|└─┤ ┌─╢ · ║ @─┤ ┌───╢ · ║ |
|::│ └─╢ · ╟─┤ └─┴─┐ ╚═╤═╝ |
|┌─┘ ::║ · ║ └───┬─┴─┐ │ ::|
|┤ ::::║ · ║ ┌───┴─┐ └─┤ ╔═|
|┤ ::::╚═╤═╝ │ ┌─┐ ├───┘ ║ |
|│ ::::::│ ::├─┘ ├─┴─┐ ::║ |
|┘ ::::::└───┘ ┌─┤ ╶─┴─┐ ╚═|
+--------------------------+

Seed 4 · 18 east, 4 north of the start · 120 steps · 28 trodden cells in view
+--------------------------+
|::::::::::::::::::::::::::|
|╗ ::::::::╷ ::╔═══════╗ ::|
|║ ::::::::│ ┌─╢ · · · ║ ::|
|╝ ┌─┬─┬─┬─┤ │ ╚═══════╝ ::|
|──┘ │ │ └─┤ └─┐ ::::::::::|
|┬───┘ │ ┌─┘ ┌─┘ ::::::::::|
|┴─────┴─┼───@───┐ ::::::::|
|════════╧═══╗ ::├─╴ ::::::|
|══════════╤═╝ ╷ │ ::::::::|
|══════╗ ::├─┐ └─┤ ::::::::|
|══╤═══╝ ::├─┤ ┌─┤ ::┌───┐ |
|──┤ ┌─┬─┬─┤ └─┘ │ ::├───┘ |
|┐ └─┘ ├─┘ ├───┐ │ ╔═╧═╗ ::|
+--------------------------+

Seed 4 · 11 east, 9 north of the start · 180 steps · 41 trodden cells in view
+--------------------------+
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::╔═@═╗ ::::::::╷ |
|::::::::::║ · ║ ::::::::│ |
|::::::::::╚═╤═╝ ┌─┬─┬─┬─┤ |
|::::::╔═══╗ └───┘ │ │ └─┤ |
|::::::║ · ╟───┬───┘ │ ┌─┘ |
|::::┌─╢ · ╟───┴─────┴─┼───|
|┌─┬─┘ ╚═╤═╝ ╔═════════╧═══|
+--------------------------+

Seed 4 · 13 east, 1 north of the start · 240 steps · 107 trodden cells in view
+--------------------------+
|::::::╚═╤═╝ ┌─┬─┬─┬─┤ │ ╚═|
|::╔═══╗ └───┘ │ │ └─┤ └─┐ |
|::║ · ╟───┬───┘ │ ┌─┘ ┌─┘ |
|┌─╢ · ╟───┴─────┴─┼───┴───|
|┘ ╚═╤═╝ ╔═════════╧═══╗ ::|
|::╔═╧═╗ ╚═══════════╤═╝ ╷ |
|──╢ · ║ ╔═══@═══╗ ::├─┐ └─|
|──╢ · ║ ╚═══╤═══╝ ::├─┤ ┌─|
|┐ ╚═╤═╝ ┌───┤ ┌─┬─┬─┤ └─┘ |
|┴─┐ │ ::└─┐ └─┘ ├─┘ ├───┐ |
|┐ └─┤ ╔═╗ │ ::╔═╧═╗ └───┘ |
|├───┘ ║ ╟─┘ ::╚═══╝ ::::::|
|┴─┐ ::║ ║ ::::::::::::::::|
+--------------------------+

Seed 4 · 19 east, 1 south of the start · 300 steps · 66 trodden cells in view
+--------------------------+
|──┘ │ ┌─┘ ┌─┘ ::╔═══════╗ |
|────┴─┼───┴───┐ ╚═╤═════╝ |
|══════╧═══╗ ::├─╴ ├─┐ ┌─┐ |
|════════╤═╝ ╷ │ ╔═╧═╧═╧═╧═|
|════╗ ::├─┐ └─┤ ╚═════════|
|╤═══╝ ::├─┤ ┌─┤ ::┌───┐ ╔═|
|┤ ┌─┬─┬─┤ └─@ │ ::├───┘ ╚═|
|└─┘ ├─┘ ├───┐ │ ╔═╧═╗ ::::|
|::╔═╧═╗ └───┘ └─╢ · ╟───┬─|
|::╚═══╝ ::::::::║ · ║ ┌─┘ |
|::::::::::::::::║ · ║ └───|
|::::::::::::::::╚═══╝ ::::|
|::::::::::::::::::::::::::|
+--------------------------+

Seed 4 · 34 east, 12 south of the start · 360 steps · 20 trodden cells in view
+--------------------------+
|::::::║ ║ │ └─╢ ╟─┤ ┌─┬─╢ |
|┐ ::::║ ╟─┘ ::╚═╝ ├─┘ ├─╢ |
|│ ::::╚═╝ ::::::::├───┘ ╚═|
|├─┬───┬─┐ ::::┌─┬─┤ ::::::|
|└─┘ ┌─┤ └─┐ ::└─┴─┘ ::::::|
|┌─┬─┴─┘ ┌─┴─────┐ ::::::::|
|┤ │ ::::├─┐ @───┴─┐ ┌─┐ ::|
|├─┘ ::::├─┴─┴───┬─┘ │ └───|
|├─╴ ╔═╗ └─┬─┐ ┌─┘ ┌─┤ ::::|
|│ ::║ ╟───┤ │ └─┐ └─┤ ::::|
|╧═╗ ╚═╝ ::│ │ ╔═╧═══╧═══╗ |
|· ║ ┌─┬─┐ ├─┴─╢ · · · · ╟─|
|══╝ └─┘ ├─┼─┬─╢ · · · · ╟─|
+--------------------------+

Seed 4 · 40 east, 16 south of the start · 420 steps · 47 trodden cells in view
+--------------------------+
|::└─┴─┘ ::::::::║ ║ ::::::|
|────┐ ::::::::::║ ║ ::::::|
|┌───┴─┐ ┌─┐ ::::╚═╝ ::::::|
|┴───┬─┘ │ └───╴ ::::::::::|
|┐ ┌─┘ ┌─┤ ::::::::::::::::|
|│ └─┐ └─┤ ::::::::::::::::|
|│ ╔═╧═══╧═══@ ::::::::::::|
|┴─╢ · · · · ╟─┐ ::::::::::|
|┬─╢ · · · · ╟─┘ ::::::::::|
|│ ╚═════════╝ ::::::::::::|
|├─┐ ::::::::::::::::::::::|
|┤ │ ::::::::::::::::::::::|
|└─┘ ::::::::::::::::::::::|
+--------------------------+

Seed 4 · 31 east, 15 south of the start · 480 steps · 78 trodden cells in view
+--------------------------+
|::::::├─┬───┬─┐ ::::┌─┬─┤ |
|┐ ┌─┐ └─┘ ┌─┤ └─┐ ::└─┴─┘ |
|├─┤ ╵ ┌─┬─┴─┘ ┌─┴─────┐ ::|
|┘ │ ┌─┤ │ ::::├─┐ ┌───┴─┐ |
|::└─┤ ├─┘ ::::├─┴─┴───┬─┘ |
|┌─┐ │ ├─╴ ╔═╗ └─┬─┐ ┌─┘ ┌─|
|┘ ├─┘ │ ::║ @───┤ │ └─┐ └─|
|╔═╧═══╧═╗ ╚═╝ ::│ │ ╔═╧═══|
|╢ · · · ║ ┌─┬─┐ ├─┴─╢ · · |
|╚═══════╝ └─┘ ├─┼─┬─╢ · · |
|::::::::::::┌─┴─┘ │ ╚═════|
|::::::::::::│ ::::├─┐ ::::|
|::::::::::::└─────┤ │ ::::|
+--------------------------+

Seed 4 · 21 east, 13 south of the start · 540 steps · 29 trodden cells in view
+--------------------------+
|│ ::┌─┐ ├─┘ ┌─┐ ::::::::└─|
|┤ ┌─┴─┘ │ ::├─┘ ::::::::::|
|│ └─┐ ::├─┬─┤ ::::::::::::|
|┘ ::│ ::│ └─┘ ::::┌─┐ ┌─┐ |
|::╔═╧═╗ └─┐ ┌───┬─┘ ├─┤ ╵ |
|::║ · ╟─┬─┘ │ ╷ └─┬─┘ │ ┌─|
|::║ · ║ ├───@─┘ ::│ ::└─┤ |
|::╚═══╝ │ ::::┌───┘ ┌─┐ │ |
|::::::::└─┬─┐ │ ┌───┘ ├─┘ |
|::::::::::└─┘ └─┘ ::╔═╧═══|
|::::::::::::::╔═╗ ┌─╢ · · |
|::::::::::::::║ ╟─┘ ╚═════|
|::::::::::::::╚═╝ ::::::::|
+--------------------------+

Seed 4 · 20 east, 6 south of the start · 600 steps · 107 trodden cells in view
+--------------------------+
|══╝ ::├─┤ ┌─┤ ::┌───┐ ╔═╧═|
|┌─┬─┬─┤ └─┘ │ ::├───┘ ╚═══|
|┘ ├─┘ ├───┐ │ ╔═╧═╗ ::::::|
|╔═╧═╗ └───┘ └─╢ · ╟───┬───|
|╚═══╝ ::┌─┐ ::║ · ║ ┌─┘ ::|
|┌───┬─┐ │ ├─┐ ║ · ║ └───┐ |
|│ ┌─┘ └─┴─┤ @ ╚═══╝ ::::└─|
|┘ │ ::┌─┐ ├─┘ ┌─┐ ::::::::|
|──┤ ┌─┴─┘ │ ::├─┘ ::::::::|
|┐ │ └─┐ ::├─┬─┤ ::::::::::|
|┴─┘ ::│ ::│ └─┘ ::::┌─┐ ┌─|
|┐ ::╔═╧═╗ └─┐ ┌───┬─┘ ├─┤ |
|┤ ::║ · ╟─┬─┘ │ ╷ └─┬─┘ │ |
+--------------------------+

```

## Seed 5

```text
600 steps asked, 600 moved, 0 with nowhere to go; 2777 cells stored in 76 patches; way onward at the end: yes

1. Start: 2 ways out of the start cell, 2 of them lead onward within the view.
2. Rooms: 50 seen, 18 entered. First seen at step 0, first entered at step 1.
   Seen, never entered: 15 cells, seen at step 507; 12 cells, seen at step 504; 32 cells, seen at step 550; 20 cells, seen at step 506; 10 cells, seen at step 538; 14 cells, seen at step 510; 8 cells, seen at step 550; 6 cells, seen at step 433; 24 cells, seen at step 0; 33 cells, seen at step 95; 15 cells, seen at step 0; 22 cells, seen at step 4; 12 cells, seen at step 399; 16 cells, seen at step 0; 27 cells, seen at step 229; 18 cells, seen at step 121; 12 cells, seen at step 11; 6 cells, seen at step 232; 27 cells, seen at step 168; 10 cells, seen at step 20; 9 cells, seen at step 239; 6 cells, seen at step 207; 16 cells, seen at step 141; 10 cells, seen at step 293; 8 cells, seen at step 64; 6 cells, seen at step 268; 9 cells, seen at step 65; 6 cells, seen at step 66; 12 cells, seen at step 282; 18 cells, seen at step 184; 9 cells, seen at step 277; 6 cells, seen at step 291.
   Room sizes in cells: 15, 12, 32, 20, 10, 14, 8, 14, 6, 15, 30, 6, 24, 33, 21, 15, 22, 20, 12, 16, 16, 28, 27, 20, 18, 32, 12, 6, 16, 27, 10, 10, 10, 12, 12, 9, 10, 8, 6, 16, 10, 6, 8, 6, 9, 6, 12, 18, 9, 6.
3. 2+ ways onward in view on 85% of steps. Any junction, doorway or room in view on 100% (junction 100%, doorway 100%, room 100%).
   Ways onward in view, by steps: 0: 0%, 1: 15%, 2: 40%, 3+: 45%.
4. Dead ends seen: 32; lengths (cells to the junction) 1×8, 2×11, 3×4, 4×5, 5, 6×2, 11; 9 longer than 3.
   Walking back over trodden cells: 43 runs, 118 steps (20% of moves); longest 10, median 2.
5. Boxed in: 0 steps with nowhere to go; the world's way onward check: holds.
6. Seen cells that changed: 0 (must be 0).
   Cells appearing inside the view: on 1% of steps (7 steps closer in than the rim); 99 cells, 99 walkable.
   By distance from the player (6 is the rim): 3: 8, 4: 18, 5: 32, 6: 41.
7. (Judged by eye from the snapshots below.) Not yet known in view: median 23 of 169 cells; hedge cells stored: 0.

Time: new world 22.81 ms; a step (move and grow) median 0.01 ms, p95 0.71 ms, max 12.19 ms; patches per step median 0, max 7.
```

### Snapshots

```text
Seed 5 · at the start · 0 steps · 1 trodden cells in view
+--------------------------+
|├─┴───┘ ├─┐ ╚═╤═╤═══╤═════|
|└─┬─────┘ │ ┌─┤ └─┬─┘ ::::|
|──┤ ╔═══╗ └─┴─┘ ╔═╧═══╗ ::|
|::├─╢ · ║ ::::::║ · · ╟─┬─|
|┌─┴─╢ · ║ ┌─┐ ::║ · · ╟─┘ |
|┤ ::║ · ╟─┘ └─┐ ║ · · ╟───|
|╧═╗ ╚═══╝ ╔═@ │ ║ · · ║ ::|
|· ╟─┐ ::::║ ║ └─╢ · · ║ ::|
|· ║ ├─╴ ::║ ║ ::╚═╤═╤═╝ ::|
|· ╟─┴─┐ ::║ ║ ┌───┘ └───┬─|
|· ╟───┴─┐ ║ ║ └─┐ ┌─┬─┐ └─|
|· ╟─┬───┘ ║ ╟───┘ ├─┴─┘ ::|
|· ╟─┤ ┌─┐ ║ ╟─────┴─┐ ┌───|
+--------------------------+

Seed 5 · 16 east, 2 south of the start · 60 steps · 21 trodden cells in view
+--------------------------+
|╧═════╧═══╗ ::::::└─┤ ┌─┘ |
|· · · · · ║ ::::::┌─┘ └─┐ |
|· · · · · ║ ::::::└─────┴─|
|╤═══════╤═╝ ╔═╗ ::┌───┐ ::|
|├─┬───┬─┘ ┌─╢ ║ ╷ ╵ ::├───|
|┘ ├─┐ └───┘ ║ ║ └─┐ ::└─┐ |
|::│ │ ┌─────@ ╟─┐ └─────┴─|
|╴ ├─┴─┴─┐ ::╚═╝ └─────┬─┐ |
|──┘ ╔═══╧═══╗ ┌─┬─┐ ::└─┘ |
|══╗ ╚═══════╝ └─┘ └─┬─┬─┐ |
|· ║ ╷ ┌───┐ ::::::┌─┴─┘ └─|
|══╝ │ ├─┬─┘ ::::┌─┤ ::::┌─|
|┐ ┌─┴─┘ ├─┐ ┌───┤ │ ::::└─|
+--------------------------+

Seed 5 · 10 east, 4 north of the start · 120 steps · 55 trodden cells in view
+--------------------------+
|║ ::::::::┌─┘ └─┘ ::┌─┐ ╵ |
|╝ ┌───┐ ::├─┐ ╔═╗ ::├─┘ ┌─|
|──┤ ::└─┐ ├─┘ ║ ║ ┌─┘ ╔═╧═|
|══╧═══╗ └─┤ ::║ ║ └───╢ · |
|╤═════╝ ┌─┴───╢ ║ ::┌─╢ · |
|┘ ::::┌─┘ ┌─┐ ╚═╝ ┌─┘ ╚═══|
|══╗ ::│ ╔═╧═@═════╧═══╗ ::|
|· ╟─┬─┤ ║ · · · · · · ║ ::|
|· ╟─┘ └─╢ · · · · · · ║ ::|
|· ╟───┐ ╚═══╤═══════╤═╝ ╔═|
|· ║ ::└───┐ ├─┬───┬─┘ ┌─╢ |
|· ║ ::::┌─┴─┘ ├─┐ └───┘ ║ |
|╤═╝ ::::└─┐ ::│ │ ┌─────╢ |
+--------------------------+

Seed 5 · 21 east, 13 north of the start · 180 steps · 24 trodden cells in view
+--------------------------+
|· · · · ╟─┘ ║ ║ ┌─┘ ::::::|
|╤═╤═══╤═╝ ::║ ║ │ ::::::::|
|┘ │ ::└───┬─╢ ╟─┤ ::::::::|
|┬─┴───┬───┘ ║ ║ ├───┐ ::::|
|└───┐ │ ┌─┐ ╚═╝ │ ::└─────|
|┌─┐ ├─┘ │ └─┬─┐ └───┬───┐ |
|├─┴─┴─┐ ├───@─┴─┬─┬─┘ ┌─┘ |
|│ ::╔═╧═╧═╗ ::┌─┤ └───┘ ::|
|└─┐ ╚═══╤═╝ ::│ │ ::┌─┬───|
|┐ ╵ ┌───┤ ┌───┘ ╵ ┌─┤ └───|
|┘ ┌─┘ ┌─┘ ├─┐ ┌─┬─┤ ├─┬─┐ |
|╔═╧═╗ ├─┐ ├─┘ └─┤ └─┘ └─┴─|
|╢ · ╟─┤ ├─┘ ┌─┐ └─┬───┬───|
+--------------------------+

Seed 5 · 14 east, 34 north of the start · 240 steps · 14 trodden cells in view
+--------------------------+
|::::::::::::::::╚═══╝ │ ::|
|::::::::::::::::┌─────┴───|
|::::::::::::::┌─┤ ::┌─┬───|
|::::::╔═╗ ::┌─┘ ├───┤ │ ┌─|
|┬─┬───╢ ╟───┤ ╷ └─┬─┤ │ └─|
|┤ ├─┐ ╚═╝ ::└─┴─┬─┘ └─┘ ┌─|
|│ │ ╵ ╔═════@ ::└─┐ ::::│ |
|┘ └─┐ ║ · · ║ ::┌─┤ ::::├─|
|┌─┬─┘ ║ · · ║ ┌─┼─┘ ┌─┬─┘ |
|│ │ ::╚═╤═╤═╝ └─┴─┬─┴─┘ ╔═|
|└─┤ ┌─┐ │ └─┬─┐ ┌─┘ ::::╚═|
|──┘ │ ╵ └─┐ └─┴─┴─┬─┬───┬─|
|╗ ::└─┐ ╔═╧═════╗ ├─┴───┴─|
+--------------------------+

Seed 5 · 26 east, 24 north of the start · 300 steps · 39 trodden cells in view
+--------------------------+
|╚═╤═╝ │ ::└─┘ ├─┬───────┐ |
|┬─┴─┐ └─┬─┐ ::└─┴─┐ ::::│ |
|┴───┘ ::└─┤ ╶───┐ │ ::::└─|
|┬─┬───┐ ┌─┴─┐ ::├─┴─┬─┬───|
|├─┴─┐ └─┘ ┌─┘ ::│ ┌─┘ ├─┐ |
|┴─┐ ├───┐ └───┐ │ │ ::│ └─|
|::│ └───┘ ╔═@ │ └─┘ ┌─┴───|
|::└─────┐ ║ ╟─┘ ╔═══╧═════|
|::╔═╗ ┌─┘ ╚═╝ ::╚═════╤═══|
|──╢ ║ └───┐ ::::::┌─┬─┴───|
|┌─╢ ║ ::┌─┘ ::┌───┤ └─┐ ::|
|┘ ║ ║ ┌─┘ ┌───┘ ::│ ┌─┘ ╔═|
|::║ ║ │ ╔═╧═════╗ └─┴─┐ ╚═|
+--------------------------+

Seed 5 · 13 east, 23 north of the start · 360 steps · 52 trodden cells in view
+--------------------------+
|└───┘ │ ╵ └─┐ └─┴─┴─┬─┬───|
|══╗ ::└─┐ ╔═╧═════╗ ├─┴───|
|· ╟─────┤ ╚═════╤═╝ │ ┌─┬─|
|· ║ ┌─┬─┘ ::::::└───┴─┘ │ |
|· ║ └─┤ ::╶───┐ ╔═══╗ ┌─┴─|
|══╝ ::└───┐ ┌─┼─╢ · ╟─┤ ::|
|::::::::::└─@ └─╢ · ╟─┤ ::|
|::::::::::::└─┐ ╚═══╝ │ ::|
|::::::::::::::└─┬───┐ ├───|
|::::::::╔═══════╧═══╧═╧═╗ |
|::::::┌─╢ · · · · · · · ╟─|
|::::::│ ╚═════╤═╤═╤═══╤═╝ |
|::::::└─┐ ::::└─┘ │ ::└───|
+--------------------------+

Seed 5 · 1 east, 29 north of the start · 420 steps · 25 trodden cells in view
+--------------------------+
|::::::::::::::::::║ · ╟─┴─|
|::::::::::::::::::║ · ║ ::|
|::::::::::::::::::║ · ╟───|
|::╶─────┐ ::::::::║ · ║ ╷ |
|::::::::├─┬─┬───┐ ║ · ╟─┘ |
|::::::::└─┤ └───┘ ║ · ╟─┐ |
|┌─┐ ┌─┐ ╔═╧═@═══╗ ╚═══╝ └─|
|└─┴─┤ ├─╢ · · · ║ ::╔═════|
|┌─┬─┘ │ ║ · · · ╟─┬─╢ · · |
|┴─┘ ┌─┘ ╚═══════╝ │ ║ · · |
|╔═══╧═══════╗ ::┌─┘ ║ · · |
|║ · · · · · ║ ┌─┘ ::╚═════|
|╚═══════╤═══╝ │ ::::::::::|
+--------------------------+

Seed 5 · 4 west, 32 north of the start · 480 steps · 32 trodden cells in view
+--------------------------+
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::::::::::::::::|
|::::::::::::@─────┐ ::::::|
|::::::::::::::::::├─┬─┬───|
|╔═╗ ::::::::::::::└─┤ └───|
|╢ ║ ╔═══╗ ┌─┐ ┌─┐ ╔═╧═════|
|╚═╝ ╚═╤═╝ └─┴─┤ ├─╢ · · · |
|┐ ┌───┤ ::┌─┬─┘ │ ║ · · · |
|│ ├─┐ └─┬─┴─┘ ┌─┘ ╚═══════|
+--------------------------+

Seed 5 · 9 west, 19 north of the start · 540 steps · 24 trodden cells in view
+--------------------------+
|════╝ ╶─┐ └─┘ └───┤ ╔═══╧═|
|══╗ ╷ ┌─┘ ╔═══╗ ┌─┘ ║ · · |
|· ╟─┘ └─┐ ║ · ║ ├─┐ ╚═════|
|══╝ ::┌─┘ ║ · ║ └─┤ ::┌─┬─|
|┬───┐ │ ┌─╢ · ║ ┌─┘ ┌─┴─┘ |
|┘ ┌─┘ │ │ ╚═╤═╝ │ ::│ ┌─┬─|
|┬─┴─╴ └─┴─┬─@───┘ ::└─┤ └─|
|┘ ::::::╔═╧═════════╗ └───|
|::::╶─┐ ╚═╤═══╤═╤═╤═╝ ┌─┬─|
|┐ ┌───┤ ╷ └─┐ └─┘ └───┘ └─|
|├─┴─┐ └─┘ ╔═╧═════════════|
|│ ::└───┐ ║ · · · · · · · |
|╧═════╗ │ ╚═══╤═╤═╤═╤═╤═╤═|
+--------------------------+

Seed 5 · 6 west, 12 north of the start · 600 steps · 39 trodden cells in view
+--------------------------+
|::╔═╧═════════╗ └───┴─╢ · |
|┐ ╚═╤═══╤═╤═╤═╝ ┌─┬─┐ ║ · |
|┤ ╷ └─┐ └─┘ └───┘ └─┘ ╚═══|
|└─┘ ╔═╧═══════════════╗ ::|
|──┐ ║ · · · · · · · · ║ ::|
|╗ │ ╚═══╤═╤═╤═╤═╤═╤═══╝ ::|
|╝ ├─┐ ::│ │ @─┴─┘ ├─┐ ┌───|
|╔═╧═╧═╗ ├─┤ ╔═════╧═╧═╧═══|
|╚═══╤═╝ │ │ ║ · · · · · · |
|┌─┐ ├───┴─┤ ╚═╤═╤═╤═╤═╤═╤═|
|│ │ └─┬─┐ │ ┌─┘ ├─┤ └─┤ └─|
|┴─┴───┘ │ └─┤ ┌─┘ │ ┌─┘ ╔═|
|::┌─┬─┐ │ ::├─┴───┘ ├─┐ ╚═|
+--------------------------+

```

