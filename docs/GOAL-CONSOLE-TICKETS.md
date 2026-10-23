# Goal console: implementation task graph

Status: approved for implementation on 23 October 2026. These records are published as project issues. The spec is [GOAL-CONSOLE-SPEC.md](GOAL-CONSOLE-SPEC.md).

All implementation is integrated on one feature branch. Each ticket uses its own implementer worktree, starts at the integration tip, follows one-test/one-behavior TDD cycles at the agreed public seams, and merges the latest integration tip before handoff. A merger agent integrates each completed ticket. A final review covers the combined change.

| Ticket | Deliverable | Blocked by |
| --- | --- | --- |
| [ZOE-1](https://github.com/wagar-meilech/zoetrope/issues/2) | Fleet inventory and console interface | None; testing seams approved |
| [ZOE-2](https://github.com/wagar-meilech/zoetrope/issues/3) | Durable goals, questions, drafts and document records | ZOE-1 |
| [ZOE-3](https://github.com/wagar-meilech/zoetrope/issues/4) | Local Codex control and coexistence | ZOE-1 |
| [ZOE-4](https://github.com/wagar-meilech/zoetrope/issues/5) | Terminal fleet graph and interactive controls | ZOE-2, ZOE-3 |
| [ZOE-5](https://github.com/wagar-meilech/zoetrope/issues/6) | VS Code MD Reader preview integration | None; explicit document input is independent of storage |
| [ZOE-6](https://github.com/wagar-meilech/zoetrope/issues/7) | End-to-end verification, docs and installed release | ZOE-4, ZOE-5 |

## ZOE-1 — Fleet inventory and console interface

Create the public console interface and stable identities above the existing single-session viewer. Reuse discovery, include archived history where available, classify internal helpers correctly and index metadata incrementally. Establish selection, expansion, current/history filtering and search through externally observable behavior. Preserve legacy session parsing and replay.

Acceptance: fixture sessions from different projects and providers are discoverable without selecting a directory; archived work appears in history; children remain attached to the correct root; equal agent identifiers in different sessions cannot collide; malformed files do not break discovery; reindexing does not move selection unexpectedly.

## ZOE-2 — Durable goals, questions, drafts and document records

Add local goal contracts with evidence and scope revisions; draft/queued feedback; unanswered questions and partial answers; generated document references. Provide public local commands for reading/updating these records and parsing supported transcript updates. Preserve unknown/expired states honestly and never mutate native history.

Acceptance: evidence-backed progress and its count agree with independent examples; missing tracking remains unknown; scope changes preserve prior evidence; process restart restores unfinished answers and feedback; expired questions are retained without false delivery; documents retain exact paths and producer identity.

## ZOE-3 — Local Codex control and coexistence

Implement compatible shared-runtime attachment, live metadata and exact target/turn validation. Separate guidance, native queueing, interruption and answers. Preserve native Codex access and approval policy. Verify coexistence using disposable sessions; degrade unsupported actions explicitly without duplicate runs or hidden parent routing.

Acceptance: external protocol-peer tests prove exact recipient and active-turn checks, failure/disconnection behavior, delivery-state distinctions, and replay isolation. A real local compatibility check establishes which actions can coexist with native Codex. No passive monitoring path invokes a model.

## ZOE-4 — Terminal fleet graph and interactive controls

Wire the console into default launch. Add the global graph, one-key history switch, search, lazy expansion and pinned selection. Surface goal progress, concise activities, persistent feedback and questions, document selection and clear controls. Keep original single-session detail and playback available.

Acceptance: public input/render tests demonstrate a user finding a prior goal, selecting its agent, drafting feedback, navigating away and back without losing it, giving guidance to the intended live target, and reading a goal's criteria/evidence. Narrow terminals show usable content and key help; normal typing never triggers graph shortcuts accidentally.

## ZOE-5 — VS Code MD Reader preview integration

Provide a small local URI bridge to open selected Markdown documents in the user's installed reader. Keep the opener a native argument-based invocation, validate selected files, distinguish dispatch from render acknowledgment, and provide a plain-editor fallback. Do not publish to a marketplace or change global renderer preferences.

Acceptance: the public launch interface carries the exact selected file; spaces, Unicode and shell-like characters remain data; unavailable files report a clear error; the bridge opens the document and invokes MD Reader through its public command. Verify one real preview on this Mac after installing the local bridge.

## ZOE-6 — End-to-end verification, docs and installed release

Exercise the combined workflows, document commands and limitations, update the old read-only contract for explicit local controls, and run repository checks. Install the rebuilt fork at the existing `zoe` command, validate a real terminal flow and MD Reader preview, resolve final code-review findings, then prepare the integration PR for review.

Acceptance: agreed behavior passes at both public seams; existing replay tests remain green; native release and portable core build; relevant browser checks pass; no new warnings; user-facing docs explain current/history, search, progress, steering, persistent questions, documents and compatibility limits; final report distinguishes verified behavior from unsupported cases.
