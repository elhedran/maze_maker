---
type: Decision Record
title: TypeScript and Vite for the Lit frontend
description: The Lit frontend is written in TypeScript and built with Vite.
status: stable
decision_id: DR-0011
decision_status: accepted
decision_kind: architecture
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [technology, frontend, lit, typescript, vite]
generated:
  by: facilitator/1
  at: 2026-10-31T14:46:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

[DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md) set the stack but left the frontend language and build tooling open. This was raised as [Q-0002](/questions/resolved/0002-typescript-and-vite.md). Ian approved the facilitator's recommendation on 2026-10-31. The architect and engineers had not yet weighed in.

# Decision

Write the Lit frontend in TypeScript and build it with Vite.

# Alternatives considered

- **JavaScript and Vite.** No type layer: less ceremony, more mistakes found at runtime.
- **No bundler (plain ES modules).** Simplest setup, but no hot reload and awkward use of npm packages.

# Consequences

- Lit's decorators and typed properties can be used as intended.
- Types on the boundary between Rust and the frontend catch mismatches early. They have to be kept in step with the Rust side.
- Vite handles the compile step and gives hot reload. It is Tauri's usual frontend tooling.
- TypeScript adds a compile step and a `tsconfig.json` to maintain.

# Concerns raised

None.

# AI guidance

- Write frontend code in TypeScript (`.ts`). Don't add `.js` source files.
- Build and serve the frontend with Vite, through npm ([DR-0006](/decisions/0006-npm-for-frontend-packages.md)).

# Links

- Stack: [DR-0005](/decisions/0005-tauri-rust-backend-lit-frontend.md)
- Package manager: [DR-0006](/decisions/0006-npm-for-frontend-packages.md)
- Question: [Q-0002](/questions/resolved/0002-typescript-and-vite.md)
