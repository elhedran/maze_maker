---
type: Decision Record
title: Parallel work shares one checkout, with rules
description: Agents and sessions working at the same time share the main checkout; only one changes code at a time, parallel work gets separate files, and only the facilitator commits, by path.
status: stable
decision_id: DR-0019
decision_status: accepted
decision_kind: process
major: false
owners: [facilitator/1]
approvers: [product-manager/1, architect/1]
decided_at: 2026-10-31T20:10:00+01:00
supersedes: []
tags: [process, git, sessions, agents]
generated:
  by: facilitator/1
  at: 2026-10-31T19:30:00+01:00
verified:
  - by: architect/1
    at: 2026-10-31T19:55:00+01:00
  - by: product-manager/1
    at: 2026-10-31T20:10:00+01:00
---

# Context

[DR-0009](/decisions/0009-commit-straight-to-main.md) has all work committed straight to `main`, with no branches. A worktree needs a branch, so every session and agent so far has worked directly in the one checkout.

On 2026-10-31 several sessions and agents worked in that checkout at the same time, and it went wrong in small ways:

- The DR-0018 session committed while the scaffold's files sat uncommitted next to its own. It had to leave them out by hand.
- The facilitator committed the scaffold with `git add -A` while the scaffold session was still checking it. That session then committed its log entry on top. Nothing broke, but `git add -A` would have taken in another agent's unfinished changes if there had been any.

Ian asked whether the agents were using worktrees appropriately.

# Decision

Keep one checkout and DR-0009, with these rules:

1. **One agent changes code at a time.** Code means everything outside `docs/` and `design/`: `apps/`, `crates/`, the build files and lockfiles at any level, `README.md`, `.gitignore`, `.claude/` and `CLAUDE.md`. Installing packages and building count as changing code, because they rewrite lockfiles. Others may read code meanwhile, but a build or test run during someone else's change says nothing reliable about either.
2. **Parallel work gets separate files.** When the facilitator runs agents or sessions in parallel, each brief names the files it may change. Shared files, `docs/log.md`, every `index.md` and `docs/roadmap.md`, belong to the facilitator during parallel work: it updates them when it commits, unless a brief hands one of them to a single named agent. The facilitator also hands out new record and question numbers in the briefs, so two agents never both write a DR-0020.
3. **Only the facilitator commits work that ran in parallel.** It commits by path (`git add <paths>`), never with `git add -A` or `git commit -a`. A session working alone may commit its own work, also by path.
4. **Check before committing.** Run `git status` first. Anything not part of this work is left out, and the facilitator finds out whose it is.

# Alternatives considered

- **A worktree and a short-lived branch for each parallel writer**, rebased and fast-forwarded onto `main` when done. This isolates work fully, but needs a record superseding DR-0009 and adds merge steps for little gain while most parallel work is docs.
- **No rules, as before.** This worked only because the files barely overlapped.

# Consequences

- Parallel code work is serialized. That costs little now, while there is one app and little code.
- The facilitator has to name files in every parallel brief.
- **Revisit** when any of these happens: two engineers need to build code at the same time; parallel work collides again despite these rules (work lost, mixed into the wrong commit, or a build broken by someone else's half-finished change); or Ian adds a remote ([DR-0009](/decisions/0009-commit-straight-to-main.md) is revisited then anyway). Worktrees with short-lived local branches are the likely next step.
- Two-way door: dropping or changing these rules costs a record and nothing else. Moving to worktrees is also cheap to do and to undo, but it supersedes [DR-0009](/decisions/0009-commit-straight-to-main.md), which is Ian's, and changes the Git section of `CLAUDE.md`, so under [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) that record needs Ian's approval.

# Concerns raised

- **Yew (architect):** these rules live in briefs and in this record, not in `CLAUDE.md` or the agent files, so an agent only follows them if its brief says so. The facilitator must put the file list and the "add by path" rule in every parallel brief. If that slips, the fix is a line in `CLAUDE.md`'s Git section, which needs Ian.
- **Yew (architect):** serializing code work is right while there is one app and two crates, but it will become the bottleneck as soon as Rowan and Ash both have plan steps in flight. The adjacency-tool plan should be sliced so that only one step at a time touches code; when that stops being possible, that is the revisit trigger, not a reason to bend rule 1.
- **Yew (architect), on review:** tightened rule 1 (code is everything outside `docs/` and `design/`, and builds count) and rule 2 (shared files default to the facilitator, which also hands out record numbers), added a collision trigger to the revisit, and corrected the note on moving to worktrees.
- **Marram (product manager):** serializing code work doesn't hurt Phase 1's sequencing today. 1.1, 1.2, 1.3 and 1.5 are sequential, each builds on the one before, and what gates them now is decisions and designs (docs work), which can still run in parallel. Marram will slice so one code step is in flight at a time, and keeps the other engineer on reviews, approvals and checking demos against their "done when" rather than idle.
- **Marram (product manager):** the first real test is close. [DR-0020](/decisions/0020-vitest-for-frontend-tests.md) and its `package.json` and lockfile changes are uncommitted alongside 1.0's check. If the next step is held up waiting on one engineer's code change for more than about a slice, Marram will treat that as the "two engineers need to build code at the same time" trigger and bring it to Yew, rather than let demos slip.
- **Marram (product manager):** rule 1 counts `README.md` as code, and every demo's run steps live there. Demo steps must be written by whoever holds the code at the time, not by a parallel docs agent.

# AI guidance

- Before starting work in parallel with another agent or session, know which files are yours, and change only those.
- In parallel work, leave `docs/log.md`, the `index.md` files and `docs/roadmap.md` to the facilitator unless your brief gives one to you, and use the record or question number your brief gives you.
- Don't install packages, build or run tests while another agent is changing code.
- Never `git add -A`, `git add .` or `git commit -a`. Add by path.
- If `git status` shows changes you didn't make, leave them alone and say so.
- Don't create worktrees or branches; [DR-0009](/decisions/0009-commit-straight-to-main.md) still applies.

# Links

- [DR-0009](/decisions/0009-commit-straight-to-main.md) commit straight to main
- [DR-0010](/decisions/0010-facilitator-manages-context.md) facilitator manages context
- [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) one-way doors go to Ian
