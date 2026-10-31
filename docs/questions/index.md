# Questions

Questions waiting on a decision, and how they were answered ([DR-0004](/decisions/0004-open-questions.md)).

## Waiting on Ian

- [Q-0009 Square grid with four neighbours?](/questions/open/0009-sign-off-grid-and-tile-set.md) - Approve DR-0014, the one-way door from the tile-set work; the tile set and glyphs are two-way doors the team approved.

## Waiting on the team

- [Q-0007 Open details of the table's probabilities and the tile set](/questions/open/0007-open-details-of-table-probabilities.md) - Points 1 to 9 are answered. Point 10, the weight model: Yew decides a calculated base plus exceptions as a two-way door, and it waits on one engineer's approval.
- [Q-0008 Which global properties must a maze have, and how are they enforced?](/questions/open/0008-global-maze-properties.md) - Wren and Yew: what local tile rules can't guarantee (a way onward, room size, short dead ends), and how to enforce it. Choose what to measure before the phase 2 samples; enforce before the game.

## Resolved

- [Q-0001 Sign off the founding decision records](/questions/resolved/0001-sign-off-founding-decisions.md) - Ian approved DR-0001 to DR-0010 as written on 2026-10-31.
- [Q-0002 TypeScript and Vite for the Lit frontend?](/questions/resolved/0002-typescript-and-vite.md) - Yes: TypeScript and Vite ([DR-0011](/decisions/0011-typescript-and-vite.md)).
- [Q-0003 Where do the adjacency table's entries come from?](/questions/resolved/0003-where-table-entries-come-from.md) - Mainly from the tiles' sides, as probabilities ([DR-0013](/decisions/0013-table-entries-from-tile-sides.md)).
- [Q-0004 Where does the tile set come from?](/questions/resolved/0004-where-the-tile-set-comes-from.md) - Wren designs it from Ian's vocabulary, with crossroads and dead ends as rare fallbacks ([DR-0016](/decisions/0016-wren-designs-the-tile-set.md)).
- [Q-0005 What do you want to find out from the table?](/questions/resolved/0005-what-to-find-out-from-the-table.md) - Explore the probabilities, filter by tile type, and trace problems seen in mazes back to them; recorded in the roadmap.
- [Q-0006 How big will the table get?](/questions/resolved/0006-expected-table-size.md) - About 30 rows and columns in the short term, and millions of cells eventually.
- [Q-0010 Where does editing probabilities go?](/questions/resolved/0010-where-editing-goes.md) - Phase 2, at slice 2.3; Marram decided it as a two-way door.
