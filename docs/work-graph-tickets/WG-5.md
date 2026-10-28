# WG-5: Make the graph and controls visually legible

## Parent

[Approved work graph specification](../WORK-GRAPH-SPEC.md), [issue #15](https://github.com/wagar-meilech/zoetrope/issues/15). This ticket is [#20](https://github.com/wagar-meilech/zoetrope/issues/20).

## What to build

Deliver the approved vertical behavior described by this ticket, following the specification and answered questionnaire. Preserve Rust/local efficiency, source attribution, deliberate exact-recipient control and portable replay.

## Acceptance criteria

- [x] Consistent status colors and glyphs, labeled views/breadcrumb/actions/legend, compact details and semantic zoom make graphs readable rather than text walls.
- [x] Reported-work counts and verified-outcome progress remain separate with source/freshness and cancellation/unknown states handled honestly.
- [x] Auto/always/never color modes respect NO_COLOR by default and provide a process-scoped explicit override; real-terminal tests verify ANSI output.
- [x] Narrow screens retain a substantial navigable graph and focused owner/purpose/next/capability details; public navigation/style tests preserve drafts and replay.

## Blocked by

WG-1.

## Implementation process

Use an isolated worktree from the integration branch, merge its current tip before handoff, and apply vertical TDD at the already approved public console and executable/external seams. Record RED/GREEN and checks in a ticket implementation note. Commit with Conventional Commits; merger integration and final independent review follow. Do not change another live project or merge into main.


## Implementation evidence

[RED/GREEN, integrated checks and visual QA](../research/wg-5-implementation.md)
record the completed behavior. The [synthetic terminal captures](../research/wg-5-terminal-captures/README.md)
include the live-sized 106×48 overview plus 150×42 and 60×28 views. [Root integration and combined review](../WORK-GRAPH-VALIDATION.md) are complete.
