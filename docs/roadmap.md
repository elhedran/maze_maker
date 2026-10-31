---
type: Roadmap
title: Maze Maker roadmap
description: Phases, deliverables and demos for Maze Maker, with Phase 1 sliced into small steps that each end in a demo.
status: draft
owners: [product-manager/1]
approvers: [human:ian]
tags: [roadmap, phase-1, phase-2, adjacency-table, tooling, demos]
generated:
  by: product-manager/1
  at: 2026-10-31T15:05:00+01:00
---

# Roadmap

Draft by Marram (product manager), waiting on Ian's approval. It reflects input from Yew, Wren and Damson, given on 2026-10-31.

## Now and next

- **Now:**
  - Ian answers [Q-0003](/questions/open/0003-where-table-entries-come-from.md), [Q-0004](/questions/open/0004-where-the-tile-set-comes-from.md) and [Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md), which block the first demo. [Q-0006](/questions/open/0006-expected-table-size.md) can wait.
  - Yew and Wren bring the grid geometry record.
  - Ian approves [DR-0012](/decisions/0012-code-layout-and-core-crate.md) (code layout), and then 1.0 starts.
- **Next:** 1.1 and 1.2. Together they are the "exploring the table" demo Ian asked for.

## How it works

- Phases follow [DR-0008](/decisions/0008-adjacency-tool-first.md). Phase 1 is the table and a tool to view and filter it. Phase 2 adds sample mazes to the same tool. The game comes after, as its own app.
- Every slice works end to end, from Rust to the screen.
- Every slice ends in a demo, written as a task Ian does himself without help (Damson's suggestion). Each demo comes with a commit on `main`, a line in `/log.md` and the steps to run it.
- A slice starts once the decisions it waits on are accepted. This roadmap names those decisions but doesn't make them.
- Marram can reorder or resize slices within a phase, because sequencing is a product matter ([DR-0002](/decisions/0002-approval-and-escalation.md)). Changing what a phase is for goes to Ian.
- Yew writes the Phase 1 plan in `/plans/`, with these slices as its steps. The plan turns "responsive", "readable" and "see and understand" into measurable tests (Yew).

## Ian's input (2026-10-31)

Ian (`human:ian`), verbatim:

> the demo I'm most keen to see is the tool exploring the probability table for the wave function collapse. I do suggest use of unicode characters to represent possible tile topology (tunnel, turn, t intersection, open room, corner room, etc). I also suggest the use of color (room/hallway).

How the team took it:

- **Marram:** the first headline demo is the tool exploring the table, so everything that gates 1.1 and 1.2 is the critical path. 1.1 shows the table, and 1.2 is where Ian explores it, so the demo he asked for is 1.1 plus 1.2. Damson brings the design for both together, so 1.2 doesn't have to wait.
- **Damson:** glyphs plus colour is the starting point for drawing tiles. Box-drawing glyphs show orientation directly (`└ ┌ ┐ ┘`).
  - A turn and a corner room would both be `┌`, and colour mustn't be the only difference: some people are colour-blind, and 1.4 will want colour for problems. So hallways get single lines (`┌`) and rooms double lines (`╔`), with colour on top.
  - Hovering a tile shows its name.
- **Wren:** room vs hallway is a property of each tile. Ian's vocabulary is the basis of Wren's proposed starter set (in 1.1).

## Phase 1: the table and the viewer

**Done when** Ian can do the tasks from [Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md) on full tile set v1, without help: "see and understand the table" ([DR-0008](/decisions/0008-adjacency-tool-first.md)). Yew's plan makes these tasks measurable.

### 1.0 Skeleton (preparation)

- **Goal:** the tool app runs, and its Lit frontend gets data from Rust.
- **Demo:** a quick check-in rather than a headline demo. One command opens the tool window, which shows a value that came from Rust.
- **Done when:**
  - it runs from a clean checkout, following the steps in the README;
  - the code is laid out as [DR-0012](/decisions/0012-code-layout-and-core-crate.md) says;
  - one Rust command reaches a Lit component;
  - Rust and TypeScript tests run.
- **Waits on:** [DR-0012](/decisions/0012-code-layout-and-core-crate.md), which is proposed and needs Rowan's review and then Ian's approval, and Yew's Phase 1 plan.
- **Status:** not started. It can be built while the 1.1 gates are being settled. How code is shared with the game in full waits until the game starts (Yew).

### 1.1 First look at the table

- **Goal:** a small starter tile set, its adjacency table built in Rust, and all of it visible in the tool.
- **Demo:** Ian opens the tool and answers a question like "can a turn sit east of a tunnel?" from the table, without help.
- **What's in it:**
  - The table has three axes: tile (with orientation), side, and neighbour (Yew). How they are laid out on screen is Damson's drawing design.
  - The frontend asks Rust for a view of the table (filters plus a range), and Rust returns just that. The starter table fits in one answer (Yew).
  - Proposed starter set (Wren, pending [Q-0004](/questions/open/0004-where-the-tile-set-comes-from.md)): six tiles, each side a wall, a passage or open to a room. That gives 19 rows:
    - tunnel (2 orientations), turn (4) and T-intersection (4), which are hallways;
    - open room (1), corner room (4) and doorway (4), which are rooms.

    The doorway is what lets rooms and hallways meet. Without it, 1.1 would show two unconnected tables.
- **Done when:**
  - the table is built in Rust from the tile set definition ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)), not in the frontend;
  - every cell shows what the table holds;
  - Wren has checked a few entries by hand and they match.
- **Waits on**, in roughly this order (see the table at the end):
  1. grid geometry: grid shape, how a neighbour's direction is represented, and orientation;
  2. where the table's entries come from ([Q-0003](/questions/open/0003-where-table-entries-come-from.md));
  3. weights vs allowed/forbidden;
  4. where the tile set comes from ([Q-0004](/questions/open/0004-where-the-tile-set-comes-from.md)), then the starter tile set;
  5. what Ian wants to find out ([Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md));
  6. how tiles and the table are drawn (Damson);
  7. how Rust and TypeScript types stay in step;
  8. the tile set file format, which comes last.
- **Status:** not started. This is the critical path.

### 1.2 Explore one tile

- **Goal:** answer "what can go next to this?" at a glance, and move around the table.
- **Demo:** Ian picks a tile and orientation. It appears in the middle, with whatever can sit on each side shown around it. He answers "what fits east of this turn?" without help, then clicks through three neighbours in a row.
- **Done when:** it works for every tile and orientation in the starter set, and agrees with the 1.1 table.
- **Waits on:** Damson's design for exploring, which is brought together with the 1.1 drawing design, and [Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md).
- **Status:** not started.

### 1.3 Filter the table

- **Goal:** narrow the table to the part you care about.
- **Demo:** Ian shows only what can sit next to rooms, hides empty rows and columns, then clears the filter. He can filter by tile, orientation, side, and room or hallway.
- **Done when:**
  - filtering happens in Rust, through the view request from 1.1, not in TypeScript ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), Yew);
  - filters can be combined;
  - it's always visible what has been filtered out;
  - one action clears all filters.
- **Waits on:** Damson's design for filtering, and [Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md).
- **Status:** not started. Filtering pays off most on the full set (Damson), so this slice can swap with 1.5 if the full set is ready first.

### 1.4 Table checks

- **Goal:** the tool finds problems, so Ian doesn't have to hunt for them.
- **Demo:** Ian opens a deliberately broken tile set, sees the list of problems, and clicks one to jump to it in the table.
- **Done when:**
  - every agreed check catches its problem in a broken set;
  - a clean set shows no problems.
- **Which checks:** Wren and Yew agree on them within the slice. One example is a tile with nothing allowed on one side. One-way pairs are another, but only if the geometry record allows them: if each pair is stored once, they can't happen (Yew).
- **Waits on:**
  - the geometry record;
  - the weights record;
  - Damson's design for how problems are shown.
- **Status:** not started.

### 1.5 Full tile set v1

- **Goal:** the real tile set at full size, still quick to explore and still readable.
- **Demo:** Ian does the [Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md) tasks on the full table, without help and without lag.
- **Done when:**
  - Wren's full set v1 loads (it adds crossroads, dead end, plain room edge, corner doorway and some variants);
  - an edited tile set can be loaded without a code change (Wren);
  - it passes the speed and readability tests in Yew's plan (Damson: a whole-table view may stop working at this size);
  - every problem the checks find is either fixed or logged.
- **Waits on:**
  - full set v1 (Wren);
  - how big the table will get ([Q-0006](/questions/open/0006-expected-table-size.md));
  - the paging and transport record (Yew, which needs Q-0006);
  - Damson's design for the full size.
- **Status:** not started. This slice can move up to any point after 1.1. Expect Phase 2 to change the set (Wren).

### Not in Phase 1

- Editing the table in the tool. DR-0008 says view and filter. If [Q-0003](/questions/open/0003-where-table-entries-come-from.md) says the table is written by hand, that changes, and editing comes back to Ian as scope.
- Generating mazes (that's Phase 2), and anything in the game.

## Phase 2: sample mazes (outline)

**Done when** Wren and Ian can judge the maps the table makes ([DR-0008](/decisions/0008-adjacency-tool-first.md)).

- **2.1 First sample maze:** generate one small maze from a seed and show it with the same tile drawing. Waits on contradiction handling, determinism (seeds), and Damson's design for the generation view.
- **2.2 Watch it collapse:** step through generation, seeing each cell's possibilities and where contradictions happen.
- **2.3 Judge many samples:** generate a batch from seeds. Samples are big enough to judge pacing over a walk, not just one screen (Wren). Measures are taken from the player's side, not just as tile counts (Wren):
  - reachable cells;
  - dead ends per area;
  - corridor run lengths;
  - room sizes and spacing.

  The loop is: edit the set, regenerate with the same seed, compare (Wren).
- **2.4 Beyond the edge:** grow a sample by generating areas next to it, to test where new areas meet old ones. Waits on persistence and on Yew's research into infinite WFC ([DR-0007](/decisions/0007-wave-function-collapse.md)).

## The game (outline)

The game is its own app (proposed in [DR-0012](/decisions/0012-code-layout-and-core-crate.md) as `apps/game/`), and comes after Phase 2 ([DR-0008](/decisions/0008-adjacency-tool-first.md)).

- **Waits on:**
  - the full decision on how code is shared with the tool (Yew);
  - persistence, meaning whether the world is kept when the player walks back;
  - game-design records from Wren on what the player does;
  - possibly technology choices, which would be Ian's.
- **Smallest first slice:** a player walks through a maze generated around them. After that comes keeping the world when the player walks back, then a debug mode with the tool's views.

## Inputs for records (not settled here)

These views belong in the records that will settle them. The roadmap takes no position on them.

- **Grid shape:**
  - Wren favours a square grid for a hedge-maze feel: straight paths, clear turns, and four directions to hold in your head. Wren would change their mind only if Yew finds that hex handles infinite-maze seams much better.
  - Damson's glyphs assume a square grid.
- **Orientation:** whether symmetric tiles keep one row per orientation or merge their repeats (`─` at 0° and 180°). Damson asks that the geometry record says which.
- **Weights:** Wren will argue for weights, because frequencies set a maze's pacing.
- **Drawing:** Wren suggests showing side types, because they explain why an entry is allowed.

## What gates each slice

"Ian?" means Ian approves the gate. DR-0007's follow-ups are likely major ([DR-0002](/decisions/0002-approval-and-escalation.md)).

| Gate | Source | Brought by | Ian? | First blocks |
|---|---|---|---|---|
| Code layout and core crate | [DR-0012](/decisions/0012-code-layout-and-core-crate.md), proposed | Yew; Rowan reviews | yes | 1.0 |
| Grid geometry: shape, neighbour direction, orientation | DR-0007 | Yew, with Wren | yes | 1.1 (first) |
| Where the table's entries come from | [Q-0003](/questions/open/0003-where-table-entries-come-from.md) | Ian | yes | 1.1 |
| Weights vs allowed/forbidden | DR-0007 | Yew, with Wren | yes | 1.1 |
| Where the tile set comes from | [Q-0004](/questions/open/0004-where-the-tile-set-comes-from.md) | Ian | yes | 1.1 |
| Starter tile set | game design | Wren | no, unless Q-0004 says so | 1.1 |
| What Ian wants to find out | [Q-0005](/questions/open/0005-what-to-find-out-from-the-table.md) | Ian | yes | 1.1, 1.2 |
| How tiles and the table are drawn (glyphs, sides, cells, sticky headers) | design | Damson; Wren reviews | yes, with Marram (it was Ian's idea) | 1.1 |
| How Rust and TypeScript types stay in step | architecture | Yew | if it uses a library | 1.1 |
| Tile set file format | architecture | Yew | Yew to say | 1.1 (last) |
| Design for exploring | design | Damson | no | 1.2 (brought with 1.1's design) |
| Design for filtering | design | Damson | no | 1.3 |
| Which checks, and how problems are shown | Wren and Yew; Damson | in the slice | no | 1.4 |
| Full tile set v1 | game design | Wren | no | 1.5 |
| How big the table will get | [Q-0006](/questions/open/0006-expected-table-size.md) | Ian | yes | 1.5 |
| Paging and transport | architecture | Yew | Yew to say | 1.5 |
| Contradiction handling | DR-0007 | Yew | yes | 2.1 |
| Determinism and persistence | DR-0007 | Yew, with Wren | yes | 2.1 (seeds); 2.4 and the game (persistence) |
| Research into infinite WFC | DR-0007 | Yew | no | 2.4 |
| How code is shared with the game, in full | DR-0008 | Yew | likely | the game |
