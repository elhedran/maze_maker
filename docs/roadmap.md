---
type: Roadmap
title: Maze Maker roadmap
description: Phases, deliverables and demos for Maze Maker. Phase 1 is closed and 2.1 is done; slice 2.2 is in progress, and 2.6 pauses while Yew re-checks DR-0031 after Ian's steering on growing mazes.
status: stable
owners: [product-manager/1]
approvers: [human:ian]
tags: [roadmap, phase-1, phase-2, adjacency-table, tooling, demos, generation]
generated:
  by: product-manager/1
  at: 2026-11-01T00:10:00+01:00
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

Ian (`human:ian`), verbatim, 2026-11-01:

> Happy to use tokens faster if we can doo more work concurrently

So Marram brought 1.3 back, a sequencing call and a two-way door ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)). It runs now, alongside 2.1, in its own session and worktrees, with separate sessions of Rowan (Rust: DR-0027's `sides` filter, with tests) and Ash (the Lit filter bar, with Vitest tests). It takes no people or sessions from 2.1; the cost is tokens and a rebase against 2.1. It touches only the table view, its command and the filter code, to keep that rebase small. 2.1 stays first in line: if the two clash, 1.3 rebases. Its demo counts once it lands on `main` ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)).

## Steering 2026-11-01: improving the mazes

Ian (`human:ian`), verbatim, 2026-11-01, as steering for the product manager for future phases:

> Ok, on that not I have some future steering for the product manager for future phases around improving the mazes.  I'd like two changes - the first is we only generate based of "open" endpoints, endpoints connected to existing maze.  This basically means looking for where a path ends, a room door opens etc rather than just every space next to where the player has been, within the bounds of the player space radius.  The second change is to add some pressure to bias towards an opening maze.  that means that if a piece that goes down would only connect to already closed maze, that should get a reduced probability.

Then Ian (`human:ian`), verbatim, 2026-11-01, on [DR-0031](/decisions/0031-endless-generation-approach.md):

> In regards to DR-0031 the world needs to be stored anyway - we want to know where the player has been, we will add loot, and hence we need to know what is looted, etc.  However given how this works, there should be quite an efficient in-memory data structure for this (sparse big table).

And Ian (`human:ian`), verbatim, 2026-11-01, on what he meant:

> and yes I mean growth follows the players path

**What the two changes are.** This is the facilitator's reading, given to Ian: a proposal, not Ian's words. The full reading is in [Q-0008](/questions/open/0008-global-maze-properties.md).

- **Change 1, grow from open endpoints:** the maze only grows beside an open side (a passage end, a door) of something already generated, following the player's path, and what's never reached stays solid.
- **Change 2, opening pressure:** a candidate whose open sides all face settled cells, so that it adds no new open endpoint, gets a multiplier below 1 in [DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 3's score.
- **Considered and not taken:** growing each chunk from its gates, so that the world needn't be stored and DR-0031 holds. Ian means growth that follows the player's path, and the world is stored anyway.

**What it changes now:**

- **2.6's approach may change.** Yew is re-checking DR-0031 against Ian's steering and is likely to supersede it (expected as DR-0036). Ian's decision that the world is stored is being recorded as DR-0035, drafted by the facilitator and pending Ian's approval of its wording. That settles the game's persistence gate.
- **2.6 step 1 pauses at its next green point.** It keeps what survives: the per-cell starting masks and the completeness check. 2.6's new shape waits on Yew's record, and so does the sequencing of what follows; the facilitator briefs it once Yew has ruled. Marram doesn't redraw 2.6 here.
- **2.2 is unchanged.**

**Where Marram puts each change.** This is sequencing, a two-way door and Marram's call ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).

- **Change 2, opening pressure, goes in 2.3 Tune and compare.** It is one tunable factor in the solver's score, whatever grows the maze, so it doesn't wait on Yew's record. It works on today's single maze, and 2.3's demo already fits it: Ian changes the pressure, regenerates the same seed, and sees the reachable share and separate areas ([DR-0030](/decisions/0030-measures-for-2-1.md)) move. 2.2's "why" trace shows it as a factor. It goes after the same pockets as Wren's tuning idea 4, damping corner-into-corner ([2.1 review](/plans/2.1-maze-review.md)), so 2.3 tries the two side by side. **Revisit before 2.3's build starts**, when 2.2 lands.
- **Change 1, growing from open endpoints, is the core of 2.6's new shape**, so it is picked up there. Where it lands in the sequence is set once Yew has ruled. **Revisit when Yew's record replacing DR-0031 is accepted.**
- **Not 2.5:** it judges batches, so both changes should be in before it, and its measures should judge the maze they make.
- **The how is settled at pickup, with no planning round.** Yew settles the fit with DR-0028 and with the record that replaces DR-0031, and whether any part is a one-way door; if one is, it goes to Ian. Wren settles what the player feels, and the tuning.

## Now and next

Updated 2026-11-01 by Marram, when Ian asked to start 2.6 (see 2.6 below), and again for Ian's steering on improving the mazes (see "Steering 2026-11-01" above).

- **Done:** Phase 1 is closed. 1.1, 1.3 and 2.1 (the first sample maze) are on `main`. 1.2's design is settled ([DR-0026](/decisions/0026-explore-one-tile-design.md)) and gets built in 2.2. [Q-0008](/questions/open/0008-global-maze-properties.md)'s first part is settled by [DR-0030](/decisions/0030-measures-for-2-1.md); the rest stays open for 2.5 and the game.
- **Now, two tracks in parallel** (Marram's call; Ian, 2026-11-01: "Happy to use tokens faster if we can doo more work concurrently"):
  - **Tool track: 2.2 is done and on `main`** (Ian approved its demo on 2026-11-01).
  - **Engine track: 2.6 step 1**, the chunk generator in `maze-core`, with no window. It pauses at its next green point, after Ian's steering, keeping the per-cell starting masks and the completeness check.
- **Next:**
  - engine track: waits on Yew's record replacing [DR-0031](/decisions/0031-endless-generation-approach.md) (expected DR-0036), which gives 2.6 its new shape; the facilitator briefs what follows once Yew has ruled;
  - tool track: 2.3 once 2.2 lands, with opening pressure in it (Ian's steering);
  - then 2.4 and 2.5.

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
- **Status:** not started; comes with 2.2. Its design gate is met: [DR-0026](/decisions/0026-explore-one-tile-design.md) accepted (2026-10-31).

### 1.3 Filter the table (in progress, in parallel with 2.1)

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
  - **Follow-ups (Damson, not blockers):** make the shade legend look less like pressed buttons; when exactly one tile type is chosen, name it in the corner sentence ("Corridor corner tile, then…").
  - **Next:** Ian's OK to land; then the facilitator rebases onto `main` (2.1 may have landed by then), reruns the build and tests, and fast-forwards `main`.

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
- Since 2026-11-01: exploring one tile (now in 2.2) and full tile set v1 (dropped). Filtering (1.3) runs in parallel with 2.1.

## Phase 2: sample mazes

**Done when** Wren and Ian can judge the maps the table makes ([DR-0008](/decisions/0008-adjacency-tool-first.md)), and Ian can trace a problem he sees in a maze back to the probabilities behind it ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)).

**Status:** in progress. 2.1 is done and on `main`. 2.2 and 2.6's first step run in parallel (since 2026-11-01). 2.6 step 1 pauses at its next green point after Ian's steering, and 2.6's new shape waits on Yew's record (see "Steering 2026-11-01").

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

### 2.6 Beyond the edge (paused; new shape waits on Yew's record)

Started 2026-11-01. Ian (`human:ian`), verbatim:

> Start slice 2.6's chunk generator in crates/maze-core, built to DR-0031. Start with the order-independence and way-onward tests, plus the completeness check when the tile set loads. It's pure Rust and needs no window.

**Paused, 2026-11-01, and its approach may change.** After Ian's steering (see "Steering 2026-11-01" above: growth follows the player's path, and the world is stored), Yew is re-checking DR-0031 and is likely to supersede it (expected as DR-0036). Step 1 pauses at its next green point, keeping what survives: the per-cell starting masks and the completeness check. 2.6's new shape, and the sequencing of what follows, wait on Yew's record. The steps below are 2.6 as planned under DR-0031, kept for reference until then.

Marram's calls, all sequencing and scope, so two-way doors ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)): 2.6 runs as an engine track beside 2.2, in three steps, and moves ahead of 2.4 and 2.5. Steps 1 and 2 are `maze-core` only, with no design gate and no window, so they take nothing from 2.2. As a whole, 2.6 still ends on screen (step 3).

- **Goal:** grow a maze past its edge in chunks, to test where new areas meet old ones. Seam-first chunks on 2.1's generator ([DR-0031](/decisions/0031-endless-generation-approach.md)); the measures and definition of done are in [Endless WFC](/plans/endless-wfc-research.md).
- **Step 1, the chunk generator (now).**
  - **In it:** DR-0031 rules 1 to 6: border labels and gates from each border's canonical key, per-cell starting masks, the gate check, the spine, the fallback run with the spine, and the fixed attempt order. A chunk's result reports attempts, spine and fallback, and carries its gate positions and spine cells (Ash's DR-0031 note), so step 3 needs no core change. S = 16; R is the engineers' call.
  - **Not in it:** the measures (step 2); any view, IPC or window (step 3); new dependencies.
  - **Done when:**
    - order independence passes as tests (measure 1): a chunk made alone and after its neighbours in three shuffled orders comes out identical; each border's labels match from both its chunks; one chunk's tiles are pinned;
    - a way onward passes as tests (measure 2): the gates are joined in each of 1,000 chunks, and every gate in a 9 × 9 window is reachable from the origin's gate;
    - the spine and the fallback run are each forced in a test, and each gives a complete chunk with its gates joined;
    - the completeness check (DR-0031 rule 6) runs on table entries when a tile set loads, as an error. `core.toml` passes. A missing variant, a `passage_passage` or `hedge_hedge` base of 0, or a straight-to-straight multiplier of 0 fails, with a message that says what is missing and that endless mazes need it. It runs where the tool already loads the tile set, so Reload's banner shows it with no tool change;
    - 2.1 is unchanged: its tests pass as they are, and its seeds give the same mazes as on `main` today;
    - clippy and the build are clean.
  - **Demo, no new window:**
    - Ian runs `cargo test -p maze-core` and reads the 2.6 tests passing, named for what they check.
    - Ian sets `passage_passage` to 0 in `core.toml` and presses Reload: the tool refuses the set and says why. He puts it back, and Reload works. This is the one change he can see: a tile set that breaks the guarantee no longer loads, in any view.
  - **Landing (Marram's call):** step 1 can land on `main` before the rest of 2.6, with Ian's OK after the demo. It is additive and leaves 2.1's path alone, and landing early keeps the `wfc.rs` rebase against 2.2 small. 2.6 isn't done until step 3.
  - **Beside 2.2:** 2.6 changes `wfc.rs` only for the starting masks, and keeps the chunk code in its own files. Either can land first; on a clash in `wfc.rs`, 2.6 adapts. Its demo steps go in its report, and into `README.md` when it lands.
- **Step 2, the measures (next, still no window).** Measures 3 to 7 from the research note, from a release build, written into the 2.6 records: fallback rate, restarts and spines, seam statistics, room sizes at S = 16 and 32, and time per chunk (median, 99th percentile, the forced worst case). Ian reads them as a short report from one command. It starts once step 1 is green, without waiting for it to land. A miss is answered by one of the research note's recorded choices (tune the seam labels, raise S, or layered blocks), not by open-ended rework. 2.5 reuses the room-size measure.
- **Step 3, the view and the review (after 2.2 lands, alongside 2.3).**
  - **In it:** the research note's "Beyond the edge" view: a world seed, a 5 × 5 window of chunks, panning, a chunk-grid overlay, and restarts, spine and fallback on hover. Plus the chunk command and its ts-rs bindings, with `i64` coordinates typed as `number` (Ash's DR-0031 note).
  - **Waits on** a small design gate: Damson's short record for the view, approved by Damson and Marram, like [DR-0029](/decisions/0029-generation-view-design.md). Damson can settle it once 2.2's link design is done, so the build starts as soon as 2.2 lands.
  - **Then** Wren and Damson review seams and rooms: measure 5's "can't reliably point out borders", and judging measures 4 and 6.
  - **Demo, which closes 2.6:** Ian pans a world, toggles the overlay, regenerates in a different order and sees the same world.
  - **Why after 2.2:** it shares the tool's shell and maze drawing with 2.2, and Damson's design time goes to 2.2 first.

### Later in Phase 2 (outline)

- **2.3 Tune and compare:** Ian's loop. Change probabilities, reload, regenerate with the same seed and compare (Wren's loop, moved up for Ian).
  - Demo: Ian changes a probability, regenerates the same seed, and points at what changed.
  - Editing probabilities in the tool comes in here ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). If the weight model ever became weights set by hand on every pair, Marram and Yew would move editing earlier.
  - Opening pressure comes in here too (Ian's steering of 2026-11-01, change 2; see "Steering 2026-11-01" above). It is one more factor in the solver's score, tuned in the same loop and shown in 2.2's trace. Yew and Wren settle the how at the start of the build.
- **2.4 Watch it collapse:** step through generation, seeing each cell's possibilities and where contradictions happen.
- **2.5 Judge many samples:** generate a batch from seeds. Samples are big enough to judge pacing over a walk, not just one screen (Wren). Measures are taken from the player's side, not just as tile counts (Wren), and follow [Q-0008](/questions/open/0008-global-maze-properties.md):
  - reachable cells;
  - dead ends per area;
  - corridor run lengths;
  - room sizes and spacing.
- **2.6 Beyond the edge:** started early; see 2.6 above.

## The game (outline)

The game is its own app (`apps/game/`, [DR-0012](/decisions/0012-code-layout-and-core-crate.md)), and comes after Phase 2 ([DR-0008](/decisions/0008-adjacency-tool-first.md)).

- **Waits on:**
  - the full decision on how code is shared with the tool (Yew, with an engineer);
  - persistence, meaning whether the world is kept when the player walks back. This is being settled by Ian's decision that the world is stored ("we want to know where the player has been, we will add loot"), recorded as DR-0035, pending Ian's approval of its wording;
  - how the global maze properties are enforced ([Q-0008](/questions/open/0008-global-maze-properties.md));
  - game-design records from Wren on what the player does;
  - possibly technology choices. Any that change the tech stack are Ian's ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).
- **Smallest first slice:** a player walks through a maze generated around them. After Ian's steering of 2026-11-01, the maze grows from open endpoints and follows the player's path (see "Steering 2026-11-01" above); how is Yew's record. After that comes keeping the world when the player walks back, then a debug mode with the tool's views.

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
| Design for filtering | design | Damson; Damson and Marram | no | done ([DR-0027](/decisions/0027-filter-table-design.md)); 1.3 in progress |
| Tiles added beyond the core set | game design, [DR-0016](/decisions/0016-wren-designs-the-tile-set.md) | Wren; Wren and Marram | no | nothing scheduled; when sample mazes show the need (1.5 dropped) |
| Paging and transport | architecture | Yew; Yew and one engineer | no | nothing scheduled; when the table grows (1.5 dropped) |
| The WFC approach for 2.1: collapse, how neighbour weights combine, contradiction handling, determinism (seeds), the maze's edge | DR-0007, [DR-0013](/decisions/0013-table-entries-from-tile-sides.md), [DR-0022](/decisions/0022-weight-model.md) | Yew, with Wren on determinism; Yew and one engineer | no, unless Yew judges a part one-way | 2.1 (settled at the start of the build) |
| Which global properties to measure | [Q-0008](/questions/open/0008-global-maze-properties.md) | Wren, Yew advises on cost; Wren and Marram | no | 2.1 (Wren picks what 2.1 shows, at the start of the build); 2.5 for the full set |
| Design for the generation view (small, settled during the 2.1 build) and the link from maze to table | design | Damson; Damson and Marram | no | 2.1, 2.2 |
| Research into infinite WFC | DR-0007 | Yew | no, unless Yew judges a part one-way | done ([DR-0031](/decisions/0031-endless-generation-approach.md)); being re-checked by Yew after Ian's steering of 2026-11-01, likely superseded (expected DR-0036), which gives 2.6 its new shape |
| Persistence | DR-0007 | Ian, 2026-11-01: the world is stored; recorded as DR-0035, drafted by the facilitator, pending Ian's approval of its wording | **yes**: Ian decided it | 2.6's new shape; the game |
| How global properties are enforced | [Q-0008](/questions/open/0008-global-maze-properties.md) | Yew, with Wren | no, unless Yew judges it one-way | Ian's steering of 2026-11-01: opening pressure in 2.3, growing from open endpoints with 2.6's new shape; the rest, the game |
| How code is shared with the game, in full | DR-0008, [DR-0012](/decisions/0012-code-layout-and-core-crate.md) | Yew; Yew and one engineer | no | the game |
| Replacing WFC as the generation approach, only if 2.6 finds WFC can't do endless mazes after all (the research found it can, DR-0031) | [DR-0007](/decisions/0007-wave-function-collapse.md) | Yew | **yes**: it changes the tech stack and throws away the generator built in Phase 2 | 2.6; the game |
| Technology for the game beyond Tauri v2, Rust and Lit, if any is needed | [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md) | Yew | **yes**: it changes the tech stack, and the game would be built on it | the game |
| Anything that costs Ian money or ties down his time, such as a paid service or publishing | [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) | whoever needs it | **yes**: the cost falls on Ian | none planned |
