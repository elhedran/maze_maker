---
type: Decision Record
title: Open Knowledge Format for all docs
description: The docs folder is a strict OKF v0.2 bundle with a fixed layout for decisions, questions and plans.
status: stable
decision_id: DR-0003
decision_status: accepted
decision_kind: process
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [process, docs, okf]
sources:
  - id: okf
    resource: https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md
    title: Open Knowledge Format specification v0.2
generated:
  by: facilitator/1
  at: 2026-10-31T14:39:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

The docs are read by Ian and by agents. A strict, simple format keeps them consistent and lets agents rely on the metadata.

# Decision

`docs/` is an Open Knowledge Format (OKF) v0.2 bundle[^okf], followed strictly.

- Every `.md` file in `docs/` except `index.md` and `log.md` starts with YAML frontmatter with a non-empty `type`. Also set `title`, a one-sentence `description`, and `tags`.
- Authorship and approval use `generated` and `verified`, with the actor IDs from [DR-0002](/decisions/0002-approval-and-escalation.md). Timestamps are ISO 8601 with an explicit UTC offset.
- Links are standard Markdown links and bundle-relative: a leading `/` means `docs/`.
- Every folder that holds documents has an `index.md` listing them under headings, each with its one-sentence description. Whoever adds, moves or archives a document updates the index.
- `docs/log.md` records changes newest first, under `YYYY-MM-DD` headings, each line starting with a bold label: `**Creation**`, `**Update**`, `**Decision**`, `**Archive**` or `**Question**`.
- `README.md`, `CLAUDE.md` and `.claude/` at the repository root are outside the bundle.

Layout:

| Path | Contents | `type` |
|---|---|---|
| `decisions/` | decision records in use | `Decision Record` |
| `decisions/archive/` | rejected, superseded and deprecated decision records | `Decision Record` |
| `questions/open/` | questions waiting on someone | `Open Question` |
| `questions/resolved/` | answered or withdrawn questions | `Open Question` |
| `plans/` | designs and task plans for major work | `Plan` |
| `roadmap.md` | phases, deliverables and demos; owned by the product manager, approved by Ian | `Roadmap` |

Plans are written for major work only.

# Alternatives considered

- **Free-form Markdown.** Nothing to learn, but it drifts, and agents can't rely on metadata.
- **A wiki or external doc tool.** Out of reach of the agents and of git history.

# Consequences

- Bundle-relative links (`/decisions/...`) don't resolve in GitHub's or VS Code's Markdown preview, which read `/` as the repository root. We accept that: these links survive archive moves, and they're what the spec recommends.
- OKF's `status` only allows `draft`, `stable` and `deprecated`, so decisions and questions carry their own status key alongside it ([DR-0001](/decisions/0001-record-every-decision.md), [DR-0004](/decisions/0004-open-questions.md)).
- Indexes are kept by hand for now. A script to generate them may come later.

# Concerns raised

None.

# AI guidance

- New document: write the frontmatter first (`type`, `title`, `description`, `tags`, `generated`), then update the folder's `index.md` and `docs/log.md`.
- Don't add document folders or `type` values outside the layout above without a decision record.

# Links

- Decision records: [DR-0001](/decisions/0001-record-every-decision.md)

[^okf]: https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md
