---
type: Decision Record
title: How a maze cell opens the explore view and shows why its tile is there
description: Click selects a maze cell and "Why is this here?" (or double-click) opens the Explore tab on its tile through DR-0026 point 10, with a band above the compass that replays the cell's draw - which neighbours were settled, each candidate's weight times its pair factors, its chance, and the one drawn.
status: stable
decision_id: DR-0033
decision_status: accepted
decision_kind: design
major: false
owners: [ux-designer/1]
approvers: [engineer-ash/1, product-manager/1]   # engineer-ash/1 stands in for the author, ux-designer/1 (DR-0018)
supersedes: []
tags: [tool, generation-view, explore-view, maze, explanation, phase-2, slice-2-2]
generated:
  by: ux-designer/1
  at: 2026-11-01T08:07:24Z
verified:
  - by: product-manager/1
    at: 2026-11-01T08:10:16Z
    note: "Approved, with one scope change: the ruled-out list is out of 2.2 and only its count stays (Concerns raised, Marram, on review)."
  - by: engineer-ash/1
    at: 2026-11-01T08:24:13Z
    note: "Stands in for the author, Damson (DR-0018). Approves; built on ash/2.2-explore-view against DR-0034's types. Build calls for Damson's review are under Concerns raised (Ash, on build)."
---

# Context

Slice 2.2 ([roadmap](/roadmap.md)): "Ian clicks a bit of maze he doesn't like and sees why it's there", plus 1.2's two demo tasks in the explore view ([DR-0026](/decisions/0026-explore-one-tile-design.md)), built in this slice. Ian's loop is: see a problem in a maze, look at the numbers behind it, change them. He's likely to click what Wren's [2.1 review](/plans/2.1-maze-review.md) found: long one-deep room bars, sealed rooms and small rings of corners.

At that moment Ian is looking at a picture with all four neighbours filled in. WFC didn't see that picture. When it drew the cell, only some neighbours were settled, and those, with the tile weights, are all the score used ([DR-0028](/decisions/0028-wfc-approach-for-2-1.md) rule 3). The rest came later and were drawn to fit this cell. The view has to make that gap obvious, or Ian will read the wrong cause and edit the wrong number.

# Decision

Damson (`ux-designer/1`). The meaning of "how its neighbours combined" is the **collapse-time trace**: what WFC saw and scored when it drew this cell, replayed by Rust from the seed. The numbers in the sketches are illustrative.

```
MAZE TAB
| [ Table ] [ Explore ] [#Maze#]     tile set bar, as DR-0029                    |
| Seed [ 48213 ] [Generate] [Random seed]                                       |
| Seed 48213 · 24×16 · first try                    | measures, as DR-0029      |
| |░░░░┌───┐ ░░╔═══════════╗░░  |                   |                           |
| |░░░░│   └───╢ · · · · · ║░░  |  <- a one-deep room bar                       |
| |░░░░╵ ░░░░░░╚══[══]═════╝░░  |  <- [══] selected: magenta outline            |
| Room wall, wall S · col 15, row 2                     <- hover readout (2.1)  |
| Selected: ═ Room wall, wall S · col 14, row 2   [Why is this here?]          |
```

```
EXPLORE TAB, opened from that cell
| [ Table ] [#Explore#] [ Maze ]     tile set bar, as DR-0029                               |
| [< Back to maze]  Trail: ═         Centre: [ ═ Room wall, wall S  v ]                      |
+---------------------------------------------------------------------------+---------------+
| WHY ═ IS HERE    seed 48213 · col 14, row 2 · drawn 41st of 384           | DR-0025 panel |
|                                                                           | for the       |
|                 · north: came later                                       | selected      |
|   west: ═     [ ═ ]     ═ east: came later                                | factor, or a  |
|   counted                                                                 | neighbour row |
|                 ░ south: counted                                          | in an arm     |
|                                                                           |               |
|   Candidate               weight  × south ░  × west ═  = score   chance   | ═ then to its |
|   ═ Room wall, wall S       1.0   × [1.06]   × 0.97    = 1.03    58.9%    | south ░ ...   |
|                                                         drawn             | Hedge-Hedge   |
|   ╝ Room corner, SE         0.7   × 0.93     × 1.10    = 0.72    41.1%    | base 1.0 ...  |
|   + 27 ruled out before the draw                                          |               |
|                                                                           |               |
|   score = tile weight × one pair factor per counted neighbour (side-pair  |               |
|   base × exceptions × noise; tile weights left out). chance = score ÷     |               |
|   1.74, the sum of the scores. Neighbours that came later were drawn to   |               |
|   fit ═, so they don't count here.                                        |               |
+---------------------------------------------------------------------------+               |
|   DR-0026's compass for ═, unchanged, except each arm marks the tile      |               |
|   the maze has there:  ═ Room wall, wall S   Floor, wall at S end  31.2 in maze [Go]      |
```

The demo reading: along the bottom wall of a bar, a wall beats a corner about 3 to 2 at every step, so walls run on. Clicking a factor shows which base, weight or exception to edit.

## What the build must do

**In the Maze tab** ([DR-0029](/decisions/0029-generation-view-design.md))

1. **Select by click.** A click selects a cell: a 2px magenta outline, the selection colour of [DR-0017](/decisions/0017-tile-glyphs-and-colours.md) and [DR-0025](/decisions/0025-table-view-design.md). If it is also the centre cell, the magenta outline shows on top.
2. **A "Selected" line** under the hover readout: "Selected: ═ Room wall, wall S · col 14, row 2 [Why is this here?]". With nothing selected it reads "Click a cell, then ask why it's there.", and the button is disabled. The hover readout is unchanged.
3. **Open** with [Why is this here?], a double-click on a cell, or Enter when the map has focus. This switches to the Explore tab.
4. **The selection lasts.** It survives tab switches, because views hide rather than close (DR-0029 point 1). It also survives Reload's regeneration: the same column and row stay selected, whatever tile is there now. Generate and Random seed clear it.
5. **Keyboard:** the map is focusable, and arrow keys move the selection.

**Opening the Explore tab**

6. **Three tabs: Table, Explore, Maze.** Table still opens first. Explore is built to DR-0026.
7. **Through DR-0026 point 10**, with the cell's (tile type, orientation), nothing selected in the arms, a new trail, and one new optional argument: the maze cell (the maze's seed, size and revision, and the cell's column and row). Only when that argument is given does the view show the band below. The Table's "Explore this tile" and the Centre picker don't pass it.
8. **Back to the same maze.** The Maze tab shows the same maze with the cell still selected. When the trail began at a maze cell, [< Back] at the trail's first stop reads "< Back to maze" and switches to the Maze tab.

**The "Why it's here" band**, above the compass in the left column. DR-0025's panel stays on the right, where DR-0026 put it.

9. **Heading:** "Why ═ is here · seed 48213 · col 14, row 2 · drawn 41st of 384". The band shows while the centre is the trail's first stop. [Go] away hides it, and [< Back] brings it back.
10. **The cross:** the cell and its four neighbours as in the maze, with DR-0017 glyphs at 20px or more. Each neighbour is labelled in words: **counted** (settled before the draw), **came later** (also dimmed), or **edge** (blank, because nothing past the edge counts; DR-0028 rule 6).
11. **The candidate table:** one row per tile still possible at the draw, sorted by chance, highest first, with ties in table order (Rust sorts them). The columns are:
    - glyph and name;
    - weight;
    - one "× direction glyph" column per counted neighbour, in N, E, S, W order;
    - "= score";
    - "chance", with one decimal and a % sign, and "<0.1%" below 0.05 as in DR-0025 point 5.

    The drawn row is bold and says "drawn" in words. It never uses magenta (kept for selection) or red (kept for problems).
12. **The foot:** "+ 27 ruled out before the draw" opens, below a divider, the ruled-out tiles with their factors. A 0 factor shows the clash with that neighbour. A row with no 0 was ruled out by cells further away, and says so. Under it goes the formula line from the sketch, with the sum of the scores.
13. **Select a factor** by click: it gets a magenta outline, and DR-0025's panel, unchanged, explains that pair. The candidate is the row tile, the counted neighbour is the column tile, and the direction runs from the cell to the neighbour. Selecting a factor clears any arm selection, and selecting an arm row clears the factor.
14. **"in maze" in the arms:** each arm marks, in words, the tile the maze has on that side. Every neighbouring pair in a generated maze is above 0 (2.1's test), so the mark is always in the visible part of the arm.

**Edge cases**

15. **No counted neighbours** (the first cell, or any cell drawn before anything around it settled): there are no factor columns, and the band says "Nothing around it was settled yet, so only tile weights counted."
16. **One candidate left**, which is where the fallback crossroads and dead ends usually come from: "Only one tile could fit here, so the draw had no choice." Its chance reads 100%. The ruled-out list (point 12) then holds the answer.
17. **Hedge:** nothing special. Hedge is a tile with a weight and factors like any other.
18. **A failed maze** draws no cells (DR-0029 point 11), so there is nothing to pick. If a later slice draws the partial grid (2.4), an empty cell can be selected, but [Why is this here?] is disabled: "Nothing was placed here: the attempt stopped first."
19. **Restarts:** the trace comes from the attempt that made the maze on screen.
20. **Never a stale trace.** The band only shows a trace from the maze's own tile set revision. After a Reload the maze regenerates (DR-0029 point 3), and the band fetches the trace again for the same cell. If the tile there changed, the view recentres on the new tile (a new trail), and the heading adds "after reload; was ═". If the revision doesn't match, Rust says so, and the band reads "The tile set was reloaded. Pick the cell again in the Maze tab."

## What the view needs from Rust (point 3; DR-0034 sets the shape)

The frontend formats and never recalculates: no score, factor, sum or chance is multiplied or divided in TypeScript (DR-0025, DR-0026).

- **Request:** the maze's seed, width, height and revision, and the cell (x, y).
- **Revision:** the revision used. If it isn't the loaded one, a readable stale error instead of a trace.
- **The cell:** its tile index, the same as the maze's cell, and its step in the collapse order (k of width × height).
- **Per direction (N, E, S, W):** one of *counted*, with the neighbour's tile index as it was then; *later*, with its tile index in the finished maze; or *edge*.
- **Candidates still possible at the draw**, sorted by chance (highest first, ties in table order). For each: the tile index, the tile weight, the pair factor for each counted direction (base × exception multiplier × noise), the score, the chance (score ÷ sum), and a drawn flag. Exactly one is drawn, and it is the cell's tile.
- **The sum of the scores.**
- **The ruled-out tiles**, in table order, with the same per-direction factors (0 where it clashes), and their count.
- **For each factor,** what DR-0025's panel needs for that pair (direction, candidate as row, neighbour as column): an `EntryView`, or a way to fetch one.

Suggested tests: for every cell of several seeds, the replayed tile equals the maze's cell, the drawn candidate is it, its chance is above 0, and the chances add up to 100 within rounding.

## The demo, and what may slip

**Must-haves:**

- In the Maze tab: click to select with the magenta outline, the Selected line with [Why is this here?], and double-click to open.
- DR-0026's must-haves.
- Opening on the cell's tile through point 10.
- The band:
  - the heading;
  - the cross with counted, came later and edge in words;
  - the candidate table (weight, factors, score, chance, drawn);
  - the ruled-out count;
  - the formula line;
  - the wording for points 15 and 16.
- A factor click opens DR-0025's panel.
- The Maze tab keeps the maze and the selection.
- Never a stale trace: clearing the band with point 20's note is enough.

**May slip:**

- the keyboard on the map (points 3 and 5);
- "< Back to maze" (point 8; the Maze tab is enough);
- "drawn 41st of 384";
- the ruled-out list (the count stays);
- "in maze" in the arms;
- refreshing the band in place after Reload (clear it with the note instead);
- dimming the later neighbours (the words stay).

# Alternatives considered

- **"What would fit here now":** scoring each candidate against all four final neighbours. It reads more easily against the picture, since it uses everything Ian sees. But it isn't why the tile is there. A neighbour that came later was drawn to fit this tile, so counting it explains the tile by its own consequences. And a high "now" score for another tile suggests it belongs there when it never had a chance. For tuning, it points at the wrong number. It could come later, in 2.3 or 2.4, as "would this cell change if it were drawn again?".
- **Opening on a single click.** One click fewer, but cells are about 20px, and a mis-click throws Ian into another tab. Selecting and then opening matches DR-0026's select-and-[Go].
- **The band in the Maze tab instead,** beside the maze. It keeps the picture in view, where the question started. But the roadmap's link is to the explore view, the panel and the arms live there, and the cross carries enough of the picture. The band is one component, so if review finds Ian flipping back and forth, moving it is cheap.
- **Showing where the random draw landed.** It explains which of two close tiles won, but Ian can't tune the roll; he can tune the chance.

# Consequences

- The demo: Ian selects a wall in a long room bar, presses [Why is this here?], reads that a wall beat a corner about 3 to 2, and clicks a factor to see what to edit. 1.2's two tasks run in the same Explore tab.
- 2.4's step-through can reuse the band: it is the same trace, at one step.
- Once Reload refreshes the band in place, it serves 2.3's loop: edit, reload, then ask "why is it here now?".

# Concerns raised

- **Damson (`ux-designer/1`):** "came later" is the hard idea here. If review shows Ian reading the band as "these four neighbours decided it", the cross's labels aren't enough. 2.4's view of the collapse order is the next step.
- **Damson (`ux-designer/1`):** there are two percentages on one screen: chance in the draw (the band) and share of the centre's neighbours (the arms). They have different labels, and only chance carries a % sign. Review will tell whether that's enough.
- **Damson (`ux-designer/1`):** for a fallback tile, the answer is in the ruled-out list. If that slips, fallbacks only get "the draw had no choice". That's acceptable for the demo, because what Ian is likely to click (bars, sealed rooms, rings) is explained by the scores.
- **Marram (`product-manager/1`), on review:** approved. The must-haves are the smallest set that takes Ian from a bit of maze he doesn't like to the number behind it: select, open, the band's numbers, and a factor click into DR-0025's panel. The may-slip list stands. One scope change, a two-way door and mine to make ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)), and three points for the build, not a new round:
  1. **The ruled-out list (point 12) is out of 2.2, not just may-slip.** [DR-0034](/decisions/0034-data-contract-for-why-is-this-here.md) sends only the count (`ruledOut`), so building the list would mean changing the contract mid-build. The count and point 16's wording stay, and so does the "What the view needs from Rust" bullet as the brief for later. Revisit at 2.4, which shows propagation, or sooner if Ian clicks fallbacks in the demo and asks why they're there.
  2. DR-0026's may-slip list, with what I added on its review, still applies to the explore view.
  3. The slice can land with any may-slip item cut. Each cut goes into the roadmap's 2.2 status as a follow-up, not a blocker.
  4. Damson's first concern is what to watch in Ian's demo. If he reads the four neighbours as the cause, the answer is 2.4's view of the collapse order, not reworking 2.2.

- **Ash (`engineer-ash/1`), on build, for Damson's review:** tabs read Table, Explore, Maze, and Explore opens on the Centre picker with a hint rather than a default tile. 0 rows have no [Go], because that edge can't be crossed. Arm rows take two lines (name, then "faces with…") so the compass fits the 1024 px window. A lone candidate shows "100.0%", keeping one decimal. Slipped from the may-slip list: keyboard on the map, refreshing the band in place after Reload (it clears with a note instead), and "after reload; was ═".

# Why it's a two-way door

It is one band and one link in the tool. The trace is replayed on request, not stored, and nothing else depends on it.

# AI guidance

- "Counted" means settled before this cell was drawn, not present in the finished maze.
- Don't compute scores, sums or chances in TypeScript. Format what Rust returns.

# Links

- Roadmap, slice 2.2: [roadmap](/roadmap.md)
- Explore view and entry point: [DR-0026](/decisions/0026-explore-one-tile-design.md)
- Maze tab: [DR-0029](/decisions/0029-generation-view-design.md)
- Explanation panel: [DR-0025](/decisions/0025-table-view-design.md)
- Glyphs and colours: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
- WFC rules 2 to 4, the score: [DR-0028](/decisions/0028-wfc-approach-for-2-1.md)
- Measures and centre: [DR-0030](/decisions/0030-measures-for-2-1.md)
- Wren's 2.1 review: [review](/plans/2.1-maze-review.md)
