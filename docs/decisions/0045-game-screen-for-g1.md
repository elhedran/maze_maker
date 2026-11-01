---
type: Decision Record
title: How the game screen looks and takes keys in G.1
description: A 13 by 13 world view centred on the player, drawn with DR-0017's glyphs two characters per cell at 32px; a status line with seed, position in words and step count; the player as a bold @; a held key walks at a set pace of about 8 steps a second; a blocked move is a small nudge with "Can't go north."; visited cells get a trodden wash; cells not yet known are a flat haze, never hedge; keys work from launch, with no click first.
status: stable
decision_id: DR-0045
decision_status: accepted
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]   # ux-designer/1 is the author and owner and doesn't sign its own record; Marram's entry approves
supersedes: []
tags: [game, game-screen, world-view, keyboard, glyphs, slice-g-1]
generated:
  by: ux-designer/1
  at: 2026-11-01T11:00:00Z
verified:
  - by: product-manager/1
    at: 2026-11-01T10:26:33Z
    note: "Approves, with the four may-slip items accepted as fallbacks (nudge, wash, hold delay, WASD), each listed as a follow-up if cut. One addition in Concerns raised: G.1's demo needs a same-seed restart without typing a seed in the window."
  - by: engineer-ash/1
    at: 2026-11-01T10:27:11Z
    note: "Buildable as written, standing in for the author as engineer. Built in apps/game on ash/g1-screen. Five build notes in Concerns raised: held keys clear on window blur, one waiting press, the set-focus permission, the launch seed in the contract, and what the nudge replays on."
---

# Context

G.1 ([roadmap](/roadmap.md), [DR-0042](/decisions/0042-the-game-starts-now.md)): Ian opens the game, walks, sees the world grow ahead of him ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)), turns round and finds the way back exactly as he left it ([DR-0035](/decisions/0035-the-world-is-stored.md)). At each moment he is asking three things: where am I, where can I go, and is this the same place as before? The screen should answer those and nothing else. The engineers are building against a stub: `start_game(seed)` and `step(direction)` return the position, whether the move happened, and the stored cells within a radius (tile, orientation, kind, visited). Wren's [DR-0044](/decisions/0044-walking-the-growing-world.md) sets what the player does: one cell per press, a radius of 6 with no fog, the unknown never shown as hedge, a trodden trail, and a bump that costs nothing. Its "What makes it fun" section is what point 10 below serves.

# Decision

Damson (`ux-designer/1`). This is a two-way door: it's one screen, and nothing is built on it yet.

```
+------------------------------------------------------------------+
| Seed 48213 · 3 east, 5 north of the start · 42 steps [New world] |
+------------------------------------------------------------------+
|                                                                  |
|            +--------------------------+                          |
|            |::░░░░░░┌───┐░░░░░░::::::|  :: not yet known (haze) |
|            |░░╔═════╧═╗ │░░░░░░░░░:::|  ░  hedge                 |
|            |░░║ · · · ║ └───┐ ░░░░░::|  ·  room floor, tinted    |
|            |░░╚═══════╝░░░░░@───────::|  @  you, always centre    |
|            |::░░░░░░░░░░░░░░╵░░░░░░::|  ── trodden wash where    |
|            |::::::::░░░░░░░░░░░░░::::|     you've stood          |
|            +--------------------------+  frame = 6 cells, the    |
|                                           edge of sight           |
| Can't go north.                          Arrows or WASD to walk  |
+------------------------------------------------------------------+
```

1. **Layout.** One status line on top, the world view in the middle, a message line at the bottom. Nothing else in G.1.
   - **Status line** (16px, system-ui): "Seed 48213 · 3 east, 5 north of the start · 42 steps", then [New world].
   - **Position is in words, relative to the start**, because that's how a walker thinks ("I'm a bit north-east of where I began"), not in coordinates. At the origin it reads "at the start". It is 0-based from the spawn cell, and the engineers confirm y grows to the south, as rows do in the tool. TypeScript only formats this. It never moves the player itself.
   - **Steps** counts moves that happened. A blocked move doesn't count.
   - **[New world]** starts a new random seed. It has no key, so a stray press can't throw away a walk. After a click it gives up focus, so Space and Enter can't press it again. Typing a seed in is left for G.3's debug mode.
2. **Tiles.** [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)'s glyphs, colours, tints and font. They're drawn two characters per cell with the east filler, as in [DR-0029](/decisions/0029-generation-view-design.md) point 7 (`apps/tool/src/maze/`), with natural line height and no frames. Light and dark themes follow Windows. **32px** gives cells about 38px square. That's bigger than the tool's 18px, because this is a screen you look into, not one you study. The mapping is copied into `apps/game` for G.1 ([DR-0012](/decisions/0012-code-layout-and-core-crate.md): apps are self-contained). Whether it moves into a shared package is Yew's call.
3. **The view is 13 × 13 cells, with the player always in the middle cell** (a centred camera). That's DR-0044 rule 4's radius of 6, with no fog. The view is sized from the radius setting, not hard-coded. The square is centred in the window, inside a 1px frame in the frame colour: the frame is the edge of sight. Outside it is plain page. A step moves the world by one cell, with no smooth scroll in G.1. At 32px the view is about 500 × 490px, which fits a 1024 × 768 window at 100% and 150% scaling. There's no minimap (DR-0044 rule 8).
4. **The player** is a bold `@` in the text colour, drawn in place of the cell's glyph. The cell keeps its tint and its east filler, so the way east still joins. `@` isn't a tile glyph, and it isn't red or magenta, which DR-0017 keeps for problems and selection. Being in the middle cell every time is the main cue.
5. **A blocked move is a bump that costs nothing** (DR-0044 rule 2). It isn't an error, so it's never red.
   - The `@` nudges about 3px toward the direction and back over 120ms.
   - The message line reads "Can't go north.", in muted text. It clears on the next move that happens.
   - With `prefers-reduced-motion` there's no nudge, only the words.
   - The words give only the direction, because the stub doesn't say why.
6. **The trodden trail** (DR-0044 rule 6) is a faint wash under the glyph of every visited cell: `rgba(0,0,0,0.07)` in light, `rgba(255,255,255,0.08)` in dark, over any room tint. It's faint enough that it never competes with what's ahead. Visited cells are never dimmed, so a corridor's lines keep their full contrast.
7. **Not yet known** (DR-0044 rule 5): any cell in the view that the stub didn't return is a flat haze with no glyph and no texture. The token is `--unknown`, #D5D9DF in light and #0B0C0F in dark.
   - It's never the hedge texture, because "not yet known" can still become a path, and stored hedge never can.
   - It isn't blank either. Blank is the corridor's own background, so a blank cell would read as floor.
   - A passage or door at the frontier is drawn reaching into the haze, which reads as "the way on".
   - DR-0036 rule 2 says unsettled cells are "drawn as solid hedge". Wren's DR-0044 and the roadmap now read that as *behaves* as solid. Yew should note that in DR-0036.
8. **Keys, pace and focus.**
   - **Keys:** arrow keys and WASD. The listener goes on `window`, not on an element, so the game takes keys from launch with no click.
   - **Focus:** on load, the app calls Tauri's `getCurrentWindow().setFocus()` and focuses the document, because WebView2 can start without keyboard focus. Ash checks this by eye on Windows.
   - **No scrolling:** `preventDefault` on these keys stops the page scrolling.
   - **One press is one step,** sent on keydown at once.
   - **Holding walks at a steady pace** set by the game, not by the OS key repeat, which varies from machine to machine. After 250ms held, it takes one step every 125ms, about 8 cells a second. That's fast enough to cross ground, and slow enough to see a junction go by. Both numbers are settings. It ignores the OS's repeat events and tracks keydown and keyup.
   - **One step at a time:** only one `step` is in flight, and the next waits for it rather than queuing, so letting go stops at once.
   - **Holding into a hedge** bumps once, then waits until the key is released or the direction changes. You don't get a stuttering row of nudges.
   - **The newest key wins:** if two keys are held, the most recent one walks.
   - **Text fields:** keys are ignored while focus is in one, if one is ever added.
9. **Readable sizes and states.**
   - The map is 32px and never below DR-0017's 16px. Text is 16px.
   - Before `start_game` returns, the view says "Growing the world…". If it fails, Rust's error shows in the problem red, with [New world] to try again.
   - The message line is an `aria-live="polite"` region. The view has `role="img"` and the label "World around you".
10. **What the screen does for "What makes it fun"** (DR-0044):
    - **Rooms are finds.** The room tint and amber walls draw at full strength. Hedge stays the quietest thing on screen, as DR-0017's low-contrast hedge glyph already is. So a room past the hedge catches the eye.
    - **Something worth seeing ahead.** The centred view gives six cells in every direction, so a choice is visible whichever way you face.
    - **The world never feels like it's assembling.** Newly stored cells appear with no animation, fade or highlight. Anything that draws the eye to growth would undo the illusion. Wren's checklist item 6 counts how often it happens in view.
    - **Dead ends are cheap.** The trail shows at a glance which branch you've already tried.

**May slip:** the nudge (the words stay); the trodden wash (the `visited` flag is still kept); the 250ms hold delay (a fixed pace from the first repeat is enough); the WASD keys (the arrows stay).

# Alternatives considered

- **A fixed camera, or one with a dead zone** that scrolls near the edge. Landmarks stay still, which helps you get your bearings. But the player has to be found again after each scroll, and the view's edge stops matching "how far you see". Easy to try if review finds the centred view tiring.
- **Coordinates ("x 3, y −5").** Exact, but it's the store's way of thinking, not the walker's. The tool and G.3 are where coordinates belong.
- **The unknown as hazy hedge** (DR-0044 allows it): the hedge texture, faded. It looks more finished, but a hedge-like cell that turns into a path is exactly what rule 5 guards against, and at a glance faded hedge and hedge are hard to tell apart. Flat haze is the cheapest look that can't be mistaken. It's one CSS token to change if review wants it.
- **OS key repeat for holding.** No code is needed, but the pace changes from machine to machine and is often 30 a second, too fast to read a junction.
- **A fade-in for new cells.** It would show the growing, which is exactly what DR-0044 says the player shouldn't notice.
- **Dimming unvisited cells** instead of washing the visited ones. Rooms you've only crossed would come out patchy, and the corridors' contrast drops.

# Concerns raised

- **Damson (`ux-designer/1`):** the haze inside sight shows voids behind the hedges. That's honest, but it may feel unfinished. Watch Ian's reaction, alongside Wren's checklist item 7.
- **Damson (`ux-designer/1`):** a centred camera with snapping moves the whole picture on every step. At 13 × 13 and 8 steps a second it should be fine, but it needs checking by eye.

- **Marram (`product-manager/1`), on approving:** G.1's demo task 4 restarts the game with the same seed. This screen gives no way to choose a seed, which is fine, but launching must be able to reuse one. Marram's call (a two-way door): the launch command takes an optional seed (for example from the command line or an environment variable, the engineers' pick, written in the README), and with none it picks a random one, shown in the status line. No seed field on screen in G.1.

- **Ash (`engineer-ash/1`), on review:** buildable. Five build calls the record doesn't spell out, all two-way doors:
  - **Lost keyups.** If the window loses focus while a key is held, its keyup never arrives and the player would walk on by itself. The game drops every held key on window `blur`.
  - **A press during a step.** Point 8 says the next step "waits rather than queuing". I read that as at most one waiting press, the newest, so a quick tap during a slow step isn't lost and holding a key still can't build a backlog. Hold ticks never wait: a tick that finds a step in flight is skipped.
  - **Focus permission.** `setFocus()` needs `core:window:allow-set-focus`, which isn't in `core:default`. Rowan adds it to the game's capabilities. If the call fails, the game carries on, because the window listener still works once the window has focus.
  - **Launch seed (Marram's concern).** The frontend can't read the command line or the environment, so Rust has to supply the launch seed. Following DR-0043, the frontend asks Rust for it with an `initial_seed` command (the launch seed if one was given, otherwise a random one), then calls `start_game(seed)` and shows that seed. [New world] sends a random seed. Rowan confirms the command.
  - **Nudge replay.** The nudge is a CSS animation on the `@`, replayed on each bump, and `prefers-reduced-motion` turns it off in CSS. A held key bumps once (point 8), so the animation never stutters.

- **Damson (`ux-designer/1`), review of the build, 2026-11-01:** **pass for the demo, once the three must-fixes below are in.** I reviewed Rowan's screenshots of the real Tauri window under Xvfb (seed 1: launch, blocked, east 5 and 10, the far end, back at the start, a restart, and every step) and `apps/game/src` on `mm-rowan-g1`. What works:
  - Keys walk with no click.
  - The `@` stays in the centre.
  - The status line reads as a walker thinks ("8 east, 4 north of the start · 14 steps").
  - "Can't go east." is quiet and not red.
  - The haze reads as "not yet known".
  - The way back matches cell for cell, and a restart is identical.
  - The pace (250ms, then 125ms) and Ash's five calls are as asked, and all are **accepted**.

  Wren's fix that draws sealed unknown pockets as hedge (░) is **accepted**, and it amends point 7. A pocket that can never be reached can never become a path, so drawing it as hedge is honest. The haze stays for unknown cells that can still grow.

  **Must fix before the demo** (all CSS or template, minutes each):
  1. **The blank bands above and below the rows** (the frame is about 646px for about 494px of rows). It isn't the font. The `.map` div has `white-space: pre`, so the newlines and indents around `${layoutGrid(...)}` in the template render as empty lines at 32px. Put the expression flush against the tags, `<div ...>${...}</div>`, as the tool's `maze-view.ts` does. The frame then hugs the rows, and the edge of sight sits where it should.
  2. **The trail.** There are two problems. The black wash makes room tint muddy. And on corridors it's a flat light grey right next to the flat grey haze, so in `far-end` the trail reads as a lighter patch of "not yet known", the one thing it mustn't look like. The fix:
     - make the trail a warm "trodden earth" tint, `--trail: rgba(160, 120, 40, 0.10)` in light and `rgba(242, 166, 90, 0.08)` in dark, so it's warm where the haze is cool;
     - leave it off room cells (`.cell.room.visited { box-shadow: none; }`). Rooms are small and their doorways carry the trail in and out. Wren, that's a small departure from "every cell stood on" (DR-0044 rule 6); say if it matters to you.
  3. **The room tint bleeds past east walls.** About 6 to 10px of amber shows outside ║, ╝ and ╗ (in `launch`, the right edges of all three rooms). A two-character cell puts its vertical lines at 25% of the cell, not 50%, so a "left" tint should stop at 25% and a "right" tint should start there:
     - `.tint-left`, `-lower-left` and `-upper-left` become `background-size: 25% …`;
     - `.tint-right`, `-lower-right` and `-upper-right` become `background-size: 75% …`, still positioned right.

     Rooms are meant to be the finds, so their edges should be clean. The tool has the same rule (DR-0043 point 5 says fix both copies), but there it's smaller at 18px, so the tool's fix can follow.

  **Check on Windows before the demo** (not fixes): that WebView2 takes keys with no click on Ian's machine; the dark theme; and 150% scaling.

  **Follow-ups**, none blocking:
  1. **For Wren:** seed 1's views are dense corridor grids, with a junction almost every cell and very little hedge, close to the "graph-paper grid" DR-0044 warns against. It's a generator question for the 2.3 tuning, not the screen's.
  2. The `@` hides the tile under it. If Ian asks "what am I standing on?", the message line could name it when he stops.
  3. My two concerns above (voids behind hedges, a snapping camera) stand until Ian has walked it.

# AI guidance

- Draw tiles from (tile id, orientation) through the copied DR-0017 mapping. Never identify a tile by its glyph.
- Don't compute moves, blocking or growth in TypeScript. Draw what `step` returns.
- Red is for errors only, never for a blocked move.
- Never animate or highlight newly stored cells.

# Links

- [DR-0017](/decisions/0017-tile-glyphs-and-colours.md) glyphs and colours; [DR-0029](/decisions/0029-generation-view-design.md) two-character cells; [DR-0025](/decisions/0025-table-view-design.md)
- [DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md), [DR-0042](/decisions/0042-the-game-starts-now.md), [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- What the player does, and what makes it fun: [DR-0044](/decisions/0044-walking-the-growing-world.md)
