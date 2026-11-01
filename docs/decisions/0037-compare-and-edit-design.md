---
type: Decision Record
title: How the tool edits a number and compares the same seed before and after
description: Any number in the explanation panel that lives in the tile set (base, tile weight, exception, opening pressure) is edited in place with its reach spelled out; a saved change puts the Maze tab into a side-by-side before-and-after of the same seed with unchanged cells faded and both measures under it; opening pressure sits under the seed row; a refused edit stays at the field with the loader's reason; the "why" band refreshes in place after any reload.
status: stable
decision_id: DR-0037
decision_status: accepted
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [product-manager/1, engineer-ash/1]   # engineer-ash/1 stands in for the author, ux-designer/1 (DR-0018), as on DR-0029 and DR-0033
supersedes: []
tags: [tool, editing, compare, generation-view, explore-view, opening-pressure, phase-2, slice-2-3]
generated:
  by: ux-designer/1
  at: 2026-11-01T08:58:00Z
verified:
  - by: product-manager/1
    at: 2026-11-01T09:05:00Z
    note: "Approved, inside 2.3's scope. Two scope calls under Concerns raised (Marram, on review): no stacked layout for narrow windows in 2.3, and the may-slip list stands as written. Status stays proposed until engineer-ash/1 stands in for the author, Damson (DR-0018), as on DR-0033."
  - by: engineer-ash/1
    at: 2026-11-01T09:07:50Z
    note: "Stands in for the author, Damson (DR-0018). Approves: buildable as written on DR-0038's commands, with two small additions to DR-0038's contract that the views need, under Concerns raised (Ash, on build)."
---

# Context

Slice 2.3 ([roadmap](/roadmap.md)) closes Ian's loop ([Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md)): he sees a bit of maze he doesn't like, asks why it's there ([DR-0033](/decisions/0033-why-is-this-here-design.md)), changes the number behind it, and sees the same seed before and after. He also tries opening pressure ([DR-0036](/decisions/0036-growth-from-open-endpoints.md) rule 9). Kept as small as [DR-0029](/decisions/0029-generation-view-design.md). Ian: "There is such a thing as polishing too much before moving on."

What's going on for Ian at that moment matters for two choices here.

- **He thinks he's changing one pair; he's changing a shared number.** He clicks "× Corridor corner 3.0" in the panel for one wall beside one corner. That weight is in every Corridor corner entry, in all orientations and directions ([DR-0022](/decisions/0022-weight-model.md)). If the editor doesn't say so, the maze will change in places he didn't expect, and he'll distrust the tool. So every edit says what it reaches, in words.
- **One different draw changes the draws after it.** Same seed, new numbers: the maze matches up to the first draw that goes differently, then can change a lot. Marking each changed cell heavily would cover most of the maze in marks. So the design fades what didn't change, and says why the changes spread.

Yew's [DR-0038](/decisions/0038-editing-core-toml.md), written alongside, gives the commands: edit takes a path to a number and a new value; compare returns both mazes, the changed cells and both measure sets; and opening pressure is its own factor in the "why" trace. Rust works out every number; the frontend formats ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), DR-0025).

# Decision

Damson (`ux-designer/1`), a two-way door. The numbers in the sketches are illustrative.

## Editing a number

```
DR-0025 PANEL (table, explore, or a factor from the "why" band)
| Where it comes from                          |
|   Passage–Passage base  1.0                  |   numbers with a dotted underline
| × Corridor straight     2.0                  |   are editable; noise is not
| × Corridor corner      [3.0]  <- clicked     |
|   +--------------------------------------+   |
|   | Corridor corner weight  [ 2.0    ]   |   |
|   | Every Corridor corner, in all four   |   |
|   |   orientations and every direction.  |   |
|   | [Save to core.toml]  [Cancel]        |   |
|   +--------------------------------------+   |
| × exceptions: none                           |
| × noise 0.97                                 |
| = entry 6.0                                  |
|                                              |
| Saved 20:31 · Corridor corner weight         |
|   3.0 -> 2.0   [See before and after]        |
| [Explore this tile]  [Show in table]         |
```

1. **What's editable.** In DR-0025's panel (`table-explanation`), wherever it appears: the side-pair base, both tile weights and each exception multiplier. Each shows as a button styled as the number, with a dotted underline, tabbable, `aria-label` "Edit Corridor corner weight, 3.0". Noise, the entry and the share are worked out, not stored, so they stay plain text.
2. **Two more places in the "why" band**, so every number in the band can be reached: a click on a candidate's **weight** cell, or on a **pressure** cell (point 15), selects it (magenta outline, as factors) and the right-hand panel shows a short single-number panel instead of DR-0025's pair panel: the name, the value, what it does in one line, and the same edit. This matters when no neighbour was counted (DR-0033 point 15): then there are no factors to click, and the weight is the only number.
3. **Changing it.** A click opens the editor in place, under that line: the field, prefilled with the exact value Rust sends (not the rounded one shown), focused and selected; one line saying what it reaches; [Save to core.toml] and [Cancel]. Enter saves, Esc cancels. One editor at a time: opening another closes the first and drops what was typed.
4. **What it reaches**, in words, from the path:
   - base: "Every Passage–Passage meeting.";
   - tile weight: "Every Corridor corner, in all four orientations and every direction." (one orientation for a tile that has one);
   - exception: "Every pair this rule matches:" and the rule as written in the data;
   - opening pressure: "Every draw where a tile would add no new way onward."
5. **Save is enabled** only when the field holds a number different from the current value. Anything not a number reads "A number, like 2.0 or 0.5." under the field. The frontend checks nothing else: ranges, 0s and completeness are the loader's, so there's one source of truth (point 16).
6. **Saving.** The button reads "Saving…" and is disabled. Rust writes `core.toml` and reloads it (DR-0038). On success, the app runs the same path as Reload tile set (DR-0029 point 3): the table and explore view refetch, keeping selection, direction, centre and trail; the maze regenerates the shown seed; the band refreshes (point 16).
7. **After a save**, the editor closes and the panel shows the new numbers, with one line under the sum: "Saved 20:31 · Corridor corner weight 3.0 → 2.0", and [See before and after] when a maze is shown, which switches to the Maze tab. The line stays until the next edit or a new selection. The old value in it is how Ian sets a number back. No tab switches by itself: Ian is in the middle of reading the band, and it updates where he is.

## Before and after

```
MAZE TAB, after a save
| Seed [ 48213 ] [Generate] [Random seed]                                          |
| Opening pressure  1.0 (off)   Below 1, a tile that adds no new way onward is     |
|                               less likely.                                       |
| Seed 48213 · 24×16 · before and after your last change          [Close compare]  |
| BEFORE · as loaded 20:14:03               AFTER · Corridor corner weight 3.0 -> 2.0 |
| +----------------------------------+      +----------------------------------+   |
| |░░░░┌───┐ ░░╔═══╗░░ (faded)        |      |░░░░┌───┐ ░░╔═══╗░░ (faded)        |   |
| |░░░░│   └───╢ · ║░░  ...          |      |░░░░│   ┌───╢ · ║░░  ...          |   |
| |  ...           changed: full     |      |  ...          changed: full      |   |
| +----------------------------------+      +----------------------------------+   |
| col 7, row 1 · before: └ Corridor corner, open N and E · after: ┌ Corridor corner... |
| 37 of 384 cells changed. [x] Fade unchanged cells                                 |
| One different draw changes the draws after it, so changes spread from the first.  |
|                              BEFORE                   AFTER                       |
| Reachable from centre        412 of 480 (86%)    ->   380 of 471 (81%)            |
| Separate areas               5                   ->   7                           |
| Selected (after): ┌ Corridor corner · col 7, row 1   [Why is this here?]          |
```

8. **When it opens.** Whenever a load changes the tile set while a maze is shown: a save in the tool, or Reload tile set after editing the file by hand. Ian's question is the same either way: "what did my last change do?". **Before** is the maze that was on screen just before the change; **after** is the same seed regenerated. A second change makes the old "after" the new "before". [Close compare], Generate and Random seed go back to DR-0029's single maze (the after one).
9. **Labels.** "BEFORE · as loaded 20:14:03" (the time of the load it came from) and "AFTER ·" what changed: "Corridor corner weight 3.0 → 2.0" for a save in the tool, "after reload at 20:31" for a Reload. Words, not colour, say which is which.
10. **Layout.** Side by side, before on the left, at 16px (DR-0017's floor) instead of 18px, so two 24×16 mazes fit the 1024px window. The measures move from the right-hand column to a two-column table under the mazes: label, before, →, after, with DR-0030's wording and no difference worked out in TypeScript. The hover readout spans both mazes: hovering a cell in either gives a 1px text-colour outline on the same cell in the other, and the readout names both tiles.
11. **Changed cells.** Rust sends them (DR-0038). With "Fade unchanged cells" on (the default), cells that are the same in both mazes drop to about 40% opacity in both, so what changed stands at full strength. It works in greyscale, puts no marks over the glyphs, and leaves the shapes readable. Unticked, both mazes show in full, to judge each as a whole. The count reads "37 of 384 cells changed", or "Nothing changed for this seed." (no fading then). The spread line sits under it.
12. **Selection and "why" work on the after maze only.** It is the current tile set. The before maze is from a tile set that's no longer loaded, so its cells don't select; hovering one says so in the readout: "Before is the old tile set, so it can't be replayed." The centre outline (DR-0029 point 9) shows on both.
13. **If either side gave up**, that side shows DR-0029 point 11's message in its box, and its measure cells read "—".

## Opening pressure

14. **Where it sits.** A line under the seed row in the Maze tab, because it shapes the maze, not the table: "Opening pressure 0.5", or "1.0 (off)" when the key is missing or 1, and the one-line meaning beside it. The number is the same edit button as point 1, with point 4's reach line and the hint "Between 0 and 1. 1 is off." under the field. Saving writes the key (DR-0038) and opens compare (point 8), so the demo is: set 0.5, save, read the measures.
15. **In the band**, once the loaded pressure is below 1: one more factor column, "× pressure", after the neighbour columns. Its cell shows the value where it applied and a muted "1" where it didn't. The formula line adds "× opening pressure, where the tile adds no new open side (a side past the maze's edge counts as new)". A click on the cell opens the single-number panel (point 2): "Opening pressure 0.5. Applied: every open side of ┘ faces a settled cell." or "Not applied: ┘ opens onto unsettled cells." At pressure 1 the column is hidden, so 2.2's band is unchanged.

## A refused edit

```
| × Corridor corner      [3.0]                 |
|   | Corridor corner weight  [ -1     ]   |   |
|   | Not saved: <the loader's reason, as   |   |   problem red, at the field
|   |   given>. core.toml is unchanged.     |   |
|   | [Save to core.toml]  [Cancel]         |   |
```

16. **At the field, not in the banner.** The editor stays open with what Ian typed, and under it, in the problem red: "Not saved: " then Rust's reason as given, then "core.toml is unchanged." Focus returns to the field. Nothing else moves: no reload, no regeneration, no compare. The shared red banner is for a tile set that won't load at all; here the loaded tile set is fine, and a banner would say the tool is broken. A failed write gets the same treatment with Rust's error. Typing clears the message; Esc puts the old value back.

## The band after a reload, and back to the table (2.2 follow-ups)

17. **The band refreshes in place** (DR-0033 point 20, now a must-have). After any successful load, if the band is showing, it fetches the trace again for the same column and row of the regenerated maze. If the tile there is the same, the band updates and the heading adds "· after reload". If it changed, the view recentres on the new tile with a new trail, and the heading adds "· after reload; was ═". A selected factor stays selected if the same candidate and column are still in the table, so after a save the panel goes on showing the number Ian just changed; otherwise it clears. If the regenerated maze gave up, the band keeps its heading and reads "No maze for seed 48213 after the reload."
18. **Until point 17 is built**, the note that replaces the band reads "The tile set was reloaded. Press Why is this here? again." (the cell stays selected in the Maze tab, so there's no need to pick it again), under the band's heading, kept so Ian still sees which cell it was about. Point 17 supersedes it; the build should only ship this if point 17 slips.
19. **DR-0026 point 11, [Show in table].** In the panel's action slot, next to "Explore this tile" or "Explore ┐", whenever the panel explains a pair, including a factor from the band. It switches to the Table tab with the direction set and the cell for (row tile, neighbour) selected and scrolled into view. Filters stay as they are; if they hide the pair, the panel still explains it and the filter bar reads "This pair is hidden by your filters. [Clear all]".

## What may slip

**Must-haves:** points 1, 3 to 9, 11, 14, 16, 17 (or 18 in its place), and point 15's column once the pressure factor is in the trace (DR-0038).

**May slip:** the weight and pressure cells in the band opening the single-number panel (point 2; the pair panel's weights cover most cases); the shared hover outline across both mazes (point 10; the readout alone is enough); "Nothing changed for this seed."; [Show in table] (point 19); a [Set back to 3.0] button on the saved line, which would just be point 3's edit with the old value. Each cut goes into the 2.3 status as a follow-up.

# Alternatives considered

- **An edit form or a separate "Tile set" tab** listing every base, weight and exception. Complete, but it follows the file, not Ian's thinking: he arrives from a bit of maze and a factor, and would have to find the same number again in a list.
- **Editing the share (15.0%) or the entry.** It's what Ian reads, but neither is stored: each comes from several shared numbers, so no edit to it has one meaning.
- **Marking changed cells with an outline or a tint.** Outlines would grid over most of a cascaded maze; tints clash with the room floor tint, and blue, amber, red and magenta are taken (DR-0017).
- **Before and after on a toggle in one place.** Flicking shows small changes well, but Ian has to compare two measure sets and point at both mazes in the demo. Side by side, with the shared hover, does both.
- **Opening pressure in the table's tile set bar.** It's in the file like the rest, but it doesn't change any table entry, so beside the table it would look broken. Beside the seed, it sits with what it changes.
- **Jumping to the Maze tab after a save.** Shows the result at once, but pulls Ian out of the band mid-thought. The band refreshes in place and the compare is one click away.
- **Range checks in the frontend for instant feedback.** Faster, but a second copy of the loader's rules that will drift.

# Consequences

- The first demo is: Why is this here?, click a factor, click its weight, type, Enter, read the band again, [See before and after], point at the changed cells and the measures.
- `table-explanation` gains edit buttons and the saved line, and is used unchanged in all three places. The Maze tab gains compare mode and the pressure line.
- The band's reload note (`RELOADED` in `why-text.ts`) changes wording (point 18), or goes once point 17 is built.
- Wren's tuning pass uses compare as is: pressure against damping corner-into-corner, one save each.

# Concerns raised

- **Damson (`ux-designer/1`):** at 16px, two mazes side by side only just fit 1024px. If the window is narrower, stack them, before above after; review should check by eye.
- **Damson (`ux-designer/1`):** fading means "the same as before" here, while in the band dimming means "came later" and while generating it means "old". They're on different screens, and the legend line names it. Review will tell whether Ian mixes them up.
- **Damson (`ux-designer/1`):** the reach line tells Ian a weight is shared, but not how many entries that is. If he's still surprised by how far a change spreads, a count from Rust ("changes 116 entries") is the next step.
- **Marram (`product-manager/1`), on review, 2026-11-01:** approved; it stays inside 2.3 (no adding or removing tiles or exceptions, undo is setting a number back via the saved line's old value, two mazes only, no batches or grown worlds) and covers all three demo tasks. Fading unchanged cells counts as "changed cells marked" for the first demo task. Scope calls, mine to make (DR-0018):
  - **No stacked layout in 2.3.** The tool's window is 1024px; build side by side only. If review by eye finds it doesn't fit, stacking is a follow-up, not part of the build. Damson's concern above stands for review.
  - **The may-slip list stands, and slipping is the default under time pressure.** Build the must-haves first, then demo; the may-slip items go into the 2.3 status as follow-ups rather than holding the slice. Point 18 ships only if point 17 slips, as written.
  - **The entry count from Rust in Damson's third concern stays out of 2.3** unless review shows Ian is surprised by a change's reach.

- **Ash (`engineer-ash/1`), on build, 2026-11-01:** buildable. Two things the views need that DR-0038's contract doesn't yet carry, raised with Yew and Rowan as additions to DR-0038 rather than changes here:
  - **The loaded opening pressure.** Point 14 shows it and prefills the editor with it, and point 15 hides the column at 1. Nothing the frontend receives carries it today. Proposed: `TableView.openingPressure` (1 when the key is absent).
  - **The base's key.** A base edit names a `[bases]` key (`passage_door`), but the panel only has the two side kinds, and turning sides into a key would copy `side.rs`'s pairing rules into TypeScript. Proposed: `EntryView.basePair`, the key, or `null` for a pair that can't meet.
  - The tile weight's reach line ("in all four orientations") counts the tile's orientations in the tile list the view already holds. That's counting rows, not working out a number, so it stays in TypeScript.

- **Damson (`ux-designer/1`), review of the build, 2026-11-01:** **pass with follow-ups**, and one README fix before Ian's demo. I reviewed from the code on `claude/slice-2-3-tune-compare` (`edit/`, `table-explanation.ts`, `maze-view.ts`, `compare-text.ts`, `why-band.ts`, `explore-view.ts`'s refresh, `maze-tool-app.ts`'s 2.3 block), its tests, and README's 2.3 section. There's no display, so the layout points below are for checking by eye.
  - **Built as designed.** Points 1, 3 to 17 are built, including two may-slip items: the shared hover outline across both mazes and "Nothing changed for this seed.". Some details I checked: the editor prefills Rust's exact value; the reach lines are in words; Save is enabled only for a different number; a refusal stays at the field in red with `role="alert"`, typing clears it, and nothing reloads; the saved line survives the refetch of the same pair; no tab switches by itself; the band refreshes in place and keeps the factor. In compare, the labels use words, unchanged cells fade at 0.4 in both mazes, the measures sit in a before → after table, only the after maze selects, and the centre shows on both. The pressure column shows "× 0.5" or a muted "1", and the formula line carries the pressure clause. No sums are worked out in TypeScript.
  - **Deviations I accept.** Esc closes the editor rather than restoring the old value. The effect is the same, because the line still shows the old number. When the tile changes after a reload and Ian has already walked on from the first stop, the build keeps his trail and only replaces the first stop, instead of starting a new trail. That's better for him than point 17 as written.
  - **Must fix before the demo (README, demo step 2).** Wren set `opening_pressure = 0.5`, so the line now reads "0.5", not "1.0 (off)". If Ian types `0.5` and presses Enter, Save stays disabled and nothing happens. Change step 2 to: "it reads 0.5, Wren's setting. Type `1` and press Enter to see the maze without it, then `0.5` again to see it back." Also say the × pressure column is hidden at 1.
  - **Follow-ups (2.3 status), besides Ash's three cuts:**
    - (a) After Save or Cancel, focus is lost with the closed editor. It should return to the number's button.
    - (b) Saving the pressure from the Maze tab with no maze shown gives no confirmation beyond the new value. A saved line like point 7's would help.
    - (c) `RELOADED` in `why-text.ts` still has the old wording. It's only reachable if no revision is known, so it's harmless.
  - **By eye on Windows, at 1024×768:**
    1. **Two mazes side by side.** By my arithmetic they fit: 48ch at 16px in Cascadia Mono is about 452px each, so about 920px with the gap. That leaves roughly 940 to 960px after the 1.5rem padding and a vertical scrollbar, which will appear because the compare is taller than 768. That's only 20 to 40px to spare. Check there's no horizontal scroll. If there is, it's Marram's stacked-layout follow-up.
    2. Box-drawing lines stay aligned at 16px in both mazes.
    3. At 40%, the faded cells are still readable, and the changed cells clearly stand out, including hedge backgrounds.
    4. A selected cell that didn't change is faded too, so its magenta outline drops to 40%. Check it's still visible. If not, a follow-up: don't fade the selected cell.
    5. The twin outline shows in the other maze on hover.
    6. In the Explore tab, the band with three or four counted neighbours plus × pressure fits beside the 20rem panel without clipping.
    7. The editor fits inside the 20rem panel, including an exception's reach line with its rule.
    8. My second concern still stands. Watch whether Ian reads "faded" in compare as the band's "came later".

# Why it's a two-way door

It's views and controls in the tool. The file format and the edit record are DR-0038's; nothing else builds on this layout.

# AI guidance

- Prefill the editor with Rust's exact value, never the formatted one.
- Don't check ranges or completeness in TypeScript; show the loader's reason.
- Don't work out differences, scores or shares in TypeScript; format what Rust returns.

# Links

- Roadmap, slice 2.3: [roadmap](/roadmap.md)
- Editing `core.toml`, the commands and the trace field: [DR-0038](/decisions/0038-editing-core-toml.md)
- Explanation panel: [DR-0025](/decisions/0025-table-view-design.md)
- Explore view, point 11: [DR-0026](/decisions/0026-explore-one-tile-design.md)
- Maze tab: [DR-0029](/decisions/0029-generation-view-design.md)
- Measures: [DR-0030](/decisions/0030-measures-for-2-1.md)
- "Why is this here?" and its follow-ups: [DR-0033](/decisions/0033-why-is-this-here-design.md)
- Opening pressure, rule 9: [DR-0036](/decisions/0036-growth-from-open-endpoints.md)
- Weight model: [DR-0022](/decisions/0022-weight-model.md)
- Glyphs and colours: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
