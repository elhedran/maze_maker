---
type: Decision Record
title: Use the GitHub remote and keep it up to date
description: origin on GitHub is the project's remote, and main is pushed to it every time anything lands. Work still lands on main. Supersedes DR-0021.
status: stable
decision_id: DR-0032
decision_status: accepted
decision_kind: process
major: false
owners: [human:ian]
approvers: [architect/1, product-manager/1, human:ian]
decided_at: 2026-11-01T10:15:00+01:00
supersedes: [/decisions/archive/0021-work-lands-on-main.md]
tags: [process, git, remote]
generated:
  by: facilitator/1
  at: 2026-11-01T09:45:00+01:00
verified:
  - by: architect/1
    at: 2026-11-01T10:00:00+01:00
  - by: product-manager/1
    at: 2026-11-01T10:00:00+01:00
  - by: human:ian
    at: 2026-11-01T10:15:00+01:00
---

# Context

[DR-0021](/decisions/archive/0021-work-lands-on-main.md) said there was no remote and nothing is pushed, and to revisit that when Ian added one. On 2026-11-01 Ian added `origin`, https://github.com/elhedran/maze_maker, and pushed `main`. Ian (`human:ian`), verbatim, 2026-11-01:

> https://github.com/elhedran/maze_maker has been set up as the remote, document where needed and let the team know.  I'll see about making use of a cloud session later today.

and then:

> lets make a firm decisions to use that remote and keep it up to date.

Cloud sessions start from what is on GitHub, so a stale remote means a cloud session starts from old work.

# Decision

Ian's decision:

- `origin` on GitHub is the project's remote, and the team uses it.
- It is kept up to date.

Facilitator's reading of "keep it up to date", for the reviewers and Ian to correct:

- Work still lands on `main`, as in DR-0021. Committing straight to `main` is still fine, and local branches and worktrees are still allowed under [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md).
- Every time anything lands on `main`, docs included, whoever landed it pushes `main` to `origin` straight away. Landing still needs Ian's OK where it did before, so pushing adds no new approval: an OK to land is also an OK to push. If the push is rejected because `origin/main` has moved, don't force it. Stop and tell the facilitator.
- Before starting work, `git fetch origin`. In the main checkout, fast-forward `main` to `origin/main`. In a worktree, never touch `main`: branch from or rebase onto `origin/main`. (Clarified 2026-11-01; see the note below.)
- Never force-push `main` and never rewrite its pushed history.
- No pull requests are required. Work lands on `main` directly.
- Pushing other branches (work in progress, or a branch a cloud session should start from) is the team's call, in DR-0019. Until DR-0019 is revised, other branches are pushed only when a cloud session needs them, and only by the facilitator. A pushed branch isn't rebased, or deleted from `origin`, until DR-0019 says how.

**Clarification, 2026-11-01** (Yew, `architect/1`, at the facilitator's request): the bullet on starting work first said "fast-forward local `main` from `origin/main`" for everyone. Ash and Marram, reviewing DR-0019, found git refuses that from a worktree while `main` is checked out in the main checkout, and that it contradicts DR-0019 rule 8 (a worktree never touches `main`). The bullet, the AI guidance and the agent-file wording now say what was meant: fetch first; only the main checkout fast-forwards `main`; a worktree works from `origin/main`. Ian's decision and his words are unchanged.

# Alternatives considered

- **Push only when Ian asks.** Leaves the remote stale between asks, which is what Ian ruled out.
- **Pull requests for every change.** A review step on GitHub, but the team already reviews before landing, and there is one human. Not asked for.

# Consequences

- `CLAUDE.md`'s Git section is headed `## Git (DR-0032, DR-0019)` and says: work lands on `main`; after it lands, `main` is pushed to `origin`; local branches and worktrees as DR-0019 says.
- The engineers' agent files change "Don't push" to: "Before starting work, `git fetch origin`. In the main checkout, fast-forward `main` to `origin/main`. In a worktree, never touch `main`: branch from or rebase onto `origin/main`. Push `main` to `origin` straight after you land work on it. Don't force-push; if a push is rejected, stop and tell the facilitator. Push other branches only as DR-0019 says."
- DR-0021 moves to `docs/decisions/archive/` as superseded. Links to it are rewritten to the archive path.
- DR-0019's "revisit when Ian adds a remote" is triggered. Yew reviews it for pushing branches and for cloud sessions.
- Not a one-way door: a remote can be ignored or removed. It comes to Ian because it replaces his record and changes `CLAUDE.md` and the agent files ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).

# Concerns raised

- **Marram (product manager), on review:** once a cloud session can push, `main` changes in two places. The draft covered pushing but not pulling. A local session that starts from an out-of-date `main` would push rejected or diverging work and stall delivery. Pulling before starting closes that gap without adding a step that needs approval. Marram also proposed fetching and integrating when a push is rejected. Yew's "stop and tell the facilitator" was taken instead, as the safer stopgap until DR-0019 is revised.
- **Yew (architect), on review:** "never force-push" and the stopgap for branches are the team's rules, not Ian's. They belong in DR-0019, and I'll move them there when I revise it. Two things the revision must settle: (a) cloud sessions make `origin/main` a second place where work can land, so landing has to start with a fetch; (b) DR-0019 rule 9 rebases branches and rule 10 deletes them, which clashes with a branch that's already pushed. Until then, the stopgap keeps both from coming up.

# AI guidance

- Before starting work, `git fetch origin`. In the main checkout, fast-forward `main` to `origin/main`. In a worktree, never touch `main`: branch from or rebase onto `origin/main`.
- Land work on `main`, docs included, then push `main` to `origin` straight away. If the push is rejected, don't force it: stop and tell the facilitator.
- Never force-push `main`.
- Push other branches only as [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) says.

# Links

- Supersedes [DR-0021](/decisions/archive/0021-work-lands-on-main.md)
- [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) parallel work
- [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) what goes to Ian
