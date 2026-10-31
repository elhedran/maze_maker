---
type: Open Question
title: TypeScript and Vite for the Lit frontend?
description: Choose the language and build tooling for the Lit frontend.
status: stable
question_id: Q-0002
question_status: resolved
waiting_on: [human:ian]
raised_by: facilitator/1
raised_at: 2026-10-31T14:15:00+01:00
resolved_by: human:ian
resolved_at: 2026-10-31T14:46:00+01:00
resolution_record: /decisions/0011-typescript-and-vite.md
related:
  - /decisions/0005-tauri-rust-backend-lit-frontend.md
  - /decisions/0006-npm-for-frontend-packages.md
tags: [technology, frontend, lit]
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
---

# Question

Should the Lit frontend be written in TypeScript and built with Vite?

# Context

Vite is Tauri's usual frontend tooling, and Lit works with it in either TypeScript or JavaScript. This is a technology choice, so it's Ian's ([DR-0002](/decisions/0002-approval-and-escalation.md)). It isn't needed until the phase 1 tool is scaffolded.

# Options

- **TypeScript and Vite.** Favoured by `facilitator/1`. Lit's decorators and typed properties work best in TypeScript, and types on the boundary between Rust and the frontend catch mistakes early. Vite handles the compile step and gives hot reload.
- **JavaScript and Vite.** No type layer: less ceremony, more mistakes found at runtime.
- **No bundler (plain ES modules).** Simplest setup, but no hot reload and awkward use of npm packages.

# Recommendation

TypeScript and Vite (`facilitator/1`). The team isn't formed yet; the architect and engineers can weigh in once they exist.

# Resolution

Ian approved the recommendation on 2026-10-31: TypeScript and Vite. Recorded as [DR-0011](/decisions/0011-typescript-and-vite.md).
