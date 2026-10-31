---
type: Decision Record
title: The core tile set
description: Ten tiles in three kinds (corridor, room, hedge), giving 29 (tile, orientation) pairs, built on five kinds of side that say which tiles may sit next to each other.
status: stable
decision_id: DR-0015
decision_status: accepted
decision_kind: game-design
major: false
owners: [game-designer/1]
approvers: [game-designer/1, product-manager/1]
decided_at: 2026-10-31T18:45:00+01:00
supersedes: []
tags: [tile-set, wfc, adjacency-table, game-design]
generated:
  by: facilitator/1
  at: 2026-10-31T15:15:00+01:00
verified:
  - by: game-designer/1
    at: 2026-10-31T16:10:00+01:00
  - by: product-manager/1
    at: 2026-10-31T18:45:00+01:00
---

# Context

WFC runs over a tile set, and the adjacency table relates every (tile, orientation) to every other ([DR-0007](/decisions/0007-wave-function-collapse.md)). Phase 1 builds that table and a tool to explore it ([DR-0008](/decisions/0008-adjacency-tool-first.md)). Wren (`game-designer/1`) designs the tile set from Ian's vocabulary, with crossroads and dead ends as very-low-probability fallbacks ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md)). The table's entries come mainly from the tiles' sides, with probability 0 where sides don't pair up ([DR-0013](/decisions/0013-table-entries-from-tile-sides.md)).

Ian's words:

- "unicode characters to represent possible tile topology (tunnel, turn, t intersection, open room, corner room, etc)" and "color (room/hallway)";
- on Q-0005, he wants to "filter by tile type (room, corridor, corridor corner)".

The game is a cozy, hedge-maze dungeon crawler: rooms should feel like places worth finding, and corridors should lead somewhere.

This record assumes the square grid of [DR-0014](/decisions/0014-square-grid.md): four sides (N, E, S, W) and 90° rotations only.

# Decision

## Kinds

Every tile has a **kind** and a **tile type**. The kinds are **Corridor**, **Room** and **Hedge**. "Corridor" is Ian's latest word, so the team and the tool use it in place of "hallway". Hedge is the solid space between paths.

## Kinds of side

Each side of a tile is one of five kinds:

| Side | Meaning |
|---|---|
| Hedge (H) | Closed. Nothing gets through. |
| Passage (P) | A narrow path through the middle of the side. |
| Door (D) | A room's doorway, opening outwards. |
| Floor (F) | Room floor, open across the whole side. The room carries on. |
| Floor with a wall at one end (F·wN, F·wE, F·wS, F·wW) | Room floor, but the room's wall reaches this side at the named end. F·wN: the wall is at the north end. |

Which sides may meet:

| Side | Meets |
|---|---|
| Hedge | hedge |
| Passage | passage or door |
| Door | passage (the likely case) or another door (less likely: one room opening straight into the next) |
| Floor | floor |
| Floor with a wall at one end | the same kind, with the wall at the same end |

Every other pair of sides has probability 0 ([DR-0013](/decisions/0013-table-entries-from-tile-sides.md)). These rules are invariants: whatever sets the numbers, no table may break them, in any zone. As game rules they mean:

- an opening never faces a wall;
- corridors enter rooms only through doorways, and a doorway always sits in a straight run of wall;
- room walls run straight, so a wall at the north end never meets one at the south end;
- a room's outer wall can face hedge, a corridor's closed side, or another room's wall.

Ian confirmed these on Q-0007: "rooms can only join at a door", a path leads into a room only through a door, and a room's open side meets only another room's open side. So a door meets another room's door, never its floor, and rooms stay rectangular. Two doors back to back become one doorway through game logic at play time, not at generation.

## The tiles

Names are kind first, then shape, as in Ian's "corridor corner", except Open room, which keeps Ian's word order. Edges are listed N, E, S, W in the base orientation (north up, turning clockwise, as description only).

| Tile type | Kind | Base orientation | Edges N, E, S, W | Orientations | Tags | Ian's word |
|---|---|---|---|---|---|---|
| Corridor straight | Corridor | runs N–S | P H P H | 2 | | tunnel |
| Corridor corner | Corridor | open N and E | P P H H | 4 | | turn, corridor corner |
| Corridor T intersection | Corridor | closed S | P P H P | 4 | junction | t intersection |
| Corridor crossroads | Corridor | open on all sides | P P P P | 1 | junction, fallback | cross-roads |
| Corridor dead end | Corridor | open N | P H H H | 4 | fallback | dead ends |
| Open room | Room | | F F F F | 1 | | open room |
| Room wall | Room | wall along N | H F·wN F F·wN | 4 | room edge | plain room edge |
| Room corner | Room | NE corner | H H F·wE F·wN | 4 | room edge | corner room |
| Room doorway | Room | door in the N wall | D F·wN F F·wN | 4 | room edge | doorway, room door |
| Hedge | Hedge | | H H H H | 1 | | |

That is **29 (tile, orientation) pairs**: 15 corridor, 13 room, 1 hedge. Each pair is a row and a column of the table, so the table has 29 × 29 entries for each of the four sides (3,364 in all). The side rules allow 916 of them (229 per side), and the rest are 0.

- **Orientations** are the distinct 90° rotations of each tile. There are no mirror images.
- **Tags** group tiles across types for filtering. A tile can carry more than one, and later tiles get their tags when they are added: *fallback* (crossroads, dead end), *junction* (T intersection, crossroads), and *room edge* (wall, corner, doorway).
- "Corner room" is read as the corner cell of a room made of several cells. A one-cell room in a corner is the alcove, a later addition.

## Rooms

A room that closes is a rectangle at least 2 cells by 2. With walls that run straight and no inner corners, local rules can't make any other shape. Nothing local stops a room growing without end (Q-0008, room size). Rectangles only is a choice for now: L-shaped rooms come with an inner-corner tile, added later and on purpose if rectangles feel samey.

Local rules can't promise anything bigger than one cell's neighbours: a door into every room, room sizes, short dead ends, or a way onward from anywhere. Those are global properties, tracked as their own question ([Q-0008](/questions/open/0008-global-maze-properties.md)). Tiles are not added to force them. Ian allows rooms that can't be entered for now, to be revisited later (Q-0007).

## How often tiles appear (intent only)

This record says what each tile is for. How the numbers are produced, and the numbers themselves, belong to the weight model. Ian wants a good-enough maze now, tuned later by review in the tool, "not perfect ahead of time".

- **Rooms are rare, and paths are more likely near other paths** (Ian, Q-0007).
- **A room's closed side** is "equally likely to be another room or a path" (Ian, Q-0007). Wren reads this as room versus corridor, with how often a room's wall faces hedge left to the team, as hedge is.
- **Rare fallbacks** (Ian, [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)): corridor crossroads and corridor dead end, and the later variants. They are there to fill gaps where nothing else fits. Wren: the inner corner, when added, is rare for variety rather than a fallback.
- **Room walls are common among room edges, and doorways are the exception** ("room walls are not rare", Ian, Q-0007). Rooms themselves are still rare.
- **Hedge is common** (the team's call, which Ian left to us). Hedge spaces the paths out, so the maze is paths through hedge rather than a dense labyrinth.
- **"In that place"** (Wren's game intent, for the weight model to test against; the default zone, if zones come later):
  - junctions spaced apart: walk, choose, walk;
  - a doorway likeliest facing a corridor straight, like a path up to a gate, and rarely facing a dead end;
  - doors spaced along a wall: doorway beside doorway rare, wall beside wall common;
  - dead ends rarely beside a doorway or another dead end.

## Later additions

Considered for after phase 2's sample mazes, each at low probability:

- inner corner, for L- and T-shaped rooms;
- corner doorway, which comes in left- and right-handed versions, so it would be two base tiles;
- alcove, a one-cell room with a doorway;
- gallery and room end (rooms one cell wide);
- landmarks (fountain, tree), preferably as decorations rather than table rows;
- a see-through hedge side.

## Not decided here

- How tiles are drawn in the tool: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md).
- How probabilities are produced (tile weights, per-pair exceptions, the variation Ian wants to see in the table, and future zones) and the numbers: the weight model, with [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md).
- Tile IDs, the format of the tile set definition, and which way rotation 0 faces in the data: Yew's data-format record.
- Global properties and how they are enforced: [Q-0008](/questions/open/0008-global-maze-properties.md).

# Alternatives considered

- **The 19-row starter set described in Q-0004:** corridor straight, corner and T, open room, room corner and doorway. No hedge and no plain room wall. Without a wall, rooms bigger than 2x2 are ringed with doorways. Without hedge, every cell is walkable and the maze becomes a dense labyrinth, and a cell closed on all four sides has no tile at all.
- **One plain floor side, not handed.** Simpler, but a north wall could meet a south wall, giving stepped rooms nobody chose (Yew).
- **A doorway whose outer side is a plain passage.** Fewer side kinds, but "a door meets a path" and "a path meets a path" become the same pair, so DR-0013's "a room door is more likely to match a path" couldn't be expressed from sides. If the weight model holds a weight per tile pair, Door isn't needed for weighting. It stays so that "what can a door face?" reads clearly (Wren).
- **L-shaped rooms from the start.** More interesting rooms, but the shape would come from WFC by accident rather than by choice. Kept for later.
- **Extra side kinds to make tiles rarer**, such as a special dead-end passage. Side kinds say what can meet, not how often; mixing the two multiplies them (Yew).

# Consequences

- The table has 29 rows and columns per side, small enough to read in the first demo, and it grows with every tile added.
- Rooms are always rectangles of at least 2x2, until an inner corner is added on purpose.
- Some rooms will have no door. Ian allows that for now, to be revisited. Some pockets may be cut off until the global-properties question is answered. Phase 2 measures how often.
- Whether mazes are fun depends heavily on the weights, which aren't set yet.
- Filters by kind, tile type and tag fit Ian's "filter by tile type (room, corridor, corridor corner)".

# Concerns raised

Wren (`game-designer/1`):

- **Boxed in.** Local rules can't make sure the player always has a way onward. In an endless maze, being sealed into a finite pocket is the worst outcome.
- **Sealed rooms.** No local rule makes sure a room has a door. A garden you can see but never enter is frustrating.
- **Graph-paper mazes.** If every allowed pair is about equally likely, mazes become grids of junctions with no rhythm, so the "slight variation" may need to be more than slight. Wren wants a say in the weights.
- **Dead ends** that run long and lead nowhere feel punishing.
- **Doors everywhere.** A room with many doors feels like a plaza rather than a destination. Weights per pair can address it.
- **Rooms back to back.** Two rooms wall to wall with no door between them read as one room you can't cross. With rooms rare overall this should be uncommon; phase 2 will show how often. (Wren wanted rooms rarely back to back; Ian's "equally likely to be another room or a path" replaces that.)

# AI guidance

- Use the tile names, kinds and side names here. Say "corridor", not "hallway".
- Identify tiles by tile type and orientation in code, tests and logs, never by glyph.
- Don't add tiles or side kinds without updating this record or superseding it. Don't add side kinds to change how often something appears.
- Don't treat "north up, clockwise" here as the data convention.

# Links

- Generation approach: [DR-0007](/decisions/0007-wave-function-collapse.md)
- Phase 1 scope: [DR-0008](/decisions/0008-adjacency-tool-first.md)
- Table entries from sides: [DR-0013](/decisions/0013-table-entries-from-tile-sides.md)
- Grid: [DR-0014](/decisions/0014-square-grid.md)
- Who designs the set, and fallbacks: [DR-0016](/decisions/0016-wren-designs-the-tile-set.md)
- How tiles are drawn: [DR-0017](/decisions/0017-tile-glyphs-and-colours.md)
- Open probability details: [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md)
- Global properties: [Q-0008](/questions/open/0008-global-maze-properties.md)
- Preview: `design/tile-set-preview.html` at the repository root, outside this bundle.
