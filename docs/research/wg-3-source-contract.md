# WG-3 public local-source contract

27 October 2026. Native-only additive tracking. WG-4 can invoke this interface;
no Matt skill or project-instruction edit is performed by the command itself.

```sh
zoe tracking enable --project /absolute/project --provider codex \
  --session EXACT_NATIVE_ROOT --source 'docs/WORK.md#Tasks' --root /local/state
zoe tracking status --project /absolute/project --provider codex --session EXACT_NATIVE_ROOT --root /local/state
zoe tracking sync --project /absolute/project --provider codex --session EXACT_NATIVE_ROOT --root /local/state
zoe tracking disable --project /absolute/project --provider codex --session EXACT_NATIVE_ROOT --root /local/state
```

Options may follow the operation in any order. `--source` repeats only for
enable. `--root` is optional and uses the same defaults as `zoe state`.
Enable is idempotent, replaces only this exact goal's source set, and opts in the
selected project. It does not bind future native roots. Disable opts the project
out and stops all its existing bindings; re-enable resumes only the explicitly
selected binding. Cached tasks, evidence, historical reports and disabled state
survive restart. Status and sync return bounded JSON containing `project`,
`project_opt_in`, `session`, `enabled` and per-source selector/identity,
fingerprint, update time, task/retired counts, stale flag and error. No native
message is sent, no instructions are rewritten, and no network/model/tool
content is run.

`--session auto` is Codex-only. Both `CODEX_SESSION_ID` (native root) and
`CODEX_THREAD_ID` (current thread) must match their own public session headers;
the current thread's ancestry must lead to that root. Missing, conflicting,
ambiguous or out-of-bounds metadata fails explicitly. No cwd/title/recency
inference occurs. Explicit UI-supplied native roots remain supported. Runtime
paths, native identities and caches belong only in local state. The setup skill
owns any authorized small portable project pointer/convention edit.

Normal Console/Inventory refresh performs bounded local source work on the
existing worker every five seconds; routine updates do not need sync. Sync is a
one-shot recovery/testing interface. Replay does not call this importer.

## Task-table subset

Select `relative/file.md#Exact heading text`. The heading must occur exactly once
outside fenced code; the first pipe table in its section is authoritative.
Require leading/trailing pipes, a normal dash separator and equal cell counts.
Escaped `\|` and `\\` are supported; Markdown links remain literal attributed
text, never another fetch source.

```markdown
## Tasks

| ID | Task | Status | Owner | Depends on | Next |
| --- | --- | --- | --- | --- | --- |
| T1 | Implement behavior | active | /root/implementation | None | Run checks |
| T2 | [Review](https://example.invalid/issues/2) | blocked | Alice | T1 | Await review |
```

Column names are case-insensitive: ID/Ticket, Task/Title and Status are required.
Owner, Depends on/Blocked by/Dependencies, Purpose, Change, Next and Blocker are
optional. Other columns are ignored. Duplicate recognized columns/IDs are invalid.
IDs use 1..64 ASCII letters, digits, `.`, `_` or `-`; task labels use 1..512 bytes.
Dependencies are comma-separated IDs within this table. `None` or `[]` explicitly
means no dependencies; blank means unknown. Missing targets, self edges and cycles
remain declared relationships for the graph to explain.

Status mappings: planned/pending/ready-for-agent → planned;
active/in_progress/in-progress → active; blocked → blocked;
done/completed/resolved → reported done; cancelled/canceled → cancelled.
Unknown or blank status remains `null` plus the raw `status_label` and a diagnostic.
No task status, source link or checkbox executes outcome verification.

Native owner references are `main`, `/root/path` and `native:EXACT_THREAD_ID`.
Their scoped reference still requires live exact-owner resolution before steering.
All other owner text, including people, Unknown and Unassigned, remains a display
label with `owner: null`; there is no coordinator fallback.

## Ticket-file subset

Select each explicit `relative/ticket.md@ticket`. The first 64 lines form the
maximum metadata area; stop at the first body heading, horizontal rule or prose.
Plain `Field: value` and `**Field:** value` are supported, with the same field
names as tables. A leading `# Title` supplies Task; ID and Status are required.
Fenced examples are ignored. Duplicate metadata fields are invalid.

```markdown
# Implement save behavior

ID: T1
Status: active
Owner: /root/implementation
Blocked by: None
Next: Run checks

## Acceptance criteria

Normal ticket prose stays here.
```

Within an explicitly bound ticket-file set, a dependency resolves across ticket
files only when its declared ID is unambiguous in that exact binding. Unknown or
ambiguous IDs remain unresolved. Similar task labels never establish identity.

## Effective plan, authority and failure

`state show` retains old `work_items` and native plans as historical/fallback data.
`state context` exposes the effective source plan as `source_tasks`, with
`source_snapshot: true` after any successful bound snapshot, even an empty one.
The Work graph must use that source plan instead of a second parallel report/plan.
Initially invalid sources leave `source_snapshot: false` and the prior explicit
reports/native plans remain the fallback. Later failures retain the canonical
last-good tasks, marked stale; they do not revive the old parallel plan.

Task IDs are stable `source:SOURCE_ID:DECLARED_ID`, scoped by the WorkRecord's exact
provider/root. Read these IDs from `state context` to supply an optional
`upsert_work_item` supplement; never match or merge by label. Source-owned fields
win. Supplemental fields absent from the source schema may enrich context, owner,
dependencies and explicit goal/revision outcome links. A recognized field remains
source-owned after its column/metadata line is deleted, so clearing a field never
revives an old supplement. Provenance maps each field to the source fingerprint
or exact supplemental report/revision. Invalid edits retain the previous field
ownership as well as previous values. Valid removal/empty snapshots retire current
tasks without inventing completion/cancellation; bounded retired observations stay
in the local tracking cache.

Bounds: 64 opted-in projects, 128 exact goal bindings, 64 total explicitly
allowlisted files, 16 files per binding, 256 KiB per source, 16 table columns,
1024 bytes per cell/context field, 512 current tasks per goal across sources,
64 dependencies per row, 8192 edges per source, 512 retired observations per
source, 8 MiB serialized tracking cache. Registry snapshots share a bounded
8 MiB/eight-entry process cache; source stat caches hold at most 64 entries.
Only resolved files within the selected canonical root are read; symlink escapes
and directory crawling are rejected. Unchanged files are statted on the worker;
identical content is a persistent no-op. The existing compact context remains
at most 64 KiB, paginated with `next_work_offset` and omission counts.
