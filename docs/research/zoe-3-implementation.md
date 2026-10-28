# ZOE-3 local Codex control

Worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-3`, branch `feat/zoe-3-control`, started at integration `dfa63cf`. Feature commit `93a8a82`; integration `c84a0bf` merged as `4c4e42a`. Main control implementation is native Rust. The independent protocol peer is Python stdlib **test support only**, with no Python application dependency. Tests use the approved actual `zoe control` executable seam talking to that Unix-WebSocket peer, which independently checks exact wire requests and supplies realistic responses. No private controller tests.

Installed runtime evidence: [FINDINGS.md](zoe-control-compatibility-2026-10-23/FINDINGS.md). No additional model probes or real mutating actions were run in this ticket.

## Vertical TDD evidence

Every cargo command uses `CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-3`. The host's sandbox prohibits Unix socket binding, so socket-peer tests run with the approved escalation. Test sockets and data are disposable under `/tmp/zoe-ctrl-*` and are cleaned by RAII.

1. `cargo test --locked --test codex_control status_reads -- --nocapture` — RED unknown `--socket`; GREEN exact child read + one latest turn with `itemsView:notLoaded`, output Working/current-turn; no resume/start.
   - Initial fixture attempts exposed the sandbox's socket restriction and macOS Unix path limits, not product defects. Short temporary paths, explicit peer readiness and an atomic uniqueness suffix make parallel cases reliable.
2. `... guidance_is ...` — RED unknown action `guide`; GREEN exact `threadId` plus pinned `expectedTurnId`, accepted acknowledgment clearly distinct from completed work.
3. `... queueing_is ...` — RED unknown action `queue`; GREEN experimental queue add with stable `clientUserMessageId`, Queued acknowledgment and explicit idle-may-start-immediately explanation.
4. `... stopping_interrupts ...` — RED unknown action `stop`; GREEN exact thread/turn interrupt; no parent routing.
5. `... idle_guidance ...` — RED rejected `--idle`; GREEN explicit loaded-idle `turn/start` with only thread and input, no settings overrides.
6. `... join_replays ...` — RED unknown action `join`; GREEN `thread/resume` with only thread and `excludeTurns:true`, replayed native question and resolution preserved as events.
7. `... an_answer_requires ...` — RED unknown action `answer`; GREEN fresh explicit join/replay plus exact thread/turn/request and explicit answers. Output Submitted/awaiting resolution, never falsely accepted.
8. `... loaded_inventory ...` — RED required a thread ID for inventory; GREEN bounded paginated already-loaded IDs without resumes/model invocation.
9. Root review found `canAcceptDirectInput` was incorrectly applied to interruption. `... input_restrictions ...` — RED exact child stop refused because input restricted; GREEN restriction now applies only to Guide/Queue. Exact child interrupt reaches peer; real existing child was not touched.
10. `... an_incompatible ...` — RED wrong acknowledged turn still reported accepted; GREEN incompatible acknowledgments leave delivery Unknown, invalidate pending claims and are never retried.

Additional external behavior coverage passed: stale local turn selection sends no input; runtime stale-turn rejection is Rejected; lost acknowledgment is Unknown; input-disabled child is unavailable without parent fallback; resolved native questions reject a saved answer and submit no default.

## Public interface for terminal integration

Native module `control`, controller only on `cfg(unix)`:

- `default_socket()` uses `CODEX_HOME` or `~/.codex`, then `app-server-control/app-server-control.sock`. Never starts a daemon or network endpoint.
- `Controller::connect(&Path)` negotiates initialize/initialized with experimental API enabled.
- `loaded_threads() -> Result<Vec<String>>` lists only already-loaded IDs, bounded to 128/page and 16 pages with repeated-cursor detection.
- `snapshot(thread_id) -> Result<ThreadSnapshot>` reads real status and one latest turn without transcript items or resume. Snapshot fields: thread_id, name, cwd, status, active_turn, can_accept_direct_input.
- `cached_snapshots() -> &BTreeMap<String,ThreadSnapshot>` performs no IO.
- `poll_events(Duration) -> Vec<RuntimeEvent>` bounds waiting to 250 ms and processing to 64 wire messages. RPC waits also retain events, so question replay before an RPC response is preserved.
- `execute(&ControlTarget { agent: AgentKey, replay: bool }, ControlAction) -> DeliveryReceipt` freshly checks exact thread/turn. The root `main` maps only to its own session thread; child IDs map only to themselves. Replay/other-provider targets are unavailable before any action.
- Actions: `Guide { text, expected_turn: Option<String> }`, `Queue { text, client_message_id }`, `Stop { expected_turn }`, `Join`, `Answer { request_id:Value, turn_id, answers:BTreeMap<String,Vec<String>> }`.
- Receipts: `state` (Accepted, Queued, Submitted, Rejected, Unavailable, Unknown), `message`, `thread_id`, `turn_id`, `queue_id`.
- Events: `ThreadStatus {thread_id,status}`, `TurnStarted/TurnCompleted {thread_id,turn_id}`, `Question(NativeQuestion {request_id,thread_id,turn_id,prompts})`, `QuestionResolved {thread_id,request_id}`, `Disconnected {reason}`, `Notice {message}`.
- NativePrompt fields match workstate's prompt fields, except `options` is `Option<Vec<NativeOption>>`, mapping to an empty vector when null. Native option has label/description. Native prompt has id/header/question/is_other/is_secret.
- RuntimeStatus: Working, WaitingForAnswer, WaitingForApproval, Idle, NotLoaded, Error, Unknown. NotLoaded never means completed.

All connection/snapshot/control/poll methods are blocking local IO. Own the controller on a dedicated worker. Only cached access belongs on a drawing path. No implicit hidden instructions, model calls, summaries, settings overrides, approvals or transcript writes occur.

Pending native IDs are trusted **only after replay/new question on an explicitly joined current connection**. Resolve/turn completion/disconnection clears their live claims. Caller must expire restored pending records before connection confirmation. Never resend unknown deliveries automatically. Question submission has no RPC acknowledgment, so Submitted awaits observed resolution. `serverRequest/resolved` establishes no longer pending; it does not independently prove whose answer or cancellation reason. Keep UI wording honest.

T4 should apply live status to fleet visibility so quiet working/waiting work remains Current beyond the transcript-only 120s fallback. Use `loaded_threads` and snapshots/global events, not broad resumes of history. Explicit selected-thread Join enables questions; ignore native approval requests so ordinary Codex remains authoritative. Non-Unix native UI must cfg-gate Controller and explain live control unavailable while retaining replay.

## Resource and compatibility evidence

- Native-only tungstenite 0.28 handshake feature, defaults/TLS disabled; small protocol dependency graph, no service, background model, GUI framework or internet transport.
- Messages/frames cap 4 MiB; native questions cap 64 KiB, 64 pending questions; event queue 128; cached snapshots 4096. Event overload invalidates pending claims and surfaces a Notice. RPC deadline 5 seconds; write timeout 3 seconds; poll wait max 250 ms. Zero-wait poll drains bounded ready messages before answer validation.
- Real Rust read-only compatibility check: listed four loaded installed Codex threads; queried all four statuses/active-turn identities in 0.055 seconds total, including subprocess starts in debug mode. Three Working, one Idle; active-turn metadata present for Working. Root accepts direct input, three child/helper threads report false. Guide/Queue respect this; Stop remains independently attempted against the exact pinned target and may be rejected by runtime.
- No real existing thread was joined, steered, queued, interrupted or answered. The earlier disposable probe remains the only real mutating compatibility evidence. It did not verify second-client answering, simultaneous replies, real approval arbitration, or all queue drain/crash semantics. Do not claim those are verified.

## Validation

Before the style-only integration merge: all 244 tests passed (207 library, 9 binary, 13 external protocol cases, 10 fleet console, 5 launcher), all legacy replay tests retained. Formatting and portable-core check passed. Final post-merge clippy/core/rustdoc/test results are recorded below when complete. No release build, installation, push or additional models launched.

Final handoff tip: `4bf1ab1` (`4c4e42a` includes latest integration, then a tiny style-only test/documentation cleanup). `cargo clippy --locked --all-targets --all-features -- -D warnings` passes after the integration fixes. `cargo fmt --all --check`, portable core, and `RUSTDOCFLAGS=-D warnings cargo doc --locked --all-features --no-deps` pass. Post-merge full tests: all 244 pass. Tree clean. Main CLI integration consists of one early `control::run_cli` dispatch and a help line; merger must preserve ZOE2's separate state dispatch.
