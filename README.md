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
