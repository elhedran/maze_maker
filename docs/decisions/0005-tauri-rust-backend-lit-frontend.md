---
type: Decision Record
title: Tauri v2 with a Rust backend and Lit frontend
description: Apps are built with Tauri v2, with generation logic in Rust and the UI in Lit web components.
status: stable
decision_id: DR-0005
decision_status: accepted
decision_kind: architecture
major: true
owners: [human:ian]
approvers: [human:ian]
decided_at: 2026-10-31T14:46:00+01:00
supersedes: []
tags: [technology, tauri, rust, lit]
sources:
  - id: tauri
    resource: https://v2.tauri.app/
    title: Tauri v2
generated:
  by: facilitator/1
  at: 2026-10-31T14:15:00+01:00
verified:
  - by: human:ian
    at: 2026-10-31T14:46:00+01:00
---

# Context

We need desktop apps for both the tooling and the game. Ian has chosen the stack.

# Decision

Apps are built with Tauri v2[^tauri]:

- **Rust backend:** all generation logic, tile data and the adjacency table.
- **Lit frontend:** UI and rendering, as Lit web components.

# Alternatives considered

These weren't evaluated in depth; the stack is Ian's choice. Listed for context:

- **Electron and TypeScript.** One language throughout, but heavier, and no Rust for the generator.
- **A native Rust game engine (e.g. Bevy).** Strong for the game, weaker for table-heavy tool UIs.
- **Godot or Unity.** Full engines, but the generation logic would live in another runtime.

# Consequences

- Generation runs in Rust and the frontend only displays results. Moving large data (the adjacency table, maze chunks) over Tauri's IPC has a cost, so interfaces should page or stream it rather than send everything at once.
- Development machines need Rust and Tauri's Windows prerequisites (MSVC build tools, WebView2).
- The frontend language and bundler were left open in [Q-0002](/questions/resolved/0002-typescript-and-vite.md) and settled by [DR-0011](/decisions/0011-typescript-and-vite.md).

# Concerns raised

None.

# AI guidance

- Keep generation and data logic in Rust. Don't reimplement it in the frontend.
- Don't add another frontend framework (React, Vue, ...) or replace Tauri without a superseding major record.

# Links

- Package manager: [DR-0006](/decisions/0006-npm-for-frontend-packages.md)

[^tauri]: https://v2.tauri.app/
