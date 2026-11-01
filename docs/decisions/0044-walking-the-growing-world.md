---
type: Decision Record
title: "G.1: walking and looking through a growing world"
description: "What the player does and sees in G.1: one cell per key press, blocked only by hedge or the unknown, starting at the origin, seeing every stored cell within 6 cells over the hedges, with a trodden trail on visited cells; what stays the same on walking back, what makes it fun, and the checklist Wren judges the demo against."
status: stable
decision_id: DR-0044
decision_status: accepted
decision_kind: game-design
major: false
owners: [game-designer/1]
approvers: [product-manager/1]   # game-designer/1 is the author and owner, so can't approve; Marram approves (DR-0018)
supersedes: []
tags: [game, slice-g-1, movement, view, discovery, game-design]
generated:
  by: game-designer/1
  at: 2026-11-01T10:23:30Z   # corrected by facilitator/1: the time written was later than the approval
verified:
  - by: product-manager/1
    at: 2026-11-01T10:24:34Z
    note: "Approves as written. In G.1's scope: the trail is small and the checklist gives the fun review a clear verdict. Roadmap G.1 updated to match rule 5 (the unknown is not drawn as stored hedge)."
---

# Context

Ian (`human:ian`) started the game alongside Phase 2 ([DR-0042](/decisions/0042-the-game-starts-now.md)). G.1, "Walk a growing world": a player walks through a world grown around them from open endpoints along their path ([DR-0036](/decisions/0036-growth-from-open-endpoints.md)), and walking back finds it as it was ([DR-0035](/decisions/0035-the-world-is-stored.md)). On 2026-11-01 Ian added, verbatim: "don't forget we have a game designer to ensure looking through the maze (based on our generator) is fun." This record says what the player does and sees, and what makes it fun. How it looks on screen and which inputs it uses are Damson's, in DR-0045.

# Decision

1. **Moving.** One press moves the player one cell north, east, south or west. Arrow keys and WASD both work. There are no diagonals and no turns to manage. Holding a key keeps walking at a steady pace. Damson sets the pace and any other inputs in DR-0045. Each step is one `step(direction)` call.
2. **What blocks.** The player can step into a neighbouring cell when it is stored and the two sides facing each other are both walkable. That is the same "joined" rule as `measures.rs` and DR-0036: any side except hedge. Passage, door, floor and floor with a wall are all walkable. Two doors back to back are one doorway you walk through ([DR-0015](/decisions/0015-tile-set.md)). Doors in G.1 are openings: nothing opens, locks or costs anything. Hedge blocks, and so does an unsettled cell. Walking into a hedge costs nothing: `moved` is false and the player stays where they are. Damson decides the feedback, which should be a soft bump and not an error.
3. **Start.** The player starts on the origin cell, which is the first patch's anchor and always has a walkable side (DR-0036 rule 2). Before the first frame, growth runs until no anchor is within r. So the first thing the player sees is already a place with ways out.
4. **What the player sees.** Every stored cell within **6 cells** (Chebyshev, so a 13 × 13 square) is shown, seen from above, over the hedges, with no line-of-sight fog. A hedge maze seen from above is the fantasy: you glimpse a room past the hedge, and the fun is finding the way in. The view radius v is a setting that must stay below DR-0036's r (8 today). That way growth mostly happens out of sight.
5. **The unknown.** An unsettled cell is not shown as a stored hedge. It reads as the unknown, the hedge beyond. Damson picks the look, and it may be hazy hedge. Yew's concern 3 on DR-0036 is why: an unsettled cell drawn as solid hedge can later turn into a path. If the player had been shown real hedge there, the promise in rule 7 would break. "Not yet known" turning into a path doesn't break it, and it rewards coming back.
6. **Visited cells are marked** with a faint trodden trail on every cell the player has stood on. The trail is there for orientation and comfort, so the player can see "I've been this way" and doesn't get lost in a way that isn't fun. It isn't a score. Damson picks the look. *2026-11-01, Wren accepts Damson's change for G.1:* the trail shows only on corridor cells, as warm trodden earth, so the room tint stays clean. A room is seen whole, so orientation holds. At the demo, Wren checks that the player can still tell which of a room's doors they've already used.
7. **Walking back finds it as it was.** Every stored cell keeps its tile and orientation forever, and the trail stays. A room you passed is the same room when you come back. Only unknown cells can become new paths. The player sees no cell they've already seen change.
8. **Not in G.1:** loot, goals, scores, quests, other characters, danger or hunger, doors that open or lock, sound, a remembered map or minimap of cells outside the view, and zones. Saving and loading is G.2. Debug views and overlays are G.3.

## What makes it fun (the game-design intent)

- **Something worth seeing ahead.** In most views there should be a choice in sight: a junction, a doorway, or a room past the hedge. Long stretches with nothing but corridor in view are tedium.
- **Rhythm.** Walk, choose, walk ([DR-0015](/decisions/0015-tile-set.md)'s "in that place"). Junctions should be spaced apart, not a graph-paper grid.
- **Rooms feel like finds.** They are rare enough to be an event, and they can be entered. A room you can see but never reach is the worst kind of glimpse.
- **Dead ends are short and cheap.** A dead end you can see from the junction costs nothing. One you walk ten cells into costs patience. Ways onward should always outnumber dead ends in view.
- **The world never feels like it's assembling.** Growth should be out of sight, and the player shouldn't catch cells appearing in the middle of their view.

## Demo checklist (Wren judges the built G.1 against this, over several seeds, before Ian's demo)

1. From the start, at least two ways to go are in view.
2. A room comes into view within roughly 30 to 60 steps of walking, and can be entered.
3. Most views (more than half the steps) show a choice: a junction, a doorway or a room.
4. Dead ends: few longer than 3 cells, and none that need a long walk back.
5. The player is never boxed in (DR-0036 rule 6 holds in play).
6. No cell already seen changes on walking back, and how often unknown cells turn into paths inside the view.
7. "Just one more corner": after a few minutes, do I still want to keep going?

Requests to change the generator (weights, opening pressure, P, r, how growth meets the player) go in that review as requests for the 2.3 and 2.6 sessions, and not into `maze-core` from the game branch.

## What this teaches the 3D game (added 2026-11-01)

Ian (`human:ian`), 2026-11-01, verbatim: "The very long term intent is to use this game as a minimal tech demo of a given approach (infinite maze) - we will later make a different 3d game (probably in unity) based on it - but that will need human designed art assets etc. Its impact here is just to keep in mind while we are making a 2d game, part of it beyond being fun is to learn what we bring to the 3d game, and hopefully have a maze generation algorithm in that can be attached to that unity game running in its own thread and working with the unity game."

- **Fun that doesn't depend on art is in the layout:** the walk-choose-walk rhythm, rooms that feel like rare finds, short dead ends, a way onward that's always there, and a world that stays as you left it. A 3D game inherits all of this from the generator.
- **What won't carry over is the view from above.** A player walking at ground level in 3D can't see over the hedges, so rule 4's glimpse of a room past the hedge goes. The fun review asks which joys come from the layout itself and which come only from seeing over the hedges. It also asks whether growth ever showed itself, because in 3D it has to stay out of sight. Requests for the generator follow from those answers. The review's checklist carries this same heading.

# Why it's a two-way door

Nothing persists in G.1. Keys, view radius, the trail and how the unknown looks are settings or drawing. Changing any of them later costs a small edit. The movement rule is DR-0036's existing "joined" rule, so it adds nothing new to reverse.

# Alternatives considered

- **Show the whole stored world.** It gives away what lies ahead and shows growth happening. That kills discovery.
- **Show only where the player has been (fog of war), or use line of sight.** It's tense, and you can't glimpse a room over the hedge, so wandering turns into groping in the dark. That's closer to horror than to cozy. Line of sight is a fair later option for some zones.
- **Draw unsettled cells as stored hedge** (as the tool does, DR-0036 rule 2). It's simpler, but a "hedge" can then become a path, and that breaks rule 7.
- **No trail.** It's cleaner to look at, but in an endless maze with no map the player gets lost quickly.

# Consequences

- The engineers need three things now: rules 1 and 2 for `step`, rule 3 for `start_game`, and rule 4's radius of 6 for the cells `step` returns.
- `visited` is per-cell game state beside the tile (DR-0035), set when the player stands on a cell.
- The stub returns stored cells only. Anything else in the view is the unknown (rule 5).

# Concerns raised

- **Wren (`game-designer/1`), the author:** a patch's window can reach sideways back toward the player. So new cells can appear inside the view even when v is below r (DR-0036 rule 3: a north-facing anchor's window runs 8 cells across). Checklist item 6 measures it. If it shows often, the fix is to ask the 2.6 session to clip windows to stay out of the view. Raising v or r is not the fix.

# Links

- [DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md), [DR-0042](/decisions/0042-the-game-starts-now.md)
- Tile set and side rules: [DR-0015](/decisions/0015-tile-set.md); tool glyphs: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md); connection rule: [DR-0030](/decisions/0030-measures-for-2-1.md)
- Global properties: [Q-0008](/questions/open/0008-global-maze-properties.md); roadmap: [roadmap](/roadmap.md), G.1
