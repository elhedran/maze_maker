---
type: Decision Record
title: How the world view shows a grown world, walked and replayed
description: A World tab, separate from the other views. A world seed and New world; a walker driven by a seeded random walk (Play, Pause, Step) or by Ian (arrow keys, or click a stored cell); the view pans and follows the player; overlays for stored cells, the frontier (outward endpoints as arrows) and the last patch; a patch debug overlay with its window, spine, discarded cells, attempts and flags; and Replay, which regrows the world from the growth log patch by patch and says whether it matches.
status: draft
decision_id: DR-0039
decision_status: proposed
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [ux-designer/1, product-manager/1]   # ux-designer/1 is the author; engineer-ash/1 may stand in for the author (DR-0018), as for DR-0033
supersedes: []
tags: [tool, world-view, growth, frontier, replay, phase-2, slice-2-6]
generated:
  by: ux-designer/1
  at: 2026-11-01T13:00:00Z
verified:
  - by: product-manager/1
    at: 2026-11-01T14:00:00Z
    note: "Approved with four scope cuts, recorded under Concerns raised (Marram overrules on scope)."
  - by: engineer-ash/1
    at: 2026-11-01T15:55:00Z
    note: "Stands in for the author, Damson (DR-0018), as for DR-0033. Approves with Marram's four cuts; building on ash/2.6-world-view against DR-0041. Build calls for Damson's review are under Concerns raised (Ash, on build)."
---

# Context

Slice 2.6 step 3 ([roadmap](/roadmap.md)) puts [DR-0036](/decisions/0036-growth-from-open-endpoints.md)'s grown world on screen. The demo that closes 2.6: "Ian walks a world in the tool, watches it grow from open endpoints with the frontier overlay on, then replays the walk and sees the same world."

Ian's questions at this screen, in the order he'll have them:

1. "Is it growing where I'm going, from the ends of paths and doors, and never in front of my face?"
2. "Is there always a way on?" (the frontier, and which endpoints are outward)
3. "What did that patch just do, and did it struggle?" (window, spine, discards, attempts)
4. "If I replay it, is it the same world?"

A grown world has no edges, so it is a different thing to look at from 2.1's fixed maze. It gets its own tab rather than a mode of the Maze tab. Kept small, like [DR-0029](/decisions/0029-generation-view-design.md).

# Decision

Damson (`ux-designer/1`), a two-way door: one new view, nothing builds on it.

```
| [ Table ] [ Explore ] [ Maze ] [#World#]       tile set bar, as DR-0029 (unchanged)  |
+--------------------------------------------------------------------------------------+
| World seed [ 48213 ] [New world] [Random seed]                                       |
| Walk seed  [ 907   ] Steps [ 200 ] [> Play] [|| Pause] [Step] Speed [Normal v]       |
| Overlays: [x] Stored (S)  [x] Frontier (F)  [x] Last patch (L)  [ ] Patch debug (D)  |
|                                                                                      |
| World 48213 · 112 steps · 37 patches · 1,912 cells     | PATCH 37 of 37              |
| +----------------------------------------------------+ | anchor x 14, y -3, grown E  |
| |                 ← ░░░░░░                            | | window x 14..29, y -11..4  |
| |   ░░╔═════╗░░─────┐ :- - - - - - - - - - - - :      | | attempts 3 (1 = clean run) |
| |   ░░║ · · ╟───┐   │ :  ┌───┐ × ×              :      | | spine: yes   fallback: no  |
| |   ░░╚═════╝░░ │ [@]└──┼───┘ ×  ○              :      | | stored 41 · discarded 23   |
| |             ↓ ╵   ░░  ║=====spine=====> →      :      | |                            |
| |                       :- - - - - - - - - - - - :      | | Frontier: 14 open,         |
| +----------------------------------------------------+ | 9 outward                   |
| Corridor corner, open N and E · x 12, y -4 · stored by patch 21    [Centre on player] |
| Legend: [@] player  → outward endpoint  ○ open, not outward  :- -: patch window       |
|         ━ spine  × discarded  dimmed = older patches  blank = unsettled              |
+--------------------------------------------------------------------------------------+
```

## What the build must do

1. **A new tab, World,** in its own component (`world-view`), after the existing tabs (after 2.3's compare tab if that lands first). It reuses the shared tile set bar and banner as they are; this record changes no shared chrome. It hides rather than unmounts, like the others.
2. **World seed.** A whole number up to the u32 limit, with DR-0029's validation and wording. It opens filled with a random seed. **New world** (or Enter in the field) grows the origin patch and growth around it, puts the player at the origin, and clears the walk. **Random seed** fills the field and starts a new world in one press.
3. **Two ways to walk, freely mixed.** The growth log records anchors, not how the walk was driven, so replay works either way.
   - **Seeded random walk:** Walk seed (whole number, opens random) and Steps (default 200). **Play** walks a step at each tick until Steps are done or Pause. **Step** walks one step. Speed: Slow (400 ms), Normal (100 ms), Fast (25 ms). Changing the walk seed mid-world carries on from where the player stands.
   - **Ian walks:** with the map focused, **arrow keys or WASD** move one cell to a joined stored neighbour. A blocked move does nothing and the readout says "Hedge to the north". **Click a stored cell** to walk there along a shortest joined path, one step per tick. Any key or click pauses Play first.
4. **The world grows visibly.** The view redraws after every walk step with the patches that step grew. A step that grows several patches shows them one per tick, so Ian sees them arrive in order. The status line counts steps, patches and stored cells.
5. **Panning.** The map fills the panel, at least 40 × 24 cells, at DR-0029's two characters per cell and 18px (never below DR-0017's 16px). **Follow player** is on by default: the view re-centres when the player comes within 6 cells of an edge. Dragging the map, or **Shift+arrows** (8 cells), pans and turns Follow off. **Centre on player** (or **C**) turns it back on. Coordinates are world x, y with the origin at 0, 0, y growing south, as in `world.rs`.
6. **Drawing.** Stored cells use DR-0017's glyphs, colours and tints exactly as the Maze tab does.
   - **Unsettled cells are solid hedge** (░ on the hedge background), what a player would see.
   - **Stored overlay (S)** draws unsettled cells blank instead (DR-0017 rule 6: blank means "not generated yet"), so the edge of the stored world shows as shape, not colour.
   - **The player** is the cell drawn inverted (text colour background, page colour glyph) with a 2px outline, never red or magenta. The origin gets a 1px dotted outline.
7. **Frontier overlay (F).** Each open endpoint is marked in its anchor cell: an **arrow pointing away from the stored cell** (→ ← ↑ ↓) when it is outward, a **hollow circle ○** when it is not. Marks are in the text colour and bold. The status panel gives the counts: "Frontier: 14 open, 9 outward".
8. **Last patch overlay (L).** The last patch's window gets a dashed 2px outline. Every stored cell from earlier patches is **dimmed** (about 50% opacity), so the new cells stand out by brightness, not hue.
9. **Patch debug overlay (D)**, on top of L:
   - the anchor cell gets a diamond outline;
   - the **spine** is drawn as a thick line through its cells' centres, from the anchor out across the window's edge, ending in an arrowhead;
   - **discarded cells** (cells in the window that the run filled but rule 4 did not keep, so still unsettled) get a centred ×;
   - the side panel shows the patch: number, anchor and the direction it grew, window bounds, **attempts** ("1 = clean run"), **spine yes/no**, **fallback yes/no**, cells stored and discarded. Spine and fallback are words, never only a colour or icon.
   With D off, the panel shows only the patch number, attempts and the frontier counts.
10. **Hover.** One readout line under the map, as DR-0029: tile name, x, y, and "stored by patch 21"; for an unsettled cell "Unsettled" plus, if it is an anchor, "open endpoint, outward". Each cell's `aria-label` is the same text.
11. **Replay.** **Replay walk** (enabled once a walk has grown at least one patch) asks Rust to regrow the world from the growth log into a fresh world, with the tile set the world was grown with. The map clears to the origin and the patches come back one per tick, with the same overlays, under a **timeline**: "Replay · patch 12 of 37", with Play, Pause, Step back, Step forward and a slider. Step back is safe because stored cells never change: the view shows patches 1 to k. When it reaches the end, the status line says, in words, **"Replayed 37 patches · same world as walked: yes (1,912 cells match)"**, or "no", with the first differing cell's coordinates, in the problem red. The answer comes from Rust comparing both worlds, not from the view. **Back to walk** leaves replay with the walked world as it was. Walking is disabled during replay.
12. **Reload.** The world keeps the tile set it was grown with. After the shared Reload succeeds, the status line adds "grown with the tile set loaded 20:14 · New world uses the reloaded one". Nothing regrows on its own.
13. **States.** Before New world: a blank map with "Press New world to grow a world from this seed." While Rust works, the buttons show "Growing…" and the map stays. A Rust error goes to the shared banner and the last good world stays.

## What the view needs from Rust

The IPC contract is Yew's DR-0041, written in parallel; this record doesn't set wire types. The view needs:

- per patch, a delta: the log entry (anchor and its direction, window, attempts, spine and fallback flags, spine cells) plus the stored cells with their tiles;
- the discarded cells of each patch (they aren't in today's `Patch`);
- the frontier after a patch: each open endpoint, its direction and whether it is outward;
- a walk step (random, by key or along a path to a clicked cell) returning the player's new cell and that step's patch deltas;
- replay of the growth log as a stream or list of the same deltas, and Rust's same-world comparison;
- positions as plain numbers ([DR-0036](/decisions/0036-growth-from-open-endpoints.md), Ash's note).

## Keyboard

With the map focused: arrows or WASD walk; Shift+arrows pan; C centres; Space plays or pauses; `.` steps (forward in replay) and `,` steps back in replay; S, F, L and D toggle the overlays. The overlay buttons are toggle buttons with `aria-pressed` and show their key.

## Colour is never the only cue

| What | Cue without colour |
|---|---|
| Unsettled vs stored | blank (S on) vs a glyph; ░ texture when S is off |
| Outward vs other endpoint | arrow vs hollow circle |
| Last patch vs older | full strength vs dimmed; dashed window outline |
| Spine | a thick line with an arrowhead |
| Discarded | × |
| Player | inverted cell with an outline |
| Spine, fallback, same world | written as words |

No overlay uses red or magenta, except the "no" of the replay check, which is also written out.

# How Ian demos it

All tasks he does himself:

1. World tab. **New world.** The origin patch appears with the player in it.
2. Frontier and Last patch on. **Play.** He watches the walker wander and patches arrive at arrows ahead of it, never in its face.
3. **Pause.** He walks with the arrow keys toward an arrow and watches a patch grow there. He clicks a far corridor and the walker goes.
4. Patch debug on. **Step** a few times and reads one patch: its window, spine, discards and attempts.
5. **Replay walk.** Play to the end. The world comes back patch by patch: "same world as walked: yes".
6. Optional: New world with the same two seeds, Play, and the same world grows.

# Alternatives considered

- **A mode of the Maze tab.** It shares little: no fixed size, no DR-0030 measures, and it adds a walker, overlays and a timeline. One more tab is simpler to read.
- **Unsettled cells always blank.** Truer to the data, but it hides what the player would see, and Yew's concern 3 (a cell drawn solid that later opens) is about that picture. The S toggle gives both.
- **Colour tints for the frontier and patches.** Fast to read, but fails in greyscale and for colour-blind readers, and competes with DR-0017's tile colours.
- **Replay as an instant rebuild.** Proves sameness, but Ian wouldn't see it grow. The timeline shows it and also makes a debugger of the log.
- **Ghosting the discarded cells' tiles.** It would show what rule 4 threw away, but it needs more data and looks like stored cells. A × is enough for 2.6.

# Consequences

- The demo is the six tasks above.
- DR-0041 must carry discarded cells and per-anchor outwardness, which today's `Patch` and `World` don't expose.
- Wren and Damson review the built view for rooms against patch edges, the frontier, and Yew's concern 3. Watching with S off is how concern 3 is seen.
- Not in step 3: "Why is this here?" ([DR-0033](/decisions/0033-why-is-this-here-design.md)) on world cells, since click walks here; step 2's measures; saving a world or walk; field of view or anything of the game's.

# Concerns raised

- **Damson (`ux-designer/1`):** with r = 8 and a 40-cell-wide view, growth is visible on screen, which is right for a debug view and wrong for a game. Don't take this view's framing as how the game shows growth.
- **Marram (`product-manager/1`), 2026-11-01: scope cuts for step 3, overruling Damson on scope (CLAUDE.md overrules; each a two-way door, added back if review of the built view asks).** The demo is walk, watch growth with the frontier on, replay, "same world: yes". These parts aren't needed for it and add build or contract work today:
  1. **Click a stored cell to walk there** (rule 3) is cut: it needs pathfinding and a third walk command. Arrow keys/WASD and the seeded walk cover the demo. Demo task 3 drops the click.
  2. **Dragging the map** (rule 5) is cut. Follow player, Shift+arrows and Centre on player stay.
  3. **Replay's Step back and slider** (rule 11) are cut. Replay keeps Play, Pause, Step forward, "Replay · patch k of n" and Rust's same-world line.
  4. **Patch debug (D) drawing** (rule 9) is cut to the side panel's words: patch number, anchor and direction, window bounds, attempts, spine yes/no, fallback yes/no, cells stored. The diamond, the drawn spine and the discarded × are deferred, so DR-0041 needn't carry discarded cells or spine cells for step 3. Last patch (L) still outlines the window.
  - The cost of cut 4: without the drawn spine and discards, D shows less of "did it struggle". Damson can add her own concern here. Marram's view: attempts, spine and fallback in words answer that for 2.6; revisit if Wren or Damson's review of the built view needs the drawing.

- **Ash (`engineer-ash/1`), on build, for Damson's review (two-way doors):** the map is a fixed 40 × 24 cell window rather than resizing to fill the panel, so the follow-player and pan rules stay simple and testable; widen it at review if it feels cramped. New world reveals its first patches one per tick, as a walk step does, so the origin patch is seen arriving first. In replay the player isn't drawn, since the log holds anchors, not the walk. Replay ends when Rust's `check` arrives, even before the last patch (Rowan's note on DR-0041). The world's four commands live in `apps/tool/src/world/`, not `ipc.ts`, to keep clear of the parallel 2.3 work. **WASD is left out:** the Keyboard section has S and D both walking (WASD) and toggling overlays (S, D), so the arrow keys walk and the letters toggle, matching the buttons; Damson to pick another walk set if she wants one. Arrow keys, Step and a click on the map pause Play; overlay keys don't, so an overlay can be turned on while watching.

# Links

- Roadmap, slice 2.6 step 3: [roadmap](/roadmap.md)
- Growth: [DR-0036](/decisions/0036-growth-from-open-endpoints.md); stored world: [DR-0035](/decisions/0035-the-world-is-stored.md)
- Glyphs and colours: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md); generation view: [DR-0029](/decisions/0029-generation-view-design.md); maze cell link: [DR-0033](/decisions/0033-why-is-this-here-design.md)
- Approvals: [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)
