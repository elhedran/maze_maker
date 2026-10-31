# Decision records

One file per decision ([DR-0001](/decisions/0001-record-every-decision.md)). Only records in use are listed here; rejected, superseded and deprecated records are in `/decisions/archive/`.

## Accepted

- [DR-0001 Record every decision as a decision record](/decisions/0001-record-every-decision.md) - Every project decision gets its own decision record in docs/decisions, with a lifecycle and an archive.
- [DR-0002 Approval and escalation rules](/decisions/0002-approval-and-escalation.md) - Ian approves major decisions and settles disputes; other decisions are approved by the relevant team members.
- [DR-0003 Open Knowledge Format for all docs](/decisions/0003-open-knowledge-format-for-docs.md) - The docs folder is a strict OKF v0.2 bundle with a fixed layout for decisions, questions and plans.
- [DR-0004 Track open questions as files](/decisions/0004-open-questions.md) - Questions waiting on a decision live in docs/questions/open and move to docs/questions/resolved when answered.
- [DR-0005 Tauri v2 with a Rust backend and Lit frontend](/decisions/0005-tauri-rust-backend-lit-frontend.md) - Apps are built with Tauri v2, with generation logic in Rust and the UI in Lit web components.
- [DR-0006 npm for frontend packages](/decisions/0006-npm-for-frontend-packages.md) - The frontend uses npm as its package manager.
- [DR-0007 Generate mazes with Wave Function Collapse](/decisions/0007-wave-function-collapse.md) - Mazes are generated around the player with Wave Function Collapse over a tile set and an adjacency table.
- [DR-0008 Adjacency-table tool first, separate from the game](/decisions/0008-adjacency-tool-first.md) - Phase 1 builds the adjacency table and a separate tool to inspect and test it; the game comes later.
- [DR-0009 Commit straight to main](/decisions/0009-commit-straight-to-main.md) - Work is committed directly to main, with no branches and no remote for now.
- [DR-0010 The facilitator manages context](/decisions/0010-facilitator-manages-context.md) - The facilitator's main job is managing context, giving each task fresh team sessions and telling Ian when to clear his session.
- [DR-0011 TypeScript and Vite for the Lit frontend](/decisions/0011-typescript-and-vite.md) - The Lit frontend is written in TypeScript and built with Vite.
- [DR-0013 Table entries come from tile sides, as probabilities](/decisions/0013-table-entries-from-tile-sides.md) - The adjacency table's entries are worked out mainly from the tiles' sides, with probability 0 where sides don't pair up and slightly varied probabilities for what is allowed.
- [DR-0016 Wren designs the tile set, with crossroads and dead ends as rare fallbacks](/decisions/0016-wren-designs-the-tile-set.md) - Wren designs the tile set from Ian's vocabulary, and crossroads and dead ends are in it at very low probability, to fix issues where nothing else fits.

## Proposed

_None._

## Archived

_None yet._
