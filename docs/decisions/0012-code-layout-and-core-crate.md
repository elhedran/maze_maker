---
type: Decision Record
title: Code layout, with a shared core crate and self-contained apps
description: One Cargo workspace at the repository root, with the UI-free maze-core library in crates/ and each app, starting with the tool, self-contained in apps/.
status: stable
decision_id: DR-0012
decision_status: accepted
decision_kind: architecture
major: true
owners: [architect/1]
approvers: [human:ian, engineer-rowan/1]
decided_at: 2026-10-31T18:58:00+01:00
supersedes: []
tags: [architecture, layout, rust, cargo, tauri, npm, boundaries]
generated:
  by: architect/1
  at: 2026-10-31T14:58:00+01:00
verified:
  - by: engineer-rowan/1
    at: 2026-10-31T15:09:21+01:00
  - by: human:ian
    at: 2026-10-31T18:58:00+01:00
---

# Context

[DR-0008](/decisions/0008-adjacency-tool-first.md) makes the tool and the game separate apps, and leaves the architect to propose how they share code. The tool's skeleton (roadmap slice 1.0) is next. The layout and the boundary between shared logic and app code should be set before there is code to move, because both get expensive to change once features are built on them. That makes this major under [DR-0002](/decisions/archive/0002-approval-and-escalation.md): it sets structure and boundaries between parts.

# Decision

**Layout:**

```
maze_maker/
  Cargo.toml            workspace root (virtual manifest, no package)
  Cargo.lock            committed
  target/               all Rust build output (ignored)
  crates/
    maze-core/          library: tiles, adjacency table, table checks, later WFC
  apps/
    tool/               the tool app, self-contained
      package.json
      package-lock.json committed
      vite.config.ts
      tsconfig.json
      index.html
      src/              Lit frontend (TypeScript)
      dist/             Vite output (ignored)
      src-tauri/        Tauri v2 app crate maze-tool, a workspace member
    game/               reserved, not created
  data/                 tile set and table data shared by the apps, created with the first data file
  docs/
```

**Cargo workspace.** One workspace, rooted at the repository root with a virtual manifest. Its members are `crates/maze-core` and `apps/tool/src-tauri`. The root `Cargo.toml` sets `resolver = "3"` and holds every `[profile.*]` section. Run `cargo` from the root.

**One core crate, `maze-core`** (`maze_core` in code). It holds the domain: tiles, the adjacency table, checks on the table, and later WFC generation. One crate, not several, because we don't know its internal seams yet. Split it only when a part needs different dependencies or different users, and record the split.

**Core boundary rules:**

- It is a plain Rust library. It never depends on Tauri (`tauri` or any `tauri-*` crate), a UI or webview crate, an async runtime, or anything in `apps/`.
- It does no I/O. It doesn't open files or sockets, read the environment, or read the clock or OS randomness. Callers hand it data (strings, bytes, values) and get values back. Parsing data it has been given is fine.
- Randomness, if needed, is passed in by the caller. This keeps [DR-0007](/decisions/0007-wave-function-collapse.md)'s determinism question open rather than answering it.
- Core never uses a dependency to do I/O, and turns off default features that bring in OS access (for example `getrandom`). Significant dependencies need a record ([DR-0002](/decisions/archive/0002-approval-and-escalation.md)).
- Dependencies point one way: apps depend on crates, never the reverse, and apps never depend on each other.

**Apps.** Each app lives in `apps/<name>/` and is self-contained, with its own `package.json`, `package-lock.json`, Vite config, `src/` and `src-tauri/`. This is the standard Tauri v2 layout moved down a level, so the template's relative paths still work. The app crate holds everything app-specific: Tauri commands, the types that cross IPC to the frontend, file loading, and paging or streaming large data ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)). The Tauri CLI is a dev dependency of the app, run with npm ([DR-0006](/decisions/0006-npm-for-frontend-packages.md)).

**No root `package.json` and no npm workspaces.** There is one frontend. Run `npm` from inside the app's folder.

**The game** will go in `apps/game/`, with the same shape as the tool, as crate `maze-game`. The folder isn't created until a record starts the game.

**Lockfiles and build output.** `Cargo.lock` at the root and `apps/tool/package-lock.json` are committed. Rust output, including Tauri's installers, goes to `target/` at the root. The frontend builds to `apps/tool/dist/`, and npm installs to `apps/tool/node_modules/`. The root `.gitignore` already ignores all three. `apps/*/src-tauri/gen/schemas/` is ignored too, because tauri-build regenerates it on every build. The rest of `gen/` is not ignored, because that is where mobile projects would go.

**Data.** Tile set and table data used by the apps lives in `data/` at the root, not inside an app and not inside `maze-core`. Both apps use it, and core has to work with any tile set. Test fixtures, such as tiny or deliberately broken tile sets, live beside the tests that use them. What goes in `data/`, and in what format, is left to the tile set and data format records.

**Deferred**, each to its own record when it is needed:

- Sharing Lit components between the tool and the game. Until then, components live in the app that uses them. Sharing would probably bring npm workspaces and move `package-lock.json` to the root.
- How the Rust and TypeScript types on the IPC boundary are kept in step (by hand or generated), and where any generated bindings live.
- How an app finds its data at run time: read from `data/`, embedded at build time, or bundled as Tauri resources. This belongs with the data format. Note for that record: `tauri dev` doesn't watch `data/`.

This record doesn't settle any of DR-0007's follow-up questions, and doesn't design any API.

# Alternatives considered

- **Everything in the tool's `src-tauri`, extracted later.** Least work today. Extraction gets harder with every feature, and in the meantime Tauri types leak into the domain logic.
- **Several core crates now** (tiles, table, WFC). The boundaries would be guesses, and each crate costs a manifest and a public API. Split when there is a reason.
- **Flat layout** (`core/`, `tool/`, `game/` at the root). Fine at this size. `crates/` and `apps/` make the dependency direction visible and give the game an obvious slot. A mild preference, not a strong one.
- **The workspace rooted in `apps/tool/src-tauri`**, as Tauri's template does on its own. That puts the game's shared code under the tool.
- **npm workspaces now.** There is nothing to share yet, so it would add a root lockfile and hoisting for no gain.
- **Core loads its own files.** Simpler at first, but it ties core to a folder layout, makes its tests need files on disk, and the game may well load data differently from the tool.
- **Data inside `maze-core`, embedded.** Ties content to code, when core also has to work with test sets.
- **Separate repositories** for core, tool and game. Far too heavy for one owner with no remote ([DR-0009](/decisions/0009-commit-straight-to-main.md)).

# Consequences

- Core can be built and tested with `cargo test -p maze-core`, with no webview and no frontend build.
- Each tool feature has two halves: domain logic in core, and IPC and paging in the tool's `src-tauri`. A little more ceremony, and the reason the game can reuse the first half.
- One `target/` serves both apps, so core is built once.
- Commands run from two places: `cargo` at the root, `npm` in `apps/tool/`. The README has to say so.
- Starting the game adds `apps/game/` and one workspace member. Nothing moves.

# Concerns raised

None. Ian approved it but said it should not have waited on him: layout decisions like this are for the architect and an engineer.

# AI guidance

- Domain logic (tiles, the adjacency table, table checks, WFC) goes in `crates/maze-core`. Tauri commands, IPC types and file loading go in the app's `src-tauri`.
- Never add `tauri`, a `tauri-*` crate, a UI crate, an async runtime or an app crate to `maze-core`'s dependencies.
- Don't do I/O in `maze-core`, or read the clock, the environment or OS randomness there. Take data and randomness as inputs.
- Run `cargo` from the repository root and `npm` from `apps/tool/`. Don't add a root `package.json`.
- Put `[profile.*]` settings in the root `Cargo.toml` only. Cargo ignores them in members.
- Don't set `default-members`. `cargo test` at the root must test every member.
- Keep `build.devUrl` set in the tool's `tauri.conf.json`. Without it, `cargo build` and `cargo test` fail until `dist/` exists.
- Don't create `apps/game/`, a new crate or a shared frontend package without a record.
- Shared tile and table data goes in `data/`. Test fixtures go beside their tests.
- When the scaffold lands, update "Where things are" in `README.md` to match this layout.

# Links

- Stack: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)
- Package manager: [DR-0006](/decisions/0006-npm-for-frontend-packages.md)
- Generation approach and its open follow-ups: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Tool and game as separate apps: [DR-0008](/decisions/0008-adjacency-tool-first.md)
- Frontend language and bundler: [DR-0011](/decisions/0011-typescript-and-vite.md)
- Roadmap, slice 1.0: [roadmap](/roadmap.md)
