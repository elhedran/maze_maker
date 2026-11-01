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

## Demo: slice 2.2, why is this here?

From `apps/tool/`, run `npm run tauri dev`. There are three tabs now: **Table**, **Explore** and **Maze**.

1. **A bit of maze you don't like, and why it's there.** In the **Maze** tab, press **Generate** and click a cell, for example a wall along a long one-deep room. It gets a magenta outline, and the line under the maze names it next to **Why is this here?**. Press the button, or double-click the cell. The **Explore** tab opens on that tile, with a band above it that replays how the cell was drawn ([DR-0033](docs/decisions/0033-why-is-this-here-design.md)):
   - the heading gives the seed, the cell and when it was drawn ("drawn 41st of 384");
   - the cross shows the four neighbours. **Counted** ones were settled before the draw and are all the score used. Ones that **came later** were drawn to fit this cell, so they don't count. **Edge** means past the maze's edge;
   - the table lists every tile still possible at the draw: its weight, times one factor per counted neighbour, equals its score, and its chance is that score's share of the sum. The tile drawn is in bold and marked "drawn".

   Click a factor. The panel on the right explains that pair the way the table does, with its base, weights, exceptions and noise, so you can see which number in `data/tile-sets/core.toml` to change. **< Back to maze** returns to the same maze with the cell still selected.
2. **A corridor corner's most likely east neighbour.** In **Explore**, pick `└ Corridor corner, open N and E` in **Centre**. Each side of the tile lists what can go there, highest share first, with the side kinds that meet shown on the shared edge. The top row of **EAST** is the answer. Click it for the panel.
3. **From a corridor into a room and back out, through doorways.** Pick `─ Corridor straight, E–W`. In **EAST**, press **Go** on `╢ Room doorway, door W`: the passage meets the door. Then go east to `· Open room`, then to `╟ Room doorway, door E`, and back out to `─`. After each step the tile you came from is selected and marked "from", so the panel shows the edge you just crossed. **< Back** (or Alt+Left) goes back one step. Click any glyph in the trail to jump back to it.

"+ N can't go here" at the foot of each side opens the 0s. **Explore this tile** in the table's panel opens the same view on the selected pair. After **Reload tile set**, the band replays the same cell of the regenerated maze (from 2.3).

## Demo: slice 2.3, tune and compare

From `apps/tool/`, run `npm run tauri dev`. Numbers that live in the tile set file are now underlined with dots: click one to change it ([DR-0037](docs/decisions/0037-compare-and-edit-design.md), [DR-0038](docs/decisions/0038-editing-core-toml.md)). A save writes the new value into `data/tile-sets/core.toml`, keeping its comments, and appends what changed, from what to what, to `data/tile-sets/core.edits.toml` beside it.

1. **Change the number behind a bit of maze, and compare.** In the **Maze** tab, press **Generate**, click a cell you don't like and press **Why is this here?**. In the band, click a factor: the panel on the right shows where it comes from. Click one of its underlined numbers, for example a tile weight. The editor opens under it with the exact value and a line saying everywhere that number reaches. Type a new value and press Enter (or **Save to core.toml**). The band refreshes where you are, with "after reload" in its heading, and the panel shows "Saved 20:31 · … 3.0 → 2.0". Press **See before and after**. The **Maze** tab shows the same seed before (left) and after (right). Cells that didn't change are faded, so what changed stands out; untick **Fade unchanged cells** to see both in full. Under them are the count of changed cells and both measures, before → after. Hover a cell in either maze to name the tile on both sides. **Close compare** goes back to one maze.
2. **Opening pressure.** In the **Maze** tab, under the seed, click the number after **Opening pressure** (it reads "1.0 (off)" at first). Type `0.5` and press Enter. The same seed shows before and after, and the **Reachable from centre** and **Separate areas** rows show how they moved. Click a cell and press **Why is this here?**: the band has a **× pressure** column, with 0.5 where the pressure applied (the tile added no new open side) and a grey 1 where it didn't.
3. **Find your edits in the file, and set one back.** Open the tile set in an editor, for example in PowerShell from the repository root:

   ```powershell
   notepad data\tile-sets\core.toml
   notepad data\tile-sets\core.edits.toml
   ```

   `core.toml` has your new values in place, with its comments kept. `core.edits.toml` beside it lists every number saved in the tool, oldest first, with `from` and `to`. To set one back, click the same number in the tool and type its `from` value (or edit `core.toml` and press **Reload tile set**). The compare's after maze is the original maze again.

A value the loader won't take, such as a negative weight, isn't saved: the editor stays open and says why in red, and `core.toml` is left unchanged. If `core.toml` was edited by hand since the last load, the save is refused too: press **Reload tile set** first.
