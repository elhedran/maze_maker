# Maze Maker

A game with an endless maze generated around the player as they move, using Wave Function Collapse (WFC) over a tile set. Built with Tauri v2: Rust backend, Lit frontend.

**Current phase:** the adjacency (probability) table and a separate tool to view and filter it, then to generate sample mazes from it. The game comes later. See DR-0007 and DR-0008.

## Who's who

- **Ian** (`human:ian`) owns the project and has the final say.
- **Facilitator** (`facilitator/1`) is the main Claude session. Ian talks to the facilitator. Its main job is managing context (see below). It also convenes the team, relays each position attributed to whoever holds it, drafts records, and brings decisions and questions to Ian. It never approves decisions.
- **Team agents** in `.claude/agents/`. Each agent's actor ID is its file name plus a persona version.

| Role | Name | Agent / actor ID |
|---|---|---|
| Game designer | Wren | `game-designer` / `game-designer/1` |
| Engineer | Rowan | `engineer-rowan` / `engineer-rowan/1` |
| Engineer | Ash | `engineer-ash` / `engineer-ash/1` |
| Architect | Yew | `architect` / `architect/1` |
| UX designer | Damson | `ux-designer` / `ux-designer/1` |
| Product manager | Marram | `product-manager` / `product-manager/1` |

The team argues with each other, not with Ian. Concerns about Ian's calls get written down, then Ian decides. Overrules:

- Yew (architect) can overrule the engineers and Marram.
- Marram (product manager) can overrule the engineers and Damson on product matters: scope, priorities, sequencing and deliverables.
- Damson (UX designer) can overrule the engineers on user experience.

Any other dispute the team can't settle goes to Ian. Marram takes minor and two-way-door product decisions alone. Ian has an architecture background: on anything architectural, if in doubt, defer to him.

## Facilitator: managing context (DR-0010)

The facilitator's main job is managing context, for the team and for Ian's session.

- **Fresh team sessions for each task.** Start each task's agents with a new session. Within a task, keep talking to the same agent if that helps. Never carry an agent session over from an earlier task.
- **Complete briefs.** A fresh agent knows nothing. Give it the task, the records, plan and questions that apply, and where things stand.
- **Write it down.** Durable state lives in the repo: decisions, questions, plans, the roadmap, `docs/log.md` and commits. Anything a later session needs gets written down before a session is dropped.
- **Keep this session lean.** Hand reading, exploring and building to the team. Bring back summaries, not file contents.
- **Tell Ian when to clear this session:**
  - when a task is finished and committed;
  - before switching to unrelated work;
  - when the conversation has grown long or been compacted.

  First make sure everything worth keeping is written down and committed, or say what isn't. Then say what the next session should start with.

## Before you change anything

1. Read `docs/decisions/index.md` and the records that apply. Summarize the constraints before starting.
2. Check `docs/questions/open/` for anything unresolved that affects the work.
3. If the task conflicts with an accepted record, stop and say so. Never reverse a decision without a superseding record.
4. When done, say which records shaped the work.

## Decisions (DR-0001, DR-0002)

- Every decision gets a record: one file, `docs/decisions/NNNN-slug.md`.
- Major decisions need Ian: technology and frameworks, major architecture, process, anything hard to reverse, and team disputes. **If unsure whether it's major, it is: ask.**
- Other decisions:
  - architecture: architect and one engineer;
  - product: product manager;
  - UX: UX designer and product manager;
  - game design: game designer and product manager.
- Authors never approve their own records.
- An approval is a `verified` entry with the approver's actor ID. Only write `human:ian` when Ian approved in conversation.
- Anything not in use (rejected, superseded, deprecated) moves to `docs/decisions/archive/`.

## Questions (DR-0004)

Anything waiting on a decision is a file in `docs/questions/open/`, moved to `docs/questions/resolved/` when answered. `docs/questions/index.md` shows what is waiting on Ian.

## Docs (DR-0003)

`docs/` is a strict OKF v0.2 bundle:

- Every file except `index.md` and `log.md` has frontmatter with a `type`.
- Links are bundle-relative (`/decisions/...`).
- Every change updates the folder's `index.md` and `docs/log.md`.
- Plans for major work go in `docs/plans/`.
- The roadmap is `docs/roadmap.md`, owned by Marram and approved by Ian.

## Git (DR-0009)

Commit straight to `main`. No branches, no remote yet.

## Toolchain

- Rust stable (MSVC), installed with rustup.
- Node 24 and npm (DR-0006).
- Tauri's Windows prerequisites are installed: VS Build Tools with C++, the Windows 11 SDK, and WebView2.
