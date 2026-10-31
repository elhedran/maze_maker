---
type: Decision Record
title: Adjacency-table tool first, separate from the game
description: Phase 1 builds the adjacency table and a separate tool to inspect and test it; the game comes later.
status: draft
decision_id: DR-0008
decision_status: proposed
decision_kind: product
major: true
owners: [human:ian]
approvers: [human:ian]
supersedes: []
tags: [roadmap, tooling, adjacency-table]
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
---

# Context

The adjacency table will be large. Ian's past experience is that it needs its own tooling before it can drive a game.

# Decision

- **Phase 1** is the adjacency table, and a tool to view and filter it.
- **Phase 2** extends the tool to generate sample mazes from the table, to test what kind of maps it makes.
- The tool is a **separate app** from the game.
- The game comes after, as its own app. It will likely get a debug mode with some of the tool's views.
- How code is shared between the tool and the game (for example a shared Rust crate) is an architecture decision still to be made. The architect will propose it.

# Alternatives considered

- **Game first, with the table hand-tuned in code.** Faster to something playable, but past experience says the table needs tooling first.
- **The tool as a mode inside the game.** One app, but it ties the tooling UI to the game's UI and release.

# Consequences

- Nothing playable for a while. Phase 1 succeeds if we can see and understand the table; phase 2 if we can judge the maps it makes.
- Two apps to build and maintain, so shared code needs a structure from the start.

# Concerns raised

None.

# AI guidance

- Phase 1 work is about the table and the tool. Don't start on game features without a decision record.

# Links

- Generation approach: [DR-0007](/decisions/0007-wave-function-collapse.md)
