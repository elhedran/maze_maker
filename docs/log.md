# Log

## 2026-11-01

- **Update** Ian added the remote `origin` (https://github.com/elhedran/maze_maker) and pushed `main`. `CLAUDE.md`'s Git section and DR-0021 record it. Agents still don't push until a record replaces DR-0021; the team was told.

- **Update** Slice 1.3 is on `main` (fast-forward to e520d07) after Ian's OK, verbatim: "Yes, land 1.3". Tests pass on `main`. Roadmap 1.3 status updated.

- **Update** Slice 1.3, filtering the table, is built to [DR-0027](/decisions/0027-filter-table-design.md) on branch `claude/busy-ritchie-ebe382`, not yet on `main`. Rowan added the `sides` filter in Rust, with a `bad_request` error unless the request names one direction. Ash built the filter bar, with nothing slipped, and fixed a lost-update bug found in the live check. 52 Rust and 37 Vitest tests pass. The facilitator drove the real Tauri window by script, and Damson passed it against DR-0027 with two follow-ups (the shade legend, and the corner sentence for one tile type). Ian, verbatim: "new filters look good". It lands on `main` after Ian's OK. Roadmap 1.3 status updated.

- **Update** Roadmap: Marram brought slice 1.3, filtering the table, back to run in parallel with 2.1, a sequencing call and a two-way door, after Ian said "Happy to use tokens faster if we can doo more work concurrently". It has its own session, worktrees and agent sessions of Rowan (Rust `sides` filter) and Ash (Lit filter bar), takes nothing from 2.1, and touches only the table view, its command and the filter code. DR-0027's slip list stands. Status lines updated.

- **Update** Ian ran slice 1.1's two demo tasks himself from `main` and approved them: "demo approved - looks good.  I tried the east, tried the blank cell, also did a bit of an explore around how paths and rooms were represented". Roadmap 1.1 status and the Phase 1 plan updated; the plan's points 9 and 10 are checked.

- **Update** Roadmap revised by Marram for Ian's "the data side is good enough to start leveraging in maze generation"; approved by Ian ("Approve, start 2.1"). Maze generation is next, starting with 2.1, the first sample maze.
  - Phase 1 closes with 1.1, which is done and on `main`. Phase 1's "done when" shrinks to 1.1's demo, which is Ian's to approve.
  - Marram moved slices, a two-way door: 1.2 (explore one tile) folds into 2.2; 1.3 (filtering) moves to later, unscheduled, with DR-0027 kept; 1.5 (full tile set v1) is dropped, its basic check moving into 2.1, with tiles added when samples need them and paging waiting until the table grows.
  - 2.1 now has a goal, demo tasks and done-when points. It waits on three items, settled at the start of the build rather than in planning rounds: the WFC approach (Yew, with an engineer), which Q-0008 measures 2.1 shows (Wren, with Marram), and a small generation view design (Damson, with Marram).
  - Stale lines fixed: Q-0008 is parked and its first part is now due; the 1.1 gates are done.

- **Decision** DR-0027 accepted (written by Damson; approved by Marram; Yew accepted the `sides` addition to DR-0024): the design for slice 1.3, filtering the table. A bar above the grid has one group for the tile (rows) and one for its neighbour (columns), each filtering by kind, tile type, tag and the side on the shared edge; direction stays the 1.1 switcher. OR within a facet, AND across facets. A "showing N of 29 rows, M of 29 columns" line with one Clear all. Rust filters through DR-0024's `rowFilter`/`columnFilter`, plus a `sides` field that needs exactly one direction. The 1.3 design gate is met; index and roadmap updated.

- **Update** Slice 1.1, first look at the table, is built on branch `claude/slice-1-1-4a2272`, waiting to land on `main`.
  - Rowan built the Rust side: the tile set model, loader and validation, seeded noise and the table in `maze-core`; the `table_view` and `reload_tile_set` commands; ts-rs bindings. There are 45 Rust tests.
  - Ash built the Lit table view and explanation panel to DR-0025, with 24 Vitest tests.
  - Wren set the first numbers in `data/tile-sets/core.toml`, and six hand-checked entries match the tool.
  - The facilitator ran both demo tasks and the edit-and-Reload loop in the real Tauri window. It also made the tile headers stay in view while the grid scrolls.
  - Two tool tests had depended on the shipped numbers; they no longer do.
  - README has the demo steps.
- **Update** DR-0019 rule 11 numbers, from the first worktrees, are recorded in DR-0019. Rust: first build 1m 40s, first test run 3.9s, 4.0 GB. Frontend: `npm install` 29.3s, test 3.0s, build 1.9s.

## 2026-10-31

- **Decision** DR-0026 accepted (written by Damson; approved by Marram): the design for slice 1.2, exploring one tile. The chosen tile sits in the middle with a list of neighbours on each side, sorted by share, and the facing side kinds on the shared edge; numbers and explanations come from the 1.1 table and DR-0025's panel; [Go] steps to a neighbour, and [< Back] and a trail go back; one entry point opens it for any tile and orientation, for 2.2 to reuse. Marram listed what the demo must have and what can slip. The 1.2 design gate is met; index and roadmap updated.

- **Decision** DR-0023, DR-0024 and DR-0025 accepted for slice 1.1. DR-0023 (Yew; Ash approves in the architect's place): IPC types in the tool crate derive ts-rs, bindings checked in, a test fails when stale. DR-0024 (Yew; Ash): the tile set is one TOML file in `data/tile-sets/`, validated by maze-core and reloaded by the tool; the `table_view` command takes filters and a window and returns entries with probability (normalised per tile and direction) and explanation; seeded noise goes in for 1.1, departing from DR-0022's hand-set default under its condition 4, so Ian sees per-cell variation. DR-0025 (Damson; Marram): one grid per direction, grouped by kind, shade plus row percentage, and an explanation panel. Phase 1 plan added in `/plans/`.

- **Update** Q-0008 parked, to be revisited before slice 2.1, after Ian asked to "bias towards executing planned work over re-planning or planning new work". Nothing new was settled, and no record was written.

- **Decision** DR-0022 accepted (Yew decides, Rowan approves; drafted by the facilitator): the weight model is a side-pair base times a weight per tile times a few exception multipliers keyed by tile type or tag and the side kinds that meet, all in the tile set data. Side rules always win, bad data is rejected, the table is reproducible, and the tool shows where each entry comes from. A two-way door; Yew overrules Marram's concern that it is one-way, which is recorded. Q-0007 resolved and moved to `/questions/resolved/`; links, indexes and the roadmap updated.

- **Update** Roadmap approved by Ian.

- **Decision** DR-0019 revised and re-accepted (written by Yew; approved by Ash in the architect's place and by Marram): parallel code work gets its own worktree in a git-ignored `.worktrees/`, which the facilitator creates, lands by rebase and fast-forward, and removes. Docs-only parallel work stays in the main checkout. Shared files and record numbers stay with the facilitator. `.worktrees/` added to `.gitignore`.

- **Decision** DR-0021 accepted (Ian, reviewed by Yew and Ash): work lands on `main`, local branches and worktrees are allowed, no remote yet. It supersedes DR-0009, which wrongly recorded a ban on branches that Ian never made; DR-0009 moved to the archive. `CLAUDE.md` and the engineers' agent files updated.

- **Update** Corrected timestamps in DR-0019, DR-0020 and DR-0021, which had been written ahead of the real clock.

- **Update** Slice 1.0 done: Rowan checked all four done-when points (fresh clone from the README, DR-0012 layout, Rust value in the real Tauri window, Rust and TypeScript tests). First frontend tests added.

- **Decision** DR-0020 accepted (Ash, Yew; written by Rowan): Vitest and happy-dom for frontend tests, with the backend faked by `mockIPC`.

- **Decision** DR-0019 accepted (Yew, Marram; drafted by the facilitator): parallel work shares one checkout with rules. One agent changes code at a time, parallel briefs name each agent's files, shared files and new record numbers belong to the facilitator, and commits are by path, never `git add -A`. Revisit when two engineers need to build code at once, on another collision, or when Ian adds a remote.

- **Update** Roadmap revised by Marram for DR-0012 and DR-0014 to DR-0018, and the resolved Q-0006, Q-0009 and Q-0010: gates are the team's two-way doors except three one-way doors kept with Ian (replacing WFC, new game technology, anything costing Ian money or time); 1.0 is built and being checked against its done-when. No phase or deliverable changed.

- **Creation** Code scaffold per DR-0012, built by Rowan: root Cargo workspace, an empty `crates/maze-core`, and the Tauri v2 tool in `apps/tool/` (Lit, TypeScript, Vite) with one command that shows maze-core's version on screen. README updated with the layout and build, run and test commands.

- **Decision** DR-0018 accepted (Ian, reviewed by Yew and Marram): one test for what goes to Ian. If a decision can be reversed without a lot of extra work, the team decides it; only one-way doors and disputes on them go to Ian. When unclear, Yew decides for technical decisions and Marram for product. Supersedes DR-0002, now archived. `CLAUDE.md` and every agent in `.claude/agents/` updated to match.

- **Decision** DR-0014 accepted (Yew, Ian): square grid with four neighbours. Ian noted it should not have been blocked on him. Q-0009 resolved.

- **Decision** DR-0012 accepted (Yew, Rowan, Ian): the code layout. Ian approved it but said it should not have waited on him; decisions like it now go to the architect and an engineer.

- **Decision** DR-0015 accepted (Wren, Marram): the core tile set, 10 tiles and 29 (tile, orientation) pairs on five kinds of side, folding in Ian's Q-0007 answers. A two-way door, per Ian's direction.
- **Decision** DR-0017 accepted (Damson, Marram): tile glyphs and colours in the tool. A two-way door.
- **Decision** DR-0014 proposed: square grid with four neighbours and 90-degree rotations. Yew judges it a one-way door, so it goes to Ian; Ash has reviewed it.
- **Question** Q-0008 raised by Yew: which global properties a maze must have, and how they are enforced. Waiting on Wren and Yew.
- **Question** Q-0009 raised: approve DR-0014. Waiting on Ian.
- **Creation** `design/tile-set-preview.html` (outside the bundle): a self-contained preview of every glyph, hand-placed arrangements checked against the side rules, and join tests, built by Ash.

- **Question** Q-0006 resolved: Ian expects about 30 rows and columns in the short term, and millions of cells eventually.
- **Question** Q-0010 resolved: Marram decided it as a two-way door, after Ian's direction that the team makes reversible decisions. Editing goes into Phase 2 at 2.3.
- **Update** Q-0007: point 10, the weight model, moves to the team. Yew decides a calculated base plus exceptions as a two-way door, and Marram sees it as one-way. It waits on one engineer.

- **Decision** DR-0016 accepted: Wren designs the tile set from Ian's vocabulary, with crossroads and dead ends as very low probability fallbacks. It records Ian's answer to Q-0004 verbatim. Q-0004 resolved.
- **Question** Q-0005 resolved: Ian's answer on what he wants from the table goes into the roadmap, with no record of its own.
- **Update** DR-0013, DR-0016 and Q-0007: Ian's answers on "mainly" and on Q-0007 points 1 to 9 recorded verbatim. Point 10, the weight model, is still with Ian. Q-0006 notes "I'm expecting a big table".
- **Question** Q-0010 raised by Marram: where does editing probabilities go? Waiting on Ian.
- **Update** Roadmap revised by Marram for DR-0013, DR-0016 and Q-0005: slice 1.4 dropped, Phase 2 reordered around Ian's review loop.

- **Update** DR-0013 and Q-0007: added the open points Wren and Yew raised (whether every room is guaranteed a door; side pairs or tile kinds for "paths match paths, rooms other rooms"), their door-to-door reading, and Yew's note that any variation must be reproducible. All still with Ian.
- **Decision** DR-0013 accepted: the adjacency table's entries come mainly from the tiles' sides, as probabilities, with 0 where sides don't pair up. It records Ian's answer to Q-0003 verbatim. Q-0003 resolved.
- **Question** Q-0007 raised: the open details of DR-0013 (slight variation, paths meeting rooms, doors, what "mainly" covers). Waiting on Ian; the main session is putting them to him together.
- **Update** Roadmap: Q-0003 links point to DR-0013, and the "weights vs allowed/forbidden" gate becomes "the weight model" (Yew).

- **Question** Q-0003 raised: where do the adjacency table's entries come from? Waiting on Ian; blocks roadmap slice 1.1.
- **Question** Q-0004 raised: where does the tile set come from? Waiting on Ian; blocks 1.1.
- **Question** Q-0005 raised: what do you want to find out from the table? Waiting on Ian; blocks 1.1 and 1.2.
- **Question** Q-0006 raised: how big will the table get? Waiting on Ian; blocks 1.5, so not urgent.
- **Creation** Roadmap drafted by Marram (`product-manager/1`), with input from Yew, Wren and Damson. Phase 1 is sliced into 1.0 to 1.5, and Phase 2 and the game are outlined. Draft, waiting on Ian's approval.
- **Update** Ian's input recorded in the roadmap: the demo he most wants is the tool exploring the probability table, and he suggests Unicode glyphs for tile shape and colour for room vs hallway. These are suggestions for Damson and Wren, not decisions yet.

- **Decision** DR-0012 proposed: the code layout, with one Cargo workspace at the root, a UI-free `maze-core` crate in `crates/`, and self-contained apps in `apps/`, starting with the tool. Waiting on Rowan's review, then Ian.

- **Decision** DR-0001 to DR-0010 accepted: Ian approved them as written. Q-0001 resolved.
- **Decision** DR-0011 accepted: the Lit frontend uses TypeScript and Vite. Q-0002 resolved.
- **Update** DR-0005: its link to Q-0002 now points to the resolved question and to DR-0011.

- **Decision** DR-0010 proposed: the facilitator manages context, with fresh team sessions per task and reminders for Ian to clear his session. Added to Q-0001 for sign-off.

- **Update** DR-0002 (still proposed): the product manager can overrule the engineers and the UX designer on product matters, and takes minor and two-way-door product decisions alone.
- **Update** DR-0003 (still proposed): added `roadmap.md` (`type: Roadmap`) to the layout.

- **Update** DR-0002 (still proposed): the UX designer can overrule the engineers on user experience.
- **Update** DR-0002 (still proposed): the architect can overrule the engineers and the product manager; defer to Ian on architecture when in doubt.

- **Creation** Docs bundle set up (OKF v0.2) with decisions, questions and plans.
- **Decision** DR-0001 to DR-0009 proposed: the founding decisions from the setup conversation, awaiting Ian's sign-off.
- **Question** Q-0001 raised: sign off the founding decision records.
- **Question** Q-0002 raised: TypeScript and Vite for the Lit frontend?
