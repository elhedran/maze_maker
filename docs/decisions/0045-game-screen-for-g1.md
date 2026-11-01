---
type: Decision Record
title: How the game screen looks and takes keys in G.1
description: One world view centred on the player, drawn with DR-0017's glyphs two characters per cell at 28px; a status line with seed, position in words and step count; the player as a bold @; a blocked move as a small nudge and "Can't go north."; visited cells as a worn-path wash; not-yet-grown space as a flat "unknown" fill, distinct from hedge; keys taken on the window from launch, with no click first.
status: stable
decision_id: DR-0045
decision_status: proposed
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]   # ux-designer/1 is the author and owner and doesn't sign its own record; Marram's entry approves
supersedes: []
tags: [game, game-screen, world-view, keyboard, glyphs, slice-g-1]
generated:
  by: ux-designer/1
  at: 2026-11-01T11:00:00Z
verified: []
---

# Context

G.1 ([roadmap](/roadmap.md), [DR-0042](/decisions/0042-the-game-starts-now.md)): Ian opens the game, walks, sees the world grow ahead of him ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)), turns round and finds the way back exactly as he left it ([DR-0035](/decisions/0035-the-world-is-stored.md)). At each moment he is asking three things: where am I, where can I go, and is this the same place as before? The screen should answer those and nothing else. The engineers are building against a stub: `start_game(seed)` and `step(direction)` return the position, whether the move happened, and the stored cells within a radius (tile, orientation, kind, visited). Wren's DR-0044 sets what the player does. Where this record depends on it, it says so and gives a default.

# Decision

Damson (`ux-designer/1`). This is a two-way door: it's one screen, and nothing is built on it yet.

```
+------------------------------------------------------------------+
| Seed 48213 · 3 east, 5 north of the start · 42 steps [New world] |
+------------------------------------------------------------------+
|                                                                  |
|            +-------------------------------+                     |
|            |::::░░░░░░┌───┐░░░░░░::::::::::|  :: not grown yet   |
|            |::░░╔═════╧═╗ │░░░░░░░░░░::::::|  ░  hedge           |
|            |::░░║ · · · ║ └───┐ ░░░░░░░::::|                     |
|            |::░░╚═══════╝░░░░░@───────────::|  @  you (centre)    |
|            |::::░░░░░░░░░░░░░░╵░░░░░░░░::::|  ── worn wash where |
|            |::::::::::░░░░░░░░░░░░░░::::::::|     you've walked   |
|            +-------------------------------+  frame = how far    |
|                                                you can see        |
| Can't go north.                          Arrows or WASD to walk  |
+------------------------------------------------------------------+
```

1. **Layout.** One status line on top, the world view in the middle, a message line at the bottom. Nothing else in G.1.
   - **Status line** (16px, system-ui): "Seed 48213 · 3 east, 5 north of the start · 42 steps", then [New world].
   - **Position is in words, relative to the start**, because that's how a walker thinks ("I'm a bit north-east of where I began"), not in coordinates. At the origin it reads "at the start". It is 0-based from the spawn cell, and the engineers confirm y grows to the south, as rows do in the tool. TypeScript only formats this. It never moves the player itself.
   - **Steps** counts moves that happened. A blocked move doesn't count.
   - **[New world]** starts a new random seed. It has no key, so a stray press can't throw away a walk. After a click it gives up focus, so Space and Enter can't press it again. Typing a seed in is left for G.3's debug mode.
2. **Tiles.** [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)'s glyphs, colours, tints and font. They're drawn two characters per cell with the east filler, as in [DR-0029](/decisions/0029-generation-view-design.md) point 7 (`apps/tool/src/maze/`), with natural line height and no frames. Light and dark themes follow Windows. **28px** gives about square cells around 34px; that's bigger than the tool's 18px, because this is a screen you look at, not one you study. The mapping is copied into `apps/game` for G.1 ([DR-0012](/decisions/0012-code-layout-and-core-crate.md): apps are self-contained). Whether it moves into a shared package is Yew's call.
3. **The view is a square of (2R+1) × (2R+1) cells, with the player always in the middle cell** (a centred camera). R is the radius Rust returns. The square is centred in the window, inside a 1px frame in the frame colour: the frame is the edge of sight. Outside it is plain page. A step moves the world by one cell, with no smooth scroll in G.1. *Depends on Wren (how far they see).* My default is R = 7, a 15 × 15 view. That's about 510 × 495px, and it fits a 1024 × 768 window at 100% and 150% scaling. R must stay below DR-0036's growth radius r (8 as built).
4. **The player** is a bold `@` in the text colour, drawn in place of the cell's glyph. The cell keeps its tint and its east filler, so the way east still joins. `@` isn't a tile glyph, and it isn't red or magenta, which DR-0017 keeps for problems and selection. Being in the middle cell every time is the main cue.
5. **A blocked move** isn't an error, so it's never red. The `@` nudges about 3px toward the direction and back over 120ms, and the message line reads "Can't go north." The message clears on the next move that happens. With `prefers-reduced-motion` there's no nudge, only the words. *Depends on Wren (what blocks).* The words say only the direction, because the stub doesn't say why.
6. **Visited cells** get a worn-path wash under the glyph: `rgba(0,0,0,0.07)` in light, `rgba(255,255,255,0.08)` in dark, over any room tint. They are never dimmed, so a corridor's lines keep their full contrast. The trail is what lets Ian check that the way back is the way he came. *Depends on Wren (whether the game shows the trail).* My default is that it shows.
7. **Not grown yet:** a flat "unknown" fill with no glyph, `--unknown` #D5D9DF in light and #0B0C0F in dark. It isn't the hedge texture, and it isn't blank. Blank is the corridor's own background, so a blank cell would read as floor. A passage or door at the frontier is drawn reaching into the fill, which reads as "the way on". DR-0036 rule 2 says unsettled cells are "drawn as solid hedge". For G.1 I'm taking that as *behaves* as solid, and drawing them as unknown, because if they looked like hedge and later turned into corridor, the world would seem to change under the player. That would break the promise DR-0035 makes. *Wren and Yew* may still want hedge in the finished game (Yew's concern 3). That's one CSS token to change.
8. **Keys and focus.**
   - Arrow keys and WASD walk. The listener goes on `window`, not on an element, so the game takes keys from launch with no click.
   - On load, the app calls Tauri's `getCurrentWindow().setFocus()` and focuses the document. WebView2 can start without keyboard focus, and Ash checks this by eye on Windows.
   - `preventDefault` on these keys stops the page scrolling.
   - Holding a key walks, at the OS key-repeat rate. Only one `step` is in flight at a time, and repeats that arrive meanwhile are dropped, not queued, so letting go stops at once.
   - Keys are ignored while focus is in a text field, if one is ever added.
9. **Readable sizes and states.**
   - The map is 28px and never below DR-0017's 16px. Text is 16px.
   - Before `start_game` returns, the view says "Growing the world…". If it fails, Rust's error shows in the problem red, with [New world] to try again.
   - The message line is an `aria-live="polite"` region. The view has `role="img"` and the label "World around you".

**May slip:** the nudge (the words stay); the visited wash (the `visited` flag is still kept); a 150ms fade-in for newly grown cells, which makes the growing visible; the WASD keys (the arrows stay).

# Alternatives considered

- **A fixed camera, or one with a dead zone** that scrolls near the edge. Landmarks stay still, which helps you get your bearings. But the player has to be found again after each scroll, and the view's edge stops matching "how far you see". Easy to try if review finds the centred view tiring.
- **Coordinates ("x 3, y −5").** Exact, but it's the store's way of thinking, not the walker's. The tool and G.3 are where coordinates belong.
- **Unknown drawn as hedge** (DR-0036 rule 2, as written). It looks finished, but it lies when that space grows (point 7).
- **Dimming unvisited cells** instead of washing visited ones. Rooms you've only crossed would come out patchy, and the corridors' contrast drops.

# Concerns raised

- **Damson (`ux-designer/1`):** the unknown fill inside sight shows voids behind hedges. That's honest for a demo of growth, but it may feel unfinished as a game. Watch Ian's reaction.
- **Damson (`ux-designer/1`):** a centred camera with snapping moves the whole picture on every step. At 15 × 15 that should be fine, but it needs checking by eye.

# AI guidance

- Draw tiles from (tile id, orientation) through the copied DR-0017 mapping. Never identify a tile by its glyph.
- Don't compute moves, blocking or growth in TypeScript. Draw what `step` returns.
- Red is for errors only, never for a blocked move.

# Links

- [DR-0017](/decisions/0017-tile-glyphs-and-colours.md) glyphs and colours; [DR-0029](/decisions/0029-generation-view-design.md) two-character cells; [DR-0025](/decisions/0025-table-view-design.md)
- [DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md), [DR-0042](/decisions/0042-the-game-starts-now.md), [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- Wren's G.1 record, DR-0044 (in progress)
