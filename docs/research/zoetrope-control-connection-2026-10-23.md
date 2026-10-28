# Zoetrope: connection to the existing Codex runtime

Read-only exploration on 23 October 2026. No thread was started, resumed, steered, interrupted, forked, or otherwise changed. No server-initiated request was answered. No product source or tests were written.

## Verified outcome

A second client **successfully initialized against the already-running local Codex 0.164.0 daemon** and read its loaded threads, historical thread summaries, metadata, goal state, and newest turn identities. The loaded tree included this ongoing research conversation and the child whose `agent_path` is `/root/codex_control_probe`; this establishes access to the current runtime rather than only historical rollout files.

`codex app-server daemon version` returned:

```json
{"status":"running","backend":"pid","managedCodexPath":"/Users/melikhov/.codex/packages/app-server-daemon/current/bin/codex","managedCodexVersion":"0.164.0","socketPath":"/Users/melikhov/.codex/app-server-control/app-server-control.sock","cliVersion":"0.164.0","appServerVersion":"0.164.0"}
```

Socket access required sandbox escalation in this research environment. No daemon startup/restart command was used.

## Transport: the prior timeout is explained

`codex app-server proxy --help` says: “Proxy stdio bytes to the running app-server control socket.” It does not promise JSONL-to-WebSocket conversion. Sending a newline-terminated `initialize` JSON object through that proxy produced zero output in a 15-second repeat probe.

The Unix control socket requires a **WebSocket HTTP Upgrade handshake**, followed by one JSON-RPC object per WebSocket text frame. The direct Unix WebSocket connection below succeeded immediately. Official OpenAI documentation confirms this Unix transport and distinguishes it from the JSONL transport of a newly spawned `codex app-server --listen stdio://`. [OpenAI App Server documentation](https://learn.chatgpt.com/docs/app-server#protocol)

Thus, simply spawning `codex app-server proxy` and writing JSONL is insufficient for this daemon. A Rust client can connect directly with a WebSocket implementation over a Unix stream. A byte proxy can also carry a WebSocket handshake/framed stream, but this was not separately tested.

## Working minimal read-only client

This uses the locally installed Python `websocket-client` package. The passed socket is an already-connected `AF_UNIX` stream; `ws://localhost/` supplies the HTTP Upgrade URI, not a TCP connection. Use an honest, stable client identity in the product.

```python
import json
import socket
import websocket

sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.settimeout(8)
sock.connect(
    "/Users/melikhov/.codex/app-server-control/app-server-control.sock"
)
ws = websocket.create_connection("ws://localhost/", socket=sock, timeout=8)

def request(request_id, method, params):
    ws.send(json.dumps({"id": request_id, "method": method, "params": params}))
    while True:
        message = json.loads(ws.recv())
        if message.get("id") == request_id and (
            "result" in message or "error" in message
        ):
            return message
        # Notifications may interleave. Do not answer server requests here.
        # A product client needs a dispatcher keyed by message kind and ID.

try:
    initialized = request(1, "initialize", {
        "clientInfo": {
            "name": "zoetrope_readonly_probe",
            "title": "Zoetrope Read-only Probe",
            "version": "0.0.0",
        },
        "capabilities": {"experimentalApi": True},
    })
    if "error" in initialized:
        raise RuntimeError(initialized["error"])
    ws.send(json.dumps({"method": "initialized", "params": {}}))
    loaded = request(2, "thread/loaded/list", {})
    recent = request(3, "thread/list", {
        "limit": 10,
        "sortKey": "updated_at",
        "useStateDbOnly": True,
    })
    # Render selected metadata; do not log whole responses or conversation text.
finally:
    ws.close()
```

Successful initialization returned `codexHome=/Users/melikhov/.codex`, `platformFamily=unix`, `platformOs=macos`, and a `userAgent` containing version `0.164.0` and the probe identity. The connection received `remoteControl/status/changed` and `account/updated` notifications without any thread subscription.

## Current runtime observations

At the first successful probe, `thread/loaded/list` returned four IDs: one root and three children. Metadata reads preserved `sessionId` and `parentThreadId` correctly. The root reported `source: "vscode"`; its rollout header reported `originator: "codex-tui"`. These are protocol source labels and should not alone be treated as proof of which graphical client is displaying a thread.

The root was `active`; children included both `active` and `idle`. The root and active children had newest turns with `status: "inProgress"`; the idle child's newest turn was `completed`. These values were read with:

```json
{"id":4,"method":"thread/read","params":{"threadId":"<loaded-thread-id>","includeTurns":false}}
{"id":5,"method":"thread/goal/get","params":{"threadId":"<loaded-thread-id>"}}
{"id":6,"method":"thread/turns/list","params":{"threadId":"<loaded-thread-id>","limit":1,"sortDirection":"desc","itemsView":"notLoaded"}}
```

All four loaded threads had `historyMode: "paginated"`; all four native goals were `null`. The metadata-only reads and turn-list query provide real thread/turn identities without printing user messages or loading full histories. `thread/list` with its default source filter also returned historical `notLoaded` conversations; these must not be presented as currently running or completed.

## Control and coexistence: established contract versus untested behavior

The exact generated 0.164.0 schema says `thread/resume` with an ID identifying a running thread **rejoins that thread**. It separately defines `turn/steer` with required `threadId`, `expectedTurnId`, and `input`; `expectedTurnId` is an active-turn precondition. `turn/interrupt` requires `threadId` and `turnId`. These are plausible control paths to the discovered native threads, with no need to create a replacement run.

However, **acceptance of a control request from this second client was deliberately not tested**. Neither the successful handshake nor a loaded status is proof that a particular feedback command will be accepted. There is no per-thread `canSteer`/`canInterrupt` field in the generated schema inspected here. Product compatibility should distinguish discovery, live observation, and verified control.

For safe coexistence that is established by these probes, keep a separate connection and use `thread/loaded/list`, `thread/list` with `useStateDbOnly: true`, metadata-only `thread/read`, goal/get, and pagination. Official documentation says `thread/read` does not resume or subscribe to thread events. Polling these reads plus existing transcript tailing can provide observation without becoming a subscriber. [OpenAI thread-read documentation](https://learn.chatgpt.com/docs/app-server#read-a-stored-thread-without-resuming)

The generated protocol exposes `thread/unsubscribe`, but no separate `thread/subscribe` request in the inspected export. Resuming a running thread is the documented rejoin path; it was not invoked. Resume parameters can override `approvalPolicy`, `approvalsReviewer`, model, instructions, and configuration, so an eventual join must not send copied defaults as overrides.

**Approval ownership remains unresolved.** The inspected initialization capabilities have no explicit passive/observer role or “leave approvals with native client” switch. Schema/docs describe server approval requests and `serverRequest/resolved`, but do not establish arbitration, broadcast, replay, timeout, or disconnect behavior with two subscribed clients. No live pending prompt was created, subscribed to, or answered in this exploration. Merely ignoring requests after an unverified subscription is not evidence of safe ownership behavior.

An initial implementation can honestly offer observation for existing native sessions and gate feedback behind verified compatibility. It should not start a second daemon, fork/resume a transcript in a new process as a substitute for live attachment, or promise to preserve native approval ownership until the co-client behavior has been established on a disposable session. The source contract suggests rejoining can avoid duplicate runs; approval coexistence still needs a separate, authorized integration check.

## Exact-version caveats and sources

The installed schema export remains the implementation reference: [generated 0.164.0 schema](zoetrope-codex-schema-0.164.0/codex_app_server_protocol.schemas.json). Useful definitions are `InitializeCapabilities` (line 3402), `ThreadGoalGetParams` (22110), `ThreadListParams` (23343), `ThreadLoadedListParams` (23494), `ThreadReadParams` (23726), `ThreadResumeParams` (24381), `TurnInterruptParams` (26200), and `TurnSteerParams` (26522).

The current online documentation contains fields/methods absent from this generated export, including parent/ancestor filters on `thread/list`. The export may omit experimental-only fields depending on generator options. Do not infer installed support solely from the current online documentation or from absence in a stable export; negotiate and handle unsupported methods/fields explicitly.

Commands inspected: `codex app-server --help`, `codex app-server proxy --help`, `codex app-server daemon --help`, `codex app-server daemon version`, and `codex agents --help`. `codex agents` describes itself as browsing all agents on the shared local daemon; the interactive TUI was not launched.
