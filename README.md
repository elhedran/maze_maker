# Maze Maker

A game with an endless maze, generated around the player as they move using Wave Function Collapse. Built with [Tauri v2](https://v2.tauri.app/): Rust backend, Lit frontend.

The first phase is tooling: an app to view and filter the tile adjacency table, then to generate sample mazes from it.

## Where things are

- `Cargo.toml`: the Cargo workspace root. Run `cargo` from the repository root.
- `crates/maze-core/`: the shared domain library (tiles, the adjacency table, later WFC). Plain Rust, no I/O, no Tauri.
- `apps/tool/`: the adjacency tool, a Tauri v2 app. Run `npm` from this folder.
  - `src/`: the Lit frontend, in TypeScript, built with Vite.
  - `src-tauri/`: the app crate `maze-tool`: Tauri commands and IPC.
- `docs/`: project documentation in [Open Knowledge Format](https://github.com/GoogleCloudPlatform/open-knowledge-format).
  - `docs/decisions/`: one decision record per decision; retired ones are in `docs/decisions/archive/`.
  - `docs/questions/`: open questions, and what was decided.
  - `docs/plans/`: plans for major work.
- `CLAUDE.md`: how the AI team works on this repository.
- `.claude/agents/`: the team's agent definitions.

The layout is set by [DR-0012](docs/decisions/0012-code-layout-and-core-crate.md).

## Prerequisites

- Rust stable (MSVC on Windows), installed with rustup.
- Node 24 and npm.
- Tauri's platform prerequisites: on Windows, VS Build Tools with C++, the Windows SDK and WebView2. See [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/).

## Build, run and test

Rust, from the repository root:

```sh
cargo build --workspace      # build everything
cargo test --workspace       # test every crate
cargo test -p maze-core      # just the core, fast
```

The tool, from `apps/tool/`:

```sh
npm install                  # once, and after package.json changes
npm run tauri dev            # run the tool; rebuilds on Rust and frontend edits
npm run build                # type-check and build the frontend into dist/
npm test                     # run the frontend tests once (Vitest)
npm run test:watch           # rerun them on every edit
npm run tauri build          # release build and installers, in target/release/bundle/
```

`npm run dev` serves only the frontend at http://localhost:1420. Tauri commands don't work there, so the page shows an error instead of the core version.

Rust tests sit next to the code in `#[cfg(test)]` modules. Frontend tests are `*.test.ts` files next to the component. They run in [happy-dom](https://github.com/capricorn86/happy-dom), with the Tauri backend faked by `mockIPC` from `@tauri-apps/api/mocks`, so they need no running app.

## Demo: slice 1.1, first look at the table

From `apps/tool/`, run `npm run tauri dev`. The window shows the adjacency table for the core tile set: one grid per direction, rows are a tile and columns its neighbour in that direction, each cell the neighbour's share of the row as a percentage. A blank cell is a 0.

1. **A corridor corner east of a straight corridor.** Pick **East**. Find the row for the east-west straight corridor `─` and the column for the corner that opens west, `┐`. Click the cell. The panel on the right gives the probability and why: which sides face each other, the base for that pair, each tile's weight, any exceptions, the noise factor, and the row total it is divided by.
2. **A 0, and which sides clash.** In the same row, click a blank cell under a room wall in the Room columns. The panel says "Can't happen" and names the two sides that clash, for example a passage meeting a room's floor with its wall at the end.

**Changing a probability needs no code change.** The numbers live in `data/tile-sets/core.toml` ([DR-0024](docs/decisions/0024-tile-set-file-format.md)). Edit a base, a tile weight, an exception or the noise, save, and press **Reload tile set**. If the file is wrong, a red banner lists every error and the last good table stays on screen.


## Demo: slice 2.1, first sample maze

From `apps/tool/`, run `npm run tauri dev` and pick the **Maze** tab. The tile set bar and **Reload tile set** sit above both tabs, so they work from either.

1. **A maze from a seed.** The seed field opens with a random seed. Press **Generate** (or Enter in the field). The maze is drawn with the table's glyphs, two characters per cell; blue single lines are corridors, brown double lines and the tinted floor are rooms, green shading is hedge. The line above it says which seed it is, its size, and whether generation had to restart. Hover a cell to read its name and position under the maze.
2. **Same seed, same maze.** Press **Generate** again: the maze doesn't change. Type a different seed, or press **Random seed**, and it does.
3. **Can you get around in it?** Beside the maze, **Reachable from centre** says how much of the walkable maze you could walk to from the outlined centre cell, and **Separate areas** how many pieces it falls into ([DR-0030](docs/decisions/0030-measures-for-2-1.md)). The maze's edge cuts paths that would carry on in the endless maze, so read these for comparing seeds, not as a pass mark.
4. **Change the data, see the maze change.** Edit a weight in `data/tile-sets/core.toml`, save, and press **Reload tile set**. The maze regenerates from the same seed with the new numbers, and the status line says "regenerated after reload". If the file is wrong, the red banner lists the errors and the last good maze stays.

If generation gives up after its restarts, the maze area says so in red and suggests another seed or a look at the table for tiles that can't meet.
