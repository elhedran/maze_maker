---
type: Open Question
title: How big will the table get?
description: Estimate how many rows (tile and orientation pairs) the adjacency table will have, in Phase 1 and later in the game.
status: stable
question_id: Q-0006
question_status: resolved
waiting_on: [human:ian]
raised_by: product-manager/1
raised_at: 2026-10-31T15:05:00+01:00
resolved_by: human:ian
resolved_at: 2026-10-31T15:40:00+01:00
related:
  - /roadmap.md
  - /decisions/0005-tauri-rust-backend-lit-frontend.md
  - /decisions/0008-adjacency-tool-first.md
  - /questions/open/0007-open-details-of-table-probabilities.md
tags: [adjacency-table, scale, phase-1]
generated:
  by: product-manager/1
  at: 2026-10-31T15:05:00+01:00
---

# Question

Roughly how many rows (tiles times their orientations) do you expect the table to have, for full tile set v1 in Phase 1 and later in the game?

# Context

- [DR-0008](/decisions/0008-adjacency-tool-first.md) says the table will be large, based on your past experience. A number lets the team plan for it.
- Entries grow with the square of the rows. With four sides (if the grid is square, which isn't decided yet):
  - 19 rows give 1,444 entries;
  - 100 rows give 40,000;
  - 500 rows give 1,000,000.
- Yew (`architect/1`) needs the estimate for the paging and transport record at slice 1.5, and for the speed tests in the Phase 1 plan ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md) consequences).
- Damson (`ux-designer/1`): at full size, a view of the whole table may stop being readable.
- Wren's (`game-designer/1`) starter set has 19 rows. Full set v1 adds crossroads, dead end, plain room edge, corner doorway and some variants.
- **Partial input from Ian, 2026-10-31:** "I'm expecting a big table" (in his answer on what else sets entries, recorded in [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md)). This question stays open for a rough number. Yew asks for it as a number of tiles. If it's in the thousands, Yew would move slice 1.5 up to straight after 1.1; the view-based interface from 1.1 already allows for paging.
- This isn't urgent, because it blocks only slice 1.5 ([roadmap](/roadmap.md)). An early number helps Yew and Damson plan.

# Options

Nobody favours a particular band; this is your estimate.

- **Under 50 rows.**
- **50 to 200 rows.**
- **200 to 1,000 rows.**
- **More than 1,000 rows.**

# Recommendation

Marram (`product-manager/1`): an order of magnitude is enough, for Phase 1 and for the game. Add what you saw last time, if you remember it.

# Resolution

Ian answered on 2026-10-31, in conversation with the main facilitator session. His words, verbatim: "For Q-0006, I expect the eventual table to be up in the order of millions of cells - although in the short term more in the order of 30 rows/columns."

So the table is small for now, about 30 rows and columns, and in the end around millions of cells. This is an estimate for planning, not a decision, so it has no record of its own. It feeds Yew's paging and transport record and the roadmap's condition on slice 1.5.
