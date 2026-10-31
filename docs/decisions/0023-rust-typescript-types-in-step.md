---
type: Decision Record
title: Rust and TypeScript IPC types kept in step by ts-rs
description: The tool's IPC types are Rust structs in the app crate that derive ts-rs; the generated TypeScript is checked in, and a Rust test fails when it is stale.
status: stable
decision_id: DR-0023
decision_status: accepted
decided_at: 2026-10-31T19:35:32+01:00
decision_kind: architecture
major: false
owners: [architect/1]
approvers: [architect/1, engineer-ash/1]
supersedes: []
tags: [ipc, tauri, typescript, rust, tooling, phase-1]
generated:
  by: architect/1
  at: 2026-10-31T19:34:00+01:00
verified:
  - by: engineer-ash/1
    at: 2026-10-31T19:35:32+01:00
---

# Context

Slice 1.1 adds the first real Tauri command: a view of the adjacency table ([DR-0024](/decisions/0024-tile-set-file-format.md)). Its request and response are nested types with enums, and Rust and TypeScript both have to agree on them. [DR-0012](/decisions/0012-code-layout-and-core-crate.md) deferred how they stay in step, and said IPC types live in the app crate. The table is about 30 × 30 × 4 entries now. Ian (`human:ian`), verbatim, 2026-10-31: "make sure we bias towards executing planned work over re-planning or planning new work." So: cheap, now.

# Decision

- **Rust is the source of truth.** IPC types are plain structs and enums in `apps/tool/src-tauri/src/ipc.rs`, deriving `serde::Serialize`/`Deserialize` and `ts_rs::TS`. They are not the `maze-core` types: the app crate maps core values into them, so `maze-core` never gains ts-rs or IPC concerns.
- **Generated TypeScript is checked in** at `apps/tool/src/bindings/`, one `.ts` file per type. The frontend imports only from there for IPC shapes.
- **A Rust test fails when the bindings are stale.** It renders every exported type with ts-rs and compares it with the checked-in file. Running it with `UPDATE_BINDINGS=1` rewrites the files instead. So `cargo test --workspace` catches drift, and nobody needs a separate generate step to build the app.
- **One typed wrapper module**, `apps/tool/src/ipc.ts`, holds the command names and calls `invoke` with the generated types (Yew's point on [DR-0020](/decisions/0020-vitest-for-frontend-tests.md)). Components and `mockIPC` tests use it, so a renamed command or field fails `tsc`.
- **Conventions on IPC types:** `#[serde(rename_all = "camelCase")]` on structs, `snake_case` string values on enums, and no `u64`/`i64` (ts-rs maps them to `bigint`); use `u32` and `f64`.

**New dependency:** `ts-rs` (current release, default features, which follow serde's attributes) in `maze-tool` only. It is a proc-macro crate; most of what it pulls in (`syn`, `quote`) is already in `Cargo.lock` through Tauri. It changes `Cargo.lock`, so it lands on the branch that holds the lockfile ([DR-0019](/decisions/0019-parallel-work-in-one-checkout.md) rule 5). No npm dependency.

# Alternatives considered

- **Hand-written TypeScript with a JSON fixture produced by Rust.** No dependency, but the check is weak: JSON imported into TypeScript widens string enums to `string`, so the fixture can't be type-checked against the union types without runtime validation code we'd also have to keep in step.
- **tauri-specta.** Also generates typed command wrappers, but it is tied to Tauri's release cycle and heavier for one or two commands.
- **typeshare or JSON Schema (schemars plus a TypeScript generator).** A second toolchain: a CLI to install, or an npm generator, for what one derive does.

# Consequences

- Changing an IPC type is: edit Rust, run the test with `UPDATE_BINDINGS=1`, commit both. Forgetting the second step fails `cargo test`.
- The bindings are Rowan's output and Ash's input. Ash never edits them by hand.
- Two-way door: dropping ts-rs means hand-writing the files it last generated, which stay valid TypeScript.

# Concerns raised

- **Ash (`engineer-ash/1`), on review:** approved. One derive, a staleness test and one wrapper is the simplest thing that makes a renamed field fail `tsc`. Two notes for the build, not a new round: (1) the bindings test should also fail on a stale file that no type exports any more, so deleted types don't linger; (2) `f64` fields can carry `NaN`, which serde_json writes as `null` and the TypeScript type won't admit, so Rust must never send one (see DR-0024's row total).

# AI guidance

- Put IPC types in `apps/tool/src-tauri/src/ipc.rs`, not in `maze-core`.
- Never edit `apps/tool/src/bindings/` by hand. Change the Rust type and regenerate.
- Call commands through `apps/tool/src/ipc.ts`, not bare `invoke` strings.
- No 64-bit integers in IPC types.

# Links

- Code layout and IPC types in the app crate: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- The view command these types carry: [DR-0024](/decisions/0024-tile-set-file-format.md)
- Frontend tests and the typed-wrapper point: [DR-0020](/decisions/0020-vitest-for-frontend-tests.md)
- Lockfiles, one writer at a time: [DR-0019](/decisions/0019-parallel-work-in-one-checkout.md)
- Plan: [Phase 1](/plans/phase-1.md)
