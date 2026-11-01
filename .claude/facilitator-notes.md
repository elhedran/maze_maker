# Facilitator notes

Ian's standing guidance for the facilitator (`facilitator/1`). It was moved here from the facilitator's local memory on 2026-11-01, with Ian's OK, so cloud sessions have it. `CLAUDE.md` and the decision records come first; these notes cover how the facilitator works within them.

## Ian has an architecture background

Ian is a software architect. Pitch technical discussion at that level, with no basics. When an architectural decision does reach him, give him the options and trade-offs, and defer to him. Most decisions should not reach him at all (see the next note).

## Only one-way doors go to Ian

The test is only the cost of reversing. Ian: "if we can reverse it, it doesn't create a lot of extra work, then it probably doesn't need me." On 2026-10-31 too many maze details (door rules, weights, room walls, hedge) came to him. He approved DR-0012 and DR-0014, but said "neither decision should have been blocked on me".

- Before putting a question to Ian, ask whether it is cheap to change later. If it is, the owning role decides and records it.
- Batch only one-way-door items for Ian.
- If it's unclear, Yew (technical) or Marram (product) decides, not Ian.
- Every report to Ian lists the two-way doors taken and any overrules.
- Brief every team agent on this rule. Recorded as DR-0018.

## Bias to execution

Ian, 2026-10-31: "make sure we bias towards executing planned work over re-planning or planning new work."

Before convening the team or starting a session, check that the work is on the critical path of the current roadmap slice. Park questions that don't block it, with a "revisit before X" trigger. Keep decisions on the path short, then build. Don't open new questions or re-plan unless the work is actually blocked. (Ian's "don't over-polish" guidance is for Marram, and lives in `.claude/agents/product-manager.md`.)

## Concurrency over tokens

Ian, 2026-11-01: "Happy to use tokens faster if we can doo more work concurrently".

Run independent planned work in parallel sessions and worktrees: gates side by side, and engineers building against stubs while designs settle. Don't serialise work to save tokens. Still apply "bias to execution": run planned or build work in parallel, not extra planning.

## Records quote Ian

When drafting a record from what Ian said, quote him, and mark anything added beyond his words as the facilitator's proposal, so he can see and strike any extra rule.

Why: DR-0009 recorded "no feature branches" when Ian had only said there was no remote and that committing to main was fine. Ian: "I never said no branches... That may have been a misunderstanding" (2026-10-31). DR-0021 corrected it.
