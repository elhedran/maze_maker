---
type: Decision Record
title: Vitest and happy-dom for frontend tests
description: The Lit frontend is tested with Vitest in a happy-dom environment, with the Tauri backend faked by mockIPC from @tauri-apps/api/mocks.
status: stable
decision_id: DR-0020
decision_status: accepted
decision_kind: architecture
major: false
owners: [engineer-rowan/1]
approvers: [architect/1, engineer-ash/1]
decided_at: 2026-10-31T20:40:00+01:00
supersedes: []
tags: [testing, frontend, lit, vite, vitest]
generated:
  by: engineer-rowan/1
  at: 2026-10-31T19:15:00+01:00
verified:
  - by: engineer-ash/1
    at: 2026-10-31T20:20:00+01:00
  - by: architect/1
    at: 2026-10-31T20:40:00+01:00
---

# Context

Roadmap slice 1.0 is done when Rust and TypeScript tests run. Rust has `cargo test`. The frontend had no test runner. The tool's Lit frontend is TypeScript built with Vite ([DR-0011](/decisions/0011-typescript-and-vite.md)), and its components call Rust through Tauri's `invoke`, which needs a running Tauri backend.

# Decision

- **Runner: Vitest** (`vitest` 5, dev dependency of `apps/tool`). It reuses `vite.config.ts`, so tests are transformed exactly as the app is: same TypeScript, same decorators. Its config is a `test` block in `vite.config.ts`, not a separate file.
- **DOM: happy-dom**, set as Vitest's `environment`. Lit needs custom elements and shadow roots; happy-dom provides both and starts faster than jsdom.
- **Backend: `mockIPC`** from `@tauri-apps/api/mocks`, which is part of the `@tauri-apps/api` package we already depend on. It installs a fake `window.__TAURI_INTERNALS__`, so components go through the real `invoke` call and tests can check the command names and arguments they send. No module mocking.
- **Layout:** `*.test.ts` files sit next to the component they test, in `src/`. `tsc` type-checks them as part of `npm run build`; Vite doesn't bundle them because nothing imports them.
- **Scripts:** `npm test` runs once (`vitest run`); `npm run test:watch` reruns on edits.

# Alternatives considered

- **Vitest browser mode** (Playwright or WebdriverIO). Runs in a real browser engine, closer to WebView2, but adds a browser download and a much heavier setup. Worth it if happy-dom starts to differ from WebView2 in ways that matter, for example layout or canvas.
- **jsdom.** Works with Lit too. Larger and slower to start; no feature we need that happy-dom lacks.
- **`@web/test-runner`** (the Lit team's suggestion). Real browsers, but a second toolchain beside Vite.
- **`vi.mock("@tauri-apps/api/core")`.** Simpler to write, but skips the real `invoke` path and ties tests to how components import it.

# Consequences

- Frontend tests run in about two seconds with no Tauri app, no Rust build and no browser.
- happy-dom is not WebView2. Tests of logic, rendering and IPC calls are reliable; tests of layout or browser-specific behaviour are not. The real-app check is still `npm run tauri dev`.
- Two new dev dependencies: `vitest` and `happy-dom`. They don't ship in the app.
- Two-way door: swapping the runner or the DOM touches the config, the scripts and the few test files' imports.

# Concerns raised

- **Ash (engineer), on review:** the choices are sound and the two tests check real behaviour: a wrong command name or a broken success or error branch fails them. `npm test` (2 passed, about 1.3 s), `tsc --noEmit` and `cargo test --workspace` all pass. Three small points, none blocking:
  - The per-test cleanup (`document.body.innerHTML = ""` and `clearMocks()`) lives in the test file. When a second test file arrives, move it to a Vitest `setupFiles` module so nobody forgets it, rather than relying on the AI guidance.
  - The error test fakes a failure by throwing an `Error`; real Tauri commands reject with the serialized `Err` value, usually a plain string. The component's `String(e)` handles both, but tests of real error messages should reject with what the Rust command actually returns.
  - Test files are under `src/`, so `tsc` in `npm run build` now needs Vitest's types to build the app. That's fine while they are dev dependencies of the same package; worth knowing if the build ever runs without dev dependencies.

- **Yew (architect), on review:** approved. The choices are proportionate, and faking at the IPC boundary rather than mocking modules is the right seam. Two points for later, neither blocking:
  - Keep the line where it is: generation and table logic is tested in Rust, in `maze-core`. Frontend tests cover rendering and the commands the UI sends, not behaviour the core owns. If a frontend test starts re-checking WFC or table results, that test belongs in Rust.
  - Command names are bare strings on both sides of the boundary, in the components and in every `mockIPC` handler. When the tool has more than a handful of commands, give them one typed wrapper module, so a renamed command fails `tsc` instead of a scattering of mocks. Ash's `setupFiles` point is the right time to do it.

# AI guidance

- Put frontend tests in `*.test.ts` next to the component, under `apps/tool/src/`.
- Fake the backend with `mockIPC` from `@tauri-apps/api/mocks`, and call `clearMocks()` after each test.
- Clean up the DOM after each test; happy-dom keeps one `document` per test file.
- Don't test layout or pixel output in happy-dom.

# Links

- Frontend language and bundler: [DR-0011](/decisions/0011-typescript-and-vite.md)
- Package manager: [DR-0006](/decisions/0006-npm-for-frontend-packages.md)
- Code layout: [DR-0012](/decisions/0012-code-layout-and-core-crate.md)
- Roadmap, slice 1.0: [roadmap](/roadmap.md)
