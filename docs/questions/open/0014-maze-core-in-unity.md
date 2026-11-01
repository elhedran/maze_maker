---
type: Open Question
title: How would maze-core attach to a Unity game?
description: Ian's long-term intent is a 3D Unity game built on the infinite maze, with the generator running in its own thread inside it; how maze-core crosses into Unity (FFI, threading, how growth requests and results cross) is open. Revisit before the 3D game starts.
status: draft
question_id: Q-0014
question_status: open
waiting_on: [architect/1]
raised_by: facilitator/1
raised_at: 2026-11-01T10:26:00Z
revisit: before the 3D game starts
related:
  - /decisions/0012-code-layout-and-core-crate.md
  - /decisions/0035-the-world-is-stored.md
  - /decisions/0036-growth-from-open-endpoints.md
  - /decisions/0042-the-game-starts-now.md
  - /roadmap.md
tags: [game, unity, ffi, threading, maze-core]
generated:
  by: facilitator/1
  at: 2026-11-01T10:26:00Z
---

# Question

How does `maze-core` attach to a Unity game: the FFI (for example a C ABI native plugin), threading (the World running in its own thread), and how growth requests and their results cross between the game and the generator?

# Context

Ian (`human:ian`), 2026-11-01, verbatim:

> I suppose this is where I give some additional context.  The very long term intent is to use this game as a minimal tech demo of a given approach (infinite maze) - we will later make a different 3d game (probably in unity) based on it - but that will need human designed art assets etc.  Its impact here is just to keep in mind while we are making a 2d game, part of it beyond being fun is to learn what we bring to the 3d game, and hopefully have a maze generation algorithm in that can be attached to that unity game running in its own thread and working with the unity game.

Until then, the constraints that keep this open are in DR-0043 (how code is shared with the game): `maze-core` stays engine-agnostic, its World is usable from its own thread, it is deterministic and seeded, and its API stays simple enough to sit behind a C ABI.

# Not now

Don't answer this yet (bias to execution). Revisit before the 3D game starts.
