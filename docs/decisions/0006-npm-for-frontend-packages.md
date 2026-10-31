---
type: Decision Record
title: npm for frontend packages
description: The frontend uses npm as its package manager.
status: stable
decision_id: DR-0006
decision_status: accepted
decision_kind: architecture
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [technology, npm, frontend]
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

The Lit frontend needs a JavaScript package manager. Node 24 and npm 11 are already installed.

# Decision

Use npm. Commit `package-lock.json`.

# Alternatives considered

- **pnpm.** Faster installs and stricter dependency resolution; another tool to install and learn.
- **bun.** Fast, and also a runtime; less mature on Windows and an extra moving part.

# Consequences

- Nothing extra to install.
- Commands in docs and scripts use `npm` and `npx`.

# Concerns raised

None.

# AI guidance

- Use `npm` only. Don't add `pnpm-lock.yaml`, `bun.lockb` or `yarn.lock`.

# Links

- Stack: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)
