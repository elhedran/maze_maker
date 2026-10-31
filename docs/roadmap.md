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
  at: 2026-10-31T15:35:00+01:00
---

# Roadmap

Draft by Marram (product manager), waiting on Ian's approval. It reflects input from Yew, Wren and Damson, and Ian's answers to Q-0003, Q-0004 and Q-0005, and to most of Q-0007, all given on 2026-10-31.

## Now and next

- **Now:**
  - Ian settles the weight model, the last open part of [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md). It blocks the first demo.
  - Ian answers [Q-0010](/questions/open/0010-where-editing-goes.md) (where editing probabilities goes). It doesn't block anything yet.
  - [Q-0006](/questions/open/0006-expected-table-size.md) (table size) can wait, but a number would settle when paging is needed.
  - Yew and Wren bring the grid geometry record. Yew brings the weight model. The tile-set session brings DR-0015, the starter tiles. Damson brings the design for drawing and exploring.
  - Ian approves [DR-0012](/decisions/0012-code-layout-and-core-crate.md) (code layout), and then 1.0 starts.
- **Next:** 1.1 and 1.2. Together they are the "exploring the table" demo Ian asked for.

## How it works

- Phases follow [DR-0008](/decisions/0008-adjacency-tool-first.md). Phase 1 is the table and a tool to view and filter it. Phase 2 adds sample mazes to the same tool. The game comes after, as its own app.
- Every slice works end to end, from Rust to the screen.
- Every slice ends in a demo, written as tasks Ian does himself without help (Damson's suggestion). Each demo comes with a commit on `main`, a line in `/log.md` and the steps to run it.
- A slice starts once the decisions it waits on are accepted. This roadmap names those decisions but doesn't make them.
- Marram can reorder or resize slices within a phase, because sequencing is a product matter ([DR-0002](/decisions/0002-approval-and-escalation.md)). Changing what a phase is for goes to Ian.
- Yew writes the Phase 1 plan in `/plans/`, with these slices as its steps. The plan turns "responsive", "readable" and "see and understand" into measurable tests (Yew).
- Slice numbers are stable, because other records refer to them. A dropped slice keeps its number.
- Demo tasks say "east", which assumes a square grid. The geometry record hasn't settled that yet.

## What Ian has asked for (2026-10-31)

Ian (`human:ian`), verbatim, on the first demo:

> the demo I'm most keen to see is the tool exploring the probability table for the wave function collapse. I do suggest use of unicode characters to represent possible tile topology (tunnel, turn, t intersection, open room, corner room, etc). I also suggest the use of color (room/hallway).

Ian, verbatim, on what he wants from the table ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)):

> For the Q-0005 - what I want from the table - is I want to explore what is written down for probabilities, filter by tile type (room, corridor, corridor corner) so I can understand what leads into the maze generation. from the options, 1, 2, 3 are important, 4 is not, 5 is not, 6 is important. Basically I expect to "see" problems in the maze, and I'll want to look at the probabilities to understand a bit more of why they are there. A problem is hard to define, it will be bits of maze I don't think works well and would be improved by changing the probabilities.

Ian, verbatim, answering [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) (extracts that bear on the roadmap):

> I want to see the variation in the table of probabilities. I plan on having in future different "zones" with different probabilities.

> room walls are not rare - but go with what you think makes good for a nice maze here, the idea of the tool is to fix this by review, not perfect ahead of time.

> Lets allow for rooms we can't get into for now. We will revisit if and how we fix this later.

What Marram takes from this:

- **The tool's purpose:** "fix this by review, not perfect ahead of time". The tile set starts as good enough. Ian reviews it in the tool and fixes what he sees.
- **First demo:** exploring the table is the headline, so it's 1.1 plus 1.2. Damson brings the design for both together, so 1.2 doesn't wait.
- **What matters to Ian in the table:**
  - what can go next to a tile;
  - why a pair is allowed or not;
  - how likely each neighbour is;
  - how rooms and corridors connect.
- **What doesn't:** which tiles are hard to place, and which are rare or common. So 1.4 (table checks) is dropped.
- **Filters:** he filters by tile type (room, corridor, corridor corner). We use his word, "corridor", not "hallway".
- **His working loop:** see a problem in a maze, look at the probabilities behind it, change them. So Phase 2 starts with a maze (2.1), then the link from a spot in the maze to the table (2.2), then a loop to tune and compare (2.3).
- **Changing probabilities:** until [Q-0010](/questions/open/0010-where-editing-goes.md) is answered, you change them by editing the tile set data, and the tool reloads it.

How the team took it:

- **Damson, on drawing:**
  - Glyphs plus colour is the starting point. Box-drawing glyphs show orientation directly (`└ ┌ ┐ ┘`).
  - Corridors get single lines (`┌`) and rooms double lines (`╔`), with colour on top. Colour is never the only difference, because some people are colour-blind.
  - Hovering a tile shows its name.
- **Damson, on the table:**
  - A probability shows as a shade plus the number, and 0 is left blank.
  - Selecting an entry shows which two side kinds face each other and where the number comes from, or which sides clash for a 0.
  - Rows and columns are grouped by kind, so the connections between rooms and corridors show up as blocks.
  - These are inputs to Damson's design record, which settles them.
- **Wren:** room or corridor is a property of each tile.

## Phase 1: the table and the viewer

**Done when** Ian can do the demo tasks from 1.1, 1.2 and 1.3 on full tile set v1, without help: "see and understand the table" ([DR-0008](/decisions/0008-adjacency-tool-first.md)). Yew's plan makes these tasks measurable.

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

- **Goal:** the starter tile set and its probability table, built in Rust, with all of it visible in the tool and every number explainable.
- **Demo tasks:**
  - Ian gives the probability of a corridor corner east of a straight corridor, and says why.
  - Ian finds a 0, such as a corridor running into a room wall rather than a door, and says which sides clash.
- **What's in it:**
  - The table has three axes: tile (with orientation), side, and neighbour (Yew). How they are laid out is Damson's design.
  - The frontend asks Rust for a view of the table (filters plus a range), and Rust returns just that. The starter table fits in one answer, and the same request already allows for paging later (Yew).
  - Every entry can explain where its number comes from, because Ian's "why" is one of the things that matter to him. That needs the weight model to stay explainable.
  - Ian wants to see the variation in the table's probabilities. Whatever variation the weight model puts in the numbers is visible, and the explanation shows it.
  - The starter tiles come from DR-0015, which the tile-set session is writing. Wren designs them ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md)). DR-0015 also holds one glossary of tile kinds, shapes and side kinds, in Ian's words (Damson).
- **Done when:**
  - the table is built in Rust from the tile set data ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), [DR-0013](/decisions/0013-table-entries-from-tile-sides.md)), not in the frontend;
  - the tile set, with whatever sets its probabilities, is data, so changing a probability needs no code change. This was Wren's criterion for 1.5, moved up because of Ian's loop;
  - every cell shows its probability, variation included, and any entry can be explained;
  - Wren has checked a few entries by hand and they match.
- **Waits on**, in roughly this order:
  1. grid geometry: grid shape, how a neighbour's direction is represented, and orientation;
  2. the weight model, the last open part of [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md), and then Yew's record for it;
  3. the starter tiles and glossary (DR-0015);
  4. how tiles and the table are drawn (Damson);
  5. how Rust and TypeScript types stay in step;
  6. the tile set file format, which comes last.
- **Status:** not started. This is the critical path.

### 1.2 Explore one tile

- **Goal:** see what can go next to a tile, how likely each neighbour is and why, and move around the table.
- **Demo tasks:**
  - Ian names a corridor corner's most likely east neighbour.
  - Ian clicks from a corridor into a room and back out, through doorways.
- **What's in it:**
  - The chosen tile is in the middle, with its neighbours on each side sorted by likelihood. The facing side kinds are drawn on the shared edge (Damson).
  - The view can be opened from anywhere, so that 2.2 can open it from a spot in a maze (Damson).
- **Done when:** it works for every tile and orientation in the starter set, and agrees with the 1.1 table.
- **Waits on:** Damson's design for exploring, which is brought together with the 1.1 drawing design.
- **Status:** not started.

### 1.3 Filter the table

- **Goal:** narrow the table to the tile types Ian cares about.
- **Demo task:** Ian shows only pairs of a room and a corridor, and finds the least likely one.
- **What's in it:**
  - Ian's "room, corridor, corridor corner" means filtering by two properties of each tile: kind (room, corridor) and shape (corner, straight, T, ...) (Damson).
  - Filters apply to the tile and to its neighbour separately, for example "corridor corners next to rooms" (Damson).
- **Done when:**
  - filtering happens in Rust, through the view request from 1.1, not in TypeScript ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), Yew);
  - filters can be combined;
  - it's always visible what has been filtered out;
  - one action clears all filters.
- **Waits on:** Damson's design for filtering, and kind and shape on every tile (DR-0015).
- **Status:** not started. This slice can swap with 1.5 if the full set is ready first.

### 1.4 Table checks (dropped)

Marram dropped this slice on 2026-10-31, as a scope call ([DR-0002](/decisions/0002-approval-and-escalation.md)).

- Ian said hard-to-place tiles and rare or common tiles aren't important to him ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)), and he expects to see problems in mazes.
- Damson suggested shrinking 1.4 to basic checks. Marram went further: one basic check stays, as a test in 1.5.
- The explanations that [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) mentions for 1.4 are now in 1.1 and 1.2.

### 1.5 Full tile set v1

- **Goal:** the real tile set at full size, still quick to explore and still readable.
- **Demo:** Ian does the tasks from 1.1, 1.2 and 1.3 on the full table, without help and without lag.
- **Done when:**
  - Wren's full set v1 loads. It includes crossroads and dead ends at very low probability ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md)); DR-0015 and [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) settle the rest;
  - it passes the speed and readability tests in Yew's plan (Damson: a view of the whole table may stop working at this size);
  - a basic check, agreed by Wren and Yew, runs as a test. For example: every side of every tile can meet something.
- **Waits on:**
  - full set v1 (Wren, approved by Wren and Marram under [DR-0016](/decisions/0016-wren-designs-the-tile-set.md));
  - how big the table will get ([Q-0006](/questions/open/0006-expected-table-size.md)). Ian has said "I'm expecting a big table". Yew still wants a rough number of tiles;
  - the paging and transport record (Yew);
  - Damson's design for the full size.
- **Status:** not started.
  - This slice can move up to any point after 1.1, and 1.3 can swap with it.
  - If Ian's answer to Q-0006 is in the thousands of tiles, Yew would move 1.5 up to straight after 1.1. That's a condition, not a move yet.
  - Expect Phase 2 to change the set (Wren).

### Not in Phase 1

- **Editing probabilities in the tool:** waiting on Ian ([Q-0010](/questions/open/0010-where-editing-goes.md)). Until then, you change probabilities by editing the tile set data and reloading it.
- Generating mazes (that's Phase 2), and anything in the game.

## Phase 2: sample mazes (outline)

**Done when** Wren and Ian can judge the maps the table makes ([DR-0008](/decisions/0008-adjacency-tool-first.md)), and Ian can trace a problem he sees in a maze back to the probabilities behind it ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)).

- **2.1 First sample maze:** generate one small maze from a seed and show it with the same tile drawing.
  - Demo: the same seed gives the same maze, and a new seed gives a different one.
  - Waits on:
    - contradiction handling (Ian's answers on fallbacks in Q-0007 bear on it);
    - determinism (seeds);
    - how the weights of several neighbours combine (the weight model, deferred to 2.1 by [DR-0013](/decisions/0013-table-entries-from-tile-sides.md));
    - Damson's design for the generation view.
- **2.2 Why is this here?:** click a spot in a maze, and the 1.2 view opens on it, showing how its neighbours combined (Damson).
  - Demo: Ian clicks a bit of maze he doesn't like and sees why it's there.
  - Waits on: Damson's design for the link.
- **2.3 Tune and compare:** Ian's loop. Change probabilities, reload, regenerate with the same seed and compare (Wren's loop, moved up for Ian).
  - Demo: Ian changes a probability, regenerates the same seed, and points at what changed.
  - How he changes them depends on [Q-0010](/questions/open/0010-where-editing-goes.md).
- **2.4 Watch it collapse:** step through generation, seeing each cell's possibilities and where contradictions happen.
- **2.5 Judge many samples:** generate a batch from seeds. Samples are big enough to judge pacing over a walk, not just one screen (Wren). Measures are taken from the player's side, not just as tile counts (Wren):
  - reachable cells;
  - dead ends per area;
  - corridor run lengths;
  - room sizes and spacing.
- **2.6 Beyond the edge:** grow a sample by generating areas next to it, to test where new areas meet old ones. Waits on persistence and on Yew's research into infinite WFC ([DR-0007](/decisions/0007-wave-function-collapse.md)).

## The game (outline)

The game is its own app (proposed in [DR-0012](/decisions/0012-code-layout-and-core-crate.md) as `apps/game/`), and comes after Phase 2 ([DR-0008](/decisions/0008-adjacency-tool-first.md)).

- **Waits on:**
  - the full decision on how code is shared with the tool (Yew);
  - persistence, meaning whether the world is kept when the player walks back;
  - game-design records from Wren on what the player does;
  - possibly technology choices, which would be Ian's.
- **Smallest first slice:** a player walks through a maze generated around them. After that comes keeping the world when the player walks back, then a debug mode with the tool's views.

## Later (not scheduled, nothing decided)

- **Zones:** Ian plans "in future different "zones" with different probabilities". That will need its own records when it comes up, after Phase 2 or in the game.
- **Rooms with no way in:** these are allowed for now. Ian: "We will revisit if and how we fix this later." Phase 2's measures of reachable cells (2.5) will show how often it happens.

## Inputs for records (not settled here)

These views belong in the records and questions that will settle them. The roadmap takes no position on them.

- **Grid shape:**
  - Wren favours a square grid for a hedge-maze feel: straight paths, clear turns, and four directions to hold in your head. Wren would change their mind only if Yew finds that hex handles infinite-maze seams much better.
  - Damson's glyphs assume a square grid.
- **Orientation:** whether symmetric tiles keep one row per orientation or merge their repeats (`─` at 0° and 180°). Damson asks that the geometry record says which.
- **Weights:**
  - [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) settles that the table holds probabilities, with 0 where sides don't pair up.
  - Ian, verbatim, on what else sets them: "probabilities are mainly "if it can work" and "should we see it often in that place" - and some randomness for to add flavour. I'm expecting a big table."
  - Ian, verbatim: "I want to see the variation in the table of probabilities."
  - Wren and Yew both lean towards a weight for each pair of tiles, held in the table, but neither is sure.
  - Wren notes that "in that place" may mean next to this neighbour, or a location in the world.
  - Damson needs every number to stay explainable.
  - The weight model, including whether pair weights are set by hand or calculated, is the one part of [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) still with Ian. His other answers are being recorded in DR-0013, DR-0016 and Q-0007.
- **Fallback tiles and room walls:** [DR-0016](/decisions/0016-wren-designs-the-tile-set.md) settles crossroads and dead ends as very low probability. Ian, verbatim, on room walls: "room walls are not rare - but go with what you think makes good for a nice maze here". His other answers on fallbacks are being recorded in DR-0016 and Q-0007.
- **Drawing:** Wren suggests showing side kinds, because they explain why an entry is allowed. Damson's design for 1.1 and 1.2 does.

## What gates each slice

"Ian?" means Ian approves the gate. DR-0007's follow-ups are likely major ([DR-0002](/decisions/0002-approval-and-escalation.md)).

Already settled:

- where the table's entries come from ([DR-0013](/decisions/0013-table-entries-from-tile-sides.md));
- who designs the tile set ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md));
- what Ian wants from the table ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md), now the demo tasks).

| Gate | Source | Brought by | Ian? | First blocks |
|---|---|---|---|---|
| Code layout and core crate | [DR-0012](/decisions/0012-code-layout-and-core-crate.md), proposed | Yew; Rowan reviews | yes | 1.0 |
| Grid geometry: shape, neighbour direction, orientation | DR-0007 | Yew, with Wren | yes | 1.1 (first) |
| Ian's answer on the weight model (the rest of Q-0007 is answered) | [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) | Ian | yes | 1.1 |
| The weight model: where weights live, symmetry, explainable; how neighbours combine can wait for 2.1 | DR-0007, [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) | Yew, with Wren | yes | 1.1; 2.1 for how neighbours combine |
| Starter tiles and glossary (kinds, shapes, side kinds) | DR-0015, in progress | Wren | no: Wren and Marram, unless major | 1.1 |
| How tiles and the table are drawn | design | Damson; Wren reviews | yes, with Marram (it was Ian's idea) | 1.1 |
| How Rust and TypeScript types stay in step | architecture | Yew | if it uses a library | 1.1 |
| Tile set file format | architecture | Yew | Yew to say | 1.1 (last) |
| Design for exploring, openable from anywhere | design | Damson | no | 1.2 (brought with 1.1's design) |
| Design for filtering | design | Damson | no | 1.3 |
| Full tile set v1 | game design, [DR-0016](/decisions/0016-wren-designs-the-tile-set.md) | Wren | no: Wren and Marram, unless major | 1.5 |
| How big the table will get | [Q-0006](/questions/open/0006-expected-table-size.md) | Ian | yes | 1.5 (earlier if thousands of tiles) |
| Paging and transport | architecture | Yew | Yew to say | 1.5 |
| Where editing probabilities goes | [Q-0010](/questions/open/0010-where-editing-goes.md) | Ian | yes | 2.3 at the latest |
| Contradiction handling | DR-0007 | Yew | yes | 2.1 |
| Determinism and persistence | DR-0007 | Yew, with Wren | yes | 2.1 (seeds); 2.6 and the game (persistence) |
| Design for the generation view and the link from maze to table | design | Damson | no | 2.1, 2.2 |
| Research into infinite WFC | DR-0007 | Yew | no | 2.6 |
| How code is shared with the game, in full | DR-0008 | Yew | likely | the game |
