# Semantic context task graph

Integration: `feat/goal-console`, PR #8. Implements [the spec](SEMANTIC-CONTEXT-SPEC.md) at the approved public console and executable seams. Each ticket uses its own worktree, vertical TDD and red/green evidence.

| Ticket | Blocked by | Complete behavior |
| --- | --- | --- |
| SEM-1: Durable work reports and compact context | None | Agents publish revision-checked work through the executable, restart, inspect compact context and preserve evidence-based progress. |
| SEM-2: Discoverable steering and tracking | None | Operators see recipient-specific capability/actions before sending and prepare coordinator tracking instructions. |
| SEM-3: Work-first goal inspection | SEM-1, SEM-2 | Goal inspection exposes native/explicit work, owners, context, dependencies, source, scope links, progress and owner-directed steering. |
| SEM-4: Reporting convention and final verification | SEM-3 | Reusable project reporting guidance, documented navigation, final checks and resource evidence match combined behavior. |

SEM-1 and SEM-2 proceed independently; SEM-3 consumes their integrated interfaces; SEM-4 completes the feature.

## SEM-1 — Durable work reports and compact context

- Bounded IDs, labels, real owners, statuses and optional purpose/change/next/blocker, parent, dependency and outcome-revision links.
- Restart persistence, stale-edit rejection, identity validation and harmless identical repeats.
- Work completion stays separate from verified evidence.
- Compact context and acknowledgments exclude feedback/question/outbox history.
- Test new behavior through the executable; retain old records/operations.

## SEM-2 — Discoverable steering and tracking

- Recipient and separate guidance/queue/stop availability before sending, including disconnected/unloaded/unsupported states.
- Labeled steering/navigation actions and keyboard/mouse routes; preserve literal text, drafts and deliberate sends.
- Editable coordinator Enable tracking instruction with exact state location and transition-only commands.
- Protocol checks preserve targeting, stale-turn rejection, no rerouting and delivery states.

## SEM-3 — Work-first goal inspection

- Work and verified progress become default selected-goal content.
- Imported plan statuses/source time and explicit work taking precedence per producer.
- Stable explicit selection and actual owner-directed steering.
- Declared/unknown dependencies, unresolved references, context and stale criterion links.
- Labeled Agents, Commands, Replay; preserve legacy navigation, narrow terminals and replay isolation.
- Bounded summaries/background IO and work/plan search where feasible.

## SEM-4 — Reporting convention and verification

- Concise opt-in project guidance and conditional snippet with exact implemented CLI examples.
- Usage/handoff/architecture/validation reflect final behavior.
- Combined native/portable/browser checks, terminal flows and public resource observations.
- Independent standards/spec review and fixes, PR #8 closes new issues; no merge into main.

## Publication status

Automatic approval review initially blocked publication. The user explicitly approved publication to `wagar-meilech/zoetrope`, push and the PR update on 27 October 2026. The [spec is issue #9](https://github.com/wagar-meilech/zoetrope/issues/9).

| Local ticket | GitHub issue | Blocked by |
| --- | --- | --- |
| [SEM-1](semantic-tickets/SEM-1.md) | [#10](https://github.com/wagar-meilech/zoetrope/issues/10) | None |
| [SEM-2](semantic-tickets/SEM-2.md) | [#11](https://github.com/wagar-meilech/zoetrope/issues/11) | None |
| [SEM-3](semantic-tickets/SEM-3.md) | [#12](https://github.com/wagar-meilech/zoetrope/issues/12) | #10, #11 |
| [SEM-4](semantic-tickets/SEM-4.md) | [#13](https://github.com/wagar-meilech/zoetrope/issues/13) | #12 |

The additional installation/update request is [issue #14](https://github.com/wagar-meilech/zoetrope/issues/14), following [its specification](INSTALLATION-SPEC.md). All issues close through PR #8 when merged.

## Local integration evidence

| Ticket | Local status | Evidence |
| --- | --- | --- |
| SEM-1 | Integrated in `1e5fba5` | [Executable RED/GREEN and CLI contract](research/sem-1-implementation.md) |
| SEM-2 | Integrated in `fc06c2b` | [Console/control/terminal RED/GREEN](research/sem-2-implementation.md) |
| SEM-3 | Integrated in `e6a6d52`, detail correction `ec35ad3` | [Work/owner/navigation RED/GREEN](research/sem-3-implementation.md) |
| SEM-4 | Implemented and verified locally; source review clear, final checks passed | [Documentation/example verification](research/sem-4-implementation.md) |

These statuses describe verified implementation. Issues remain open until
PR #8 merges. The user has not requested a merge into main.

Final evidence: [validation](SEMANTIC-CONTEXT-VALIDATION.md), [two-axis review](research/semantic-context-review.md), and [review/bootstrap regressions](research/semantic-context-review-fixes.md). The semantic implementation and separately requested install/update helper are complete. [Installation evidence](research/local-installation.md) records five additional public tests, clear reviews and the real connected-Linux install/update checks.
