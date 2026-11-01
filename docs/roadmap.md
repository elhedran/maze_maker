---
type: Roadmap
title: Maze Maker roadmap
description: Phases, deliverables and demos for Maze Maker. Phase 1 is closed; 2.1 and 2.2 are on main. 2.3 Tune and compare starts on the tool track, and 2.6 grows a stored world from open endpoints (DR-0035, DR-0036) on the engine track, with step 1 built.
status: stable
owners: [product-manager/1]
approvers: [human:ian]
tags: [roadmap, phase-1, phase-2, adjacency-table, tooling, demos, generation]
generated:
  by: product-manager/1
  at: 2026-11-01T08:50:20Z
verified:
  - by: human:ian
    at: 2026-11-01T00:20:00+01:00
---

# Roadmap

By Marram (product manager), approved by Ian on 2026-10-31. Ian approves its phases and deliverables; Marram reorders and resizes slices within them ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). It reflects input from Yew, Wren and Damson, and Ian's answers to Q-0003 to Q-0007, all given on 2026-10-31.

Revised 2026-10-31 for [DR-0012](/decisions/0012-code-layout-and-core-crate.md), [DR-0014](/decisions/0014-square-grid.md), [DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md), [DR-0017](/decisions/0017-tile-glyphs-and-colours.md) and [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md), the resolved [Q-0006](/questions/resolved/0006-expected-table-size.md), [Q-0009](/questions/resolved/0009-sign-off-grid-and-tile-set.md) and [Q-0010](/questions/resolved/0010-where-editing-goes.md), and the 1.0 scaffold. No phase or deliverable changed.

## Revised 2026-11-01: generation is next

This revision waits on Ian's approval. Ian (`human:ian`), verbatim, 2026-11-01:

> I think the data side is good enough to start leveraging in maze generation. There is such a thing as polishing too much before moving on.

And on 2026-10-31: "make sure we bias towards executing planned work over re-planning or planning new work."

What Marram changed. Moving slices is a sequencing call, a two-way door and Marram's to make ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)): nothing is thrown away, and every slice can move back.

- **Phase 1 closes with 1.1.** Its "done when" shrinks to 1.1's demo, which Ian has seen ("looks good"). That changes what Phase 1 is for, so it is Ian's to approve with this revision.
- **1.2 Explore one tile folds into 2.2.** 2.2 opens the explore view from a spot in a maze, so the view gets built when it has a maze to be opened from. Its design ([DR-0026](/decisions/0026-explore-one-tile-design.md)) stands.
- **1.3 Filter the table moves to later.** Generation doesn't need it, and at 29 rows the table is readable without it. Its design ([DR-0027](/decisions/0027-filter-table-design.md)) stands, so it can be picked up as soon as Ian or tuning wants it.
- **1.5 Full tile set v1 is dropped as a slice.** Wren expects Phase 2 to change the set anyway, so tiles get added when sample mazes show the need. Its one generation-critical piece, the basic check that every side of every tile can meet something, moves into 2.1. Paging waits until the table grows ([Q-0006](/questions/resolved/0006-expected-table-size.md)).
- **No separate planning round for 2.1.** Its open items are settled at the start of the build, by the roles named in 2.1.

## Update 2026-11-01: 1.3 runs in parallel with 2.1

1.3 has since landed on `main` (see 1.3 below). This section is kept as the record of the call.

Ian (`human:ian`), verbatim, 2026-11-01:

> Happy to use tokens faster if we can doo more work concurrently

So Marram brought 1.3 back, a sequencing call and a two-way door ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). It runs now, alongside 2.1, in its own session and worktrees, with separate sessions of Rowan (Rust: DR-0027's `sides` filter, with tests) and Ash (the Lit filter bar, with Vitest tests). It takes no people or sessions from 2.1; the cost is tokens and a rebase against 2.1. It touches only the table view, its command and the filter code, to keep that rebase small. 2.1 stays first in line: if the two clash, 1.3 rebases. Its demo counts once it lands on `main` ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)).

## Steering 2026-11-01: improving the mazes

Ian (`human:ian`), verbatim, 2026-11-01, as steering for the product manager for future phases:

> Ok, on that not I have some future steering for the product manager for future phases around improving the mazes.  I'd like two changes - the first is we only generate based of "open" endpoints, endpoints connected to existing maze.  This basically means looking for where a path ends, a room door opens etc rather than just every space next to where the player has been, within the bounds of the player space radius.  The second change is to add some pressure to bias towards an opening maze.  that means that if a piece that goes down would only connect to already closed maze, that should get a reduced probability.

Then Ian (`human:ian`), verbatim, 2026-11-01, on [DR-0031](/decisions/archive/0031-endless-generation-approach.md):

> In regards to DR-0031 the world needs to be stored anyway - we want to know where the player has been, we will add loot, and hence we need to know what is looted, etc.  However given how this works, there should be quite an efficient in-memory data structure for this (sparse big table).

And Ian (`human:ian`), verbatim, 2026-11-01, on what he meant:

> and yes I mean growth follows the players path

**What the two changes are.** This is the facilitator's reading, given to Ian: a proposal, not Ian's words. The full reading is in [Q-0008](/questions/open/0008-global-maze-properties.md).

- **Change 1, grow from open endpoints:** the maze only grows beside an open side (a passage end, a door) of something already generated, following the player's path, and what's never reached stays solid.
- **Change 2, opening pressure:** a candidate whose open sides all face settled cells, so that it adds no new open endpoint, gets a multiplier below 1 in [DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 3's score.
- **Considered and not taken:** growing each chunk from its gates, so that the world needn't be stored and DR-0031 holds. Ian means growth that follows the player's path, and the world is stored anyway.

**What it changed, and where that stands now** (updated 2026-11-01, after the records landed):

- **The world is stored:** [DR-0035](/decisions/0035-the-world-is-stored.md), accepted with Ian's approval. It settles the game's persistence gate.
- **2.6's new shape:** [DR-0036](/decisions/0036-growth-from-open-endpoints.md), Yew's patch growth from open endpoints, superseding DR-0031. It is accepted (Rowan and Ash approved; Rowan's narrowing of rules 4 and 6 is folded in). 2.6 below is redrawn for it, and its step 1 is built to it.
- **2.2 was unchanged,** and has landed.

**Where Marram puts each change.** This is sequencing, a two-way door and Marram's call ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).

- **Change 2, opening pressure, goes in 2.3 Tune and compare.** It is one tunable factor in the solver's score, whatever grows the maze, so it doesn't wait on Yew's record. It works on today's single maze, and 2.3's demo already fits it: Ian changes the pressure, regenerates the same seed, and sees the reachable share and separate areas ([DR-0030](/decisions/0030-measures-for-2-1.md)) move. 2.2's "why" trace shows it as a factor. It goes after the same pockets as Wren's tuning idea 4, damping corner-into-corner ([2.1 review](/plans/2.1-maze-review.md)), so 2.3 tries the two side by side. **Revisited 2026-11-01, when 2.2 landed:** DR-0036 rule 9 defines the factor, and 2.6 step 1 builds it into the engine for world growth only. 2.3 brings it into the tool's single maze and its "why" trace, and tunes it (see 2.3).
- **Change 1, growing from open endpoints, is the core of 2.6's new shape**, so it is picked up there. **Revisited 2026-11-01:** 2.6 is redrawn for DR-0036 (see 2.6), and stays on the engine track beside 2.3.
- **Not 2.5:** it judges batches, so both changes should be in before it, and its measures should judge the maze they make.
- **The how is settled at pickup, with no planning round.** Yew settles the fit with DR-0028 and with the record that replaces DR-0031, and whether any part is a one-way door; if one is, it goes to Ian. Wren settles what the player feels, and the tuning.

## Now and next

Updated 2026-11-01 by Marram, after 2.2 landed and [DR-0036](/decisions/0036-growth-from-open-endpoints.md) gave 2.6 its new shape. Ian wants work to keep going today, so both tracks run now.

- **Done:** Phase 1 is closed. 1.1, 1.3, 2.1 and 2.2 (which includes 1.2) are on `main`. Ian, verbatim, on 2.2: "I looked at the 2.2 demo, its approved and can land on main". [DR-0035](/decisions/0035-the-world-is-stored.md) (the world is stored) and [DR-0036](/decisions/0036-growth-from-open-endpoints.md) (growth from open endpoints) are accepted. [Q-0008](/questions/open/0008-global-maze-properties.md)'s first part is settled by [DR-0030](/decisions/0030-measures-for-2-1.md); the rest stays open for 2.5 and the game.
- **Now, two tracks in parallel** (Marram's call; Ian, 2026-11-01: "Happy to use tokens faster if we can doo more work concurrently"):
  - **Tool track: 2.3 Tune and compare.** Its two gates first, side by side: Damson's design record for compare and edit (DR-0037) and Yew's record on editing `core.toml` (DR-0038). The engineers can build against stubs while they settle.
  - **Engine track: 2.6 step 1**, built as `a813a73` on `rowan/2.6-chunks` and being rebased onto `main` by Rowan. Then its no-window demo for Ian, and it lands with his OK. Step 2, the measures, starts as soon as the rebase is green.
- **Next:**
  - engine track: 2.6 step 3, the view of a growing world with a walk, once step 1 has landed. Damson writes its small design record straight after DR-0037;
  - then 2.5 Judge many samples, then 2.4 Watch it collapse (see "Later in Phase 2" for why that order);
  - then the game, starting with G.1.

## How it works

- Phases follow [DR-0008](/decisions/0008-adjacency-tool-first.md). Phase 1 is the table and a tool to view and filter it. Phase 2 adds sample mazes to the same tool. The game comes after, as its own app.
- Every slice works end to end, from Rust to the screen.
- Every slice ends in a demo, written as tasks Ian does himself without help (Damson's suggestion). Each demo comes with a commit on `main`, a line in `/log.md` and the steps to run it.
- A slice starts once the decisions it waits on are accepted. This roadmap names those decisions but doesn't make them.
- Most gates are the team's: if a decision can be reversed without a lot of extra work, the owning roles decide it and don't wait on Ian ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). Only one-way doors go to Ian. "What gates each slice" says which gates are which.
- Marram can reorder or resize slices within a phase, because sequencing is a product matter ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). Changing what a phase is for goes to Ian.
- Yew writes the Phase 1 plan in `/plans/`, with these slices as its steps. The plan turns "responsive", "readable" and "see and understand" into measurable tests (Yew).
- In Phase 2, open items are settled at the start of each slice's build, by the roles the slice names, rather than in separate planning rounds (Ian, 2026-10-31 and 2026-11-01). Yew can still ask for a plan where a slice is major work.
- Slice numbers are stable, because other records refer to them. A dropped slice keeps its number.
- Directions in demo tasks ("east") are the square grid's four: north, east, south and west ([DR-0014](/decisions/0014-square-grid.md)).

## What Ian has asked for (2026-10-31)

Ian (`human:ian`), verbatim, on the first demo:

> the demo I'm most keen to see is the tool exploring the probability table for the wave function collapse. I do suggest use of unicode characters to represent possible tile topology (tunnel, turn, t intersection, open room, corner room, etc). I also suggest the use of color (room/hallway).

Ian, verbatim, on what he wants from the table ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)):

> For the Q-0005 - what I want from the table - is I want to explore what is written down for probabilities, filter by tile type (room, corridor, corridor corner) so I can understand what leads into the maze generation. from the options, 1, 2, 3 are important, 4 is not, 5 is not, 6 is important. Basically I expect to "see" problems in the maze, and I'll want to look at the probabilities to understand a bit more of why they are there. A problem is hard to define, it will be bits of maze I don't think works well and would be improved by changing the probabilities.

Ian, verbatim, answering [Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md) (extracts that bear on the roadmap):

> I want to see the variation in the table of probabilities. I plan on having in future different "zones" with different probabilities.

> room walls are not rare - but go with what you think makes good for a nice maze here, the idea of the tool is to fix this by review, not perfect ahead of time.

> Lets allow for rooms we can't get into for now. We will revisit if and how we fix this later.

Ian, verbatim, on table size ([Q-0006](/questions/resolved/0006-expected-table-size.md)):

> For Q-0006, I expect the eventual table to be up in the order of millions of cells - although in the short term more in the order of 30 rows/columns.

What Marram takes from this:

- **The tool's purpose:** "fix this by review, not perfect ahead of time". The tile set starts as good enough. Ian reviews it in the tool and fixes what he sees.
- **First demo:** exploring the table is the headline, so it was 1.1 plus 1.2. Since 2026-11-01, 1.1 alone closed it, and 1.2 comes with 2.2.
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

**Done when** Ian can do the demo tasks from 1.1 on the core tile set, without help: "see and understand the table" ([DR-0008](/decisions/0008-adjacency-tool-first.md)). Yew's plan makes these tasks measurable.

Until 2026-11-01 this also needed 1.2 and 1.3 on full tile set v1. Ian judged the data side good enough to move on, so Phase 1 closed with 1.1 (see "Revised 2026-11-01" above).

**Status:** closed (2026-11-01).

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
- **Waited on**, all done, all two-way doors decided by the team:
  1. the weight model ([DR-0022](/decisions/0022-weight-model.md));
  2. how the table is drawn and explained (Damson, approved with Marram: [DR-0025](/decisions/0025-table-view-design.md));
  3. how Rust and TypeScript types stay in step (Yew, with an engineer: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md));
  4. the tile set file format (Yew, with an engineer: [DR-0024](/decisions/0024-tile-set-file-format.md));
  5. Yew's [Phase 1 plan](/plans/phase-1.md), with 1.1 as its first step.
- **Status:** done (2026-11-01), on `main` (ae9a6c3). Ian has seen it and said it "looks good".
  - Ian ran both demo tasks himself from `main` (4e51f6e), 2026-11-01. Ian (`human:ian`), verbatim: "demo approved - looks good.  I tried the east, tried the blank cell, also did a bit of an explore around how paths and rooms were represented".
  - the table is built in Rust from `data/tile-sets/core.toml`, with Wren's first numbers;
  - changing a number is an edit and a Reload, with no rebuild;
  - every cell shows its share of the row, and seeded noise is included and shown in each explanation;
  - Wren's six hand-checked entries match;
  - the facilitator ran both demo tasks in the real Tauri window.

### 1.2 Explore one tile (folded into 2.2)

Moved by Marram on 2026-11-01, a two-way door. It is built as part of 2.2, which opens this view from a spot in a maze. The design and demo tasks below still apply.

- **Goal:** see what can go next to a tile, how likely each neighbour is and why, and move around the table.
- **Demo tasks:**
  - Ian names a corridor corner's most likely east neighbour.
  - Ian clicks from a corridor into a room and back out, through doorways.
- **What's in it:**
  - The chosen tile is in the middle, with its neighbours on each side sorted by likelihood. The facing side kinds are drawn on the shared edge (Damson).
  - The view can be opened from anywhere, so that 2.2 can open it from a spot in a maze (Damson).
- **Done when:** it works for every tile and orientation in the core set, and agrees with the 1.1 table.
- **Waits on:** Damson's design for exploring, which is brought together with the 1.1 table design (Damson and Marram).
- **Status:** done (2026-11-01), built and landed as part of 2.2. Its design gate was met: [DR-0026](/decisions/0026-explore-one-tile-design.md) accepted (2026-10-31).

### 1.3 Filter the table (done)

Moved to later by Marram on 2026-11-01, then brought back the same day to run in parallel with 2.1, after Ian said he was happy to spend tokens faster for more concurrent work (see "Update 2026-11-01" above). Both moves are two-way doors. Nothing in Phase 2 waits on it.

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
- **Status:** done (2026-11-01), on `main` (e520d07). Ian (`human:ian`) approved landing it: "Yes, land 1.3". Design gate met: [DR-0027](/decisions/0027-filter-table-design.md) accepted 2026-11-01.
  - **Built:** everything in DR-0027, nothing slipped. Kind, Tile type, Tag and Side facets for the tile (rows) and the neighbour (columns); the count line with the filters in words; Clear all; the empty state. Rust filters through DR-0024's request, plus DR-0027's `sides` field; a side filter with other than one direction is a `bad_request` error (Rowan). The bar is `apps/tool/src/table/filter-bar.ts` (Ash).
  - **Verified:** 52 Rust tests and 36 Vitest tests pass, and the build passes. The facilitator drove the real Tauri window by script: corridor corners next to rooms shows 4 of 29 rows, 13 of 29 columns; the demo (Room rows, Corridor columns, East) shows 13 of 29 rows, 15 of 29 columns; side labels follow the switcher; the empty state and Clear all work. Damson checked it against DR-0027 and passed it, with no overrules.
  - **Ian, 2026-11-01, verbatim:** "new filters look good".
  - **Bug found in the live check, and fixed:** two filter changes made before the bar redrew lost the first one. Ash fixed it, with a test that fails without the fix; the facilitator rechecked it in the live window (8 of 8 runs correct). After the fix: 52 Rust and 37 Vitest tests pass, and the build passes.
  - **Follow-ups (Damson, not blockers):** make the shade legend look less like pressed buttons; when exactly one tile type is chosen, name it in the corner sentence ("Corridor corner tile, then…"). Both landed with 2.2.

### 1.4 Table checks (dropped)

Marram dropped this slice on 2026-10-31, as a scope call, which is Marram's to make ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md); at the time, [DR-0002](/decisions/archive/0002-approval-and-escalation.md)).

- Ian said hard-to-place tiles and rare or common tiles aren't important to him ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)), and he expects to see problems in mazes.
- Damson suggested shrinking 1.4 to basic checks. Marram went further: one basic check stays, as a test in 1.5, and since 2026-11-01 in 2.1.
- The explanations that [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) mentions for 1.4 are now in 1.1 and 1.2.

### 1.5 Full tile set v1 (dropped)

Dropped by Marram on 2026-11-01, a scope call and a two-way door. What happens to its parts:

- **The basic check** (every side of every tile can meet something) moves into 2.1, because a side that can meet nothing makes generation fail.
- **Full set v1:** Wren adds tiles when sample mazes show the need, under [DR-0016](/decisions/0016-wren-designs-the-tile-set.md). Wren expects Phase 2 to change the set anyway.
- **Paging and speed at full size:** wait until the table grows towards Ian's "millions of cells" ([Q-0006](/questions/resolved/0006-expected-table-size.md)). DR-0024's view request already allows for paging.

The original definition, kept for reference:

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
- **Status:** dropped (2026-11-01).

### Not in Phase 1

- **Editing probabilities in the tool:** in Phase 2, at 2.3 ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). Until then, you change probabilities by editing the tile set data and reloading it.
- Generating mazes (that's Phase 2), and anything in the game.
- Since 2026-11-01: exploring one tile (now in 2.2) and full tile set v1 (dropped). Filtering (1.3) ran in parallel with 2.1 and is done.

## Phase 2: sample mazes

**Done when** Wren and Ian can judge the maps the table makes ([DR-0008](/decisions/0008-adjacency-tool-first.md)), and Ian can trace a problem he sees in a maze back to the probabilities behind it ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)).

**Status:** in progress. 2.1 and 2.2 are done and on `main`. Two tracks run in parallel: 2.3 Tune and compare (tool track) starts with its gates, and 2.6 (engine track) is redrawn for [DR-0036](/decisions/0036-growth-from-open-endpoints.md), with step 1 built and being rebased onto `main`. Then 2.5, then 2.4.

### 2.1 First sample maze

- **Goal:** one small maze, generated in Rust by WFC from the core tile set and a seed, shown in the tool with the same tile drawing as the table.
- **Demo tasks** (Ian, without help):
  - Ian types a seed, presses Generate, and sees a maze drawn in [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)'s glyphs and colours.
  - Ian generates the same seed again and gets the same maze; a new seed gives a different one.
  - Ian changes a number in `data/tile-sets/core.toml`, presses Reload, regenerates the same seed, and sees the maze change. Reload already exists from 1.1, so this costs little and starts Ian's loop early. Comparing side by side is still 2.3.
  - Ian reads the measures Wren chose for 2.1 next to the maze (see "Waits on" item 2).
- **What's in it, and what isn't:**
  - One fixed-size maze that fits one screen. The size is a setting; the engineers pick the first one, a two-way door.
  - Hovering a cell shows its tile name, as in the table.
  - Not in it: stepping through the collapse (2.4), batches (2.5), clicking through to the table (2.2), growing past the edge (2.6), persistence.
- **Done when:**
  - generation runs in `maze-core`, in Rust, not in the frontend ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), [DR-0012](/decisions/0012-code-layout-and-core-crate.md));
  - the same seed and the same tile set always give the same maze, checked by a test;
  - every pair of neighbouring cells in a generated maze has a probability above 0 in the 1.1 table, checked by a test over many seeds;
  - a contradiction never crashes the tool or hangs it: it is handled the way Yew's record says, and the tool says when it happened (for example "restarted 2 times");
  - the basic check from 1.5 runs as a test: every side of every tile can meet something (agreed by Wren and Yew);
  - the demo steps are in the README, and the slice is on `main` with a line in `/log.md`.
- **Waits on** three items, all two-way doors unless Yew judges otherwise. Each is settled at the start of the build, not in a separate planning round:
  1. **The WFC approach (Yew, approved by Yew and one engineer).** One record covering the DR-0007 follow-ups that 2.1 needs: how a cell is chosen and collapsed, how the weights of several neighbours combine (deferred to 2.1 by [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) and [DR-0022](/decisions/0022-weight-model.md)), contradiction handling, determinism (seeded randomness), and what lies past the maze's edge. Persistence and infinite WFC stay out; they wait for 2.6. Fallback tiles are already low-weight tiles in the set ([DR-0015](/decisions/0015-tile-set.md), [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)). Yew judges whether any part is a one-way door; if one is, it goes to Ian. This record also stands as 2.1's plan.
  2. **Which [Q-0008](/questions/open/0008-global-maze-properties.md) measures 2.1 shows (Wren, approved by Wren and Marram).** This is Q-0008's first part, now due. Marram's suggestion is one cheap measure for "there is always a way onward", such as the share of cells reachable from the centre. Zero measures is acceptable if Wren prefers to wait for 2.5. How properties are enforced stays parked until before the game.
  3. **The generation view (Damson, approved by Damson and Marram).** Kept small: a seed field, Generate, the maze, the contradiction note and Wren's measures. Damson works it out with the engineer building the view, and records it in a short record.
- **Status:** done (2026-11-01), on `main` and pushed. Ian (`human:ian`), verbatim: "just saw a demo of the generated maze - looking good so far." Then: "Yes, land 2.1".
  - **Records:** [DR-0028](/decisions/0028-wfc-approach-for-2-1.md) (WFC approach and plan), [DR-0029](/decisions/0029-generation-view-design.md) (generation view) and [DR-0030](/decisions/0030-measures-for-2-1.md) (measures), all accepted.
  - **Built:** Rowan built WFC and the two measures in `maze-core`, the `generate_maze` command and ts-rs bindings. Ash built Maze and Table tabs under a shared tile set bar and the maze view, now on the real command. The branches were rebased onto slice 1.3 and joined.
  - **Tests:** 74 Rust tests pass, covering determinism, every neighbouring pair allowed over 200 seeds, the basic check, and a contradiction that fails cleanly after 20 restarts. 67 Vitest tests pass. Clippy and the build are clean. A 24×16 maze takes about 1.8 ms in release.
  - **Demo, run by the facilitator in the real Tauri window, driven by script:**
    - Generate draws the maze in DR-0017's glyphs.
    - The same seed gives the same maze, and a new seed gives a different one. Enter generates too.
    - Hover shows the tile name, and the centre is outlined.
    - Editing `core.toml` (hedge weight 1.5 to 6.0) and pressing Reload regenerates the same seed into a different maze; reachable went from 20% to 80%. Reverting the edit restores the maze.
    - A broken `core.toml` shows the red banner and keeps the last good maze.
    - With the fallback and hedge weights at 0, seed 11 showed "restarted 1 time".
  - **Reviews:**
    - Damson passed the view against DR-0029, with nothing blocking. Follow-up: the tile set path wraps mid-word; show the file name with the full path on hover.
    - Wren reviewed six mazes ([review](/plans/2.1-maze-review.md)). They look like hedge mazes but are poor to walk: 8–88% reachable from the centre, and 5–19 separate areas. Wren's tuning ideas are for 2.3 and 2.5.
  - **Not verified:** the give-up message after 20 restarts was not seen in the window, because the core set never failed. It is covered by Rust and Vitest tests.

### 2.2 Why is this here? (includes 1.2)

- **Goal:** click a spot in a maze, and the explore view from 1.2 opens on it, showing how its neighbours combined (Damson).
- **Demo tasks** (Ian, without help): the 2.2 task, and 1.2's two tasks with it.
  - Ian generates a maze, clicks a bit he doesn't like (a long one-deep room bar from Wren's [2.1 review](/plans/2.1-maze-review.md), say), presses [Why is this here?], and sees why it's there: which neighbours counted when it was drawn, the chance each candidate had, and, from a click on a factor, the number behind it.
  - Ian names a corridor corner's most likely east neighbour.
  - Ian clicks from a corridor into a room and back out, through doorways.
- **What's in it, and what isn't:**
  - 1.2's explore view in its own Explore tab, built to [DR-0026](/decisions/0026-explore-one-tile-design.md)'s must-haves.
  - The link from the maze and the "Why it's here" band, built to [DR-0033](/decisions/0033-why-is-this-here-design.md)'s must-haves. "Why" means what WFC saw when it drew the cell, replayed from the seed, not what is around the cell now.
  - Not in it: the ruled-out list, only its count (Marram, on review of DR-0033; revisit at 2.4); "what would fit here now" (2.3 or 2.4); refreshing the band in place after Reload (it clears with a note instead); editing in the tool (2.3).
- **Done when:**
  - **Explore, 1.2's done-when:** it works for every tile and orientation in the core set, and agrees with the 1.1 table. A test checks all 29 pairs in four directions: each arm is exactly that row's entries from the 1.1 table, sorted in Rust, with the 0s split off and counted.
  - **The trace is the maze on screen:** any cell of a generated maze can be traced. Tests show the replay picks each cell's tile over 200 seeds and leaves generation as it was, so 2.1's seeds give the same mazes and 2.1's tests pass unchanged.
  - **Its numbers are the solver's and the table's:** weight × factors equals the solver's score exactly, the chances add up to 100%, and each factor carries its 1.1 table entry, so DR-0025's panel explains it. Rust sorts and works out every number, and the frontend only formats them ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), [DR-0025](/decisions/0025-table-view-design.md)).
  - **Never wrong, never a crash:** a reloaded tile set (`stale`), a cell outside the maze, a bad size, no tile set and a failed maze each have a test, and none of them panics. After a Reload the band never shows an old trace.
  - **Checks:** the bindings are regenerated, and the Rust tests, Vitest, clippy and the build are clean.
  - **The view:** DR-0026's and DR-0033's must-haves are built, and Damson has reviewed the view against both records. Any may-slip item that was cut is listed under Status as a follow-up.
  - **The demo:** the facilitator has run all three demo tasks in the real Tauri window. The steps are in the README, and the slice is on `main` with a line in `/log.md`.
  - The technical half is DR-0034's build plan and tests ([DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)).
- **Waits on:** DR-0033 (Damson's link design; Damson and Marram) and DR-0034 (Yew's data contract, which is also 2.2's plan; Yew and one engineer). Both are two-way doors, settled at the start of the build. The explore view's design is settled (DR-0026).
- **Status:** done (2026-11-01), on `main`. Ian (`human:ian`), verbatim: "I looked at the 2.2 demo, its approved and can land on main".
  - Built by Rowan (Rust: observer hook, trace, `explore_tile`, `trace_cell`, bindings) and Ash (Explore tab, maze link, band, README), joined with no conflicts. 98 Rust and 124 Vitest tests pass; clippy and the build are clean. DR-0033 and DR-0034 are accepted.
  - Two Done-when points were met differently, because this ran in a cloud session with no window: Ian ran the demo in the Tauri window himself, and the facilitator didn't. Damson's review of the built view against DR-0026 and DR-0033 hasn't happened yet, and is a follow-up, with Ash's layout calls listed in DR-0033's Concerns raised.
  - Slipped, as follow-ups: keyboard on the map, refreshing the band in place after Reload (it clears with a note), "after reload; was ═", DR-0026 point 11. The ruled-out list waits for 2.4 (Marram).
  - Also landed with it: Damson's three follow-ups (the tile set path shows the file name; the flatter shade legend; the corner sentence names a single tile type).
  - History, before landing:
  - **Gates, approvals pending:**
    - [DR-0033](/decisions/0033-why-is-this-here-design.md): Marram approved it, with the ruled-out list out of 2.2; Ash's approval, standing in for Damson, is pending.
    - [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md): approvals pending. It gained four small additions, A1 to A4, so that it serves DR-0033: each of the four neighbours marked counted, came later or edge; candidates sorted in Rust; a ruled-out count; and each factor carrying its table entry.
  - **Build:** Rowan does the Rust side: the observer hook in `wfc.rs`, the trace, the `explore_tile` and `trace_cell` commands, and the bindings. Ash does the frontend: the Explore tab, the link and the band. Each works in their own worktree (`rowan/2.2-cell-why`, `ash/2.2-explore-view`), against DR-0034's types.
  - **Alongside 2.6** (Marram's call): 2.2 holds `README.md`, and its shape wins if the two clash in `wfc.rs`.

### 2.3 Tune and compare (now, tool track)

Readied by Marram on 2026-11-01, to start now. Ian, 2026-11-01: "There is such a thing as polishing too much before moving on." So 2.3 is kept small: edit a number, see the same seed change side by side, and try opening pressure. Scope and sequencing are Marram's calls, two-way doors ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).

- **Goal:** Ian's loop in one place ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)): he sees a problem in a maze, changes the probability behind it in the tool, and compares the same seed before and after. Editing comes in here ([Q-0010](/questions/resolved/0010-where-editing-goes.md)), and so does Ian's opening pressure (see "Steering 2026-11-01").
- **Demo tasks** (Ian, without help):
  - Ian generates a maze, opens "Why is this here?" on a bit he doesn't like, changes the number behind one of its factors in the tool, and saves. The same seed regenerates beside the old maze, the changed cells are marked, and he points at what changed and how the two measures moved.
  - Ian sets opening pressure to 0.5 in the tool, regenerates the same seed, and sees the reachable share and separate areas ([DR-0030](/decisions/0030-measures-for-2-1.md)) move. "Why is this here?" on a cell shows opening pressure as its own factor.
  - Ian opens `data/tile-sets/core.toml` in an editor, sees his edits in place with the file's comments kept, and finds the record of what he changed, from what to what. He sets one number back, and the maze returns.
- **What's in it:**
  - **Editing the numbers the tile set already has:** the weights and exceptions that 1.1's explanation already shows ([DR-0022](/decisions/0022-weight-model.md)), and the `opening_pressure` key ([DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 9). Editing opens from a number in the explanation panel, wherever it already appears (table, explore, the "why" band). Where exactly is Damson's design.
  - **Saving writes `core.toml`** through Rust ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)), then reloads it through the existing load path. So an edit the loader refuses, such as one that fails the completeness check (DR-0036 rule 7), is refused with the loader's reason and the file is left as it was. Each saved edit is recorded, as DR-0038 says.
  - **Compare:** the same seed before and after the last save, side by side, with changed cells marked and both sets of measures. Rust works out which cells changed and the measures; the frontend draws them.
  - **Opening pressure in the single maze.** 2.6 step 1 builds the factor into the engine for world growth only. 2.3 brings it into the tool's single-maze path, where a side past the maze's edge counts as new (DR-0036 rule 9), and into the "why" trace: [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md)'s contract gains the factor (Ash's note on DR-0036: the trace needs a pressure field once the pressure isn't 1).
  - **The band refreshes in place after a save,** instead of clearing (Damson's 2.2 follow-up). Editing makes reloads constant, so this is in, not a follow-up.
  - **Wren's tuning pass:** Wren tries opening pressure against tuning idea 4, damping corner-into-corner ([2.1 review](/plans/2.1-maze-review.md)), side by side in the tool, and sets the first `opening_pressure` value in `core.toml`. A game-design call (Wren, approved by Wren and Marram), written up in a short review note.
- **Not in it:** adding or removing tiles, sides or exceptions in the tool; undo beyond setting a number back; comparing more than two mazes, or batches (2.5); grown worlds (2.6 step 3); zones; "what would fit here now" (2.4); the ruled-out list (2.4).
- **Done when:**
  - a saved edit is written to `core.toml` by Rust, keeps the file's comments and layout, loads, and is recorded as DR-0038 says;
  - an edit the loader refuses changes nothing on disk and says why, with a test;
  - the same seed and the same numbers give the same maze; with no `opening_pressure` key, 2.1's pinned seeds are unchanged, with a test;
  - weight × factors still equals the solver's score exactly, with opening pressure included, over 200 seeds at a pressure below 1; values outside 0 < value ≤ 1 are refused at load;
  - compare's changed cells and measures come from Rust, with a test;
  - Damson has reviewed the view against DR-0037, and Wren's tuning note is written;
  - bindings regenerated; Rust tests, Vitest, clippy and the build are clean; the demo steps are in the README, and the slice is on `main` with a line in `/log.md`.
- **Waits on** two gates, both two-way doors unless Yew judges otherwise, settled at the start of the build by the roles named, with no planning round:
  1. **Compare and edit design, DR-0037** (Damson, approved by Damson and Marram), kept small like [DR-0029](/decisions/0029-generation-view-design.md): where editing opens and how a number is changed; the before-and-after layout and how changed cells are marked; where the opening-pressure control sits; and what a refused edit looks like.
  2. **How the tool edits `core.toml` and keeps a record of it, DR-0038** (Yew, approved by Yew and one engineer): writing the file without losing comments and layout; what the edit record is and where it lives; the edit and compare commands and their bindings ([DR-0023](/decisions/0023-rust-typescript-types-in-step.md)); and opening pressure in the single-maze path, with the factor added to DR-0034's trace contract. Yew judges whether any part is a one-way door; if one is, it goes to Ian.
  - **Also uses 2.6 step 1's** `opening_pressure` key and factor. If step 1 hasn't landed when 2.3 needs them, 2.3 builds editing and compare first and picks up the factor when step 1 lands. Nothing in 2.3 waits on it to start.
- **Beside 2.6** (Marram's call): separate sessions and worktrees. 2.3 holds the tool's views and `README.md`; on a clash in `wfc.rs`, whichever lands second adapts.
- **Status:** ready to start (2026-11-01). Its gates are next.

### 2.6 Beyond the edge (engine track; redrawn for DR-0036)

Started 2026-11-01. Ian (`human:ian`), verbatim:

> Start slice 2.6's chunk generator in crates/maze-core, built to DR-0031. Start with the order-independence and way-onward tests, plus the completeness check when the tile set loads. It's pure Rust and needs no window.

Then Ian steered how mazes grow (see "Steering 2026-11-01"), verbatim: "and yes I mean growth follows the players path". [DR-0035](/decisions/0035-the-world-is-stored.md) stores the world, and [DR-0036](/decisions/0036-growth-from-open-endpoints.md), accepted, replaces DR-0031's seam-first chunks with patches grown from open endpoints. Marram redrew 2.6 for it on 2026-11-01 (a two-way door). Order independence no longer applies: the world depends on the walk, so step 1 proves replay instead. The steps stay in the same order, and 2.6 still ends on screen (step 3).

Marram's calls, all sequencing and scope, so two-way doors ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)): 2.6 runs as an engine track beside 2.3. Steps 1 and 2 are `maze-core` only, with no design gate and no window.

- **Goal:** grow a stored world from open endpoints along the player's path, keep a way onward, and show it in the tool, walked and replayed. The technical definition of done is DR-0036's.
- **Step 1, the stored world and patch growth (built).**
  - **In it:** DR-0036 rules 1 to 10. A paged sparse world of palette indices (DR-0035); growth at the nearest anchor within r, by patches, with the keep flood; the outward check, the spine and the fallback; opening pressure as an optional key, default 1; a growth log with replay; and the completeness check at load, citing DR-0036 rule 7. P = 16, r = 8, R = 3.
  - **Not in it:** the measures (step 2); any view, IPC or window (step 3); new dependencies.
  - **Done when:**
    - **replay:** the same seed and the same walk give the same world, and a patch regenerated from the world before it comes out the same;
    - over 100 seeded random walks of 1,000 steps: the frontier rule (DR-0036 rule 1) holds after every patch; stored cells never change; every pair of neighbouring stored cells has an entry weight above 0; every stored cell is joined to the origin; there is an outward endpoint after every patch;
    - the spine and the fallback are each forced in a test;
    - the completeness check runs when a tile set loads, as an error, and its messages cite DR-0036 rule 7. `core.toml` passes; a `passage_passage` of 0 fails, and so does a doorway-against-passage weight of 0 (rule 7's new clause), each saying what is missing;
    - with no `opening_pressure` key, 2.1's pinned seeds are unchanged, and 2.1's and 2.2's tests pass as they are;
    - clippy and the build are clean.
  - **Status:** built by Rowan as `a813a73` on `rowan/2.6-chunks`. 134 Rust tests pass, at about 1 ms a patch in release. Rowan is rebasing it onto `main`, where its `wfc.rs` change to `collapse` meets 2.2's observer hook (2.6 adapts).
  - **Demo, no new window:**
    - Ian runs `cargo test -p maze-core` and reads the 2.6 tests passing, named for what they check.
    - Ian sets `passage_passage` to 0 in `core.toml` and presses Reload: the tool refuses the set and says why. He puts it back, and Reload works.
  - **Landing (Marram's call):** step 1 can land on `main` before the rest of 2.6, with Ian's OK after the demo, once the rebase is green. It is additive and leaves 2.1's path alone while the pressure is 1. 2.6 isn't done until step 3.
- **Step 2, the measures (built, no window).** DR-0036's list, re-cut for patches, from a release build:
  - per patch: the restart, spine and fallback rates;
  - the frontier (open and outward endpoints) over a walk, with pressure at 1 and at Wren's value from 2.3;
  - room sizes at P = 16 and P = 32, and whether rooms avoid the edges of patches (Yew's concern 1);
  - time per patch (median, 99th percentile and the forced worst case), and how the rule 6 check's time grows with the explored area (Yew's concern 2);
  - memory per 1,000 stored cells (DR-0035).

  - **Status:** built by Rowan, judged by Wren and checked by Yew: every measure passes ([report](/plans/2.6-measures.md)). The command is `cargo run --release -p maze-core --example measures`. Restarts are 0–0.1%, with no spines or fallbacks; the frontier never closes; grown worlds are 100% reachable against 2.1's median 59%; a P = 16 patch takes about 1 ms. Wren keeps the world's pressure at 1, and P stays 16; both are watched on screen in step 3. The incremental rule 6 check is deferred to the game, with Yew's trigger in DR-0036.
  Ian reads them as a short report from one command, and they are written into the 2.6 records. It starts as soon as step 1's rebase is green, without waiting for it to land. A miss gets one of the recorded answers, not open-ended rework: a larger P for rooms that avoid patch edges (DR-0036), an incremental rule 6 check for growth time (the engineers' call), or a new pressure value for a closing frontier (Wren). 2.5 reuses the frontier and room-size measures.
- **Step 2b, a walker that avoids backtracking, and two more measures (straight after step 2 lands; Marram's call, a two-way door).** First the walker, after Ian's steering (`human:ian`, 2026-11-01), verbatim: "I think that "random walk" needs some work - there needs to be some bias against back-tracking - keep a list of where it has walked and make it a much lower chance (not impossible) for it to walk over a spot it walked before rather than a spot it hasn't walked to." Rowan builds it in `maze-core` as a `Walker` that keeps the cells it has walked, and Wren picks how much lower the chance is; 2.6 step 3's tool switches to it. Then, from Wren's walks in step 3: cells stored per patch, and the frontier right after the first patch (seed 1 opens with 2 endpoints on one side, and its random walk never grows past patch 1). Plus Yew's check that each walk in step 1's test grows at least some number of patches. The report is re-run once at the end. If 2b runs past about half a day, the walker ships on its own as a commit Ian can run. Not in it: how often a hedge cell the player has seen later opens (Yew's concern 3), which waits for G.1. See the [report](/plans/2.6-measures.md).
- **Step 3, the view and the review (after step 1 lands, alongside 2.3).**
  - **In it:** DR-0036's view: a world seed and a walker (a seeded random walk, or click to walk), with overlays for stored cells, the frontier and the last patch, and a way to replay the walk. Unsettled cells are drawn as solid hedge. Plus the world commands and their ts-rs bindings, with what Ash listed on DR-0036: a frontier accessor, per-patch deltas over IPC, and `i64` positions typed as `number`.
  - **Waits on** a small design gate: Damson's short record for the view, approved by Damson and Marram, like [DR-0029](/decisions/0029-generation-view-design.md). It takes the next free number after 2.3's records. Damson writes it straight after DR-0037, so it is ready when step 1 lands.
  - **Then** Wren and Damson review it: rooms against patch edges, the frontier, and Yew's concern 3 (a cell drawn solid that later opens).
  - **Demo, which closes 2.6:** Ian walks a world in the tool, watches it grow from open endpoints with the frontier overlay on, then replays the walk and sees the same world.
  - **Why after step 1 lands, not after step 2:** it needs only step 1's core, and step 2's measures don't change the view.

### Later in Phase 2 (outline)

**Order, Marram's call (a two-way door): 2.5, then 2.4.** 2.5 is what closes Phase 2's "done when", judging the maps the table makes, and it should judge mazes with opening pressure in (2.3) and grown worlds' measures (2.6 step 2). 2.4 is a deeper debugging view, and 2.2's "why" trace already answers most of what it was for. **Revisit after 2.5:** whether 2.4 is still needed, or shrinks to the ruled-out list.

- **2.3 Tune and compare:** now; see 2.3 above.
- **2.5 Judge many samples:** generate a batch from seeds. Samples are big enough to judge pacing over a walk, not just one screen (Wren). Measures are taken from the player's side, not just as tile counts (Wren), and follow [Q-0008](/questions/open/0008-global-maze-properties.md):
  - reachable cells;
  - dead ends per area;
  - corridor run lengths;
  - room sizes and spacing.

  DR-0030's two measures don't apply to grown worlds (DR-0036), so Wren settles at the start of the build whether 2.5 judges single mazes, walked worlds, or both. It reuses 2.6 step 2's frontier and room-size measures.
- **2.4 Watch it collapse:** step through generation, seeing each cell's possibilities and where contradictions happen. The ruled-out list from 2.2 waits here.
- **2.6 Beyond the edge:** on the engine track now; see 2.6 above.

## The game (outline)

The game is its own app (`apps/game/`, [DR-0012](/decisions/0012-code-layout-and-core-crate.md)), and comes after Phase 2 ([DR-0008](/decisions/0008-adjacency-tool-first.md)).

- **Waits on:**
  - the full decision on how code is shared with the tool (Yew, with an engineer);
  - **persistence is settled:** the world is stored ([DR-0035](/decisions/0035-the-world-is-stored.md), Ian: "we want to know where the player has been, we will add loot"). What's left is how the game writes saves to disk, its own record when the game starts, and it gates G.2 only. Yew re-checks DR-0036's frontier rule (rule 1) before the game first saves a world;
  - how the rest of the global maze properties are enforced ([Q-0008](/questions/open/0008-global-maze-properties.md)); the way onward is guaranteed by DR-0036 rule 6;
  - game-design records from Wren on what the player does;
  - possibly technology choices. Any that change the tech stack are Ian's ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).
- **Game slices,** numbered G.1 onwards so they don't clash with Phase 2's numbers (Marram's call, a two-way door). Like Phase 2's, numbers are stable once given.
  - **G.1 Walk a growing world:** a player walks through a world grown around them from open endpoints along their path ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)), and walking back finds it as it was. Keeping the world is no longer its own slice: DR-0035 stores it in `maze-core`, so G.1 gets it for free.
  - **G.2 Save and load:** the world and the player survive closing the game. Waits on the save-format record (DR-0035 leaves it to the game). This is where the one-way part of DR-0035 starts to bite, so its record may go to Ian.
  - **G.3 Debug mode:** the tool's views (the "why" trace, overlays) inside the game.

## Later (not scheduled, nothing decided)

- **Zones:** Ian plans "in future different "zones" with different probabilities". That will need its own records when it comes up, after Phase 2 or in the game. Yew's weight model allows for them as extra sets of weights over the same base.
- **Rooms with no way in:** these are allowed for now. Ian: "We will revisit if and how we fix this later." Phase 2's measures of reachable cells (2.5) will show how often it happens.

## Inputs for records (not settled here)

These views belong in the records and questions that will settle them. The roadmap takes no position on them.

- **Weights:**
  - [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) settles that the table holds probabilities, with 0 where sides don't pair up.
  - Ian, verbatim, on what else sets them: "probabilities are mainly "if it can work" and "should we see it often in that place" - and some randomness for to add flavour. I'm expecting a big table."
  - Ian, verbatim: "I want to see the variation in the table of probabilities."
  - Yew chose a calculated base plus a few exceptions for particular pairs, in the tile set data, as a two-way door ([Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md) point 10). It needs one engineer's approval, then Yew's record.
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
| The weight model: where weights live, symmetry, explainable | [Q-0007](/questions/resolved/0007-open-details-of-table-probabilities.md) point 10, [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) | Yew, with Wren; Yew and one engineer | no | done ([DR-0022](/decisions/0022-weight-model.md)) |
| How the table is drawn and explained | design | Damson, Wren reviews; Damson and Marram | no | done ([DR-0025](/decisions/0025-table-view-design.md)) |
| How Rust and TypeScript types stay in step | architecture | Yew; Yew and one engineer | no | done ([DR-0023](/decisions/0023-rust-typescript-types-in-step.md)) |
| Tile set file format | architecture | Yew; Yew and one engineer | no | done ([DR-0024](/decisions/0024-tile-set-file-format.md)) |
| Phase 1 plan | plan | Yew | no | done ([Phase 1 plan](/plans/phase-1.md)) |
| Design for exploring, openable from anywhere | design | Damson; Damson and Marram | no | done ([DR-0026](/decisions/0026-explore-one-tile-design.md)); used in 2.2 |
| Design for filtering | design | Damson; Damson and Marram | no | done ([DR-0027](/decisions/0027-filter-table-design.md)); 1.3 done |
| Tiles added beyond the core set | game design, [DR-0016](/decisions/0016-wren-designs-the-tile-set.md) | Wren; Wren and Marram | no | nothing scheduled; when sample mazes show the need (1.5 dropped) |
| Paging and transport | architecture | Yew; Yew and one engineer | no | nothing scheduled; when the table grows (1.5 dropped) |
| The WFC approach for 2.1: collapse, how neighbour weights combine, contradiction handling, determinism (seeds), the maze's edge | DR-0007, [DR-0013](/decisions/0013-table-entries-from-tile-sides.md), [DR-0022](/decisions/0022-weight-model.md) | Yew, with Wren on determinism; Yew and one engineer | no, unless Yew judges a part one-way | 2.1 (settled at the start of the build) |
| Which global properties to measure | [Q-0008](/questions/open/0008-global-maze-properties.md) | Wren, Yew advises on cost; Wren and Marram | no | 2.1 (Wren picks what 2.1 shows, at the start of the build); 2.5 for the full set |
| Design for the generation view (small, settled during the 2.1 build) and the link from maze to table | design | Damson; Damson and Marram | no | 2.1, 2.2 |
| Research into infinite WFC, and how the world grows | DR-0007 | Yew; two engineers (Yew authored) | no, unless Yew judges a part one-way | done: [DR-0031](/decisions/archive/0031-endless-generation-approach.md), superseded by [DR-0036](/decisions/0036-growth-from-open-endpoints.md) (accepted 2026-11-01), which gives 2.6 its shape |
| Persistence | DR-0007 | Ian, 2026-11-01: the world is stored | **yes**: Ian decided it | done ([DR-0035](/decisions/0035-the-world-is-stored.md)); how saves are written to disk is the game's own record, gating G.2 |
| How global properties are enforced | [Q-0008](/questions/open/0008-global-maze-properties.md) | Yew, with Wren | no, unless Yew judges it one-way | Ian's steering of 2026-11-01: growing from open endpoints and the way onward, done ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)); opening pressure in the single maze, 2.3 (DR-0038); the rest, the game |
| Compare and edit design (DR-0037) | design | Damson; Damson and Marram | no | 2.3 (settled at the start of the build) |
| How the tool edits `core.toml` and records edits, and opening pressure in the single maze and its trace (DR-0038) | architecture, [DR-0024](/decisions/0024-tile-set-file-format.md), [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md) | Yew; Yew and one engineer | no, unless Yew judges a part one-way | 2.3 (settled at the start of the build) |
| Design for the growing-world view (next free number) | design | Damson; Damson and Marram | no | 2.6 step 3 |
| How the game saves the world to disk | [DR-0035](/decisions/0035-the-world-is-stored.md) | Yew; Yew and one engineer | likely **yes**, if Yew judges the save format one-way | G.2 |
| How code is shared with the game, in full | DR-0008, [DR-0012](/decisions/0012-code-layout-and-core-crate.md) | Yew; Yew and one engineer | no | the game |
| Replacing WFC as the generation approach, only if 2.6 finds WFC can't do endless mazes after all (the research found it can, DR-0031, and DR-0036 builds on it) | [DR-0007](/decisions/0007-wave-function-collapse.md) | Yew | **yes**: it changes the tech stack and throws away the generator built in Phase 2 | 2.6; the game |
| Technology for the game beyond Tauri v2, Rust and Lit, if any is needed | [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md) | Yew | **yes**: it changes the tech stack, and the game would be built on it | the game |
| Anything that costs Ian money or ties down his time, such as a paid service or publishing | [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) | whoever needs it | **yes**: the cost falls on Ian | none planned |
