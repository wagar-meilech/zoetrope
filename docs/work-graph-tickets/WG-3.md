# WG-3: Follow structured project artifacts locally

## Parent

[Approved work graph specification](../WORK-GRAPH-SPEC.md), [issue #15](https://github.com/wagar-meilech/zoetrope/issues/15). This ticket is [#18](https://github.com/wagar-meilech/zoetrope/issues/18).

## What to build

Deliver the approved vertical behavior described by this ticket, following the specification and answered questionnaire. Preserve Rust/local efficiency, source attribution, deliberate exact-recipient control and portable replay.

## Acceptance criteria

- [x] Idempotent enable/bind, status, disable and refresh commands keep project opt-in separate from exact per-goal source scope; restart and disabled state persist.
- [x] Explicit local Markdown task tables and individual ticket metadata import through bounded background IO; ordinary source edits update the public work view without state JSON or model/network calls.
- [x] Stable source identities, per-field authority, supplemental context and human/unassigned owners remain honest; source status never verifies outcomes.
- [x] Valid task removal differs from missing/malformed input; stale last-good data, duplicates, empty sources, errors, recovery, bounds and unchanged content are tested through actual executable/file seams.
- [x] Existing reports and native plans remain compatible fallbacks, including projects without Matt skills.

Implementation and validation: [WG-3 evidence](../research/wg-3-implementation.md).
The [public source contract](../research/wg-3-source-contract.md) defines the CLI,
supported Markdown subset and WG-4 handoff.

## Blocked by

None.

## Implementation process

Use an isolated worktree from the integration branch, merge its current tip before handoff, and apply vertical TDD at the already approved public console and executable/external seams. Record RED/GREEN and checks in a ticket implementation note. Commit with Conventional Commits; merger integration and final independent review follow. Do not change another live project or merge into main.
