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
  at: 2026-10-31T19:10:00+01:00
---

# Roadmap

Draft by Marram (product manager). Ian approves its phases and deliverables; Marram reorders and resizes slices within them ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). It reflects input from Yew, Wren and Damson, and Ian's answers to Q-0003 to Q-0007, all given on 2026-10-31.

Revised 2026-10-31 for [DR-0012](/decisions/0012-code-layout-and-core-crate.md), [DR-0014](/decisions/0014-square-grid.md), [DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md), [DR-0017](/decisions/0017-tile-glyphs-and-colours.md) and [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md), the resolved [Q-0006](/questions/resolved/0006-expected-table-size.md), [Q-0009](/questions/resolved/0009-sign-off-grid-and-tile-set.md) and [Q-0010](/questions/resolved/0010-where-editing-goes.md), and the 1.0 scaffold. No phase or deliverable changed.

## Now and next

- **Now:**
  - 1.0 is done: all four "done when" points checked by Rowan on 2026-10-31.
  - The weight model, point 10 of [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md): Yew chose a calculated base plus a few exceptions, as a two-way door. It needs one engineer's approval, then Yew's record. This is the critical path to 1.1.
  - Damson brings the design for the table and for exploring one tile. How tiles look is settled ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md)).
  - Yew brings the Phase 1 plan, how Rust and TypeScript types stay in step, and then the tile set file format.
  - Wren and Yew work on [Q-0008](/questions/open/0008-global-maze-properties.md) (global maze properties). It doesn't block Phase 1.
  - Nothing is waiting on Ian except approving this roadmap.
- **Next:** 1.1 and 1.2. Together they are the "exploring the table" demo Ian asked for.

## How it works

- Phases follow [DR-0008](/decisions/0008-adjacency-tool-first.md). Phase 1 is the table and a tool to view and filter it. Phase 2 adds sample mazes to the same tool. The game comes after, as its own app.
- Every slice works end to end, from Rust to the screen.
- Every slice ends in a demo, written as tasks Ian does himself without help (Damson's suggestion). Each demo comes with a commit on `main`, a line in `/log.md` and the steps to run it.
- A slice starts once the decisions it waits on are accepted. This roadmap names those decisions but doesn't make them.
- Most gates are the team's: if a decision can be reversed without a lot of extra work, the owning roles decide it and don't wait on Ian ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). Only one-way doors go to Ian. "What gates each slice" says which gates are which.
- Marram can reorder or resize slices within a phase, because sequencing is a product matter ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). Changing what a phase is for goes to Ian.
- Yew writes the Phase 1 plan in `/plans/`, with these slices as its steps. The plan turns "responsive", "readable" and "see and understand" into measurable tests (Yew).
- Slice numbers are stable, because other records refer to them. A dropped slice keeps its number.
- Directions in demo tasks ("east") are the square grid's four: north, east, south and west ([DR-0014](/decisions/0014-square-grid.md)).

## What Ian has asked for (2026-10-31)

Ian (`human:ian`), verbatim, on the first demo:

> the demo I'm most keen to see is the tool exploring the probability table for the wave function collapse. I do suggest use of unicode characters to represent possible tile topology (tunnel, turn, t intersection, open room, corner room, etc). I also suggest the use of color (room/hallway).

Ian, verbatim, on what he wants from the table ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)):

> For the Q-0005 - what I want from the table - is I want to explore what is written down for probabilities, filter by tile type (room, corridor, corridor corner) so I can understand what leads into the maze generation. from the options, 1, 2, 3 are important, 4 is not, 5 is not, 6 is important. Basically I expect to "see" problems in the maze, and I'll want to look at the probabilities to understand a bit more of why they are there. A problem is hard to define, it will be bits of maze I don't think works well and would be improved by changing the probabilities.

Ian, verbatim, answering [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) (extracts that bear on the roadmap):

> I want to see the variation in the table of probabilities. I plan on having in future different "zones" with different probabilities.

> room walls are not rare - but go with what you think makes good for a nice maze here, the idea of the tool is to fix this by review, not perfect ahead of time.

> Lets allow for rooms we can't get into for now. We will revisit if and how we fix this later.

Ian, verbatim, on table size ([Q-0006](/questions/resolved/0006-expected-table-size.md)):

> For Q-0006, I expect the eventual table to be up in the order of millions of cells - although in the short term more in the order of 30 rows/columns.

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
- **Changing probabilities:** editing in the tool comes in 2.3 ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). Until then, you change them by editing the tile set data, and the tool reloads it.
- **Size:** about 30 rows for now, so Phase 1's table is small. Millions of cells is the long-term size, which the view request from 1.1 and Yew's paging record must allow for.

How the team took it:

- **Damson, on drawing** (now settled in [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)):
  - Glyphs plus colour is the starting point. Box-drawing glyphs show orientation directly (`└ ┌ ┐ ┘`).
  - Corridors get single lines (`┌`) and rooms double lines (`╔`), with colour on top. Colour is never the only difference, because some people are colour-blind.
  - Hovering a tile shows its name.
- **Damson, on the table:**
  - A probability shows as a shade plus the number, and 0 is left blank.
  - Selecting an entry shows which two side kinds face each other and where the number comes from, or which sides clash for a 0.
  - Rows and columns are grouped by kind, so the connections between rooms and corridors show up as blocks.
  - These are inputs to Damson's design for the table, which settles them.
- **Wren:** room or corridor is a property of each tile. [DR-0015](/decisions/0015-tile-set.md) gives every tile a kind (corridor, room or hedge).

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
- **Waits on:** nothing. [DR-0012](/decisions/0012-code-layout-and-core-crate.md) is accepted. Yew's Phase 1 plan now gates 1.1 instead.
- **Status:** done (2026-10-31). Rowan checked all four points, including a run from a fresh clone and the value from Rust in the real Tauri window. Frontend tests use Vitest ([DR-0020](/decisions/0020-vitest-for-frontend-tests.md)). How code is shared with the game in full waits until the game starts (Yew).

### 1.1 First look at the table

- **Goal:** the core tile set and its probability table, built in Rust, with all of it visible in the tool and every number explainable.
- **Demo tasks:**
  - Ian gives the probability of a corridor corner east of a straight corridor, and says why.
  - Ian finds a 0, such as a corridor running into a room wall rather than a door, and says which sides clash.
- **What's in it:**
  - The tiles are [DR-0015](/decisions/0015-tile-set.md)'s core set: 10 tiles in three kinds (corridor, room, hedge), giving 29 (tile, orientation) pairs on five kinds of side. Wren designed them ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md)). DR-0015 is also the glossary of kinds, tile types, side kinds and tags, in Ian's words.
  - Tiles are drawn with [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)'s glyphs and colours.
  - The table has three axes: tile (with orientation), side, and neighbour (Yew). On the square grid that is 29 × 29 entries for each of four sides ([DR-0014](/decisions/0014-square-grid.md)). How they are laid out is Damson's design.
  - The frontend asks Rust for a view of the table (filters plus a range), and Rust returns just that. The core table fits in one answer, and the same request already allows for paging later (Yew).
  - Every entry can explain where its number comes from, because Ian's "why" is one of the things that matter to him. The weight model keeps that possible: a calculated base plus a few exceptions, each shown with where it came from (Yew, Q-0007 point 10).
  - Ian wants to see the variation in the table's probabilities. Whatever variation the weight model puts in the numbers is visible, and the explanation shows it.
- **Done when:**
  - the table is built in Rust from the tile set data ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), [DR-0013](/decisions/0013-table-entries-from-tile-sides.md)), not in the frontend;
  - the tile set, with whatever sets its probabilities, is data, so changing a probability needs no code change. This was Wren's criterion for 1.5, moved up because of Ian's loop;
  - every cell shows its probability, variation included, and any entry can be explained;
  - Wren has checked a few entries by hand and they match.
- **Waits on**, in roughly this order. All are two-way doors, decided by the team:
  1. the weight model: Yew's choice on [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) point 10 needs one engineer's approval, then Yew's record;
  2. how the table is drawn and explained (Damson, approved with Marram);
  3. how Rust and TypeScript types stay in step (Yew, with an engineer);
  4. the tile set file format, which comes last (Yew, with an engineer);
  5. Yew's Phase 1 plan, with 1.1 as its first step.
- **Settled since the last revision:** grid shape, neighbour directions and orientation ([DR-0014](/decisions/0014-square-grid.md), [DR-0015](/decisions/0015-tile-set.md)); the tiles and glossary ([DR-0015](/decisions/0015-tile-set.md)); glyphs and colours ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md)).
- **Status:** not started. This is the critical path.

### 1.2 Explore one tile

- **Goal:** see what can go next to a tile, how likely each neighbour is and why, and move around the table.
- **Demo tasks:**
  - Ian names a corridor corner's most likely east neighbour.
  - Ian clicks from a corridor into a room and back out, through doorways.
- **What's in it:**
  - The chosen tile is in the middle, with its neighbours on each side sorted by likelihood. The facing side kinds are drawn on the shared edge (Damson).
  - The view can be opened from anywhere, so that 2.2 can open it from a spot in a maze (Damson).
- **Done when:** it works for every tile and orientation in the core set, and agrees with the 1.1 table.
- **Waits on:** Damson's design for exploring, which is brought together with the 1.1 table design (Damson and Marram).
- **Status:** not started.

### 1.3 Filter the table

- **Goal:** narrow the table to the tile types Ian cares about.
- **Demo task:** Ian shows only pairs of a room and a corridor, and finds the least likely one.
- **What's in it:**
  - Ian's "room, corridor, corridor corner" means filtering by properties of each tile: kind (corridor, room, hedge) and tile type (corner, straight, T, ...) (Damson). [DR-0015](/decisions/0015-tile-set.md) also gives tags (fallback, junction, room edge), which can be filters too.
  - Filters apply to the tile and to its neighbour separately, for example "corridor corners next to rooms" (Damson).
- **Done when:**
  - filtering happens in Rust, through the view request from 1.1, not in TypeScript ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), Yew);
  - filters can be combined;
  - it's always visible what has been filtered out;
  - one action clears all filters.
- **Waits on:** Damson's design for filtering (Damson and Marram). Kinds, tile types and tags on every tile are settled ([DR-0015](/decisions/0015-tile-set.md)).
- **Status:** not started. This slice can swap with 1.5 if the full set is ready first.

### 1.4 Table checks (dropped)

Marram dropped this slice on 2026-10-31, as a scope call, which is Marram's to make ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md); at the time, [DR-0002](/decisions/archive/0002-approval-and-escalation.md)).

- Ian said hard-to-place tiles and rare or common tiles aren't important to him ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)), and he expects to see problems in mazes.
- Damson suggested shrinking 1.4 to basic checks. Marram went further: one basic check stays, as a test in 1.5.
- The explanations that [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) mentions for 1.4 are now in 1.1 and 1.2.

### 1.5 Full tile set v1

- **Goal:** the real tile set at full size, still quick to explore and still readable.
- **Demo:** Ian does the tasks from 1.1, 1.2 and 1.3 on the full table, without help and without lag.
- **Done when:**
  - Wren's full set v1 loads. It is DR-0015's core set, which already has crossroads and dead ends at very low probability ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md)), plus whichever of DR-0015's later additions Wren puts in v1 (for example the inner corner or the corner doorway);
  - it passes the speed and readability tests in Yew's plan (Damson: a view of the whole table may stop working at this size);
  - a basic check, agreed by Wren and Yew, runs as a test. For example: every side of every tile can meet something.
- **Waits on**, all two-way doors decided by the team:
  - full set v1 (Wren, approved by Wren and Marram under [DR-0016](/decisions/0016-wren-designs-the-tile-set.md));
  - the paging and transport record (Yew, with an engineer). It plans for Ian's long-term "millions of cells" ([Q-0006](/questions/resolved/0006-expected-table-size.md)), even though v1 is small;
  - Damson's design for the full size (Damson and Marram).
- **Status:** not started.
  - This slice can move up to any point after 1.1, and 1.3 can swap with it.
  - The condition for moving it up early (a table in the thousands of tiles) isn't met: Ian expects about 30 rows in the short term ([Q-0006](/questions/resolved/0006-expected-table-size.md)). So 1.5 stays last.
  - If v1 adds little to the core set, 1.5 shrinks to the speed and readability tests and the basic check. Marram's call, a two-way door.
  - Expect Phase 2 to change the set (Wren).

### Not in Phase 1

- **Editing probabilities in the tool:** in Phase 2, at 2.3 ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). Until then, you change probabilities by editing the tile set data and reloading it.
- Generating mazes (that's Phase 2), and anything in the game.

## Phase 2: sample mazes (outline)

**Done when** Wren and Ian can judge the maps the table makes ([DR-0008](/decisions/0008-adjacency-tool-first.md)), and Ian can trace a problem he sees in a maze back to the probabilities behind it ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)).

- **2.1 First sample maze:** generate one small maze from a seed and show it with the same tile drawing.
  - Demo: the same seed gives the same maze, and a new seed gives a different one.
  - Waits on:
    - contradiction handling (Yew). Fallback tiles are already low-weight tiles in the set ([DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md));
    - determinism (seeds);
    - how the weights of several neighbours combine (the weight model, deferred to 2.1 by [DR-0013](/decisions/0013-table-entries-from-tile-sides.md));
    - Damson's design for the generation view;
    - which global properties to measure ([Q-0008](/questions/open/0008-global-maze-properties.md), Wren and Yew), chosen before the first samples.
- **2.2 Why is this here?:** click a spot in a maze, and the 1.2 view opens on it, showing how its neighbours combined (Damson).
  - Demo: Ian clicks a bit of maze he doesn't like and sees why it's there.
  - Waits on: Damson's design for the link.
- **2.3 Tune and compare:** Ian's loop. Change probabilities, reload, regenerate with the same seed and compare (Wren's loop, moved up for Ian).
  - Demo: Ian changes a probability, regenerates the same seed, and points at what changed.
  - Editing probabilities in the tool comes in here ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). If the weight model ever became weights set by hand on every pair, Marram and Yew would move editing earlier.
- **2.4 Watch it collapse:** step through generation, seeing each cell's possibilities and where contradictions happen.
- **2.5 Judge many samples:** generate a batch from seeds. Samples are big enough to judge pacing over a walk, not just one screen (Wren). Measures are taken from the player's side, not just as tile counts (Wren), and follow [Q-0008](/questions/open/0008-global-maze-properties.md):
  - reachable cells;
  - dead ends per area;
  - corridor run lengths;
  - room sizes and spacing.
- **2.6 Beyond the edge:** grow a sample by generating areas next to it, to test where new areas meet old ones. Waits on persistence and on Yew's research into infinite WFC ([DR-0007](/decisions/0007-wave-function-collapse.md)).

## The game (outline)

The game is its own app (`apps/game/`, [DR-0012](/decisions/0012-code-layout-and-core-crate.md)), and comes after Phase 2 ([DR-0008](/decisions/0008-adjacency-tool-first.md)).

- **Waits on:**
  - the full decision on how code is shared with the tool (Yew, with an engineer);
  - persistence, meaning whether the world is kept when the player walks back;
  - how the global maze properties are enforced ([Q-0008](/questions/open/0008-global-maze-properties.md));
  - game-design records from Wren on what the player does;
  - possibly technology choices. Any that change the tech stack are Ian's ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).
- **Smallest first slice:** a player walks through a maze generated around them. After that comes keeping the world when the player walks back, then a debug mode with the tool's views.

## Later (not scheduled, nothing decided)

- **Zones:** Ian plans "in future different "zones" with different probabilities". That will need its own records when it comes up, after Phase 2 or in the game. Yew's weight model allows for them as extra sets of weights over the same base.
- **Rooms with no way in:** these are allowed for now. Ian: "We will revisit if and how we fix this later." Phase 2's measures of reachable cells (2.5) will show how often it happens.

## Inputs for records (not settled here)

These views belong in the records and questions that will settle them. The roadmap takes no position on them.

- **Weights:**
  - [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) settles that the table holds probabilities, with 0 where sides don't pair up.
  - Ian, verbatim, on what else sets them: "probabilities are mainly "if it can work" and "should we see it often in that place" - and some randomness for to add flavour. I'm expecting a big table."
  - Ian, verbatim: "I want to see the variation in the table of probabilities."
  - Yew chose a calculated base plus a few exceptions for particular pairs, in the tile set data, as a two-way door ([Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) point 10). It needs one engineer's approval, then Yew's record.
  - Wren notes that "in that place" may mean next to this neighbour, or a location in the world. [DR-0015](/decisions/0015-tile-set.md) lists Wren's "in that place" intents for the weight model to test against.
  - Damson needs every number to stay explainable.
- **Fallback tiles and room walls:** settled in [DR-0015](/decisions/0015-tile-set.md) and [DR-0016](/decisions/0016-wren-designs-the-tile-set.md): crossroads and dead ends are very low probability, and hedge is common. Ian, verbatim, on room walls: "room walls are not rare - but go with what you think makes good for a nice maze here". The numbers come from the weight model and get tuned by review.
- **Drawing:** Wren suggests showing side kinds, because they explain why an entry is allowed. Damson's design for 1.1 and 1.2 does.

Settled since the last revision, and no longer inputs: the grid shape and orientation ([DR-0014](/decisions/0014-square-grid.md): square, four neighbours, 90-degree rotations only; [DR-0015](/decisions/0015-tile-set.md): a symmetric tile keeps only its distinct orientations, so 29 pairs).

## What gates each slice

"Ian?" says whether Ian decides the gate. Under [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md), only one-way doors go to him: decisions that would create a lot of work to reverse, or tie down his own time or money. Everything else is decided by the approvers named here. When it's unclear, Yew judges technical gates and Marram product, UX and game-design gates. Yew re-checks that a foundational decision is still a two-way door before major work builds on it (DR-0018, Yew's concern).

Already settled:

- the code layout ([DR-0012](/decisions/0012-code-layout-and-core-crate.md));
- where the table's entries come from ([DR-0013](/decisions/0013-table-entries-from-tile-sides.md));
- the grid: square, four neighbours, rotations only ([DR-0014](/decisions/0014-square-grid.md));
- the core tile set and glossary ([DR-0015](/decisions/0015-tile-set.md));
- who designs the tile set ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md));
- tile glyphs and colours ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md));
- what Ian wants from the table ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md), now the demo tasks);
- how big the table will get ([Q-0006](/questions/resolved/0006-expected-table-size.md): about 30 rows now, millions of cells eventually);
- where editing probabilities goes ([Q-0010](/questions/resolved/0010-where-editing-goes.md): 2.3).

| Gate | Source | Brought by; approved by | Ian? | First blocks |
|---|---|---|---|---|
| The weight model: where weights live, symmetry, explainable; how neighbours combine can wait for 2.1 | [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md) point 10, [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) | Yew, with Wren; Yew and one engineer | no | 1.1; 2.1 for how neighbours combine |
| How the table is drawn and explained | design | Damson, Wren reviews; Damson and Marram | no | 1.1 |
| How Rust and TypeScript types stay in step | architecture | Yew; Yew and one engineer | no | 1.1 |
| Tile set file format | architecture | Yew; Yew and one engineer | no | 1.1 (last) |
| Phase 1 plan | plan | Yew | no | 1.1 |
| Design for exploring, openable from anywhere | design | Damson; Damson and Marram | no | 1.2 (brought with 1.1's design) |
| Design for filtering | design | Damson; Damson and Marram | no | 1.3 |
| Full tile set v1 | game design, [DR-0016](/decisions/0016-wren-designs-the-tile-set.md) | Wren; Wren and Marram | no | 1.5 |
| Paging and transport | architecture | Yew; Yew and one engineer | no | 1.5 |
| Contradiction handling | DR-0007 | Yew; Yew and one engineer | no | 2.1 |
| Determinism (seeds) | DR-0007 | Yew, with Wren; Yew and one engineer | no | 2.1 |
| Which global properties to measure | [Q-0008](/questions/open/0008-global-maze-properties.md) | Wren and Yew; Wren and Marram | no | 2.1 (chosen before the first samples); 2.5 |
| Design for the generation view and the link from maze to table | design | Damson; Damson and Marram | no | 2.1, 2.2 |
| Research into infinite WFC | DR-0007 | Yew | no | 2.6 |
| Persistence | DR-0007 | Yew, with Wren | no; Yew re-checks before the game builds on it | 2.6; the game |
| How global properties are enforced | [Q-0008](/questions/open/0008-global-maze-properties.md) | Yew, with Wren | no, unless Yew judges it one-way | the game |
| How code is shared with the game, in full | DR-0008, [DR-0012](/decisions/0012-code-layout-and-core-crate.md) | Yew; Yew and one engineer | no | the game |
| Replacing WFC as the generation approach, only if the research finds it can't do endless mazes | [DR-0007](/decisions/0007-wave-function-collapse.md) | Yew | **yes**: it changes the tech stack and throws away the generator built in Phase 2 | 2.6; the game |
| Technology for the game beyond Tauri v2, Rust and Lit, if any is needed | [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md) | Yew | **yes**: it changes the tech stack, and the game would be built on it | the game |
| Anything that costs Ian money or ties down his time, such as a paid service or publishing | [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) | whoever needs it | **yes**: the cost falls on Ian | none planned |
