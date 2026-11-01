---
type: Decision Record
title: How the tool and the game share code
description: The game is apps/game, a Tauri v2 app with its own maze-game crate in the workspace and its own Lit frontend. It uses maze-core's World as it is, and puts what it adds to core in a new play module. Its IPC types and ts-rs bindings are its own. It copies the tool's glyph module, with a note, until G.3. It reads data/tile-sets/core.toml at each new game. It needs no technology beyond DR-0005. maze-core stays engine-agnostic, Send, seeded and simple, for a later Unity plugin. Also G.1's command contract.
status: stable
decision_id: DR-0043
decision_status: proposed
decision_kind: architecture
major: false   # two-way door: nothing here is hard to move; the layout was already set by DR-0012
owners: [architect/1]
approvers: [architect/1, engineer-rowan/1, engineer-ash/1]   # architect/1 is the author; one engineer's approval, Rowan's or Ash's, is enough (DR-0018)
supersedes: []
tags: [architecture, layout, code-sharing, game, ipc, ts-rs, tile-set, slice-g-1, engine-agnostic, unity]
generated:
  by: architect/1
  at: 2026-11-01T10:24:01Z
verified: []
---

# Context

[DR-0042](/decisions/0042-the-game-starts-now.md) starts the game now, with G.1 "Walk a growing world". [DR-0008](/decisions/0008-adjacency-tool-first.md) and [DR-0012](/decisions/0012-code-layout-and-core-crate.md) left "how code is shared with the game, in full" to the architect, and the roadmap names it as one of the game's gates. DR-0012 already fixed the outline: the game goes in `apps/game/` as crate `maze-game`, with the same shape as the tool, on top of `maze-core`. This record fills in the rest, cheaply, for G.1.

Three other tracks are working in `maze-core` at the same time: 2.3 in `wfc.rs`, `trace.rs` and `core.toml`, and 2.6 steps 2, 2b and 3. The engineers are already building G.1 against a stub contract that the facilitator set. Point 7 below confirms that contract, with small changes.

Ian (`human:ian`), 2026-11-01, verbatim, on the long term:

> The very long term intent is to use this game as a minimal tech demo of a given approach (infinite maze) - we will later make a different 3d game (probably in unity) based on it [...] hopefully have a maze generation algorithm in that can be attached to that unity game running in its own thread and working with the unity game.

Point 10 turns that into constraints on `maze-core` now. How it gets attached to Unity is [Q-0014](/questions/open/0014-maze-core-in-unity.md), not this record.

# Decision

1. **Layout.** `apps/game/` is laid out like `apps/tool/` (DR-0012):
   - `package.json`, a committed `package-lock.json`, `vite.config.ts`, `tsconfig.json`, `index.html` and `src/` (Lit, TypeScript, [DR-0011](/decisions/0011-typescript-and-vite.md); tests with Vitest and happy-dom, [DR-0020](/decisions/0020-vitest-for-frontend-tests.md));
   - `src-tauri/` holds the Tauri v2 crate `maze-game`, lib `maze_game`. It is desktop only, like the tool's: no `staticlib` or `cdylib`.
   - `apps/game/src-tauri` is added to the root workspace's `members`. There is still no root `package.json`. Run `npm` from `apps/game/`.
   - `tauri.conf.json` uses identifier `dev.mazemaker.game` and productName "Maze Maker". The dev server runs on **port 1430**, with HMR on 1431, so the game and the tool (on 1420) can run at the same time. Keep `build.devUrl` set (DR-0012).
   - The game never depends on the tool, or the tool on the game. Both depend on `maze-core`.
2. **What the game uses from `maze-core`.** It uses `World` ([DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md)) through its public API, as built:
   - `TileSet::from_toml`, `Table`, and `World::new(set, table, seed, Settings::default())`;
   - `World::grow_around(player)` after every move;
   - `World::get` and `World::tile_set()` to read cells.

   The game owns its own state, in `maze-game`: the player's position and the **visited** set, a `BTreeSet` keyed by core's `Pos`, as DR-0035 allows for game-only state. Visited moves into core only if a later record needs it there, for example the save format (G.2).
3. **Adding to `maze-core` for the game.** Game logic that belongs in the domain still goes in core. Whether the player can step from one cell to the next is domain logic, for example. Additions go like this:
   - **In a new module, `crates/maze-core/src/play.rs`**, built on the public API and on crate-visible helpers (`measures::connected`), plus one `pub mod play;` line in `lib.rs`. Inherent methods on `World` may go in an `impl World` block in `play.rs`. No other track touches the file, so it can't clash with them. G.1 needs one function there: whether a stored cell joins its stored neighbour in a direction. It uses the same rule as `random_walk` and `measures.rs`.
   - **If something can't be built that way** (it needs private fields, for example), it goes in one contiguous block at the end of the file it belongs in, marked `// Game (DR-0043)`.
   - **Never change existing public behaviour in core for the game** (signatures, results, seeds or settings) without raising it first. Raise it with Yew and with whoever is running the other `maze-core` tracks, through the facilitator. Don't refactor shared code to tidy it, either. For example, leave `random_walk` alone even though it repeats the step check.
4. **IPC types, per app.** The game follows [DR-0023](/decisions/0023-rust-typescript-types-in-step.md) on its own:
   - `apps/game/src-tauri/src/ipc.rs` holds its own types, deriving serde and `ts_rs::TS`;
   - the bindings are checked in at `apps/game/src/bindings/`, with their own staleness test (`UPDATE_BINDINGS=1` rewrites them);
   - one typed wrapper, `apps/game/src/ipc.ts`;
   - DR-0023's conventions apply: camelCase fields, snake_case enum values, no 64-bit integers.

   The game defines its own small `Direction` and `TileKind` and doesn't import the tool's. There is no shared IPC crate: the two apps send different shapes, and the duplicate is a few lines.
5. **TypeScript sharing: copy, with a note.** For G.1 the game copies `apps/tool/src/table/tile-glyphs.ts` ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md)'s presentation mapping), and whatever it needs of the tool's drawing code: `draw-tile.ts`, the east filler in `maze/maze-text.ts` ([DR-0029](/decisions/0029-generation-view-design.md) point 7), and DR-0017's CSS tokens and font stack. That set is **G.1's copy set**. Each copy goes into `apps/game/src/` and starts with a comment like this:

   ```ts
   // Copied from apps/tool/src/table/tile-glyphs.ts at <commit> (DR-0043 point 5).
   ```

   A copy imports the game's own bindings, never the tool's. A fix to a copy is made in both places in the same change, unless Damson has decided that the game draws differently (DR-0017 allows it).

   **Revisit** this with its own record, which is likely npm workspaces and a shared package, moving `package-lock.json` to the root (DR-0012), at the first of:
   - G.3 (debug mode), which brings the tool's views into the game;
   - copying anything outside G.1's copy set;
   - a copy drifting from the original when it shouldn't have.
6. **Tile set data.**
   - `start_game` reads `data/tile-sets/core.toml` ([DR-0024](/decisions/0024-tile-set-file-format.md)), with the path built from `CARGO_MANIFEST_DIR` at compile time, as the tool does. It reads the file again at every `start_game`, so a new game picks up Wren's latest tuning with no rebuild and no reload button.
   - It is not embedded with `include_str!` (except in tests), because then a tuning change would need a rebuild.
   - Bundling the file as a Tauri resource waits for the game's first release build, as DR-0024 says for the tool.
   - A tile set that fails to load, or fails the completeness check, is returned as an error to show, not a panic.
7. **G.1's command contract.** The facilitator's stub stands, with these changes, all marked "changed":
   - `start_game(seed: u32) -> Result<GameView, GameError>`. It reads the tile set, builds a new `World` and replaces any current game. The player starts at (0, 0), which is visited.
   - `step(direction: Direction) -> Result<GameView, GameError>`, with `Direction` being `"north" | "east" | "south" | "west"`.
     - If the player's cell and the next one are both stored and joined, the player moves: the new cell is marked visited, `grow_around` runs, and `moved` is `true`.
     - Otherwise nothing changes and `moved` is `false`.
     - Calling `step` before `start_game` is an error.
   - **changed:** both commands return `Result`. `GameError { message: String }` carries the load errors, one per line, or "no game started".
   - `GameView { player: Pos, moved: bool, radius: u32, cells: Vec<ViewCell> }`, with `Pos { x: i32, y: i32 }`.
     - Core's `i64` positions are converted with `i32::try_from`. Overflow is an error, which no walk can reach.
     - **changed (meaning only):** `radius` is the **view radius**, a game setting. `cells` holds every stored cell within Chebyshev distance `radius` of the player, in (y, x) order. Unsettled cells are not sent. They behave as solid (DR-0036 rule 2), and how they are drawn is [DR-0045](/decisions/0045-game-screen-for-g1.md)'s. The view radius is **6**, from Wren's [DR-0044](/decisions/0044-walking-the-growing-world.md) point 4. It must stay below the world's growth radius r (8 as built; DR-0036 rule 2 says r must be more than the player can see), and a test asserts that.
   - `ViewCell { x: i32, y: i32, tile: String, orientation: u32, kind: TileKind, visited: bool }`.
     - `tile` is the DR-0024 tile id, and `orientation` is k.
     - **changed:** `orientation` is `u32`, to match core and DR-0023. TypeScript sees `number` either way.
     - **changed:** `kind` is the enum `TileKind` (`"corridor" | "room" | "hedge"`), not a `String`.
   - **Size.** At view radius 6 the view is at most 169 cells, so sending the whole view on every step is fine. Deltas wait until a measure says otherwise.
   - **Blocking.** If a step that grows patches takes long enough to stall the window, the engineers may declare the commands `async`, so they don't run on the main thread. That is their call, with no record needed.
   - **Additive changes** (a new field, or a `seed` echoed back) need no new record. A rename or a removed field does.
8. **Technology.** G.1 needs nothing beyond Tauri v2, Rust and Lit ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)). The view is at most a few hundred glyph cells, redrawn on a key press. Lit with the tool's text or grid rendering, or a plain `<canvas>` 2D context, does it with no new dependency. Both are browser APIs, not libraries, and the engineers choose between them. Keyboard input is DOM events. **Adding a game, rendering or canvas library is a change to the stack and goes to Ian first.** Smooth scrolling, sprites or animation at scale are the likely triggers.
9. **Lockfiles.** Adding the workspace member changes the root `Cargo.toml` and `Cargo.lock`, and `apps/game/package-lock.json` is new. Under [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) rule 5, the facilitator names the one G.1 branch that holds them. Its versions of `tauri`, `ts-rs`, `serde`, `lit`, `vite` and so on match the tool's, so the Rust lockfile only gains the `maze-game` entry.
10. **`maze-core` stays fit to be attached to another engine** (Ian, above). These constraints hold for every track, the game's included:
    - **Engine-agnostic.** No Tauri, UI, IPC or ts-rs types in core, as DR-0012 already says. The game's types stay in `maze-game`.
    - **Usable from its own thread.** `World` is `Send`, and core keeps no global or thread-local state: no `static mut`, `thread_local!` or lazy globals. Checked on 2026-11-01: `World` is `Send` and `Sync` today, with no globals. `play.rs` holds a compile-time assertion that `World: Send`, so a change that breaks it fails the build.
    - **Deterministic and seeded.** It already is (DR-0012, DR-0028 rule 5, DR-0036 rules 8 and 10): no clock, no OS randomness, no `HashMap` order.
    - **A simple API.** World's public surface stays plain data in and out: integers, ids as strings, slices and owned values. No callbacks it keeps hold of, no lifetimes leaking out of long-lived handles, and no async. Then a later C-ABI wrapper crate (a Unity native plugin) can sit on top without changing core. Core panics on bad input in places (`grow_at`, settings), so a wrapper would have to catch them at the boundary. That's for Q-0014.

    **A two-way door.** Nothing is built for Unity now. These are constraints that code already meets, and the wrapper would be a new crate on top of core. What could become a one-way door is the choice of how Unity calls the generator: a native plugin on Rust, or a port to C#. That waits for Q-0014, before the 3D game starts.

# Alternatives considered

- **A shared frontend package now (npm workspaces).** It is the right end state if the game takes on the tool's views. For a few hundred lines of copied code, though, it would mean a root lockfile, hoisting and a change to both apps while three tracks are in flight. Point 5 says when to revisit it.
- **Importing across apps** (`../../tool/src/...`). It costs nothing today, but it breaks DR-0012's "apps never depend on each other", and a refactor in the tool would silently break the game.
- **A shared IPC crate** (`crates/maze-ipc`). It would hold a few lines in common, and it would put ts-rs and IPC concerns into a crate both apps build. Revisit at G.3, with the shared frontend.
- **Game additions inside `world.rs`.** That is the natural home, but it is the file most likely to clash with 2.6's tracks right now. `play.rs` can move into `world.rs` later in one cut.
- **Embedding `core.toml` in the game binary.** It is simpler to ship, but every tuning change would need a rebuild.

# Consequences

- The game gets a growing, stored world without touching existing core code. Its only core addition is one new file.
- Two copies of the glyph mapping exist until G.3 or another trigger in point 5. Damson's call that the game draws differently would end the duty to keep them in step.
- Moving `play.rs` into `world.rs`, swapping copies for a package or switching to a shared IPC crate is each a mechanical change. That is why this is a two-way door.
- `cargo test` at the root now builds and tests `maze-game` too.
- `README.md`'s "Where things are" gains `apps/game/`. Only the branch the facilitator names holds `README.md` (DR-0019).

# Concerns raised

- **Yew (`architect/1`), the author:**
  1. Copying is a debt with a date on it: G.3. If G.3 slips, the second trigger (a third copied module) still applies.
  2. The game's view radius and the world's growth radius are two numbers that must stay in order, and they live in different crates. That's why point 7 asks for a test.
  3. Wren's DR-0044 concern stands: a patch window can reach sideways into the view. If her checklist shows it often, the fix is in DR-0036's window (the 2.6 track), not in the game's radius, and that change goes through point 3's "raise it first".

# AI guidance

- Don't import from `apps/tool/` in the game, or from `apps/game/` in the tool.
- Put the game's additions to `maze-core` in `play.rs`. Don't change existing public behaviour in core for the game without raising it first.
- Keep game-only state (player, visited) in `maze-game`, keyed by core's `Pos`.
- When you change a copied TypeScript file, change its original too, unless Damson has said the game draws differently. Keep the "Copied from" note.
- Never edit `apps/game/src/bindings/` by hand. Change `ipc.rs` and regenerate.
- No new frontend or game library without Ian's approval (point 8).
- Keep `World` `Send`, with no globals, and keep core free of engine and UI types (point 10). Don't remove the `Send` assertion to make a build pass. Raise it with Yew.

# Links

- Stack: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md); tool first: [DR-0008](/decisions/0008-adjacency-tool-first.md); the game starts: [DR-0042](/decisions/0042-the-game-starts-now.md)
- Layout: [DR-0012](/decisions/0012-code-layout-and-core-crate.md); parallel work and lockfiles: [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)
- G.1's game design and screen: [DR-0044](/decisions/0044-walking-the-growing-world.md), [DR-0045](/decisions/0045-game-screen-for-g1.md)
- Glyphs: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md); IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md); tile set file: [DR-0024](/decisions/0024-tile-set-file-format.md)
- World: [DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md)
- Roadmap: [roadmap](/roadmap.md), "The game (outline)", G.1
- Attaching `maze-core` to a Unity game: [Q-0014](/questions/open/0014-maze-core-in-unity.md)
