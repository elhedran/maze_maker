---
type: Decision Record
title: Commit straight to main
description: Work is committed directly to main, with no branches and no remote for now.
status: stable
decision_id: DR-0009
decision_status: accepted
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [process, git]
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

The repository is new and has no remote yet. Ian will add one later.

# Decision

Commit directly to `main`. No feature branches or pull requests for now.

# Alternatives considered

- **Branches and pull requests.** Gives a review step, but adds overhead without a remote or other human reviewers.

# Consequences

- No pull-request review step, so decision records and their approvals carry the review.
- Revisit when Ian adds a remote. That change gets a record that supersedes this one.

# Concerns raised

None.

# AI guidance

- Commits go to `main`. Don't create branches, and don't push.
- Keep commits small and focused, with messages that say what changed and why.

# Links

- Decision records: [DR-0001](/decisions/0001-record-every-decision.md)
