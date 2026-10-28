# ZOE-2 implementation

Branch `feat/zoe-2-state`, worktree `/Users/melikhov/Repos/zoetrope-worktrees/zoe-2`, confirmed integration base `dfa63cf` and clean before changes. Rust 1.88, isolated `CARGO_TARGET_DIR=/Users/melikhov/Repos/zoetrope/target/zoe-2`.

Read TDD, tests/mocking references, codebase-design, CLAUDE and approved spec/task graph. Tests use only the approved public Console + executable seams. Each cycle records a meaningful RED before adding the corresponding implementation.

## Intended interface

Native `workstate` module, versioned per-session local records, atomic writes and OS advisory lock for concurrent processes. Console state-enabled constructor, update/read methods, cached search/render projection; all IO intended for the inventory/control worker. `zoe state [--root DIR] show|apply PROVIDER SESSION [JSON]` exposes the same record for the operator and Codex. No model calls, native transcript mutation or automatic input submission.

## TDD cycles

1. Executable goal tracking survives restart; missing tracking shows unknown. RED: existing executable rejects `--root` (exit101). GREEN: `cargo test --locked --test workstate local_goal_tracking_survives_process_restart_and_missing_tracking_is_unknown`, passed.
2. Evidence-backed weighted progress, independent worked example: 1 of2 verified with weights3+1 is75%, empty evidence rejected, explicit user attestation completes remaining criterion. RED: `verify` unknown WorkUpdate variant. GREEN: same executable-seam test suite passes2/2.
3. Scope revision preserves prior evidence, reuses unchanged criteria, resets changed-label criteria, rejects verification for stale revision. RED: revised scope dropped credit (0/3 vs expected1/3). GREEN:3/3 executable tests passed.
4. Exact-agent draft survives navigation/restart via public Console. RED: missing with_state/SaveDraft. GREEN:4/4 tests passed. Console refresh stat-gates local records; cached work/render has no IO.
5. Structured native question + partial answers survive expiry. RED: capture_question unknown. GREEN:5/5 passed. Expiry preserves draft; explicit answer_as_feedback creates a local draft and never claims original request answered.
6. Outbox queue/accepted/uncertain delivery are distinct and do not change progress. RED: create_feedback unknown. GREEN:6/6 passed. Sending interrupted across process boundaries becomes Unknown with no automatic replay.
7. Sixteen actual CLI processes concurrently saving different agent drafts. RED: only1 of16 drafts survived. Added tiny native-only fs2 advisory lock (OS releases on process exit) around read-modify-write; records atomically replaced within the same directory. GREEN pending below.
7. GREEN: all16 concurrent process drafts survive;7/7 tests passed.
8. Search includes local goal labels and exact Unicode/metacharacter artifact paths with producer; repeated document import is idempotent. RED: missing documents projection. GREEN:8/8 passed.
9. Bounded native transcript projection: recorded plan stays draft, only completed Markdown file changes become artifacts, question text captured as expired with no selected default, failed patch ignored, native transcript byte-identical. RED: missing plan_drafts projection. GREEN:9/9 passed.
10. Explicit reopen removes credit while retaining criterion evidence and a prior status snapshot; cannot bypass verification by setting status=verified. RED: unknown set_criterion_status. GREEN:10/10 passed.
11. Explicit criteria proposal saves only a target-specific draft; checkpoints default off; adopting completed native plan still starts unverified. RED: missing checkpoint default. GREEN:11/11 passed.
12. Invalid extra CLI arguments must not mutate state. RED: apply wrote the goal before reporting extra argument error. GREEN: parse/validate whole invocation before write;12/12 passed.
13. Runtime reconnect expires stored native requests; fresh pending evidence can rejoin without losing answer; resolved request cannot be reopened by replay. RED: unknown expire_pending_questions. GREEN:13/13 passed.

## Resource evidence

Compact WorkSummary (title/progress/attention) retained per session. Full records are bounded to32 entries/8MiB encoded bytes, plus a pinned selected record; a record itself cannot exceed8MiB. `work()` can return None after eviction; `load_work()` is the explicit blocking worker operation. Search reads uncached local records sequentially only on an explicit query and drops them immediately. All graph/render access remains cached. This replaces the initial unbounded full-record cache after the root's resource review.

A standalone Rust debug-build probe at the public Console seam loaded128 real-format session/state fixtures containing about64MiB prior scope evidence. macOS /usr/bin/time measured21,217,280 bytes peakRSS,17,629,472 bytes peakfootprint. Cold refresh452ms;20 unchanged refreshes averaged2.767ms; searching an uncached early artifact275ms;100,000 borrowed cached graph reads0.633ms. Full selected scope history restored after eviction. These are local synthetic fixture observations, not CI latency guarantees. Source: zoe-2-resource-probe.rs; exact results: zoe-2-resource-results.txt. No model calls or production session controls.
14. Small CLI edits must not echo large drafts/history into an agent's context. RED: applying a checkpoint update returned >1KiB including full draft. GREEN: apply now returns compact saved/session/revision/summary acknowledgment; explicit show still returns the full record. All14 public workstate tests passed. Help lists every update and explains saved-vs-sent/checkpoint behavior.

## Public interface for ZOE-4

All APIs below are native-only except existing fleet identities/commands. All IO must run on the inventory/control worker.

- `Console::with_state(DiscoveryRoots, PathBuf)` performs no construction IO.
- `Console::refresh(SystemTime)` updates inventory, stat-gated local records and bounded structured work import.
- `Console::work(&SessionKey) -> Option<&WorkRecord>` is cached only; may be evicted.
- `Console::work_summary(&SessionKey) -> Option<&WorkSummary>` always retains compact title/progress/attention after successful refresh.
- `Console::load_work(&SessionKey) -> anyhow::Result<&WorkRecord>` explicitly reloads an evicted selected record, blocking.
- `Console::update_work(&SessionKey, WorkUpdate) -> anyhow::Result<&WorkRecord>` atomic/concurrent local edit, updates cached summary/search and graph.
- `Console::work_errors() -> &[String]` surfaces corrupt/future-version/oversized records and import failures; don't silently hide these in UI.
- `workstate::Store::{new(PathBuf),default_root(),read(&SessionKey),apply(&SessionKey,WorkUpdate),apply_many(&SessionKey,IntoIterator<Item=WorkUpdate>)}` supports a dedicated worker without owning a Console.
- Default state home: `ZOETROPE_STATE_DIR`, else `XDG_STATE_HOME/zoetrope`, else `~/.local/state/zoetrope` (`USERPROFILE` fallback for Windows). Files and lock files are0600 on Unix; native histories never written.
- `zoe state [--root DIR] show PROVIDER SESSION` explicitly prints full JSON. `apply ... JSON` returns a compact acknowledgment. `zoe state --help` lists all fields/operations.

`WorkRecord` fields:

- `goal: Option<GoalContract>`: `title,revision,criteria:Vec<Criterion>,provenance`. Criterion has `id,label,weight,status,evidence`. CriterionStatus: Pending, InProgress, Verified. Evidence has `kind:Reference|UserAttestation,detail,provenance`.
- `scope_history:Vec<ScopeRevision>` keeps prior whole contract, reason and timestamp. `criterion_history` preserves criterion snapshots when reopened.
- `drafts:Vec<FeedbackDraft { target:AgentKey,text }>`; exact recipient persisted.
- `questions:Vec<QuestionRecord>` with `id,target,request_id:Option<Value>,turn_id:Option<String>,prompts:Vec<QuestionPrompt>,state,answers:BTreeMap<String,Vec<String>>`. Prompt fields `id,header,question,options:Vec<QuestionOption{label,description}>,is_other,is_secret`. Handle secret answers appropriately in the display.
- QuestionState: Pending, Expired, Answered, AnsweredElsewhere, Resolved, Dismissed. Use Resolved for native resolution whose actor/cause is unknown. Keep text/answers for expired/resolved questions. No default selection/submission is introduced.
- `outbox:Vec<FeedbackRecord>` with `id,target,text,mode,expected_turn:Option<String>,source_question:Option<String>,state,message:Option<String>,queue_id:Option<String>`.
- FeedbackMode: Guidance, Queue, Stop, Answer, CriteriaRequest. DeliveryState: Draft, Sending, Queued, Accepted, Submitted, Rejected, Unavailable, Unknown. Reading a record left Sending by an interrupted process projects Unknown and never resends it. A runtime acknowledgment never changes goal progress.
- `documents:Vec<DocumentRecord>` with `id,path:PathBuf,producer:AgentKey,source,change:Created|Changed|Deleted,changed_at:Option<String>`. Keep exact absolute selected path; no canonicalization or automatic open.
- `plan_drafts:Vec<PlanProposal>` with `id,producer,steps:Vec<PlanStep{step,status}>,source`. Native status "completed" earns no verified credit.
- `checkpoints:bool` defaults false. This is a per-goal preference, not an automatic Codex permission-policy change. The explicit RequestCriteria draft includes an instruction to ask at meaningful milestones when enabled; the user deliberately sends it. UI should make that distinction clear.
- `WorkRecord::progress()` returns verified count, total, optional weighted percent and UI-ready label. Missing/empty contract says Progress not tracked. `summary()` is compact; `matches(query)` searches goal/criterion labels and artifact paths.

WorkUpdate variants (also serde `op` snake_case):

- `SetGoal {title,criteria:Vec<CriterionInput{id,label,weight(default1)}>,reason,provenance}`; preserves unchanged ID+label credit/evidence, snapshots scope, resets changed-label current credit.
- `Verify {id,evidence:Vec<Evidence>,revision:Option<u64>}` requires nonempty evidence/attestation. Pass inspected revision from UI to reject a stale scope.
- `SetCriterionStatus {id,status,revision:Option<u64>,reason}` allows Pending/InProgress only and snapshots prior criterion; marking Verified must use Verify.
- `ImportPlan {proposal}` records a draft. `AdoptPlan {proposal_id,title,reason,provenance}` explicitly turns its steps into pending criteria.
- `RequestCriteria {target}` saves a readable criteria-request draft for the exact target. Does NOT send to runtime. `SetCheckpoints {enabled}` saves the preference.
- `SaveDraft {target,text}`.
- `CreateFeedback {id,target,text,mode,expected_turn:Option<String>,source_question:Option<String>}` starts Draft. `RecordDelivery {id,state,message:Option<String>,queue_id:Option<String>}` records explicit control results; target is immutable.
- `CaptureQuestion {question}` preserves existing partial answers. `SaveAnswer {question_id,answers}` never submits. `ResolveQuestion {question_id,state}` cannot invent Pending. `ExpirePendingQuestions {target:Option<AgentKey>}` invalidates stored native pending IDs on disconnect/new connection. Fresh current runtime CaptureQuestion can restore an Expired request to Pending; terminal Resolved/Dismissed/Answered state does not reopen from replay. Control must independently validate IDs; store persistence is not proof of current pending status.
- `AnswerAsFeedback {question_id,feedback_id,mode}` only for Expired; creates a Draft for the original exact recipient with answer text, leaves original question Expired. User must separately send/queue it.
- `AddDocument {document}` upserts exact path+producer reference.

Structured import currently supports Codex direct update_plan/request_user_input function-call records and explicit completed FileChange Markdown records. It consumes only the metadata inventory's existing bounded head/tail/append bytes, skips child replayed parent ordinals, captures no default answers and does not interpret code strings. Unsupported/unseen records remain unknown; Claude viewing remains unchanged. Historical questions are Expired because transcript data does not establish a live native request ID.

## Handoff commits and checks

Feature6761dab; integration fa10ab0 merged as3bac506 (resolved main.rs by preserving both state and control dispatch); compact CLI acknowledgment cefe2bb. Final verification recorded below.

Final head `cefe2bb`, clean tree, integration tip fa10ab0 is an ancestor. Final258 tests pass:207 library+9 binary+13 control+10 fleet+5 document launch+14 workstate. fmt passes. Clippy all targets/all features with -D warnings passes. Portable core without default features passes. Rustdoc all features with -D warnings passes. Full control suite requires escalated local Unix-socket permission in this sandbox; default sandbox run failed because disposable peer socket binds were denied, then the authorized local-peer run passed. No release build, install, native model probe, push or PR mutation performed by this ticket.
