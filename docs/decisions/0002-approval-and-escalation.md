---
type: Decision Record
title: Approval and escalation rules
description: Ian approves major decisions and settles disputes; other decisions are approved by the relevant team members.
status: draft
decision_id: DR-0002
decision_status: proposed
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
supersedes: []
tags: [process, decisions, team]
generated:
  by: facilitator/1
  at: 2026-10-31T14:39:00+01:00
---

# Context

Ian works with a team of AI agents (game designer, two engineers, architect, UX designer, product manager) through a facilitator, the main Claude session. Ian wants the final say on anything significant without approving every small call. He wants the team to argue with each other, not with him.

# Decision

**Actors.** Approvals are OKF `verified` entries using these actor IDs:

- `human:ian`: Ian.
- `<agent-name>/<version>`: a team agent. `<agent-name>` is its file name in `.claude/agents/`; `<version>` goes up when its persona changes meaningfully. For example `architect/1`.
- `facilitator/1`: the main session. The facilitator drafts records and convenes the team. It never approves.

**Major decisions need Ian.** A decision is major if it:

- chooses or changes a technology, framework or significant library;
- is a major architectural decision (structure, data model, boundaries between parts);
- changes how the team works (every `process` decision);
- sets major product direction;
- would be expensive to reverse (a "one-way door"); or
- is one the team can't agree on.

**If it's unclear whether a decision is major, treat it as major and ask Ian.** A major record lists `human:ian` in `approvers`; the relevant team members still review it.

**Other decisions** are approved by:

| Kind | Approvers |
|---|---|
| `architecture` | architect and one engineer |
| `product` | product manager |
| `design` (UX) | UX designer and product manager |
| `game-design` | game designer and product manager |
| `process` | always major |

The product manager takes minor product decisions and "two-way-door" decisions (cheap to reverse) alone, so Ian can focus on the bigger picture.

Nobody approves their own record. If the author is a required approver, another relevant team member (or Ian) takes their place.

**Disagreement.**

- The team argues with each other, not with Ian. A team member who disagrees with any decision, including Ian's, gets their concern written into `# Concerns raised`, attributed to them. Ian has the final say.
- **Who can overrule whom.** Only these:
  - the architect can overrule the engineers and the product manager;
  - the product manager can overrule the engineers and the UX designer on product matters: scope, priorities, sequencing, and what goes into each deliverable;
  - the UX designer can overrule the engineers on how things look, feel and work for the user.

  When someone is overruled, their concern goes into `# Concerns raised`, and the record says whose call it was. Major decisions still need Ian.
- Any other disagreement goes to Ian as an open question ([DR-0004](/decisions/0004-open-questions.md)), with each position stated and attributed. That includes the architect and the UX designer pulling the engineers in different directions. It is not settled by majority vote or by the facilitator.
- **Ian has an architecture background.** On anything architectural, if in any doubt, defer to him.

**Point of contact.** Ian talks to the facilitator. The facilitator convenes the team, relays each position attributed to whoever holds it, and brings decisions and open questions to Ian.

# Alternatives considered

- **Ian approves everything.** Safest, but makes Ian a bottleneck for trivial calls.
- **A written threshold for "major" up front.** Faster, but we don't know where the line is yet. We'll learn it, erring towards asking.
- **Majority vote among agents for disputes.** Agents can talk each other into things, and disputes are exactly where Ian's judgement is wanted.
- **Every dispute goes to Ian, with no overrules.** Simplest rule, but Ian would become the referee for every engineering, UX and scope argument.

# Consequences

- Early on, many decisions will come to Ian. That should drop as the line for "major" becomes clearer; refine it with a superseding record.
- Concerns stay visible in the record even when overruled.
- Overrules keep work moving without waiting on Ian for every disagreement. The facilitator mentions overrules when reporting to Ian, so he can reopen one if he wants.

# Concerns raised

None.

# AI guidance

- Set `major` honestly. When in doubt, set `major: true` and ask.
- Only sign `verified` as yourself. Never write a `human:ian` entry unless Ian approved in conversation.
- Write dissent into `# Concerns raised`; don't smooth it out of the record.

# Links

- Decision records: [DR-0001](/decisions/0001-record-every-decision.md)
- Open questions: [DR-0004](/decisions/0004-open-questions.md)
