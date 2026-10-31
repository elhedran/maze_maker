---
type: Open Question
title: Where does editing probabilities go?
description: Decide which phase, if any, adds editing the table's probabilities inside the tool, given that Phase 1 is view and filter only.
status: draft
question_id: Q-0010
question_status: open
waiting_on: [human:ian]
raised_by: product-manager/1
raised_at: 2026-10-31T15:25:00+01:00
related:
  - /roadmap.md
  - /decisions/0008-adjacency-tool-first.md
  - /decisions/0013-table-entries-from-tile-sides.md
  - /questions/resolved/0005-what-to-find-out-from-the-table.md
  - /questions/open/0007-open-details-of-table-probabilities.md
tags: [tool, scope, editing, probabilities, roadmap]
generated:
  by: product-manager/1
  at: 2026-10-31T15:25:00+01:00
---

# Question

You want to change probabilities to fix problems you see in mazes. Should the tool let you edit them, and if so, in which phase?

# Context

- In your answer to [Q-0005](/questions/resolved/0005-what-to-find-out-from-the-table.md), you said problems "will be bits of maze I don't think works well and would be improved by changing the probabilities".
- In your answers to [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md), you said: "the idea of the tool is to fix this by review, not perfect ahead of time."
- [DR-0008](/decisions/0008-adjacency-tool-first.md) makes Phase 1 "view and filter", and makes Phase 2 generating sample mazes. Neither includes editing. Adding it changes a phase's scope, which is your call.
- [DR-0013](/decisions/0013-table-entries-from-tile-sides.md) computes the table from the tile set. Whatever sets the probabilities lives in the tile set data, so changing a probability without in-tool editing means editing that data and reloading it in the tool. The roadmap requires this to work from slice 1.1, with no code change.
- How much editing you'd need depends on two points already with you in [Q-0007](/questions/open/0007-open-details-of-table-probabilities.md). This question doesn't ask them again:
  - **The weight model.** Set by hand on every pair would mean well over a thousand numbers once the set is full size, which really needs editing in the tool. A computed base plus a few exceptions in the tile set data, which Yew (`architect/1`) recommends, keeps edits few and small.
  - **Randomness.** Whether the table's numbers carry noise.
- Your loop is: see a problem in a maze, look at the probabilities behind it, change them. In the [roadmap](/roadmap.md), seeing a maze is 2.1, opening the table from a spot in the maze is 2.2, and tuning and comparing with the same seed is 2.3.
- Wren (`game-designer/1`) raised this. If you expect to tune the table yourself, Phase 1 rules out the editing your Q-0005 answer describes.
- Nothing is blocked yet. At the latest, it's needed by 2.3.

# Options

- **1. No editing in the tool for now.** You change probabilities by editing the tile set data, and the tool reloads it. We revisit in-tool editing once you've used the tuning loop in 2.3. This fits a computed base with few exceptions, and is painful if every pair is set by hand. It also sits less well with "fix this by review": the review would happen in the tool, but the fix wouldn't.
- **2. Editing in Phase 2, as part of tuning (2.3).** From a spot in a maze you open the table, change a probability, save it to the tile set data, regenerate the same seed and compare. Phase 1 stays view and filter. Until 2.3 you edit the data by hand.
- **3. Editing in Phase 1, after filtering (1.3).** You can tune the table before mazes exist. It makes Phase 1 bigger, and you've said problems show up in mazes, not in the table.
- **4. Later, with the game's debug mode.**

Positions:

- **Wren (`game-designer/1`)** raised the question and hasn't stated a preference.
- **Yew (`architect/1`)** hasn't given a view on this question. Yew's recommendation on the weight model in Q-0007 (a computed base plus a few exceptions) would keep option 1 workable.
- **Marram (`product-manager/1`)** favours option 2, below.

# Recommendation

Marram (`product-manager/1`) recommends option 2: editing joins Phase 2, at 2.3.

- That's where your loop needs it: you see the problem in a maze, open its probabilities, change one, and regenerate the same seed.
- Phase 1 stays view and filter, so the first demos don't get bigger.
- Until 2.3, editing the tile set data and reloading covers it.

If your Q-0007 answer is weights set by hand on every pair, Marram would move editing to Phase 1 (option 3) instead.

# Resolution

_Open._
