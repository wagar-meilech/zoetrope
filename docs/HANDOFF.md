# Zoetrope project handoff

Use the `feat/goal-console` branch in this repository. Its `origin` remote is
[wagar-meilech/zoetrope](https://github.com/wagar-meilech/zoetrope); the
implementation is tracked in [PR #8](https://github.com/wagar-meilech/zoetrope/pull/8).
The original implementation and installed-release verification completed on
25 October 2026. Documentation was consolidated here on 27 October.

## Read first

1. [CLAUDE.md](../CLAUDE.md): contributor instructions, constraints and commands.
2. [Goal-console specification](GOAL-CONSOLE-SPEC.md): agreed requirements.
3. [Answered questionnaire](CUSTOMIZATION-QUESTIONS.md) and
   [implementation approvals](IMPLEMENTATION-REVIEW.md): the user's decisions
   and approved public testing seams.
4. [Usage guide](GOAL-CONSOLE.md) and
   [validation results](GOAL-CONSOLE-VALIDATION.md): implemented controls,
   verification and compatibility limits.
5. [Research archive](research/README.md): supporting implementation notes,
   reviews, measurements and exact protocol schemas. The
   [build-status snapshot](research/zoetrope-build-status.json) records the
   completed implementation and installation details.

Use [ARCHITECTURE.md](ARCHITECTURE.md) and [DESIGN.md](DESIGN.md) when changing
the code. The [ticket graph](GOAL-CONSOLE-TICKETS.md) and
[issue-tracker instructions](agents/issue-tracker.md) describe the approved
GitHub workflow. Check GitHub for current PR state before taking further action;
merging into main requires a user request.

## Product direction

The app is a terminal-first graph across projects, goals and agents. Steering,
persistent questions/drafts, evidence-based progress and readable descriptions
are the priorities. The main app stays Rust; the VS Code preview bridge is
small and dependency-free. Passive monitoring invokes no model. Preserve the
bounded caches, background IO, portable replay core and ordinary Codex access.

The recorded validation includes 290 native tests, 176 portable tests, editor
host checks, an installed terminal flow and real MD Reader preview. The
validation document dates and qualifies the resource observations. These are
results of the completed implementation, rather than a claim that checks ran
again during this documentation move.

## Latest user discussion

The user found historical inspection difficult to discover: pressing Enter or
Space on a coordinator showed its overview/status rather than the detailed
past activity graph. Current behavior is:

- Enter/Space expand or collapse the fleet overview.
- Shift+R opens the selected goal's saved activity graph and timeline.
- In replay, `g` or End jumps to the latest recorded state; `q` returns.
- “Not loaded in Codex” describes live runtime availability. Local historical
  inspection can still work when its recording exists.

No subsequent navigation change has been implemented for that feedback. The
discussion did not request restoration of an actual Git checkout.

## Documents and local artifacts

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
