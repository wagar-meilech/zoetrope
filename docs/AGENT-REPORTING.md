# Reporting tracked Zoetrope work

## Opt in for a project

Copy this guide into a chosen project as `docs/ZOETROPE-REPORTING.md`, then add this small pointer to that project's existing `AGENTS.md` or `CLAUDE.md`:

> Tracked Zoetrope work: when a tracking request supplies the provider, full session ID and state root, read `docs/ZOETROPE-REPORTING.md` once and report meaningful transitions.

The operator chooses the projects. Keep their existing instructions intact. With the convention installed, a tracking request can supply the exact identity/root and refer to this guide instead of repeating the full reporting procedure.

## Agent convention

1. Use the provider, full native session ID and exact state root from the tracking request. If any is missing, obtain it before writing. A project path or title does not establish a session identity.
2. Read compact context when tracking begins, and after a conflict or scope change. Reuse returned revisions between reports. If criteria are missing, propose concise editable `set_goal` outcomes for the agreed scope, including tests and final checks; keep them unverified until evidence exists.
3. Publish short, stable work items at meaningful transitions: plan or scope changes, work starting/completing, a blocker, an important finding, or verification. Batch related changes; routine commands and file operations need no separate report.
4. Describe the actual owner and useful purpose/change/next/blocker. Record dependencies and outcome links only when known. Keep reported work status separate from verified goal evidence.

## Compact read and partial reports

These are syntax examples. Replace `/exact/state/root` and every `SESSION_ID` with the supplied values before running them. The examples use a Codex coordinator; use the supplied provider and actual child ID when applicable.

```bash
zoe state --root '/exact/state/root' context codex 'SESSION_ID'
```

The response contains the current goal/revision, compact criteria/progress, work reports and recorded plans. It excludes feedback, questions and delivery history. If relevant work is omitted, continue from `next_work_offset`:

```bash
zoe state --root '/exact/state/root' context codex 'SESSION_ID' --offset 16 --limit 8
```

Use the actual returned offset. A clipped or omitted field is partial context, not proof that information does not exist.

Create a work item with expected revision 0:

```bash
zoe state --root '/exact/state/root' apply codex 'SESSION_ID' '{"op":"upsert_work_item","expected_revision":0,"item":{"id":"controls","label":"Explain available input","owner":{"session":{"provider":"codex","session_id":"SESSION_ID"},"agent_id":"main"},"status":"active","source":"coordinator","purpose":"Make steering availability clear before sending"}}'
```

The acknowledgment returns `work_item.revision`. Update only changed fields, using that revision; for example, if it returned 1:

```bash
zoe state --root '/exact/state/root' apply codex 'SESSION_ID' '{"op":"upsert_work_item","expected_revision":1,"item":{"id":"controls","status":"blocked","blocker":"Waiting for the runtime capability result","next":"Check the selected recipient"}}'
```

## Identity, revision and evidence rules

- Preserve work IDs across updates. `owner.session` stays the exact provider-qualified goal/session; `owner.agent_id` is `main` for its coordinator or the actual child identity.
- Status is `planned`, `active`, `blocked`, `done` or `cancelled`. `done` is an attributed report; verify an outcome separately with actual evidence through the existing goal interface.
- Omitted patch fields retain earlier values. Explicit `null` clears optional context, parent or dependencies; `depends_on: []` declares no dependencies. `outcome_links: []` clears links.
- Outcome links use actual `criterion_id` and `goal_revision` from context. Review them when scope changes; preserve earlier evidence rather than transferring credit by matching labels.
- Use each successful acknowledgment's revision. On a stale differing report, reread context and reconcile the intended change. Identical immediate retries are harmless; repeating an old different report is not a conflict-resolution strategy.
- `zoe state --help` is the field reference. State commands update local Zoetrope records; they do not steer or pause the native agent.

Passive viewing adds no model calls. The working agent's reports have input/output/tool-token cost; short transition reports and partial updates keep that overhead bounded in practice.
