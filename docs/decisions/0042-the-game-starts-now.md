---
type: Decision Record
title: The game starts now, alongside Phase 2
description: Ian's decision that work on the game starts now, in parallel with Phase 2, because the tool is good enough to build on while it keeps improving; amends DR-0008's "the game comes after".
status: stable
decision_id: DR-0042
decision_status: proposed
decision_kind: product
major: false   # two-way door: the game can pause again at no cost to the tool; DR-0008's one-way parts (two apps, tool first) are unchanged
owners: [human:ian]
approvers: [human:ian, product-manager/1, architect/1]   # Ian's decision; facilitator/1 drafted it; process and product approvers per CLAUDE.md
amends: [DR-0008]
supersedes: []
tags: [roadmap, game, sequencing, slice-g-1]
generated:
  by: facilitator/1
  at: 2026-11-01T10:20:41Z
verified:
  - by: human:ian
    at: 2026-11-01T10:20:41Z
    note: "Said to the parent facilitator (session_01AqTzintpyngWme4BK9LQr6) on 2026-11-01 and relayed to this session; time is when it was recorded. Verbatim: \"That said - this should not hold up attempting the actual game now - the tooling shows we are \"good enough\" to work on the game while we improve the tool\". Then: \"I'm happy for both existing branches to land - this advice is more for future work\". The proposals below are the facilitator's, not Ian's."
---

# Context

[DR-0008](/decisions/0008-adjacency-tool-first.md) put the tool first: Phase 1 the table, Phase 2 sample mazes, and "the game comes after, as its own app". The roadmap read that as the game starting after Phase 2.

On 2026-11-01 Ian (`human:ian`) steered the random walker (2.6 step 2b, not part of this record) and then said, verbatim:

> That said - this should not hold up attempting the actual game now - the tooling shows we are "good enough" to work on the game while we improve the tool

and then:

> I'm happy for both existing branches to land - this advice is more for future work

# Decision

Ian's words:

1. The game starts now. The tool is "good enough" to build the game on, and keeps improving at the same time.

The facilitator's proposals (Ian can strike any of them):

2. DR-0008's "the game comes after" is amended to "the game starts once the tool is good enough to build on, and runs alongside it". The rest of DR-0008 stands: the tool and the game are separate apps, and the tool came first.
3. The game runs as its own track, in parallel with Phase 2's tracks (2.3, 2.6 step 2b and later slices), which carry on as planned. Starting the game does not pause or reorder them.
4. The game starts with G.1, "Walk a growing world", as the roadmap names it. Its gates are settled at the start of the build, like Phase 2's, and the engineers can build against stubs while they settle.
5. The game reuses `maze-core`'s World ([DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md)). Changes to `maze-core`'s public behaviour that the game needs are raised with whoever runs the other tracks in `maze-core` first.

# Alternatives considered

- **Keep the game until after Phase 2.** Matches DR-0008 as written, but Ian said the tool should "not hold up attempting the actual game now".

# Consequences

- Three or more tracks run at once, two of them in `maze-core`. Clashes in `world.rs`, `wfc.rs` and `trace.rs` are more likely. Game additions to `maze-core` stay in their own contiguous blocks.
- Problems found in the game can feed back into the tool's tuning sooner.
- The game's gates in the roadmap (code sharing, game-design records) are settled now, rather than after Phase 2.

# Concerns raised

None yet.

# AI guidance

- The game is in scope now. DR-0008's "don't start on game features without a decision record" still applies: G.1's records are DR-0043 to DR-0045.
- Don't pause Phase 2 work for the game, or the game for Phase 2, without Marram's call.

# Links

- Amends [DR-0008](/decisions/0008-adjacency-tool-first.md)
- [DR-0035](/decisions/0035-the-world-is-stored.md), [DR-0036](/decisions/0036-growth-from-open-endpoints.md)
- [Roadmap](/roadmap.md)
