---
type: Decision Record
title: Work lands on main; local branches and worktrees are allowed
description: Work is committed to main and there is no remote yet; local branches and worktrees may be used, as long as the work lands on main. Corrects DR-0009, which wrongly recorded a ban on branches.
status: deprecated
decision_id: DR-0021
decision_status: superseded
decision_kind: process
major: false
owners: [human:ian]
approvers: [architect/1, engineer-ash/1, human:ian]
decided_at: 2026-10-31T19:18:00+01:00
supersedes: [/decisions/archive/0009-commit-straight-to-main.md]
superseded_by: [/decisions/0032-use-the-remote.md]
tags: [process, git, worktrees]
generated:
  by: facilitator/1
  at: 2026-10-31T19:15:00+01:00
verified:
  - by: architect/1
    at: 2026-10-31T19:16:00+01:00
  - by: engineer-ash/1
    at: 2026-10-31T19:17:00+01:00
  - by: human:ian
    at: 2026-10-31T19:18:00+01:00
---

# Context

[DR-0009](/decisions/archive/0009-commit-straight-to-main.md) says "No feature branches or pull requests for now", and `CLAUDE.md` turned that into "No branches". Ian never said that. On 2026-10-31 he said:

> "I never said no branches, just that we were not working with a remote, and committing to main was fine. That may have been a misunderstanding"

The facilitator added the ban when drafting DR-0009, and Ian signed the record without the extra rule standing out. Because of it, [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) kept every agent in one checkout and ruled out worktrees. Ian had asked whether the agents were using worktrees appropriately, and said of DR-0009:

> "would have expected something about git worktrees in DR-0009"

This record states what Ian decided and nothing more. How the team uses branches and worktrees is a team process decision, and lives in DR-0019.

# Decision

- Work lands on `main`. There is no remote yet, so there are no pull requests and nothing is pushed.
- Committing straight to `main` is fine.
- Local branches and git worktrees are allowed. Work done on one lands on `main`.
- When and how the team uses them (when an agent gets a worktree, how work lands, who lands it, cleaning up afterwards) is the team's call, in [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md). Yew revisits that record once this one is accepted.

# Alternatives considered

- **Leave DR-0009 and only revise DR-0019.** DR-0009 would still record a rule Ian didn't make, and `CLAUDE.md` and the engineers' agent files would keep repeating it.

# Consequences

- `CLAUDE.md`'s Git section is headed `## Git (DR-0021, DR-0019)` and becomes: "Work lands on `main`. Local branches and worktrees are fine; DR-0019 says when agents use them. No remote yet."
- The engineers' agent files currently say "Don't branch or push" and link DR-0009. They become (Ash's wording): "Commit to `main` only when Ian or the facilitator asks. Don't push. Use local branches and worktrees only as DR-0019 says, including when to commit on them." They link this record and DR-0019.
- Yew's revision of DR-0019 is accepted and committed together with this record, so `CLAUDE.md` and DR-0019 never disagree.
- DR-0009 moves to `docs/decisions/archive/`. Links to it (DR-0010, DR-0012, DR-0019, Q-0001, and this record) are rewritten to the archive path, as was done for DR-0002.
- Revisit when Ian adds a remote.
- This is not a one-way door. It comes to Ian because it replaces his record and changes `CLAUDE.md` and the agent files ([DR-0018](/decisions/0018-one-way-doors-go-to-ian.md)).

# Concerns raised

- **Facilitator:** the mistake came from drafting. The facilitator wrote a rule into DR-0009 that Ian hadn't asked for, and the record didn't make it stand out. When the facilitator drafts a record from Ian's words, it quotes him, and marks anything it adds as its own proposal.
- **Yew (architect), on review:** the draft repeated DR-0009's mistake in a small way. "The branch is then deleted", "don't leave branches lying around" and "fine when nothing runs in parallel" were rules Ian didn't make. They are reasonable, but they are the team's to make, so they move to DR-0019. What remains here is Ian's: main is fine, branches and worktrees are allowed, no remote. "Work lands on `main`" is my reading of "committing to main was fine" with no remote; Ian should correct it if he meant less.
- **Yew (architect), on review:** the agent files say "Don't branch", not just a link to DR-0009, so they need their wording changed, not only their link. Until they and `CLAUDE.md` change, an agent will still refuse a worktree it is briefed to use.
- **Yew (architect), on review:** the `CLAUDE.md` line now points at DR-0019. Without it, an agent reading only `CLAUDE.md` would learn that worktrees are allowed but not the rules for using them, which is the gap I raised on DR-0019.
- **Ash (engineer), on review:** the agent-file wording "commit only when Ian or the facilitator asks" reads as a ban on committing at all. Work in a worktree has to be committed on its branch before it can land on `main`, so an agent briefed to use one would stop short. Suggested wording for the agent files: "Commit to `main` only when Ian or the facilitator asks. Don't push. Use local branches and worktrees only as [DR-0019](/docs/decisions/0019-parallel-work-in-one-checkout.md) says, including when to commit on them." That keeps Ian's part (main, no remote) and leaves the rest to DR-0019.
- **Ash (engineer), on review:** between this record's acceptance and Yew's revision of DR-0019, `CLAUDE.md` will say worktrees are fine while DR-0019 still says "Don't create worktrees or branches" and links an archived DR-0009. That is safe, since the stricter team rule holds, but it should be said: until DR-0019 is revised, its current rules (one checkout, no worktrees) still apply. Better still, accept the DR-0019 revision together with this one so the window doesn't exist.
- **Ash (engineer), for the DR-0019 revision:** each worktree gets its own `target/` and `node_modules/`, so the first Rust and Tauri build in one is a full build and takes disk space. Worth measuring once before deciding when a worktree is worth it, and worth saying where worktrees live (outside the repo folder, or ignored by git) and who removes them.

# AI guidance

- Land work on `main`. Don't push, since there is no remote.
- A local branch or worktree is allowed. Its work ends up on `main`.
- Follow [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) for when to use one, how work lands, and cleaning up.

# Update 2026-11-01: a remote now exists

Ian (`human:ian`), verbatim, 2026-11-01: "https://github.com/elhedran/maze_maker has been set up as the remote, document where needed and let the team know.  I'll see about making use of a cloud session later today."

This triggers this record's "revisit when Ian adds a remote". Ian pushed `main` himself (`origin/main` at debaa44). Facilitator's note, not Ian's rule: until a record replaces this one, agents still don't push, and everything else here stands. That record decides who pushes, when, and whether branches or pull requests are used. It comes to Ian, because it replaces his record and changes `CLAUDE.md` and the agent files.

# Links

- Superseded by [DR-0032](/decisions/0032-use-the-remote.md)
- Supersedes [DR-0009](/decisions/archive/0009-commit-straight-to-main.md)
- [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) parallel work
- [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) what goes to Ian
