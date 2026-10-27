# WG-6: Integrate, review and update the live installation

## Parent

[Approved work graph specification](../WORK-GRAPH-SPEC.md), [issue #15](https://github.com/wagar-meilech/zoetrope/issues/15). This ticket is [#21](https://github.com/wagar-meilech/zoetrope/issues/21).

## What to build

Deliver the approved vertical behavior described by this ticket, following the specification and answered questionnaire. Preserve Rust/local efficiency, source attribution, deliberate exact-recipient control and portable replay.

## Acceptance criteria

- [ ] Combined formatting, locked native/portable tests, strict Clippy, declared minimum Rust, browser/package/document checks pass.
- [ ] Independent Standards and Spec reviews are complete; confirmed findings are fixed and rechecked.
- [ ] Representative graph/source/idle resources and terminal rendering are measured through public interfaces and documented with practical limits.
- [ ] Usage/handoff, installed app and user tracking skill are updated; the existing live project is observed without automatic guidance or instruction edits.
- [ ] The integration PR closes this spec and all related tickets when merged; no merge into main is performed.

## Blocked by

WG-2, WG-4, WG-5.

## Implementation process

Use an isolated worktree from the integration branch, merge its current tip before handoff, and apply vertical TDD at the already approved public console and executable/external seams. Record RED/GREEN and checks in a ticket implementation note. Commit with Conventional Commits; merger integration and final independent review follow. Do not change another live project or merge into main.
