# Work graph and tracking skill — exploration, 27 October 2026

Read-only investigation for [the design questions](../WORK-GRAPH-QUESTIONS.md). Source baseline: integration `7c17a36`. Live observations are time-qualified; project work continues independently.

## Work canvas and visual flow

The native fleet canvas renders projects, goals and agents only. `w` changes selected work/detail without changing canvas topology. Existing work projection is bounded to 512 entries and already carries stable IDs, owners, status, context, optional parents, declared dependencies and revision-qualified outcome links. The observed goal had 11 items and usable declared dependencies; no inference is needed for that graph.

Existing rataflow rendering, hit-testing, pan/zoom and directed-layout support can be reused. Isolated nodes need explicit placement because the layout helper constructs its vertex set from edges. Missing references and cycles are valid input possibilities; the UI must label them and remain bounded. Parent grouping is distinct from a prerequisite edge. Source updates must retain selection and avoid repeated full layout.

Fleet currently uses fixed-size cards and a tall grid, with weak overview detail. The wide layout splits graph/details 60:40; narrow work mode gives the canvas only five rows. A dedicated work graph needs explicit navigation and a useful narrow layout. Preserve editor-pinned recipients, expired native-plan selection and portable replay.

## Color diagnosis

The running Zoetrope process and tmux server inherited `NO_COLOR=1`. The pane advertised `TERM=tmux-256color` and `COLORTERM=truecolor`, but its captured styled screen contained no ANSI color escapes. The installed crossterm implementation suppresses colors for a nonempty NO_COLOR value. Changing a parent environment will not repair an already running process’s environment/color decision.

The console also uses few colors: selected yellow accents and a verified-progress gauge, with most work/status content rendered as plain strings. Replay already has a shared theme, status glyphs/colors and semantic zoom patterns. Color restoration and broader semantic styling are separate implementation needs. Preserve a usable monochrome mode.

## Skill and existing project artifacts

Installed Matt Pocock `to-spec`, `to-tickets`, `implement-spec` and wayfinding workflows consult the project tracker instructions. They have no documented hook API. A conditional project pointer and tracker convention can connect a separate tracking skill without rewriting upstream skills.

Current Zoetrope reporting supports compact context and revision-checked partial updates, but has no source binding/import or identity-lookup command. A skill can remove repeated setup instructions; removing repeated model-side JSON reporting requires a deterministic helper/adapter. Arbitrary prose is not a reliable universal status parser. GitHub issue closure may wait for a final integration PR and therefore differ from locally implemented/integrated work.

Persistent project opt-in and per-goal native binding must remain separate. Keep machine/session-specific values in local state and preserve exact identity for multiple or resumed sessions. Source selection, synchronization and precedence still require product decisions.

## Owner resolution

Some live work reports identify child owners using collaboration paths instead of native thread IDs. Native control currently passes non-main IDs directly to the runtime. Public Codex session metadata already records both the child’s native ID and its collaboration path, scoped to its root session. This permits exact mapping through the provider boundary without private reasoning or KV access. Missing/ambiguous mappings must not guess or reroute to the coordinator.

## Exploration artifacts

Detailed read-only notes, including code pointers and bounded live examples, remain in `/tmp/zoetrope-work-graph-exploration.md` and `/tmp/zoetrope-tracking-skill-exploration.md`. No application changes, skill installation, project instruction edits, native input or external mutations were performed during exploration.


## Answered design and mechanism follow-up

Q1–Q5 approved the separate Work graph, persistent project opt-in, stronger
visual hierarchy and separate reported/verified counts. Existing artifacts are
preferred, with an explicit fallback when Matt's workflow is absent. The
local-first mechanism review is in
`/tmp/zoetrope-work-graph-mechanism-review.md`; the specification incorporates
source scoping, field precedence, removals versus malformed input, non-native
owners and source/replay timestamps.

This runtime exposes `CODEX_THREAD_ID` for the current child and
`CODEX_SESSION_ID` for its root. Bounded first-record metadata checks confirmed
the distinction. They can provide exact automatic binding at normal task entry
without another user/coordinator request. Treat missing or mismatched values
explicitly; this is a verified runtime capability, not a promise for every
provider/frontend. No full environment or private reasoning was read.
