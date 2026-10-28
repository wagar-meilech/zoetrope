# Research and implementation archive

These files were consolidated from this project's local research directory and
temporary probes on 27 October 2026. Start with [the project handoff](../HANDOFF.md)
and the current specification/usage documents. Many notes below record earlier
investigation stages, WIP checkpoints or superseded hypotheses; the completed
validation and final review resolutions take precedence.
Source links retain the original research's line numbers; use code search if
subsequent changes have moved a definition.

## Text readability continuation

[Requested behavior, working status and TDD evidence](../TEXT-READABILITY.md)
track completed issue #22 work (closure follows PR #8). [Terminal previews](text-readability-captures/README.md),
[resource observations](text-readability-resource-results.json) and the
[separate review reports](text-readability-review.md) preserve final evidence
and original findings/resolutions. The preceding work graph evidence below
remains dated to that earlier release.

## Visual work graph and persistent tracking

The [approved specification](../WORK-GRAPH-SPEC.md),
[answered questions](../WORK-GRAPH-QUESTIONS.md) and
[live task frontier](../WORK-GRAPH-PLAN.md) describe issues #15–21.
All six tickets are integrated. [Final validation](../WORK-GRAPH-VALIDATION.md)
records passing checks, clear independent reviews, qualified resource samples,
installed app/skill and a read-only live demonstration. Issue closure follows PR #8.

| Work | Evidence |
| --- | --- |
| Work graph and navigation | [WG-1](wg-1-implementation.md) |
| Exact native ownership | [WG-2](wg-2-implementation.md) |
| Local source imports | [WG-3](wg-3-implementation.md), [public source contract](wg-3-source-contract.md) |
| Tracking skill and installer | [WG-4](wg-4-implementation.md) |
| Visual flow and color modes | [WG-5](wg-5-implementation.md), [synthetic terminal captures](wg-5-terminal-captures/README.md) |
| Integration and independent review | [integration](work-graph-integration.md), [review](work-graph-review.md), [fixes](work-graph-review-fixes.md) |
| Final checks and resources | [validation](../WORK-GRAPH-VALIDATION.md), [resource observations](work-graph-resource-results.json), [reproduction](probes/work-graph-resource.py) |

The [tracking skill](../../skills/zoetrope-tracking/SKILL.md) is the authoritative
entrypoint for project setup/continuation. Source imports use configured local
artifacts; source status never verifies outcomes and the monitor does not poll
remote trackers or call a model. Older sections below remain historical.

## Semantic-context continuation — 27 October 2026

Current requirements: [accepted spec](../SEMANTIC-CONTEXT-SPEC.md),
[answered questionnaire](../SEMANTIC-CONTEXT-QUESTIONS.md), and
[local task graph](../SEMANTIC-CONTEXT-TICKETS.md). The
[reporting convention](../AGENT-REPORTING.md) is opt-in for selected projects.
The user explicitly approved publication, push and the PR update on 27 October
2026. The task graph links published issues #9–13; issue closure occurs through
PR #8 when merged. [Installation/update evidence](local-installation.md) covers
the additional user request tracked as issue #14.

| Work | Implementation and verification notes |
| --- | --- |
| Durable work and compact context | [SEM-1](sem-1-implementation.md) |
| Steering discovery and tracking setup | [SEM-2](sem-2-implementation.md) |
| Work-first inspection and owner routing | [SEM-3](sem-3-implementation.md) |
| Reporting guidance and documentation | [SEM-4](sem-4-implementation.md) |

Semantic checks run on Linux and are distinct from the 25 October installed Mac
release evidence below. Final [validation](../SEMANTIC-CONTEXT-VALIDATION.md), [two-axis review](semantic-context-review.md), [review/bootstrap fixes](semantic-context-review-fixes.md), and [resource observations](semantic-context-resource-results.json) record the completed local work.

## Completed implementation and review

- [Work graph review fixes](work-graph-review-fixes.md): source validation, typed
  status cues, compiler compatibility and public regression evidence.
- [Build-status snapshot](zoetrope-build-status.json): installed release,
  verification, resource samples, source baseline and compatibility pointers.
- [Final independent review](zoetrope-final-review-2026-10-25.md): original
  findings followed by their verified resolutions.
- [Review-fix implementation](zoe-review-fixes-implementation.md): RED/GREEN
  evidence and the final save, copy, refresh and placement behavior.
- [Merge notes](zoetrope-merge-notes.md) and
  [integration validation](zoetrope-integration-validation-2026-10-23.md).
- [Final resource samples](zoetrope-final-resource-samples-2026-10-25.json) and
  [history normalization proof](zoetrope-history-normalization-2026-10-25.json).
- [Earlier root review](zoetrope-root-review-2026-10-24.md) and
  [candidate resource samples](zoetrope-candidate-resource-samples.json) are
  historical and precede the final release.

## Ticket handoffs and TDD evidence

| Work | Notes |
| --- | --- |
| Fleet inventory | [ZOE-1](zoe-1-implementation.md) |
| Durable state and evidence | [ZOE-2](zoe-2-implementation.md) |
| Native Codex control | [ZOE-3](zoe-3-implementation.md) |
| Terminal interface | [ZOE-4 UI](zoe-4-implementation.md), [runtime](zoe-4-runtime-implementation.md), [earlier UI handoff](zoe-4-ui-handoff.md) |
| Markdown preview | [ZOE-5](zoe-5-implementation.md), [bridge setup](../../editors/vscode/README.md) |

[Issue bodies and issue map](zoetrope-issue-bodies-2026-10-23/) preserve the
published task graph. The [saved PR description](zoetrope-pr-body.md) and
[commit-message helper](zoetrope-normalize-commit-messages.py) are workflow
artifacts; consult the current Git state before applying historical commands.

## Interface and steering research

- [Initial interface research](codex-agent-interfaces-2026-10-23.md).
- [Third-party interface comparison](codex-agent-visual-interfaces-third-party-2026-10-23.md).
- [Steering alternatives](codex-steering-alternatives-2026-10-23.md).
- [Goal and steering synthesis](codex-goal-steering-synthesis-2026-10-23.md).
- [Zoetrope fork audit](zoetrope-forks-steering-2026-10-23.md).
- [Implementation facts](zoetrope-implementation-facts-2026-10-23.md),
  [control connection](zoetrope-control-connection-2026-10-23.md) and
  [UI implementation map](zoetrope-ui-implementation-map-2026-10-23.md).

These reports are dated research snapshots. Their early recommendations and
descriptions of upstream Zoetrope are not descriptions of the completed fork.

## Protocol exports and probes

[Compatibility findings](zoe-control-compatibility-2026-10-23/FINDINGS.md) record
what disposable probes established on Codex 0.164, including the limits of
second-client answering and approval coexistence. Its directory contains the
original probe scripts and experimental schema export. The earlier
[schema export](zoetrope-codex-schema-0.164.0/) is retained for comparison.

[Public-interface reproduction sources](probes/) were moved from temporary
storage. The [state resource probe](zoe-2-resource-probe.rs) and
[its output](zoe-2-resource-results.txt) are also preserved. These are archived
manual probes, distinct from the maintained tests under `tests/`. Some scripts
record original machine/worktree paths or live disposable-session operations;
read the associated findings before reusing them. No probes ran during the move.

## Local-only artifacts

The following are inside the repository on this Mac and deliberately ignored by
Git:

- `.local/research-artifacts/zoetrope-install-backups/`: original executable
  rollback copy.
- `.local/research-artifacts/tmp/`: compiled probes, the packaged preview bridge,
  old PID markers and raw temporary event output.
- `.local/check-tools/`: the downloaded lint executables and their archives.
- `.local/document-migration/manifest.json`: original-to-current paths and
  hashes for every moved file.
- Raw JSONL recordings, probe output text and disposable thread-ID markers in
  `zoe-control-compatibility-2026-10-23/`, plus generated `__pycache__` files.

The compatibility findings and scripts remain versioned without publishing raw
runtime/session captures. A clone contains the documentation, schemas and probe
source; these local artifacts require a separate copy if another machine needs
them. Old paths on this Mac are compatibility links, not additional data copies.
