---
type: Decision Record
title: Track open questions as files
description: Questions waiting on a decision live in docs/questions/open and move to docs/questions/resolved when answered.
status: stable
decision_id: DR-0004
decision_status: accepted
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [process, questions]
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

Ian wants one place to see what is waiting on him, and a record of what was asked and how it was answered.

# Decision

Any question that needs someone to decide is tracked as a file.

- An open question is `docs/questions/open/NNNN-short-slug.md`, with ID `Q-NNNN`. Questions have their own sequence, never reused.
- When it's answered or withdrawn, fill in `# Resolution` and move the file (`git mv`) to `docs/questions/resolved/`.
- If the answer is a decision, it also gets a decision record ([DR-0001](/decisions/0001-record-every-decision.md)), and the question links to it in `resolution_record`.
- `docs/questions/index.md` lists open questions under **Waiting on Ian** and **Waiting on the team**, followed by resolved ones.
- Disputes the team can't settle ([DR-0002](/decisions/archive/0002-approval-and-escalation.md)) are raised as questions waiting on Ian, with each position stated and attributed.

# Schema

```yaml
type: Open Question
title: <the question, short>
description: <one sentence>
status: draft | stable                  # OKF: open -> draft, resolved or withdrawn -> stable
question_id: Q-NNNN
question_status: open | resolved | withdrawn
waiting_on: [<actor>]
raised_by: <actor>
raised_at: <ISO 8601 with offset>
resolved_by: <actor>                    # once resolved
resolved_at: <ISO 8601 with offset>     # once resolved
resolution_record: </decisions/...md>   # when the answer is a decision
related: [</...md>]
tags: [<tag>]
generated: { by: <actor>, at: <ISO 8601 with offset> }
```

Body sections, in order:

- `# Question`
- `# Context`
- `# Options`: each with who favours it and why.
- `# Recommendation`: if anyone has one, attributed.
- `# Resolution`: filled in when resolved.

# Alternatives considered

- **Questions only in chat.** Easy to lose, and nothing shows what is waiting on Ian.
- **One questions file.** Simple, but it grows without bound and mixes open and settled questions.

# Consequences

- Ian can check `/questions/index.md` to see everything waiting on him.
- Every question costs a small file. Quick questions answered in the same conversation don't need one; anything left waiting does.

# Concerns raised

None.

# AI guidance

- If work is blocked on a decision you can't make, raise a question file instead of guessing, and say so.
- Keep `/questions/index.md` current when you raise, answer or move a question.

# Links

- Approval and escalation: [DR-0002](/decisions/archive/0002-approval-and-escalation.md)
- Questions: [index](/questions/index.md)
