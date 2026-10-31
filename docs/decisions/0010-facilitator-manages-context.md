---
type: Decision Record
title: The facilitator manages context
description: The facilitator's main job is managing context, giving each task fresh team sessions and telling Ian when to clear his session.
status: stable
decision_id: DR-0010
decision_status: accepted
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [process, facilitator, context, team]
generated:
  by: facilitator/1
  at: 2026-10-31T14:42:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

Agent sessions get worse as their context grows. Stale details linger, earlier tasks bleed into new ones, and every turn costs more. Both the team agents and Ian's own session with the facilitator build up context over time. The project's durable knowledge already lives in the repo (decisions, questions, plans, the log, git), so sessions can be disposable.

# Decision

The facilitator's main job is **managing context**, for the team and for Ian's session.

- **Fresh team sessions for each task.** Each task starts with fresh agent sessions. Within a task, the facilitator may keep talking to the same agent session. Once the task is done, that session is dropped and the next task starts fresh. A task is a piece of work with its own outcome: a plan step, a decision, a question resolved, a fix.
- **Complete briefs.** A fresh agent knows nothing. Its brief covers the task, the decision records, plan and questions that apply, and where things stand.
- **Durable state lives in the repo.** That means decisions, questions, plans, the roadmap, `docs/log.md` and commits, never only a session's memory. Anything a later session will need is written down before a session is dropped.
- **Keep Ian's session lean.** The facilitator hands reading, exploring and building to the team, and brings back summaries rather than file contents.
- **Tell Ian when to clear his session:**
  - when a task is finished and committed;
  - before switching to unrelated work;
  - when the conversation has grown long or has been compacted.

  Before suggesting a clear, the facilitator makes sure everything worth keeping is written down and committed, or says what isn't. It also says what the next session should start with.

# Alternatives considered

- **Long-lived agent sessions per role.** Agents would remember earlier work, but they'd build up stale context and drift, and what they "remember" is invisible to Ian.
- **Ian manages context himself.** No rules needed, but it adds to Ian's load, which is what the facilitator is there to take away.

# Consequences

- Fresh agents spend some effort at the start of each task reading the records that apply. Good briefs keep that small.
- Continuity comes from the docs, not from agents' memories, so the docs must stay current. The process already demands that.
- Ian gets regular prompts to clear his session, at points where nothing is lost by doing so.

# Concerns raised

None.

# AI guidance

- Facilitator: start each task's agents with a new session; don't continue one from an earlier task. Never rely on an agent remembering earlier work. Put what it needs in the brief.
- Before dropping a session or suggesting Ian clear his, update the docs (log, questions, plan status), and get the work committed or say what isn't.

# Links

- Approval and the facilitator's role: [DR-0002](/decisions/archive/0002-approval-and-escalation.md)
- Commits: [DR-0009](/decisions/0009-commit-straight-to-main.md)
