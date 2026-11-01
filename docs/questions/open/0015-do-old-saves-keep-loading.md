---
type: Open Question
title: Do old saves keep loading across versions?
description: A one-way door for Ian before the game's first release - throw saves away between versions, migrate forward forever, or migrate cells and replay where tiles are gone. DR-0046 refuses other-version saves until then.
status: draft
question_id: Q-0015
question_status: open
waiting_on: [human:ian]
raised_by: architect/1
raised_at: 2026-11-01T10:45:00Z
revisit: before the game's first release
related:
  - /decisions/0035-the-world-is-stored.md
  - /decisions/0046-saving-and-loading-the-world.md
tags: [game, saves, versioning, slice-g-2]
generated:
  by: facilitator/1
  at: 2026-11-01T10:45:00Z
---

# Question

Before the first release, do we promise that old saves keep loading? Yew's options, from [DR-0046](/decisions/0046-saving-and-loading-the-world.md):

- **(a) Saves are thrown away between versions.** Cheapest; players lose their worlds on updates.
- **(b) A forward migration for every version bump, kept forever, and tile ids never reused.** Players keep their worlds; every format change costs a migration and its test.
- **(c) Migrate the cells, and replay the anchor log where a tile is gone.** A replayed world can differ from what the player saw, which DR-0036 rule 5 forbids for seen cells, so it only suits unseen worlds.

# Not now

Until the first release, DR-0046 refuses a save from another version with a message and leaves the file alone. Nothing needs Ian yet; revisit before the first release.
