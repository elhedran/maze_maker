---
type: Decision Record
title: Parallel code work gets its own worktree; shared files stay with the facilitator
description: Each engineer changing code in parallel works in their own git worktree on a short-lived local branch, which the facilitator rebases and fast-forwards onto main. Docs-only parallel work stays in the main checkout, with separate files per agent and commits by path.
status: stable
decision_id: DR-0019
decision_status: accepted
decision_kind: process
major: false
owners: [facilitator/1]
approvers: [product-manager/1, engineer-ash/1]
decided_at: 2026-10-31T19:21:16+01:00
supersedes: []
tags: [process, git, sessions, agents, worktrees]
generated:
  by: architect/1
  at: 2026-10-31T19:19:00+01:00
verified:
  - by: engineer-ash/1
    at: 2026-10-31T19:20:00+01:00
  - by: product-manager/1
    at: 2026-10-31T19:21:16+01:00
---

**Revised 2026-10-31** by Yew (architect) after [DR-0021](/decisions/0021-work-lands-on-main.md). The first version, drafted by the facilitator and accepted earlier the same day, kept everyone in one checkout because it read [DR-0009](/decisions/archive/0009-commit-straight-to-main.md) as banning branches. Its approvals are cleared, since the substance changed. The file name is kept so links still work.

# Context

On 2026-10-31 several sessions and agents worked in the one checkout at the same time, and it went wrong in small ways:

- The DR-0018 session committed while the scaffold's files sat uncommitted next to its own. It had to leave them out by hand.
- The facilitator committed the scaffold with `git add -A` while the scaffold session was still checking it. That session then committed its log entry on top. Nothing broke, but `git add -A` would have taken in another agent's unfinished changes if there had been any.

Ian asked whether the agents were using worktrees appropriately. The first version of this record said they couldn't: it took [DR-0009](/decisions/archive/0009-commit-straight-to-main.md) to mean "no branches", and a worktree needs a branch. That reading was wrong. Ian never banned branches; [DR-0021](/decisions/0021-work-lands-on-main.md) corrects it. Work lands on `main`, there is no remote, and local branches and worktrees are allowed. When and how agents use them is left to this record.

# Decision

## Who works where

1. **Each engineer changing code in parallel gets their own worktree** on a short-lived local branch. Code means everything outside `docs/` and `design/`: `apps/`, `crates/`, the build files and lockfiles at any level, `README.md`, `.gitignore`, `.claude/` and `CLAUDE.md`. Installing packages and building count as changing code.
2. **Within any one checkout, one agent changes code at a time.** This covers the main checkout and every worktree. A build or test run during someone else's change in the same checkout says nothing reliable about either. An engineer working alone, with nobody else changing code, may work in the main checkout.
3. **Docs-only parallel work stays in the main checkout**, under rules 6 to 8.

## Worktrees

4. **Where they live and who sets them up.** Worktrees go in `.worktrees/` at the repo root, which git ignores. The folder is named after the branch, with `/` replaced by `-`. Branches are named `<engineer>/<slice>`, for example `ash/1.2-table-view`. The facilitator creates the worktree from `main` (`git worktree add .worktrees/ash-1.2-table-view -b ash/1.2-table-view main`) and gives its path and branch in the brief.
5. **Builds.** Each worktree has its own `target/` and `node_modules/`, so its first build is a full build. Don't share a target directory between worktrees. **Lockfile changes stay one writer at a time across all checkouts**: only one in-flight branch may add, remove or upgrade dependencies (`Cargo.lock`, `package-lock.json`, and the manifests they come from), and the brief says which one. Two branches that both rewrite a lockfile can't be rebased cleanly.

## Files, numbers and commits

6. **Parallel work gets separate files.** Each brief names the files the agent may change. Shared files, `docs/log.md`, every `index.md` and `docs/roadmap.md`, belong to the facilitator during parallel work, in the main checkout and in every worktree: it updates them when it lands or commits the work, unless a brief hands one to a single named agent. The facilitator also hands out new record and question numbers in the briefs, so two agents never both write a DR-0022.
7. **Commit by path.** Never `git add -A`, `git add .` or `git commit -a`. Run `git status` first. Anything not part of this work is left out, and the facilitator finds out whose it is.
8. **Who commits where.** An engineer in a worktree commits their own work on their own branch, when their brief says to, and never touches `main`. In the main checkout, only the facilitator commits work that ran in parallel. A session working alone may commit its own work to `main`.

## Landing and clean-up

9. **The facilitator lands the work.** It rebases the branch onto `main` in the worktree, the engineer (or the facilitator) reruns the build and tests there, and then it fast-forwards `main` (`git merge --ff-only <branch>` in the main checkout). No merge commits. If the rebase conflicts, the branch goes back to the engineer who owns it.
10. **Clean-up.** After landing, the facilitator removes the worktree (`git worktree remove`) and deletes the branch (`git branch -d`). Work that is abandoned is removed the same way, with `-D`, once Ian or the facilitator has said it is abandoned. Before telling Ian to clear a session ([DR-0010](/decisions/0010-facilitator-manages-context.md)), the facilitator checks `git worktree list` and `git branch` and says what is still open.

## Measuring the cost once

11. The first time a worktree is used, the engineer measures how long its first full build takes (`npm install`, the Rust build and the Tauri app build, from a fresh worktree) and how much disk it uses, and reports the numbers. The facilitator records them in `docs/log.md` and adds them to this record. From then on, the facilitator uses them to decide, per brief, whether a short code step is worth a worktree or should simply wait its turn under rule 2.

**Measured 2026-11-01, on slice 1.1** (the first use, rule 11):
- Rowan's Rust worktree: first `cargo build --workspace` took 1m 40s, and the first `cargo test --workspace` 3.9s. The folder was 4.0 GB, almost all of it `target/`.
- Ash's frontend worktree: `npm install` 29.3s, `npm test` 3.0s and `npm run build` 1.9s.
- One thing to know: a session that runs in its own worktree can only edit files inside it. The engineers' worktrees went inside the facilitator's worktree (`<its root>/.worktrees/`, which is git-ignored too), not at the repository root.

# Alternatives considered

- **One checkout for everyone, code work serialized** (the first version of this record). Safe, but it makes the second engineer wait, and it rested on a ban that never existed. It remains the fallback: when a worktree isn't worth its build cost (rule 11), code work waits its turn in the main checkout.
- **Worktrees outside the repo folder** (for example a sibling `maze_maker-worktrees/`). Needs no `.gitignore` entry, but the agents' file tools and permissions are scoped to the repo folder, so every worktree would need extra permission grants, and those settings are Ian's. A git-ignored folder inside the repo avoids that, and search tools that respect `.gitignore` don't see duplicate code.
- **A worktree for every agent, docs included.** Full isolation, but docs work is small, rarely builds, and already stays apart through separate files and facilitator-owned shared files. It would add landing steps for no gain.
- **A shared `CARGO_TARGET_DIR` across worktrees** to save build time. Builds on different branches would contend for the same lock and keep invalidating each other's output.
- **Merge commits instead of rebase and fast-forward.** Keeps the branch shape, but `main` stays linear and easy to read, and with short-lived local branches nothing is lost by rebasing.

# Consequences

- Two engineers can change and build code at the same time, each in their own worktree.
- The facilitator gains work: creating worktrees, landing, and cleaning up. It is a few commands each time, but it must not be skipped, or stale branches and folders pile up.
- Each worktree costs a full first build and its own disk. Rule 11 puts a number on that.
- The facilitator still names files in every parallel brief, and still owns the shared files and record numbers.
- **Revisit** when any of these happens: landing (rebase conflicts or reruns) regularly costs more than the parallel work saved; parallel work collides despite these rules (work lost, mixed into the wrong commit, or a build broken by someone else's half-finished change); the measured build cost makes worktrees rarely worth it; or Ian adds a remote ([DR-0021](/decisions/0021-work-lands-on-main.md) is revisited then anyway).
- Two-way door: dropping or changing these rules costs a record and a `.gitignore` line. Since [DR-0021](/decisions/0021-work-lands-on-main.md), using worktrees is the team's call and doesn't need Ian.

# Concerns raised

- **Yew (architect):** these rules live in briefs and in this record. `CLAUDE.md` and the engineers' agent files now point here ([DR-0021](/decisions/0021-work-lands-on-main.md)), but an agent still only gets the specifics (its worktree, its branch, its files, whether it may touch a lockfile, when to commit) from its brief. The facilitator must put them in every parallel brief.
- **Yew (architect), on the revision:** I authored this revision, so I don't approve it. Process records need the product manager and the architect; following the precedent of [DR-0012](/decisions/0012-code-layout-and-core-crate.md), where the architect authored and an engineer approved in the architect's place, Ash approves alongside Marram. Ash raised the worktree concerns this revision answers, so he is best placed to check them.
- **Yew (architect), on the revision:** the plan should still slice work so that the two engineers' steps touch different files where it can. Worktrees stop two builds trampling each other; they don't stop two branches editing the same file, and that cost shows up at the rebase.
- **Yew (architect), on review of the first version:** tightened the definition of code (everything outside `docs/` and `design/`, and builds count) and made shared files default to the facilitator, which also hands out record numbers. Both carry over.
- **Ash (engineer), on [DR-0021](/decisions/0021-work-lands-on-main.md):** each worktree gets its own `target/` and `node_modules/`, so the first build is full and takes disk; measure once before deciding when a worktree is worth it, and say where worktrees live and who removes them. Answered by rules 4, 5, 10 and 11: `.worktrees/`, git-ignored; the facilitator creates and removes them; the first use is measured.
- **Ash (engineer), on review of the revision:** my DR-0021 concerns are answered. Where worktrees live, who removes them and measuring the first build are rules 4, 5, 10 and 11, and `.gitignore` already has the `.worktrees/` line. The window where the rules disagree closes if this record is accepted and committed with DR-0021, as DR-0021 says. The `.gitignore` line must be committed before the first worktree is created, or `.worktrees/` shows up as untracked in `git status`. I approve. These points are for the briefs, or for a later edit to the AI guidance:
  - *Tools reach the worktree.* Read, Edit and Write take absolute paths, so they work. I tested ripgrep, which the search tools use, on a throwaway repo laid out the same way. A search from the repo root skips `.worktrees/`, which is what we want. A search given the worktree's path, or run from inside it, finds its files. So an engineer has to pass the worktree path to every search, or they will search `main` without noticing.
  - *The shell starts in the main checkout.* Agent shells reset their working directory between calls, so a bare `cargo build` or `npm test` runs in the main checkout, which breaks rule 2. Every command needs to name the worktree path. The brief should give that path as an absolute path and say this.
  - *Only one dev server at a time.* `apps/tool/vite.config.ts` pins port 1420 with `strictPort`, because Tauri's `devUrl` expects it, and `.claude/launch.json` starts the frontend from the main checkout. Two worktrees can't run `npm run tauri dev` or `npm run dev` at the same time; the second one fails to start. Builds and tests aren't affected. Until that changes, the brief says which checkout may run the app.
  - *Name the commands.* "The Tauri app build" in rule 11 could be read as `npm run tauri build`, the release build with installers, which is slow and isn't what a worktree needs. Measure from a fresh worktree: `cargo build --workspace` and `cargo test --workspace` at the worktree root, then `npm install`, `npm test` and `npm run build` in `apps/tool/`. Time each one, and measure the worktree folder's size at the end. Rule 9's rerun after a rebase is the same set, without `npm install` unless `package-lock.json` changed.
- **Marram (product manager), on the first version:** serializing code work didn't hurt Phase 1's sequencing, since 1.1, 1.2, 1.3 and 1.5 are sequential and what gates them is mostly docs work; Marram slices so one code step is in flight where possible, and keeps the other engineer on reviews and checking demos. That still holds; worktrees now cover the case where two code steps can usefully overlap.
- **Marram (product manager), on the first version:** `README.md` counts as code, and every demo's run steps live there. Demo steps are written by whoever holds the code at the time, in their own checkout, not by a parallel docs agent.
- **Marram (product manager), on review of the revision:** I approve. It works for slicing Phase 1. These points are for the plan and the briefs, not blockers:
  - *Worktrees are the exception in Phase 1, not the default.* 1.1 is the critical path and 1.2 builds on 1.1's view request, so most of Phase 1 is still one code step at a time in the main checkout. The likely overlaps are 1.2 with 1.3 or 1.5 once 1.1 has landed, and those share the view request, which is exactly Yew's same-file cost at the rebase. I'll slice them so each touches different files where possible, and when they can't, the second one waits rather than racing to the rebase.
  - *`README.md` is one writer at a time, like the lockfiles.* Every slice adds its demo steps to `README.md`, so two in-flight branches would both edit it and conflict at the rebase. The brief says which branch holds `README.md`; the other writes its demo steps into its report, and the facilitator or the holder adds them after landing.
  - *A demo counts once it has landed.* With one dev server at a time (Ash), and Ian running demos from the main checkout, a slice's demo is shown from `main` after rule 9's landing and rerun, never from a worktree. A slice isn't done while its branch is still open. This matches the roadmap: "each demo comes with a commit on `main`".
  - *Measure on real work.* Rule 11's measurement happens on the first slice that actually gets a worktree, not as a separate chore before it. It shouldn't delay a slice.

# AI guidance

- Before starting parallel work, know your checkout (the main one, or the worktree in your brief), your branch, and which files are yours. Change only those.
- If you are changing code in parallel with another engineer, work only in your worktree under `.worktrees/`. Don't create, remove or switch worktrees or branches yourself; the facilitator does.
- Within one checkout, don't install packages, build or run tests while another agent is changing code there.
- Don't add, remove or upgrade dependencies unless your brief says your branch holds the lockfiles.
- Leave `docs/log.md`, the `index.md` files and `docs/roadmap.md` to the facilitator unless your brief gives one to you, and use the record or question number your brief gives you.
- In a worktree, commit on your own branch when your brief says to. Never commit to, merge into or rebase `main`.
- Never `git add -A`, `git add .` or `git commit -a`. Add by path.
- If `git status` shows changes you didn't make, leave them alone and say so.
- On your first worktree, time the first full build and note the disk it uses, and report both.

# Links

- [DR-0021](/decisions/0021-work-lands-on-main.md) work lands on main; branches and worktrees allowed
- [DR-0009](/decisions/archive/0009-commit-straight-to-main.md) commit straight to main (superseded)
- [DR-0010](/decisions/0010-facilitator-manages-context.md) facilitator manages context
- [DR-0012](/decisions/0012-code-layout-and-core-crate.md) precedent for an engineer approving in the architect's place
- [DR-0018](/decisions/0018-one-way-doors-go-to-ian.md) one-way doors go to Ian
