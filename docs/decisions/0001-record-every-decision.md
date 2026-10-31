---
type: Decision Record
title: Record every decision as a decision record
description: Every project decision gets its own decision record in docs/decisions, with a lifecycle and an archive.
status: stable
decision_id: DR-0001
decision_status: accepted
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [process, decisions]
sources:
  - id: glukhov
    resource: https://www.glukhov.org/app-architecture/documentation/decision-records-ai-driven-development/
    title: Decision records for AI-driven development
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

Most of the team is AI agents. Code shows what was built but not why, and agents lose context between sessions. Without written decisions they reopen settled debates or quietly break constraints they never knew about.

# Decision

Every decision gets a decision record (DR): one Markdown file per decision in `docs/decisions/`, following the decision-records approach[^glukhov] and written as an OKF concept ([DR-0003](/decisions/0003-open-knowledge-format-for-docs.md)).

- **Kinds:** `architecture`, `product`, `design` (UX), `game-design`, `process`. One flat folder; the kind is a frontmatter key, not a subfolder.
- **Numbering:** `NNNN-short-slug.md`, four digits, one global sequence, never reused. The ID is `DR-NNNN`.
- **Lifecycle:** `proposed` becomes `accepted` or `rejected`. An accepted record later becomes `superseded` (a newer record replaces it) or `deprecated` (no longer followed, nothing replaces it).
- **Archive:** only records in use (`proposed`, `accepted`) stay in `docs/decisions/`. Anything `rejected`, `superseded` or `deprecated` moves (`git mv`) to `docs/decisions/archive/` with the same file name.
- **Changing direction:** never rewrite an accepted record. Write a new one that `supersedes` it, set `superseded_by` on the old one, then archive the old one.
- **Approval:** who must approve is set by [DR-0002](/decisions/0002-approval-and-escalation.md). A record is `accepted` only when every actor in `approvers` has signed in `verified`.

# Schema

```yaml
type: Decision Record
title: <short title>
description: <one sentence>
status: draft | stable | deprecated     # OKF: proposed -> draft, accepted -> stable, anything else -> deprecated
decision_id: DR-NNNN
decision_status: proposed | accepted | rejected | superseded | deprecated
decision_kind: architecture | product | design | game-design | process
major: true | false                     # true means human:ian must approve (DR-0002)
owners: [<actor>]                       # who drives the decision
approvers: [<actor>]                    # who must sign before it is accepted
decided_at: <ISO 8601 with offset>      # set when accepted or rejected
supersedes: [</decisions/...md>]
superseded_by: </decisions/...md>
tags: [<tag>]
generated: { by: <actor>, at: <ISO 8601 with offset> }
verified:                               # one entry per approval
  - { by: <actor>, at: <ISO 8601 with offset> }
```

Body sections, in order:

- `# Context`
- `# Decision`
- `# Alternatives considered`
- `# Consequences`
- `# Concerns raised`: disagreement, attributed to whoever raised it. It stays in the record after the decision is accepted.
- `# AI guidance`
- `# Links`

# Alternatives considered

- **Decisions only in chat or commit messages.** No effort up front, but invisible to agents in later sessions.
- **One running decision log file.** Simple, but superseding and archiving single decisions gets messy, and it breaks OKF's one concept per file.
- **A subfolder per kind** (`architecture/`, `product/`, ...). The article's suggestion; one flat folder is easier to scan at our size.

# Consequences

- Each decision costs a few minutes of writing. Trivial choices (local naming, obvious bug fixes) don't need a record: if reversing it would need a discussion, it needs a record.
- `docs/decisions/` only ever shows what is in force, so agents can read it as the current rules.
- Two status keys (`status` for OKF, `decision_status` for us) must be kept in step.

# Concerns raised

None.

# AI guidance

- Before changing code or docs, read `/decisions/index.md` and the accepted records that apply, and summarize the constraints before starting.
- Follow accepted records. If a task conflicts with one, stop and say so; don't work around it.
- Never edit the decision in an accepted record. Propose a superseding record instead.
- When you finish, say which records shaped the work.
- Draft the record before implementing, while the alternatives are still fresh.

# Links

- Approval rules: [DR-0002](/decisions/0002-approval-and-escalation.md)
- Document format: [DR-0003](/decisions/0003-open-knowledge-format-for-docs.md)

[^glukhov]: https://www.glukhov.org/app-architecture/documentation/decision-records-ai-driven-development/
