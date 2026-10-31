---
type: Open Question
title: What do you want to find out from the table?
description: List the questions Ian wants the tool to answer about the adjacency table, so they shape its design and become the demo tasks.
status: stable
question_id: Q-0005
question_status: resolved
waiting_on: [human:ian]
raised_by: product-manager/1
raised_at: 2026-10-31T15:05:00+01:00
resolved_by: human:ian
resolved_at: 2026-10-31T15:11:00+01:00
related:
  - /roadmap.md
  - /decisions/0008-adjacency-tool-first.md
tags: [tool, ux, phase-1, demos]
generated:
  by: product-manager/1
  at: 2026-10-31T15:05:00+01:00
---

# Question

When you open the tool, what questions do you want it to answer about the table?

# Context

- You are the tool's main user. Phase 1 succeeds if we can "see and understand the table" ([DR-0008](/decisions/0008-adjacency-tool-first.md)).
- Your answers shape three designs: how the table is drawn (roadmap slice 1.1), how you explore one tile (1.2), and the filters (1.3) ([roadmap](/roadmap.md)).
- Damson (`ux-designer/1`) asks that this is answered now rather than at 1.3, and that each demo is a task you do without help. Yew (`architect/1`) will turn "see and understand" into measurable tests in the Phase 1 plan. Your answers become those tasks and tests.
- This question blocks slices 1.1 and 1.2.

# Options

These are examples. Pick any, change them, or add your own.

- **1. What can go next to this tile, on each side?** Damson expects this to be your first question, and slice 1.2 is built around it.
- **2. Why is this pair allowed, or not?** Wren (`game-designer/1`) suggests showing side types for this.
- **3. How likely is each neighbour?** This only applies if the weights record decides on weights.
- **4. Which tiles are hard to place,** with few or no neighbours on some side, and where would generation get stuck?
- **5. Which tiles are rare or common,** and is any tile never usable?
- **6. How do rooms and hallways connect?** Wren's doorway tile is the link.

# Recommendation

Damson (`ux-designer/1`) recommends answering now, in your own words. Marram (`product-manager/1`) suggests three to five questions, in order of importance. The top one becomes the 1.2 demo.

# Resolution

Ian answered on 2026-10-31, in conversation with the main facilitator session. His words, verbatim: "For the Q-0005 - what I want from the table - is I want to explore what is written down for probabilities, filter by tile type (room, corridor, corridor corner) so I can understand what leads into the maze generation. from the options, 1, 2, 3 are important, 4 is not, 5 is not, 6 is important. Basically I expect to "see" problems in the maze, and I'll want to look at the probabilities to understand a bit more of why they are there. A problem is hard to define, it will be bits of maze I don't think works well and would be improved by changing the probabilities."

Options 1, 2, 3 and 6 are important; 4 and 5 are not. His answer says what the tool must help him do, so it goes into the [roadmap](/roadmap.md) as demo tasks and slice changes, which he approves with the roadmap, rather than into a decision record of its own.
