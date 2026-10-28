# Zoetrope implementation facts for grilling and spec

Read-only code/CLI exploration on 23 October 2026. No product code or tests were written, and no agent was started, steered, stopped, or resumed. A schema bundle was generated outside the repository. Earlier intent/research: [goal/steering synthesis](codex-goal-steering-synthesis-2026-10-23.md).

## Repository and constraints

- Checkout: `/Users/melikhov/Repos/zoetrope`, clean at exploration start; origin is `https://github.com/wagar-meilech/zoetrope.git`.
- No applicable `AGENTS.md`, `GLOSSARY.md`, or ADR files were found in the checkout. The parent agent already checked ancestor instructions.
- Existing domain vocabulary is Session, Agent, Fact/Statement, Timeline, Transport, Camera, Provider and Scope. A session is a main/root thread with related child-agent files. Overview is a camera mode, not a fleet view.
- The architecture explicitly promises local read-only transcript reconstruction. Local control IPC is an intentional extension of that contract; keep control separate from replay. A subprocess proxy using stdin/stdout can preserve no-internet operation without adding an HTTP client.
- `.github/workflows/ci.yml` defines formatting, clippy, locked tests, portable-core/wasm checks, docs and packaging. There is no checked-in issue/triage configuration beyond the GitHub origin. No external tracker changes were made.

Sources: [architecture](../ARCHITECTURE.md), [design](../DESIGN.md), [CI](../../.github/workflows/ci.yml).

## Existing discovery and display seams

`provider::sweep(&Scope::ALL, only)` is already a public all-provider/all-session discovery operation. `Scope` supports project, modified-since and ID-prefix filtering. `assemble` groups known files by provider and session, excludes orphan groups without a root, and sorts sessions by last modification. `open` selects one session by file, ID or newest session in a project. This is substantial reusable fleet groundwork, but not a fleet UI.

The present app owns exactly one `SessionModel`, timeline, session info store and graph. `App::handle_ui_event` rejects stale-session batches. `handler::handle_event` is a public input boundary, and `ui::draw` is the public full-screen rendering boundary. The native TUI loop owns state and routes tailer messages and terminal events; controls currently mutate local presentation only. Its tail request sender is held open but not used by input. Fleet selection will need explicit session switching or multiple cached session states, rather than allowing the tailer's newest-session auto-switch to silently override user choice.

Sources: [discovery public API](../../src/provider/mod.rs#L447), [app state](../../src/state/mod.rs#L179), [input](../../src/handler.rs#L20), [renderer](../../src/ui/mod.rs#L34), [native event loop](../../src/tui.rs).

## Goal progress and labels

The existing Codex provider does not model native goals, plan updates or completion criteria. `SessionInfo` stores title, generic labeled fields and tallies only. Existing facts encode agents/tools/activity and messages; no goal-progress domain exists. New goal progress must therefore have a defined source and scope, not be inferred from the playback slider or count of completed agents.

The current Codex adapter puts assistant message text into `FactKind::Reasoning`, even though much of that text is user-facing commentary. It can supply human explanations already present in the transcript, but this field is not proof of hidden model reasoning or of one-to-one command intent. Tool-name/action mappings can make labels readable locally; missing intent should stay unavailable rather than being invented.

Sources: [session metadata](../../src/state/info.rs), [Codex adapter](../../src/provider/codex/mod.rs), [fact vocabulary](../../src/fact.rs).

## Current local Codex facts

Installed CLI and running shared app-server daemon both report **0.164.0**. The daemon reports a local control socket at `~/.codex/app-server-control/app-server-control.sock`. CLI help exposes `codex agents`, `codex app-server proxy [--sock PATH]`, and `codex --remote unix://PATH`; `--no-daemon` remains available, so some sessions can exist outside that shared runtime.

The exact installed schema is saved in [zoetrope-codex-schema-0.164.0](zoetrope-codex-schema-0.164.0). Relevant verified contracts:

- `thread/list` returns paginated metadata including `id`, `name`, `cwd`, `status`, `sessionId`, `parentThreadId`, `source`, `path`, `preview` and history mode. Default source filtering includes interactive sources only; child sources must be requested deliberately. `useStateDbOnly` avoids rollout repair scans.
- `thread/loaded/list` lists threads currently loaded by the connected runtime. It is not equivalent to every historical local session.
- Runtime thread status distinguishes `notLoaded`, `idle`, `systemError`, and `active`, with flags for waiting on approval or user input. `notLoaded` should not be presented as completed.
- `thread/read` supports metadata-only reading. Full history is deprecated for paginated threads; use item/turn pagination if needed. A file-only viewer cannot assume all current history will always be complete in one rollout.
- `turn/steer` requires `threadId`, `expectedTurnId`, and input. Its response returns accepted `turnId`; stale or wrong active-turn identity must fail without routing to a different turn.
- `turn/interrupt` requires thread and turn identity. It is distinct from queueing and steering.
- `thread/goal/get` returns objective/status/token/time accounting; **there is no native percentage-complete field**. `turn/plan/updated` provides human steps with pending/in-progress/completed status and thread/turn identity.

The daemon version request succeeded after sandbox escalation for socket access. A minimal read-only proxy `initialize` probe timed out; no thread-list or control request was reached. Thus runtime client attachment is still unverified. The schema and running daemon establish a promising interface, not proof that any arbitrary existing Desktop-owned session is steerable. An implementation must verify attachment and ownership/capability, and show external sessions as observation-only when it cannot prove control.

## Discovery edge case worth resolving

Sampled only the first-line metadata shapes of the latest 100 local rollout files, without printing conversation text. Many are internal `guardian_review` agents. The current `SessionMeta::is_root` treats every `thread_source` except exactly `subagent` as a root, even when the source is a spawn object. Consequently guardian review helpers can be mistaken for independent sessions in an all-session sweep. Decide whether to show such internal helpers by default, hide them, or place them under their parent. Ordinary user work must not be drowned in automatic approval-review threads.

Source: [root classification](../../src/provider/codex/wire.rs#L84). Current metadata includes `session_id` and `parent_thread_id`, so preserving parentage is feasible.

## Proposed minimum TDD seams for user agreement

Prefer **one high application seam** for most behavior, with a small executable smoke seam:

1. **Fleet console as used by a human:** feed session/goal updates and keyboard/mouse input through the public app interface, then observe rendered terminal text, selection, and externally visible control results. Use ratatui's test terminal/backend; exercise real parsing against synthetic on-disk fixtures and a local protocol peer. Catches cross-project discovery, nested children, filters, readable labels, progress rules, replay/live separation, and feedback target/result. Misses actual font/mouse quirks of a particular terminal and vendor-runtime compatibility.
2. **Installed executable/CLI:** launch the binary against an isolated temporary Codex home and verify startup mode, arguments, help, headless output, and clean quit. Catches wiring/PATH/argument/session-resolution regressions. Misses broad visual quality and real model behavior.

The **live-control peer** is the external system boundary of seam 1, not a mock of internal collaborators. It should record observable protocol requests and simulate successful acceptance, stale-turn rejection, disconnected/unowned sessions, and interruption acknowledgments. Tests must demonstrate no feedback is sent while examining a historical replay, and no selected child silently routes to its parent. A separate read-only real-daemon compatibility check can establish connection/version/status without model calls; only explicitly selected disposable sessions should be used for later mutating integration checks.

Existing prior art: input tests call `handler::handle_event`; state tests use `App::handle_ui_event`; render tests use a ratatui buffer; provider conformance tests parse public fixture sessions and verify independently reviewed model/timeline goldens. New tests should assert user behavior at those boundaries, not private fields or helper call counts. No tests were written during this exploration, pending the skill-required seam confirmation.

## Decisions the interview still needs

- Must existing Desktop/CLI sessions be controllable immediately, or is honest observation plus control for supported/shared sessions acceptable?
- Should system/approval-review helpers appear by default?
- What proves goal completion: native plan steps only, explicit acceptance checklist maintained by the working agent, user confirmation, or some combination? What is displayed when no checklist exists?
- Is direct child steering required, or may a clearly labeled request to the coordinator fulfill some cases?
- Is the first release Codex-only control while retaining read-only Claude viewing?
- Should feedback deliver mid-turn, queue for next turn, or ask per message? Which interrupt behavior should be exposed separately?

These are product decisions; filesystem/API facts above should not be asked of the user.
