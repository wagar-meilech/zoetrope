# WG-2: Steer the actual work owner

## Parent

[Approved work graph specification](../WORK-GRAPH-SPEC.md), [issue #15](https://github.com/wagar-meilech/zoetrope/issues/15). This ticket is [#17](https://github.com/wagar-meilech/zoetrope/issues/17).

## What to build

Deliver the approved vertical behavior described by this ticket, following the specification and answered questionnaire. Preserve Rust/local efficiency, source attribution, deliberate exact-recipient control and portable replay.

## Acceptance criteria

- [ ] Logical owner paths resolve through exact provider/root-scoped public native metadata rather than cwd/title/recency.
- [ ] Guidance, queue and stop target the unique native thread; unavailable and ambiguous identities are explained without coordinator rerouting.
- [ ] Open feedback drafts retain their native recipient across owner reassignment, metadata refresh and graph navigation.
- [ ] Public console and independent runtime-peer tests demonstrate exact targeting and preserve historical/non-steerable ownership.

## Blocked by

None.

## Implementation process

Use an isolated worktree from the integration branch, merge its current tip before handoff, and apply vertical TDD at the already approved public console and executable/external seams. Record RED/GREEN and checks in a ticket implementation note. Commit with Conventional Commits; merger integration and final independent review follow. Do not change another live project or merge into main.
