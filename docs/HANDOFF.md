# Zoetrope project handoff

Continue on `feat/goal-console`. Its `origin` is
[wagar-meilech/zoetrope](https://github.com/wagar-meilech/zoetrope); the original
console and subsequent iterations are in [PR #8](https://github.com/wagar-meilech/zoetrope/pull/8).
The semantic-context continuation was authorized on 27 October 2026 through the
completed questionnaire and “Answered, proceed.” The user subsequently approved
publication, push and the PR update on the same date. The semantic spec/tickets
are published as issues #9–13; installation and updates are issue #14. Merging
into main still requires a user request.

## Current iteration

The current [text-readability continuation](TEXT-READABILITY.md), issue #22,
replaces gold accents, formats structured detail text and preserves selected
public-message paragraphs through a bounded lazy cache. Its working-status table
records review fixes and final checks/installation. Earlier work below is complete.

The approved [visual work graph and persistent tracking specification](WORK-GRAPH-SPEC.md)
is complete. All six tickets in the [frontier](WORK-GRAPH-PLAN.md) are integrated,
checked, reviewed and installed. Native sub-issues #16–21 belong to spec #15;
issues close through PR #8 when merged, independently of local working status.
[Final validation](WORK-GRAPH-VALIDATION.md) records source identity, practical
limits and the read-only live demonstration. WG-6's earlier pending handoff notes are historical; the new readability request
continues on the same integration branch.

## Read first

1. [CLAUDE.md](../CLAUDE.md): contributor constraints and commands.
2. [Work graph specification](WORK-GRAPH-SPEC.md), [answered questions](WORK-GRAPH-QUESTIONS.md)
   and [frontier](WORK-GRAPH-PLAN.md): current accepted behavior and completion.
3. [Usage guide](GOAL-CONSOLE.md): Work/Agents/Commands/Replay navigation,
   steering availability, criteria, color controls and tracking setup.
4. For tracking setup/resume/disable, use the [tracking skill](../skills/zoetrope-tracking/SKILL.md).
   For supplemental work context or outcome evidence, follow its reporting pointer;
   ordinary workers maintain their assigned project artifacts.
5. [Research index](research/README.md): per-ticket RED/GREEN evidence, independent
   reviews and dated verification. The earlier [semantic spec](SEMANTIC-CONTEXT-SPEC.md)
   and [original spec](GOAL-CONSOLE-SPEC.md) record previous accepted behavior.
   [Implementation approvals](IMPLEMENTATION-REVIEW.md) define the public testing seams.

Use [ARCHITECTURE.md](ARCHITECTURE.md) and [DESIGN.md](DESIGN.md) for code changes.
[Issue-tracker instructions](agents/issue-tracker.md) retain GitHub as the
configured tracker. PR #8 closes the original, semantic, installation and current
work graph issues when merged. Do not merge into main without a user request.

## Current behavior

Fleet selects projects/goals; Work is a dedicated graph of stable task nodes and
only declared prerequisite edges. Isolated tasks, missing references, cycles and
bounded omissions remain explicit. Native plans/compact reports work immediately.
A successfully bound canonical source snapshot supplies current work instead of
a parallel old plan; invalid edits preserve stale last-good data. Source-managed
fields win, supplements fill gaps, and valid task removal expires old selection.

- `w`, Enter or **Work** opens the selected goal's graph. Arrows or `[` / `]`
  select tasks. `o` fits the overview; `r` restores readable selection. Back/Esc
  restores the preceding view/camera. Task selection/camera survive ordinary refresh.
- `a` / **Agents** opens agent inspection; Space expands agents in Fleet/Agents.
  `x` / **Commands** reads the selected owner's cached public messages/commands.
- Status colors and glyphs distinguish planned, active, blocked, reported done,
  cancelled and unknown work. Gold selection, legend and compact owner/purpose/
  next/blocker context remain usable in narrow terminals. Full tool output stays
  behind Commands. `--color auto|always|never` controls actual terminal colors.
- Reported-work counts are separate from evidence-backed weighted outcome progress.
  A task marked done does not verify a criterion or estimate remaining effort.
- `f` / **Steer** edits guidance for the exact selected owner. Logical owner paths
  resolve only through provider/root-scoped public native metadata. Missing or
  ambiguous matches remain unavailable. Open drafts keep their original recipient
  after ownership changes. Human/unassigned source owners are not steerable.
- `C` / **Coordinator** deliberately selects the goal coordinator. `T` / **Enable
  tracking** prepares an editable setup request naming the installed skill,
  executable and exact local identity. Review it, then deliberately send/queue.
  Saving a draft changes no project instructions and sends no native input.
- F2/F3/F4 or Ctrl-S/Ctrl-Q/Ctrl-X deliberately send guidance / queue / stop.
  Runtime acceptance does not prove work completed.
- `R` / **Replay** opens saved activity read-only; `q` returns to the same fleet.
  Today's source documents are never silently overlaid onto historical replay.
- **Not loaded in Codex** concerns live runtime availability; historical inspection
  remains available when its local recording exists.

One explicit project setup reuses its tracker convention and selected structured
local task artifacts. The skill adds only managed pointers/missing source fields;
it leaves upstream Matt skills intact. Later goals validate their own exact native
binding. Without suitable conventions, it can seed one minimal task document;
Matt's skills are optional. Existing native/report fallback remains available.

Normal document edits refresh on the background local-IO worker about every five
seconds, without repeated JSON reporting, coordinator prompts or monitor model
calls. Remote issues are not polled: local artifacts are an attributed working
projection. Project disable persists locally and overrides old committed pointers.

## Runtime, verification and installation

The app remains Rust with bounded incremental metadata, cached exact-owner joins,
lazy details and background IO. No helper model was added. The agreed next step
for any future helper is to assess the no-model flow, then settle role, trigger
and budget with explicit opt-in. Working agents still spend tokens maintaining
ordinary artifacts and reporting evidence; the monitor itself calls no model.

Current [work graph validation](WORK-GRAPH-VALIDATION.md) records 373 native tests,
189 portable tests, strict linting on Rust 1.99 and the declared 1.88 minimum,
wasm/rustdoc/package checks, actual terminal captures and qualified resources.
The separate [Standards and Spec reviews](research/work-graph-review.md) are clear
following independently rechecked fixes. Source-graph idle samples used about
8.3 MiB for 12 tasks, 18.3 MiB for sparse 512 tasks and 32.8 MiB for a dense
512-task fixture; they are observations rather than universal guarantees.

The user selected this connected Linux environment for installation. `zoe` and
`zoe-update` are installed in `~/.local/bin`, already on PATH. The tracking skill
is installed at `~/.codex/skills/zoetrope-tracking`. Launch with `zoe`; use
`zoe --color always` if the environment inherits `NO_COLOR`. After local changes,
run `zoe-update` from any directory to rebuild and atomically reinstall the app
and skill. It does not fetch or change branches. Keep this permanent checkout in
place; updates require Rust/Cargo, while normal launch runs the binary directly.
The actual updater was checked from `/tmp` and after the final installer polish.

The old attached tmux session `zoe-live` remains intact. The new `zoe-work-graph`
session runs the installed colored app and observes the active bot project.
Outside tmux: `tmux attach -t zoe-work-graph`. Already in tmux: Ctrl-b, `:`, then
`switch-client -t zoe-work-graph`. Its current work uses existing reports;
the new persistent source skill has not been activated in that separate project.
Only read-only viewer navigation was performed. An operator may deliberately
review/send its new `T` request to establish that project's convention.

Earlier [semantic validation](SEMANTIC-CONTEXT-VALIDATION.md) and
[initial Linux installation](research/local-installation.md) remain historical.
The [25 October Mac validation](GOAL-CONSOLE-VALIDATION.md) and
[build-status snapshot](research/zoetrope-build-status.json) concern the earlier
Apple Silicon release; they do not describe the current Linux binary. Existing
native compatibility limits remain; local peers and this live observation do not
prove every Codex runtime or terminal configuration.

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
