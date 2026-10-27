# Zoetrope project handoff

Continue on `feat/goal-console`. Its `origin` is
[wagar-meilech/zoetrope](https://github.com/wagar-meilech/zoetrope); the original
console is in [PR #8](https://github.com/wagar-meilech/zoetrope/pull/8).
The semantic-context continuation was authorized on 27 October 2026 through the
completed questionnaire and “Answered, proceed.” The user subsequently approved
publication, push and the PR update on the same date. The semantic spec/tickets
are published as issues #9–13; installation and updates are issue #14. Merging
into main still requires a user request.

## Current iteration

The user approved the [visual work graph and persistent tracking specification](WORK-GRAPH-SPEC.md)
and [six-ticket frontier](WORK-GRAPH-PLAN.md). Native sub-issues #16–21 belong to
spec #15. WG-1/2/3 implement graph navigation, exact owner resolution and local
source tracking independently; skill activation and visual polish follow their
prerequisites. Continue through TDD, merger integration, independent reviews and
installation without another design-approval request. The table in the plan is
the current coordinator-reported status source.

## Read first

1. [CLAUDE.md](../CLAUDE.md): contributor constraints and commands.
2. [Semantic-context specification](SEMANTIC-CONTEXT-SPEC.md),
   [answered questionnaire](SEMANTIC-CONTEXT-QUESTIONS.md), and
   [task graph](SEMANTIC-CONTEXT-TICKETS.md): accepted behavior and local status.
3. [Usage guide](GOAL-CONSOLE.md): current work/agent/command/replay navigation,
   steering availability, criteria and tracking.
4. [Agent reporting convention](AGENT-REPORTING.md): compact updates for an
   explicitly tracked goal and an opt-in selected-project pointer.
5. [Research index](research/README.md): current ticket handoffs, RED/GREEN
   evidence and dated verification. The original
   [goal-console spec](GOAL-CONSOLE-SPEC.md) and
   [implementation approvals](IMPLEMENTATION-REVIEW.md) record the existing
   public console and executable testing seams.

Use [ARCHITECTURE.md](ARCHITECTURE.md) and [DESIGN.md](DESIGN.md) for code changes.
[Issue-tracker instructions](agents/issue-tracker.md) retain GitHub as the
configured tracker. The [task graph](SEMANTIC-CONTEXT-TICKETS.md) links the
published issues; they close through PR #8 when it merges.

## Current behavior

A selected goal shows a bounded list of meaningful work and its verified-outcome
count. Explicit reports supply stable work IDs, actual owners, status, context,
source/time, declared dependencies and revision-qualified outcome links. Native
plans are an attributed fallback per producer. Reported done never verifies a
criterion; missing dependencies/context and stale links remain explicit.

- Press `w` or click **Work** to focus work; arrows or `[` / `]` choose entries.
  Without work focus, arrows navigate the fleet.
- `a` / **Agents** opens agent inspection. Enter/Space retains agent expansion.
- `x` / **Commands** reads the chosen owner's cached public message and command.
- `f` / **Steer** opens feedback for the selected recipient; an open editor keeps
  its recipient even if work ownership changes. Separate guidance/queue/stop
  availability is shown before sending.
- `C` / **Coordinator** explicitly selects the goal's coordinator. Child drafts
  stay with their original recipients.
- `T` / **Enable tracking** prepares an editable coordinator draft with exact
  local executable/state/session identity and transition-only reporting. Missing criteria
  trigger a request for `set_goal` outcomes including tests/final checks.
- F2/F3/F4 or Ctrl-S/Ctrl-Q/Ctrl-X deliberately send guidance / queue / stop.
  Saving a draft does not send it; runtime acceptance does not prove completion.
- `R` / **Replay** opens saved activity read-only; `q` returns to the same fleet.
  In replay, `g` or End jumps to the latest recorded state.
- **Not loaded in Codex** concerns live runtime availability. Local historical
  inspection remains available when its recording exists.

Labeled Work/Agents/Commands/Replay actions now expose the inspection routes
involved in the earlier discovery feedback. No helper model was added. The agreed next
step for any future helper is to assess the no-model flow first, then settle its
role/trigger/budget with explicit global or per-project opt-in.

## Runtime, verification and installation

The app remains Rust with bounded incremental metadata, selected-owner activity
loading, lazy detail and background IO. Passive monitoring calls no model.
Working-agent reports do consume tokens; partial updates, compact context and
short acknowledgments reduce that overhead. The reusable project convention is
operator-installed and conditional, rather than an automatic instruction edit.

The original installed-release verification and
[validation record](GOAL-CONSOLE-VALIDATION.md) are from 25 October 2026 on the
original Apple Silicon Mac. Its 290 native / 176 portable tests, installed
terminal/editor flows and resource observations are historical evidence, not
claims about the new semantic code. The
[build-status snapshot](research/zoetrope-build-status.json) also describes that
earlier installed release.

The semantic iteration is built and checked on Linux. Final [semantic validation](SEMANTIC-CONTEXT-VALIDATION.md) records 319 native and 176 portable tests, both strict Rust lint runs, browser/package checks and the qualified Linux resource observations. The [independent review](research/semantic-context-review.md) is clear on both axes. Per-ticket evidence remains in the research index.

The user selected this connected Linux environment for installation. `zoe` and
`zoe-update` are installed in `~/.local/bin`, already on PATH, from this permanent
checkout. Launch with `zoe`; after local changes, run `zoe-update` from any
directory to rebuild and atomically reinstall. It does not pull or change Git
branches. Keep the checkout in place; updates require Rust/Cargo, while normal
launch runs the installed binary directly. [Installation evidence](research/local-installation.md)
records five additional public process tests, both clear review axes, the real
installed PTY/compact-context check and an actual update from `/tmp`.
Existing native Codex compatibility limits still apply; local protocol peers
verify behavior without claiming universal live runtime compatibility.

## Documents and local artifacts

The following records the earlier Mac documentation migration; those local-only
artifacts and compatibility links are not guaranteed on this Linux checkout.

The project documents and supporting research now live in this repository.
Former research and temporary-artifact paths on this Mac redirect to the new
locations through compatibility symlinks, keeping saved document references
and historical commands usable. New work should use repository paths.

Executable rollback copies and compiled probe/check tools are under `.local/`
and Git-ignored. Raw runtime capture files and Python caches are also retained
locally and ignored; the findings, probe source and generated schemas are
versioned. See the research index for details. The file-move inventory and
original content hashes are in `.local/document-migration/manifest.json`.

Zoetrope's per-session runtime state remains in its configured state directory;
native Codex history and credentials remain in their own locations. Those are
runtime data rather than project documents.
