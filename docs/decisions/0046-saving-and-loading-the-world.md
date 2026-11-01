---
type: Decision Record
title: How the game saves and loads the world and the player
description: A save is one JSON file in the game's local app data folder. It holds the stored cells and their palette as the truth, plus the seed, settings and anchor log for replay; the player and the visited set sit beside them. The world snapshot is a plain-data type in maze-core; the file, the envelope and the player state are the game's. A save from another format version, or one the current tile set can't honour, is refused, never repaired. Also the re-check of DR-0036 rule 1. Two-way today; the promise to keep old saves loading is the one-way door, and it waits for the first release.
status: stable
decision_id: DR-0046
decision_status: accepted
decision_kind: architecture
major: false   # two-way door today: the only saves are Ian's dev saves, and reversing this throws away those and the snapshot code
owners: [architect/1]
approvers: [engineer-rowan/1, engineer-ash/1]   # architect/1 is the author; one engineer's approval, Rowan's or Ash's, is enough (DR-0018)
supersedes: []
tags: [architecture, persistence, save-format, versioning, game, slice-g-2, frontier, engine-agnostic]
generated:
  by: architect/1
  at: 2026-11-01T10:39:08Z
verified:
  - by: engineer-ash/1
    at: 2026-11-01T10:43:03Z
    note: "Approves. Buildable against world.rs at cac4b35: every field restore needs (palette, palette_variant, Cells pages/count/bounds, log, settings, seed) is reachable from an end-of-file block, and EMPTY is the sentinel already. Four build points in Concerns raised; the first (what log() holds after a restore) must be settled when it's built."
---

# Context

G.2 "Save and load": the world and the player survive closing the game ([roadmap](/roadmap.md)). [DR-0035](/decisions/0035-the-world-is-stored.md) stores the world and leaves the save format to the game. It names the one-way part: the save format, the palette's tile ids and [DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 1, because new code must grow from an old save's frontier. The roadmap asks Yew to re-check rule 1 before the game first saves a world. [DR-0043](/decisions/0043-code-shared-by-tool-and-game.md) point 10 keeps `maze-core` engine-agnostic, `Send`, seeded and plain-data, for a later Unity game ([Q-0014](/questions/open/0014-maze-core-in-unity.md), not answered here).

What `World` holds today (`crates/maze-core/src/world.rs`): a palette of (tile id, orientation), 32 × 32 pages of palette indices, the seed, `Settings`, and a growth log of `Patch` entries. The frontier is not kept: `anchor_at` derives it from the cells. A patch's seed mixes (world seed, anchor, attempt) only (DR-0036 rule 8), so growth after a load is the same as growth without one, given the same tile set and code.

# Decision

1. **What is saved: the cells are the truth, the recipe rides along.**
   - **Truth:** the palette, in its order, and the pages. Loading never regenerates anything.
   - **For replay and bug reports:** the world seed, the settings, the anchor log (each patch's anchor cell and direction, oldest first; not the `Patch` bodies, which replay rebuilds), the tile set fingerprint and a `GENERATOR_VERSION` constant in core, bumped by whoever changes what a patch produces for the same inputs. Replay works only when both match; otherwise they are information.
   - **Not saved:** the frontier. It is a function of the cells, so it can't drift out of step with them. (This drops it from DR-0035's proposal.)
   - **The player:** position; the **visited** set, as a list of positions in key order. **Loot later:** the game's part of the file is a struct whose new fields carry `#[serde(default)]`, so loot, looted and anything else is an additive change (point 4).
2. **The format: JSON through serde and `serde_json`.** Both are already in the workspace (the tool uses them, and Tauri needs them), so this is no change to the stack. JSON is readable in a bug report, diffable in a test, and readable from C# later. Positions are `i64` in the file; the file never crosses IPC, so DR-0023's no-64-bit rule doesn't apply. A page is written as its key and one array of palette indices, with a sentinel for unsettled. If a save grows past a few megabytes, the engineers may switch the page encoding (runs, or base64 of the `u16`s) under a format version bump, with no new record. A binary format such as `bincode` or `postcard` waits for a measure that says JSON is too slow or too big.
3. **Where the format lives.**
   - **`maze-core`:** a plain-data `WorldSnapshot` (palette, pages, seed, settings, anchor log, fingerprint, generator version) deriving serde, with `World::snapshot()` and `World::restore(set, table, snapshot) -> Result<World, Vec<LoadError>>`. No file I/O and no `serde_json` in core (DR-0012). `restore` needs private fields, so it goes in one block at the end of `world.rs`, marked `// Game (DR-0043)` (DR-0043 point 3). Deriving serde on `Settings`, `Stage`, `Anchor` and `Direction` is additive. The world is what a later Unity game or the tool's world view (G.3) would share, so it belongs in core.
   - **`maze-game`:** the file envelope, `{ "format": "maze-maker-save", "version": 1, "world": WorldSnapshot, "game": { player, visited, ... } }`, the reading and writing, and the paths. The player and visited stay game state (DR-0043 point 2); a 3D game would have its own.
4. **Versioning.**
   - The envelope's `version` is one integer. An additive change (a new field with a default) keeps it. A rename, a removal or a change of meaning bumps it.
   - **Until the first release, there are no migrations.** A save with another version is refused with a message ("this save is from another version of the game; start a new world"), and the file is left untouched.
5. **When the tile set or the generator changes under an old save.** Load runs these checks against the current tile set and table, and refuses the save if any fails, naming the tile or cell. It never changes a stored cell to make one pass (DR-0036 rule 5).
   - every palette entry's tile id exists in the current set, with that orientation;
   - **rule 1 holds:** every stored cell shows hedge, passage or door to every unsettled neighbour;
   - every pair of neighbouring stored cells has an entry weight above 0;
   - there is an outward endpoint (DR-0036 rule 6);
   - the player stands on a stored cell.

   They are O(stored cells), once, at load. What follows from them: weights, noise, exceptions, settings and generator changes all leave old saves loadable, and new growth uses the current rules (DR-0035). Saves break only if a stored tile is renamed or removed, or its sides change where it matters. A restored world uses the **current** settings, as a new world would; the saved settings are only for replay.
6. **Where the save lives.** Tauri's `app_local_data_dir()`, from `tauri::Manager::path()` with no plugin: on Windows, `%LOCALAPPDATA%\dev.mazemaker.game\saves\`. Local, not Roaming, because saves grow with the world. Writes go to a temporary file in the same folder, then a rename over the old save, so a crash mid-write never leaves half a file. How many slots, and when the game saves (on close, every so many steps, on a key), are Marram's, Wren's and Damson's calls; this record allows any of them.
7. **The re-check of DR-0036 rule 1.** It stands, unchanged, and it is what old saves rely on. Point 5 enforces it at load, at run time, not only in tests. One correction to DR-0036's "loosening rule 1 later needs a migration": old saves meet any looser rule, so loosening keeps them loadable. What it costs is redoing rule 7's fallback proof for floor at the frontier, and saves made after it won't load in older code. **Tightening** rule 1 is what would break old saves. Either is DR-0036's to decide, through a superseding record.

## The one-way door, which this record doesn't open

Nothing here is a one-way door today. The only saves are Ian's dev saves, and point 4 refuses rather than migrates. The door is **promising that old saves keep loading**: once someone other than Ian has saves worth keeping, every format, tile id and rule-1 change carries a migration from then on. That decision goes to Ian before the first release. Its options, for then:

- **(a) Saves are disposable between versions** (point 4 as is). Cheapest; players lose worlds on updates.
- **(b) Forward migrations,** one per version bump, kept forever, with tile ids never reused. Players keep worlds; every format change costs a migration and its test.
- **(c) Migrate cells, fall back to replay** when a tile is gone: replay the anchor log under the new set. Replayed worlds differ from what the player saw, which DR-0036 rule 5 forbids for seen cells, so (c) only suits worlds nobody looked at.

# Alternatives considered

- **Seed, settings and growth log only, replayed at load.** Smallest file, but any change to weights, the tile set or the generator gives a different world, or a panic in `replay` when an anchor stops being an anchor. That is DR-0031's one-way door, which DR-0035 removed.
- **Cells only, no log.** It works, but bug reports lose the replay that DR-0035 asked to build from the start. The anchor log is a few bytes a patch.
- **Saving the frontier.** It is derived, so storing it adds a way to disagree with the cells.
- **A binary format now** (`bincode`, `postcard`). Smaller and faster, but opaque in a bug report, and no measure asks for it.
- **The whole save in the game.** It would keep `world.rs` untouched, but `restore` needs core's private fields either way, and a Unity game couldn't share the world format.
- **Roaming app data, or Documents.** Roaming syncs megabytes for no reason; Documents is the player's space.

# Consequences

- `maze-core` gains `WorldSnapshot`, `snapshot`, `restore`, `GENERATOR_VERSION`, a tile set fingerprint, and serde derives on four types. Its public behaviour doesn't change.
- `world.rs` gets an end-of-file block while 2.6's tracks may also be in it. The facilitator names the branch that holds it (DR-0019).
- G.2's technical done, for Marram to slice:
  - save then load then walk on gives the same world, cells and position as walking on without the save, over many seeds;
  - a save survives a weights-only change to `core.toml`, and is refused, untouched and with a message, when a stored tile id is removed or a frontier side becomes floor;
  - a save from another `version` is refused the same way;
  - a killed write leaves the previous save loadable;
  - the save size and load time for a 1,000-step walk are written down.

# Concerns raised

- **Yew (`architect/1`), the author:**
  1. `GENERATOR_VERSION` is only as good as the habit of bumping it. A test that replays a fixed seed and walk and compares a hash of the cells catches a forgotten bump; the engineers should add one.
  2. Refusing old saves is right now, and wrong the day someone else plays. The one-way door above needs putting to Ian before the first release, not after.

- **Ash (`engineer-ash/1`), on review:** buildable. Four points for whoever builds it:
  1. **What `log()` holds after a restore.** The snapshot keeps anchors, not `Patch` bodies, so a restored world has no `window` or `stored` for its old patches. Today `world_measures::rooms` indexes `patch_of[&start]` and would panic on a restored world, and `apps/tool/src-tauri/src/world.rs` indexes `log()[index]`. Pick one: keep the anchor log in its own field with `log()` holding only patches grown since the load, or rebuild the bodies by replay when the fingerprint and generator version match. Say which in the code and test it.
  2. **Settings at restore.** Point 5 says a restored world uses the current settings, so `restore` should take them, `restore(set, table, settings, snapshot)`, the same as `World::new`, and not assume the defaults.
  3. **Don't trust the file's structure.** Before point 5's checks, validate the snapshot: every page has 32 × 32 entries, every index is `EMPTY` or within the palette, and there are no duplicate palette entries. Recompute `count` and `bounds` from the pages rather than reading them. A malformed file is refused like any other bad save, never a panic. `PAGE` becomes part of the format, so a change to it bumps `version`.
  4. **What the fingerprint covers.** Hash the parsed tile set (tiles, sides, weights, bases, exceptions, noise, opening pressure) with the existing `hash`, not the TOML text, so a comment or reordering in `core.toml` doesn't stop replay. Yew's concern 1, a fixed-seed cell-hash test, is worth adding in the same change.

# AI guidance

- Load never changes a stored cell. If a check in point 5 fails, refuse the save.
- Don't save the frontier; derive it.
- Keep file I/O and `serde_json` out of `maze-core`.
- Bump `GENERATOR_VERSION` when a patch's output for the same world, anchor and tile set changes. Bump the envelope `version` for any change that isn't additive.
- Write saves to a temporary file and rename. Never write over a save in place.
- Add loot and other game state as new fields with `#[serde(default)]`.

# Links

- The world is stored: [DR-0035](/decisions/0035-the-world-is-stored.md); growth and rule 1: [DR-0036](/decisions/0036-growth-from-open-endpoints.md); replaced recipe-only approach: [DR-0031](/decisions/archive/0031-endless-generation-approach.md)
- Code sharing and core constraints: [DR-0043](/decisions/0043-code-shared-by-tool-and-game.md); layout: [DR-0012](/decisions/0012-code-layout-and-core-crate.md); stack: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)
- Determinism and seeds: [DR-0028](/decisions/0028-wfc-approach-for-2-1.md); IPC types: [DR-0023](/decisions/0023-rust-typescript-types-in-step.md); tile set file: [DR-0024](/decisions/0024-tile-set-file-format.md)
- One-way doors: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md); parallel work: [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)
- Unity: [Q-0014](/questions/open/0014-maze-core-in-unity.md); roadmap: [roadmap](/roadmap.md), G.2
