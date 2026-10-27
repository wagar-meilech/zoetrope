# Reporting tracked Zoetrope work

## Project tracking

The source installer ships [zoetrope-tracking](../skills/zoetrope-tracking/SKILL.md)
into `${CODEX_HOME:-$HOME/.codex}/skills/zoetrope-tracking`. Enable tracking (`T`)
in the console prepares an editable exact-target request; the operator explicitly
sends or queues it. Installation and draft preparation do not opt in a project.

The skill owns authorized setup through the project's existing instruction/tracker
convention, including Matt-style ticket workflows. It selects the current goal's
existing task artifacts, adds only missing structured fields and a small managed
conditional pointer, and leaves upstream skills intact. Its
[setup reference](../skills/zoetrope-tracking/references/project-setup.md) is the
authority for setup, continuation and managed-pointer removal; its
[source reference](../skills/zoetrope-tracking/references/sources.md) describes
supported tables/tickets and the minimal non-Matt fallback.

Project opt-in persists separately from each exact native goal/source binding.
On later coordinator startup, check local status using the current validated
identity; an old committed pointer cannot override `project_opt_in: false`.
For Codex, `--session auto` validates `CODEX_SESSION_ID` and `CODEX_THREAD_ID`
against their public metadata/ancestry. Explicit UI-supplied roots also work.
Runtime paths/session IDs remain local rather than in project instructions.

```bash
zoe tracking status --project '/absolute/project' --provider codex --session 'EXACT_ROOT' --root '/exact/state/root'
zoe tracking disable --project '/absolute/project' --provider codex --session 'EXACT_ROOT' --root '/exact/state/root'
```

Use the supplied executable and actual values. Disable stops every binding for
that project while preserving history/evidence; the skill removes only managed
instruction additions. `zoe tracking --help` documents enable/source selectors
and optional one-shot sync. Normal console refresh imports ordinary artifact
edits locally on its worker every five seconds. Sources are attributed local
projections, not freshly fetched remote issue state; source status never verifies
outcomes.

Workers maintain assigned tasks in their usual artifacts at meaningful
transitions. They need not reload this guide or repeat setup/report JSON for
supported sources. Keep owner/dependencies honest. Source-managed fields require
source edits; optional reports enrich fields outside that source's schema.

## Compact reporting fallback

If the installed skill is unavailable or no suitable source is selected, retain
existing native plans and compact work reports. Use the CLI executable, provider,
full native root and state root supplied by the tracking request. Missing identity
must be resolved before writing; project names, cwd and recency do not establish
it. Read compact context at tracking start and after conflicts/scope changes.
Propose editable completion criteria when missing, including relevant tests and
final checks; keep them unverified until actual evidence exists.

Use short stable work items at meaningful plan/scope/status/blocker transitions.
Describe actual owner and useful purpose/change/next context; batch related
changes. The examples below apply to fallback or optional supplements, not every
routine source edit. For source tasks, obtain the stable `source:...` ID and field
provenance from context before supplementing; never merge by label.

## Compact read and partial reports

These are syntax examples. Replace `zoe` with the supplied CLI executable (preserving shell quoting), `/exact/state/root` and every `SESSION_ID` with the supplied values before running them. The examples use a Codex coordinator; use the supplied provider and actual child ID when applicable.

```bash
zoe state --root '/exact/state/root' context codex 'SESSION_ID'
```

The response contains the current goal/revision, compact criteria/progress, effective source tasks, work reports and recorded plans. It excludes feedback, questions and delivery history. If relevant work is omitted, continue from `next_work_offset`:

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
