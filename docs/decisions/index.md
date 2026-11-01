# Decision records

One file per decision ([DR-0001](/decisions/0001-record-every-decision.md)). Only records in use are listed here; rejected, superseded and deprecated records are in `/decisions/archive/`.

## Accepted

- [DR-0001 Record every decision as a decision record](/decisions/0001-record-every-decision.md) - Every project decision gets its own decision record in docs/decisions, with a lifecycle and an archive.
- [DR-0003 Open Knowledge Format for all docs](/decisions/0003-open-knowledge-format-for-docs.md) - The docs folder is a strict OKF v0.2 bundle with a fixed layout for decisions, questions and plans.
- [DR-0004 Track open questions as files](/decisions/0004-open-questions.md) - Questions waiting on a decision live in docs/questions/open and move to docs/questions/resolved when answered.
- [DR-0005 Tauri v2 with a Rust backend and Lit frontend](/decisions/0005-tauri-rust-backend-lit-frontend.md) - Apps are built with Tauri v2, with generation logic in Rust and the UI in Lit web components.
- [DR-0006 npm for frontend packages](/decisions/0006-npm-for-frontend-packages.md) - The frontend uses npm as its package manager.
- [DR-0007 Generate mazes with Wave Function Collapse](/decisions/0007-wave-function-collapse.md) - Mazes are generated around the player with Wave Function Collapse over a tile set and an adjacency table.
- [DR-0008 Adjacency-table tool first, separate from the game](/decisions/0008-adjacency-tool-first.md) - Phase 1 builds the adjacency table and a separate tool to inspect and test it; the game comes later.
- [DR-0010 The facilitator manages context](/decisions/0010-facilitator-manages-context.md) - The facilitator's main job is managing context, giving each task fresh team sessions and telling Ian when to clear his session.
- [DR-0011 TypeScript and Vite for the Lit frontend](/decisions/0011-typescript-and-vite.md) - The Lit frontend is written in TypeScript and built with Vite.
- [DR-0013 Table entries come from tile sides, as probabilities](/decisions/0013-table-entries-from-tile-sides.md) - The adjacency table's entries are worked out mainly from the tiles' sides, with probability 0 where sides don't pair up and slightly varied probabilities for what is allowed.
- [DR-0014 Square grid with four neighbours](/decisions/0014-square-grid.md) - The maze is a square grid; each cell has four neighbours (north, east, south, west) and tiles turn in 90-degree steps.
- [DR-0016 Wren designs the tile set, with crossroads and dead ends as rare fallbacks](/decisions/0016-wren-designs-the-tile-set.md) - Wren designs the tile set from Ian's vocabulary, and crossroads and dead ends are in it at very low probability, to fix issues where nothing else fits.
- [DR-0015 The core tile set](/decisions/0015-tile-set.md) - Ten tiles in three kinds (corridor, room, hedge), giving 29 (tile, orientation) pairs, built on five kinds of side that say which tiles may sit next to each other.
- [DR-0017 Tile glyphs and colours in the tool](/decisions/0017-tile-glyphs-and-colours.md) - Each (tile, orientation) shows as one Unicode box-drawing or block character, with corridors in blue single lines, rooms in amber double-line walls on a floor tint, and hedge as texture.
- [DR-0012 Code layout, with a shared core crate and self-contained apps](/decisions/0012-code-layout-and-core-crate.md) - One Cargo workspace at the repository root, with the UI-free maze-core library in crates/ and each app, starting with the tool, self-contained in apps/.
- [DR-0018 Easily reversed decisions are the team's; only one-way doors go to Ian](/decisions/0018-one-way-doors-go-to-ian.md) - If a decision can be reversed without a lot of extra work, the team decides it; only decisions that would be costly to reverse, and disputes on them, go to Ian.
- [DR-0019 Parallel code work gets its own worktree; shared files stay with the facilitator](/decisions/0019-parallel-work-in-one-checkout.md) - Each engineer changing code in parallel works in their own git worktree on a short-lived branch, which the facilitator rebases and fast-forwards onto main after fetching origin. Docs-only parallel work stays in the main checkout, with separate files per agent and commits by path. Nothing is ever force-pushed; a pushed branch is never rewritten on origin, and is deleted from origin once it lands.
- [DR-0020 Vitest and happy-dom for frontend tests](/decisions/0020-vitest-for-frontend-tests.md) - The Lit frontend is tested with Vitest in a happy-dom environment, with the Tauri backend faked by mockIPC from @tauri-apps/api/mocks.
- [DR-0022 The weight model, a calculated base plus a few exceptions](/decisions/0022-weight-model.md) - Each table entry is a side-pair base times a weight per tile, with a few exceptions for particular pairs, all kept in the tile set data; side rules always win, and the tool shows where every number came from.
- [DR-0023 Rust and TypeScript IPC types kept in step by ts-rs](/decisions/0023-rust-typescript-types-in-step.md) - The tool's IPC types are Rust structs in the app crate that derive ts-rs; the generated TypeScript is checked in, and a Rust test fails when it is stale.
- [DR-0024 The tile set file is TOML in data/, reloaded by the tool on request](/decisions/0024-tile-set-file-format.md) - The tile set, side-pair bases, tile weights, exceptions and seeded noise live in one TOML file, parsed and validated by maze-core and reloaded by the tool; the table view command takes filters and a window and returns entries with probabilities and explanations.
- [DR-0025 How the table is laid out and how an entry explains itself](/decisions/0025-table-view-design.md) - One 29 by 29 grid per direction with a direction switcher, rows grouped by kind, each cell a shade plus a row percentage, and a side panel that explains the selected entry.
- [DR-0026 How exploring one tile works](/decisions/0026-explore-one-tile-design.md) - One tile in the middle with a list of neighbours on each side, sorted by likelihood, facing sides on the shared edge, DR-0025's explanation panel, a step-to-centre action with a trail back, and one entry point that takes any tile and orientation.
- [DR-0027 How filtering the table works](/decisions/0027-filter-table-design.md) - A filter bar with one group for the tile (rows) and one for its neighbour (columns), by kind, tile type, tag and side on the shared edge; OR within a facet, AND across; a "showing N of 29" count and one Clear all; Rust filters through DR-0024's view request, plus one `sides` field.
- [DR-0031 Endless generation with seam-first chunks](/decisions/0031-endless-generation-approach.md) - The endless maze is made of fixed-size chunks, each generated by the 2.1 WFC against border labels both neighbours derive from the world seed, with one forced gate per side; order-independent with nothing stored, a pinned spine guarantees a way onward, and a corridor-and-hedge fallback means no chunk fails.
- [DR-0032 Use the GitHub remote and keep it up to date](/decisions/0032-use-the-remote.md) - origin on GitHub is the remote; main is pushed every time anything lands, and local main is fast-forwarded before work starts. Supersedes DR-0021.

## Proposed


## Archived

- [DR-0002 Approval and escalation rules](/decisions/archive/0002-approval-and-escalation.md) - Superseded by DR-0018.
- [DR-0009 Commit straight to main](/decisions/archive/0009-commit-straight-to-main.md) - Superseded by DR-0021.
- [DR-0021 Work lands on main; local branches and worktrees are allowed](/decisions/archive/0021-work-lands-on-main.md) - Superseded by DR-0032.
