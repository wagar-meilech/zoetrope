# Codex 0.164.0 two-client control compatibility — 23 October 2026

These are real shared-daemon protocol results from two honest probe clients, using only two new ephemeral threads and one new durable thread titled **Zoetrope compatibility test - DISPOSABLE 2026-10-23**. No existing user/agent thread was controlled. No shell commands, file tools, network tools, access grants, daemon/settings changes, or real VS Code GUI actions were performed. The durable disposable thread was archived after its turns completed; the ephemeral turns completed or were interrupted before their connections closed. Further model probes have stopped.

## Results suitable for implementation

| Action | Verified installed behavior |
|---|---|
| Active guidance from an unsubscribed second client | `turn/steer` with exact `threadId`, `expectedTurnId`, `input` succeeded. Owner continued and its final answer was `STEER_ACCEPTED`. No `thread/resume` was sent beforehand. |
| Stale-turn protection | A random wrong `expectedTurnId` was rejected with `-32600`, naming expected and actual active IDs. Never retry against a newer turn automatically. |
| Stop from an unsubscribed second client | `turn/interrupt` with exact disposable `threadId` and `turnId` returned `{}`; owner received `turn/completed` with `status: interrupted`. |
| Idle feedback from an unsubscribed second client | `turn/start` with only the loaded thread ID and input (plus an explicit disposable Plan-mode setting to expose a harmless native question) started a new turn without `thread/resume`. Original owner received its question. This proves loaded-idle direct start, not loading arbitrary historical threads. |
| Observation without joining | The unsubscribed second client received global `thread/started` and `thread/status/changed`, but no `turn/*`, `item/*`, dynamic-tool request or native question events. `thread/read` did not add those subscriptions. Sending `turn/start` also did not subscribe that client. |
| Durable rejoin | `thread/resume({threadId, excludeTurns: true})` rejoined the already-running disposable durable thread. No model/approval/sandbox/instruction overrides were sent. |
| Pending-question replay | Before observer rejoin only owner held `item/tool/requestUserInput` ID **7**. Rejoin replayed that same request with ID **7** to observer. Owner answered; both clients received `serverRequest/resolved(7)`, and the turn completed `QUESTION_OK`. |
| New question after rejoin | A later question was broadcast to both clients with the same ID **8**. Owner answered; both got `serverRequest/resolved(8)`, and the turn completed `QUEUE_OK`. Observer ignoring its copy did not block owner completion. |
| Policy preserved on join | Before/after join: `approvalPolicy: on-request`, `approvalsReviewer: user`, `sandbox: {type: readOnly, networkAccess: false}`, same model. The new disposable thread deliberately used read-only sandbox. |
| Native queue CRUD | An unsubscribed second client could `thread/queue/add`, `list`, `update`, `reorder`, and `delete` while an owner question blocked an active turn. Reordering returned `{}` and subsequent list reflected the exact requested order; deletion returned `{deleted: true}`. |
| **Queue add is active delivery** | On an **idle durable thread**, `thread/queue/add` immediately dequeued the submitted message and automatically started a turn. A later idle add also completed `QUEUE_OK` and left an empty queue. It is not a passive draft or a “wait for operator to run” command. A quick update can race consumption and return `queued submission not found`. |
| Queue restrictions | Ephemeral `thread/queue/add` failed `-32600: ephemeral thread does not support queued submissions`. Queue reads on our archived durable thread failed with an explicit unarchive requirement. Pending queue data survived archive/unarchive; only our own leftover probe item was then deleted. |
| Queue start | Exact method exists in experimental schema; empty queue call returned `-32600: queue is empty`. Nonempty explicit `queue/start`, selected-item start, automatic transition from an active turn's completion, and duplicate-add idempotency were not tested. |
| Ephemeral rejoin | `thread/resume` on our active ephemeral thread failed `-32600: no rollout found for thread id …`. Direct control/start still worked on that loaded ephemeral thread. |

## Exact payload shapes

All connections used Unix WebSocket HTTP Upgrade over `/Users/melikhov/.codex/app-server-control/app-server-control.sock`, initialized once per connection with `capabilities: {experimentalApi: true}`, then sent `initialized`. Clients were named `zoe_disposable_*`; default source/originator labels returned by the daemon do **not** prove a real native GUI was used.

```json
{"method":"turn/steer","params":{"threadId":"<thread>","expectedTurnId":"<active-turn>","input":[{"type":"text","text":"<guidance>"}]}}
{"result":{"turnId":"<same-active-turn>"}}

{"method":"turn/interrupt","params":{"threadId":"<thread>","turnId":"<active-turn>"}}
{"result":{}}

{"method":"turn/start","params":{"threadId":"<loaded-idle-thread>","input":[{"type":"text","text":"<feedback>"}]}}
{"result":{"turn":{"id":"<new-turn>","status":"inProgress"}}}

{"method":"thread/resume","params":{"threadId":"<running-durable-thread>","excludeTurns":true}}

{"method":"thread/queue/add","params":{"threadId":"<durable-thread>","clientUserMessageId":"<stable-client-message-id>","input":[{"type":"text","text":"<next-input>"}]}}
{"result":{"queuedSubmission":{"id":"<queue-item>","clientUserMessageId":"<stable-client-message-id>","input":[{"type":"text","text":"<next-input>","text_elements":[]}]}}}

{"method":"thread/queue/list","params":{"threadId":"<durable-thread>","limit":10}}
{"result":{"data":["<queuedSubmission objects>"],"nextCursor":null}}

{"method":"thread/queue/update","params":{"threadId":"<durable-thread>","queuedSubmissionId":"<queue-item>","input":[{"type":"text","text":"<replacement>"}]}}
{"method":"thread/queue/reorder","params":{"threadId":"<durable-thread>","queuedSubmissionIds":["<second>","<first>"]}}
{"method":"thread/queue/delete","params":{"threadId":"<durable-thread>","queuedSubmissionId":"<queue-item>"}}

{"id":7,"result":{"answers":{"probe_choice":{"answers":["Alpha"]}}}}
{"method":"serverRequest/resolved","params":{"threadId":"<durable-thread>","requestId":7}}
```

The `turn/start` example above gives the minimum schema shape. The exact successful idle probe also supplied `collaborationMode: {mode: "plan", settings: {model: "gpt-6.1-sol", developer_instructions: "…disposable probe…"}}` to expose `request_user_input`; these were new disposable-thread settings, not recommended overrides when joining user work. Full exact payloads, IDs, results, timestamps and native question contents are in the JSONL evidence.

## Evidence and scripts

The findings, scripts and schemas are versioned. Raw recordings, output text
and disposable thread markers below remain local Git-ignored artifacts in this
directory; their filenames are preserved for correlation with the original run.

- **Successful durable coexistence and queue CRUD:** `transcript-durable-continued.jsonl`, `probe-durable-continued-output.txt`. The final line reports `CONTINUED_PROBE_SUCCESS`; archive returned `{}`.
- **Successful active guidance, exact interruption, idle start; ephemeral resume limitation:** `transcript-control.jsonl`, `probe-control-output.txt`. The run intentionally stopped after discovering ephemeral resume was unsupported; cleanup interrupted its own question turn.
- **First ephemeral queue rejection:** `transcript-queue-ephemeral-rejection.jsonl`, `probe-output.txt`.
- **Discovery that idle queue/add immediately starts:** `transcript-durable.jsonl`, `probe-durable-output.txt`. Cleanup archived its own active disposable thread and its turn was interrupted.
- **Archived queue rejection:** `transcript-durable-archived-queue-rejection.jsonl`.
- **Scripts:** [probe.py](probe.py) is the external Unix-WebSocket client and ephemeral control probe. Its final source handles the discovered ephemeral-resume error so a rerun can answer its own question. [probe_durable_continue.py](probe_durable_continue.py) is the successful durable probe, limited to the recorded disposable thread. [probe_durable.py](probe_durable.py) preserves the initial failing idle-queue assumption as investigation evidence; do not use it as the workflow model. `python3 -m py_compile` passed for all three scripts. There are no product-source/test changes.
- **Installed exact schema:** [schema-experimental/codex_app_server_protocol.schemas.json](schema-experimental/codex_app_server_protocol.schemas.json), generated by installed `codex app-server generate-json-schema --experimental`. Queue methods were omitted from the previous stable export. All schema `v2/ThreadQueue*Params.json` and response files are present here.

Total model activity was limited to seven short disposable turn starts across the investigation: three completed final markers (`STEER_ACCEPTED`, `QUESTION_OK`, `QUEUE_OK`) and four interrupted turns. No simultaneous model turns were intentionally run. The only durable thread ID was `01a16981-e61b-7641-8534-eb9457972674`; it is archived. The two ephemeral IDs were `01a1697f-9520-7240-859e-45e453ac2d76` and `01a16980-46d3-73b0-8b7f-270e681aad63`.

## Remaining limits

This establishes installed protocol co-client behavior using two probe clients, not a real VS Code GUI acceptance test. Native command/file/permission approval requests were not created or answered. Question broadcast/replay/resolution does not establish all approval arbitration semantics. Simultaneous replies, second-client answering, stale responses after resolution, disconnect ownership transfer, runtime restart queue persistence, crash recovery, and automatic drain of a previously active thread were not tested. Acknowledgment means accepted input, not completed user intent. Treat native request IDs as pending only until `serverRequest/resolved` or clear termination evidence; never submit defaults or replay uncertain feedback automatically.

Official supporting documentation: [Codex App Server](https://learn.chatgpt.com/docs/app-server). Its thread/turn lifecycle supports the payload interpretation above; exact installed behavior and experimental queue fields come from the local export and probes.
