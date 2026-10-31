---
type: Decision Record
title: Easily reversed decisions are the team's; only one-way doors go to Ian
description: If a decision can be reversed without a lot of extra work, the team decides it; only decisions that would be costly to reverse, and disputes on them, go to Ian.
status: stable
decision_id: DR-0018
decision_status: accepted
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T19:04:00+01:00
supersedes: [/decisions/archive/0002-approval-and-escalation.md]
tags: [process, decisions, team, escalation]
generated:
  by: facilitator/1
  at: 2026-10-31T19:00:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T19:04:00+01:00
---

# Context

[DR-0002](/decisions/archive/0002-approval-and-escalation.md) sent every "major" decision to Ian and said: if unsure whether it's major, treat it as major and ask. It expected the line to be learned over time.

On 2026-10-31 the team sent Ian roughly fifteen detail questions: door rules, weights, room walls, hedge, the grid shape and the code layout. The tool exists so details like these can be tuned later by reviewing sample mazes, so they never needed Ian up front. Ian's words:

> "too many details in the maze came to me and the intent of the tool was to be able to defer those. The whole team needs to learn about two way door decisions (they make) versus one way door decisions (I make)."

Approving [DR-0012](/decisions/0012-code-layout-and-core-crate.md) (code layout) and [DR-0014](/decisions/0014-square-grid.md) (square grid):

> "approve both, but neither decision should have been blocked on me"

And, on the first draft of this record:

> "decisions that can easily be undoable can be left to the AI to decide - basically if we can reverse it, it doesn't create a lot of extra work, then it probably doesn't need me to decide."

Yew had judged the grid a one-way door, and Marram had judged the weight model one. Ian disagreed both times. The bar for what reaches him is higher than the team has been setting it.

This record supersedes DR-0002. It carries over what DR-0002 got right (actors, approvers, overrules) and replaces how decisions reach Ian.

# Decision

## The rule

One test: **if we can reverse it without a lot of extra work, the team decides it.** Ian doesn't need to.

- **Two-way doors** can be undone easily. The owning roles decide them, with the approvers in the table below, and move on. They don't go to Ian and don't wait on him. If one turns out wrong, reverse it with a superseding record.
- **One-way doors** would create a lot of extra work to reverse. Only these go to Ian.

The test is the cost of reversing, not how important the decision feels. A foundational decision is still a two-way door if undoing it is a refactor, a regenerated table or an edited record.

`major: true` in a record means a one-way door: `human:ian` is in `approvers`, and the record says in a sentence what reversing it would cost.

## Examples

**The team's call.** All of these came to Ian on 2026-10-31 and shouldn't have:

- the tile set and its sides ([DR-0015](/decisions/0015-tile-set.md));
- tile glyphs and colours ([DR-0017](/decisions/0017-tile-glyphs-and-colours.md));
- probabilities, weights and the weight model ([DR-0013](/decisions/0013-table-entries-from-tile-sides.md), Q-0007);
- door, path, room-wall and hedge rules (Q-0007);
- the code layout ([DR-0012](/decisions/0012-code-layout-and-core-crate.md));
- the grid shape ([DR-0014](/decisions/0014-square-grid.md));
- the order of work and what goes into each deliverable, within the roadmap's phases ([Q-0010](/questions/resolved/0010-where-editing-goes.md)). Ian approves the roadmap's phases and deliverables; Marram reorders within them.

Anything the tool exists to explore and tune is the team's call.

**Still Ian's call**, because reversing would create a lot of work or the cost falls on Ian:

- changing the tech stack: the framework, the language or the generation approach ([DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md), [DR-0007](/decisions/0007-wave-function-collapse.md));
- anything that throws away a lot of finished work;
- anything that ties down Ian's own time or money, such as paid services, accounts or publishing something. This one is added on purpose: it can be cheap to reverse, but the cost falls on Ian, so he decides it.

These are examples of the test, not a list to check against. Something not on either list is judged by the test.

## When it's unclear

When it's unclear whether reversing would cost a lot, the owning role doesn't ask Ian. It asks:

- **Yew** (architect), for architecture and technical decisions;
- **Marram** (product manager), for product, UX and game-design decisions.

Their call settles it, and it doesn't wait on Ian. He sees it in the facilitator's report, like any other two-way door.

## Approvers

| Kind | Approvers (two-way door) |
|---|---|
| `architecture` | architect and one engineer |
| `product` | product manager |
| `design` (UX) | UX designer and product manager |
| `game-design` | game designer and product manager |
| `process` | product manager and architect; Ian, if it changes how decisions reach him, who approves what, the overrules, or `CLAUDE.md` or the agent files |

A one-way door adds `human:ian` to `approvers`; the relevant team members still review it first. Marram takes minor product decisions alone. Nobody approves their own record. If the author is a required approver, another relevant team member (or Ian) takes their place.

## Actors

Approvals are OKF `verified` entries using these actor IDs:

- `human:ian`: Ian.
- `<agent-name>/<version>`: a team agent. `<agent-name>` is its file name in `.claude/agents/`; `<version>` goes up when its persona changes meaningfully. For example `architect/1`.
- `facilitator/1`: the main session. The facilitator drafts records and convenes the team. It never approves.

## Disagreement

- The team argues with each other, not with Ian. A team member who disagrees with any decision, including Ian's, gets their concern written into `# Concerns raised`, attributed to them. Ian has the final say.
- **Overrules.** Only these:
  - the architect can overrule the engineers and the product manager;
  - the product manager can overrule the engineers and the UX designer on product matters: scope, priorities, sequencing, and what goes into each deliverable;
  - the UX designer can overrule the engineers on how things look, feel and work for the user.

  When someone is overruled, their concern goes into `# Concerns raised`, and the record says whose call it was.
- **Escalate only real disputes.** On a two-way door, a disagreement is settled by trying it: pick one, record the concern, and let review of the tool or the sample mazes show who was right. The role that owns the record's `decision_kind` picks what to try first (Wren for game design, Damson for UX, and so on); if ownership is shared or unclear, Yew picks for technical decisions and Marram for product. A dispute goes to Ian as an open question ([DR-0004](/decisions/0004-open-questions.md)) only when it's on a one-way door. Each position is stated and attributed. It is not settled by majority vote or by the facilitator.
- **Ian has an architecture background.** When an architectural decision does go to him, pitch it at that level and defer to him.

## Point of contact

Ian talks to the facilitator. The facilitator convenes the team, relays each position attributed to whoever holds it, and brings one-way doors and real disputes to Ian. Every report to Ian lists, briefly, the two-way doors the team took and any overrules, so he can reopen one. This list is never skipped.

# Alternatives considered

- **Keep DR-0002, and learn the line case by case.** That is what produced about fifteen questions in one session. Ian has now drawn the line.
- **Ian approves everything.** Safest, but makes him the bottleneck, and defeats the tool's purpose of deferring details to review.
- **Categories that always go to Ian** (process, product direction, architecture). Simple to apply, but it sends Ian easily reversed decisions again. The cost of reversing is the only test.

# Consequences

- Far fewer questions reach Ian. Most open details of the table and tile set (Q-0007, Q-0008) are now the team's.
- The team will sometimes get a two-way door wrong. That's expected: the tool is built to show it, and the decision is reversed with a superseding record.
- Yew and Marram carry the "is it one-way?" judgement. The safety net is the facilitator's report: it lists the two-way doors the team took, so Ian can reopen any of them.
- `CLAUDE.md` and every agent in `.claude/agents/` are updated to apply this rule.

# Concerns raised

- **Yew (architect):** reversibility decays. The grid and tile sides are cheap to change today, but stop being cheap once the generator and the game build on them. Before major work builds on a foundational decision, Yew checks again whether it's still a two-way door.
- **Marram (product manager):** Yew and Marram now carry the "is it one-way?" judgement. The safety net is the facilitator listing the two-way doors taken in every report to Ian, so that list must not be skipped.

# AI guidance

- Before asking Ian anything, ask: could we reverse this without a lot of extra work? If yes, decide it with the right approvers and move on.
- If you can't tell, ask Yew (technical) or Marram (product), not Ian.
- Don't send Ian a menu of details. If something must go to him, say what's being decided, why it's one-way, and the team's recommendation.
- Set `major: true` only for one-way doors, and say in the record why it's one.
- Only sign `verified` as yourself. Never write a `human:ian` entry unless Ian approved in conversation.
- Write dissent into `# Concerns raised`; don't smooth it out of the record.

# Links

- Superseded: [DR-0002](/decisions/archive/0002-approval-and-escalation.md)
- Decision records: [DR-0001](/decisions/0001-record-every-decision.md)
- Open questions: [DR-0004](/decisions/0004-open-questions.md)
