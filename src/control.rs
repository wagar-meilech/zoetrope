//! Explicit local Codex control. Connection, observation and control perform
//! blocking local IO: run them on an IO worker, never the terminal drawing thread.
//! Cached metadata access performs no IO; passive reads invoke no model.

use crate::console::AgentKey;
use crate::provider::Provider;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(unix)]
use serde_json::json;
use std::collections::BTreeMap;
#[cfg(unix)]
use std::collections::{BTreeSet, VecDeque};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;
#[cfg(unix)]
use std::time::{Duration, Instant};

/// Current runtime state; `NotLoaded` is not evidence that a goal is complete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeStatus {
    Working,
    WaitingForAnswer,
    WaitingForApproval,
    Idle,
    NotLoaded,
    Error,
    Unknown,
}

/// Bounded metadata for one exact native thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadSnapshot {
    pub thread_id: String,
    pub name: Option<String>,
    pub cwd: Option<String>,
    pub status: RuntimeStatus,
    pub active_turn: Option<String>,
    pub can_accept_direct_input: Option<bool>,
}

/// Exact UI selection, with a separate replay guard. The root `main` maps only
/// to its own native thread; a child maps only to its own agent ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlTarget {
    pub agent: AgentKey,
    pub replay: bool,
}
impl ControlTarget {
    pub fn thread_id(&self) -> anyhow::Result<&str> {
        anyhow::ensure!(
            !self.replay,
            "Replay is read-only; return to live control before sending input"
        );
        anyhow::ensure!(
            self.agent.session.provider == Provider::Codex,
            "This provider supports transcript viewing only"
        );
        let id = if self.agent.agent_id == "main" {
            &self.agent.session.session_id
        } else {
            &self.agent.agent_id
        };
        anyhow::ensure!(
            !id.is_empty(),
            "The selected agent has no native thread identity"
        );
        Ok(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlAction {
    Guide {
        text: String,
        expected_turn: Option<String>,
    },
    Queue {
        text: String,
        client_message_id: String,
    },
    Stop {
        expected_turn: String,
    },
    Join,
    Answer {
        request_id: Value,
        turn_id: String,
        answers: BTreeMap<String, Vec<String>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptState {
    Accepted,
    Queued,
    Submitted,
    Rejected,
    Unavailable,
    Unknown,
}
/// Runtime acknowledgment is distinct from completion of the requested work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub state: ReceiptState,
    pub message: String,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub queue_id: Option<String>,
}

#[cfg(unix)]
#[derive(Debug)]
struct RuntimeRejection(String);
#[cfg(unix)]
impl std::fmt::Display for RuntimeRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
#[cfg(unix)]
impl std::error::Error for RuntimeRejection {}

/// One native prompt. This records the public question, never protected reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePrompt {
    pub id: String,
    #[serde(default)]
    pub header: String,
    pub question: String,
    #[serde(default)]
    pub options: Option<Vec<NativeOption>>,
    #[serde(default)]
    pub is_other: bool,
    #[serde(default)]
    pub is_secret: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeOption {
    pub label: String,
    #[serde(default)]
    pub description: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeQuestion {
    #[serde(default)]
    pub item_id: Option<String>,
    pub request_id: Value,
    pub thread_id: String,
    pub turn_id: String,
    pub prompts: Vec<NativePrompt>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    ThreadStatus {
        thread_id: String,
        status: RuntimeStatus,
    },
    TurnStarted {
        thread_id: String,
        turn_id: String,
    },
    TurnCompleted {
        thread_id: String,
        turn_id: String,
    },
    Question(NativeQuestion),
    QuestionResolved {
        thread_id: String,
        request_id: Value,
    },
    Disconnected {
        reason: String,
    },
    Notice {
        message: String,
    },
}

/// The installed shared runtime socket; this never starts a replacement daemon.
pub fn default_socket() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"))
        .join("app-server-control/app-server-control.sock")
}

/// One local connection, owned by an IO worker. No global settings are changed.
#[cfg(unix)]
pub struct Controller {
    socket: tungstenite::WebSocket<std::os::unix::net::UnixStream>,
    next_id: u64,
    events: VecDeque<RuntimeEvent>,
    pending: BTreeMap<String, NativeQuestion>,
    joined: BTreeSet<String>,
    snapshots: BTreeMap<String, ThreadSnapshot>,
    disconnected: bool,
}

#[cfg(unix)]
impl Controller {
    pub fn connect(path: &Path) -> anyhow::Result<Self> {
        use anyhow::Context;
        let stream = std::os::unix::net::UnixStream::connect(path)
            .with_context(|| format!("Local Codex connection unavailable: {}", path.display()))?;
        stream.set_read_timeout(Some(Duration::from_millis(250)))?;
        stream.set_write_timeout(Some(Duration::from_secs(3)))?;
        let config = tungstenite::protocol::WebSocketConfig::default()
            .max_message_size(Some(4 * 1024 * 1024))
            .max_frame_size(Some(4 * 1024 * 1024));
        let (socket, _) =
            tungstenite::client::client_with_config("ws://localhost/", stream, Some(config))
                .map_err(|e| anyhow::anyhow!("Local Codex WebSocket handshake failed: {e}"))?;
        let mut this = Self {
            socket,
            next_id: 0,
            events: VecDeque::new(),
            pending: BTreeMap::new(),
            joined: BTreeSet::new(),
            snapshots: BTreeMap::new(),
            disconnected: false,
        };
        this.call("initialize", json!({"clientInfo":{"name":"zoetrope","title":"Zoetrope goal console","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}}))?;
        this.send(json!({"method":"initialized","params":{}}))?;
        Ok(this)
    }

    /// IDs of already-loaded threads, with bounded pagination. This does not
    /// resume stored histories or invoke a model. Call `snapshot` for status.
    pub fn loaded_threads(&mut self) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::new();
        let mut cursor: Option<String> = None;
        let mut seen = BTreeSet::new();
        for _ in 0..16 {
            let mut params = json!({"limit":128});
            if let Some(cursor) = &cursor {
                params["cursor"] = json!(cursor);
            }
            let reply = self.call("thread/loaded/list", params)?;
            let page = reply["data"].as_array().ok_or_else(|| {
                anyhow::anyhow!("Codex returned an incompatible loaded-thread list")
            })?;
            anyhow::ensure!(
                page.len() <= 128,
                "Codex returned an oversized loaded-thread page"
            );
            ids.extend(page.iter().filter_map(Value::as_str).map(str::to_owned));
            cursor = reply["nextCursor"].as_str().map(str::to_owned);
            let Some(next) = &cursor else {
                return Ok(ids);
            };
            anyhow::ensure!(
                seen.insert(next.clone()),
                "Codex repeated a loaded-thread cursor"
            );
        }
        anyhow::bail!("Live inventory exceeds 2048 entries; further automatic discovery is bounded")
    }

    /// Read real runtime metadata without joining, starting, or resuming work.
    pub fn snapshot(&mut self, thread_id: &str) -> anyhow::Result<ThreadSnapshot> {
        let reply = self.call(
            "thread/read",
            json!({"threadId":thread_id,"includeTurns":false}),
        )?;
        let thread = &reply["thread"];
        anyhow::ensure!(
            thread["id"].as_str() == Some(thread_id),
            "Codex returned a different thread; control unavailable"
        );
        let status = parse_status(&thread["status"]);
        let active_turn = if matches!(
            status,
            RuntimeStatus::Working
                | RuntimeStatus::WaitingForAnswer
                | RuntimeStatus::WaitingForApproval
        ) {
            let turns = self.call("thread/turns/list", json!({"threadId":thread_id,"limit":1,"sortDirection":"desc","itemsView":"notLoaded"}))?;
            turns["data"]
                .as_array()
                .and_then(|a| a.first())
                .filter(|t| t["status"] == "inProgress")
                .and_then(|t| t["id"].as_str())
                .map(str::to_owned)
        } else {
            None
        };
        let snapshot = ThreadSnapshot {
            thread_id: thread_id.to_owned(),
            name: thread["name"].as_str().map(str::to_owned),
            cwd: thread["cwd"].as_str().map(str::to_owned),
            status,
            active_turn,
            can_accept_direct_input: thread["canAcceptDirectInput"].as_bool(),
        };
        if self.snapshots.len() >= 4096
            && !self.snapshots.contains_key(thread_id)
            && let Some(key) = self.snapshots.keys().next().cloned()
        {
            self.snapshots.remove(&key);
        }
        self.snapshots.insert(thread_id.into(), snapshot.clone());
        Ok(snapshot)
    }

    /// Execute an explicit user action, freshly validating the exact target and
    /// pinned turn. Stale/uncertain input is never retried or sent to a parent.
    pub fn execute(&mut self, target: &ControlTarget, action: ControlAction) -> DeliveryReceipt {
        let mut receipt = DeliveryReceipt {
            state: ReceiptState::Unavailable,
            message: String::new(),
            thread_id: None,
            turn_id: None,
            queue_id: None,
        };
        let id = match target.thread_id() {
            Ok(id) => id,
            Err(e) => {
                receipt.message = e.to_string();
                return receipt;
            }
        };
        receipt.thread_id = Some(id.to_owned());
        let snapshot = match self.snapshot(id) {
            Ok(s) => s,
            Err(e) => {
                receipt.message = e.to_string();
                return receipt;
            }
        };
        if let ControlAction::Answer {
            request_id,
            turn_id,
            answers,
        } = &action
        {
            let events = self.poll_events(Duration::ZERO);
            self.events.extend(events);
            let key = format!("{id}:{request_id}");
            let question = self.pending.get(&key);
            if self.disconnected
                || snapshot.active_turn.as_deref() != Some(turn_id)
                || !question.is_some_and(|q| q.thread_id == id && q.turn_id == *turn_id)
            {
                receipt.message = "The original question is no longer confirmed pending on this connection. Keep the draft or explicitly send it as feedback".into();
                return receipt;
            }
            let question = question.unwrap();
            if answers.len() != question.prompts.len()
                || question.prompts.iter().any(|p| {
                    !answers
                        .get(&p.id)
                        .is_some_and(|a| !a.is_empty() && a.iter().all(|x| !x.trim().is_empty()))
                })
            {
                receipt.message =
                    "Answer every prompt explicitly; no default answer was submitted".into();
                return receipt;
            }
            let answers: BTreeMap<_, _> = answers
                .iter()
                .map(|(id, value)| (id, json!({"answers":value})))
                .collect();
            receipt.turn_id = Some(turn_id.clone());
            match self.send(json!({"id":request_id,"result":{"answers":answers}})) {
                Ok(()) => {
                    self.pending.remove(&key);
                    receipt.state = ReceiptState::Submitted;
                    receipt.message = "Answer submitted; awaiting native runtime resolution. Submission is not a confirmation that the answer was accepted".into();
                }
                Err(e) => {
                    self.disconnect(e.to_string());
                    receipt.state = ReceiptState::Unknown;
                    receipt.message = format!("Answer delivery unknown; not retried: {e}");
                }
            }
            return receipt;
        }
        if snapshot.can_accept_direct_input == Some(false)
            && matches!(
                action,
                ControlAction::Guide { .. } | ControlAction::Queue { .. }
            )
        {
            receipt.message = "The selected thread does not accept direct input; explicitly select another agent if intended".into();
            return receipt;
        }
        let (method, params, accepted_state, success_message) = match action {
            ControlAction::Guide {
                text,
                expected_turn,
            } => {
                if text.trim().is_empty() {
                    receipt.message = "Guidance is empty; nothing was sent".into();
                    return receipt;
                }
                if snapshot.active_turn != expected_turn
                    || (expected_turn.is_none() && snapshot.status != RuntimeStatus::Idle)
                {
                    receipt.message = "The active turn changed or is unavailable; review the target before sending again".into();
                    return receipt;
                }
                receipt.turn_id = expected_turn.clone();
                if expected_turn.is_none() {
                    (
                        "turn/start",
                        json!({"threadId":id,"input":[{"type":"text","text":text}]}),
                        ReceiptState::Accepted,
                        "Codex accepted new input on the idle thread; this does not mean the requested work is completed",
                    )
                } else {
                    (
                        "turn/steer",
                        json!({"threadId":id,"expectedTurnId":expected_turn,"input":[{"type":"text","text":text}]}),
                        ReceiptState::Accepted,
                        "Accepted by Codex; this does not mean the requested work is completed",
                    )
                }
            }
            ControlAction::Answer { .. } => unreachable!("answer handled above"),
            ControlAction::Join => {
                if !matches!(
                    snapshot.status,
                    RuntimeStatus::Working
                        | RuntimeStatus::WaitingForAnswer
                        | RuntimeStatus::WaitingForApproval
                        | RuntimeStatus::Idle
                ) {
                    receipt.message = "Only an already-loaded live thread can be joined; historical work was not resumed".into();
                    return receipt;
                }
                self.joined.insert(id.into());
                (
                    "thread/resume",
                    json!({"threadId":id,"excludeTurns":true}),
                    ReceiptState::Accepted,
                    "Joined the existing thread for native questions without changing its settings",
                )
            }
            ControlAction::Stop { expected_turn } => {
                if snapshot.active_turn.as_deref() != Some(&expected_turn) {
                    receipt.message =
                        "The active turn changed or is unavailable; nothing was stopped".into();
                    return receipt;
                }
                receipt.turn_id = Some(expected_turn.clone());
                (
                    "turn/interrupt",
                    json!({"threadId":id,"turnId":expected_turn}),
                    ReceiptState::Accepted,
                    "Codex accepted the interruption request; completion will be observed separately",
                )
            }
            ControlAction::Queue {
                text,
                client_message_id,
            } => {
                if text.trim().is_empty() || client_message_id.trim().is_empty() {
                    receipt.message = "Queue text and a stable message ID are required".into();
                    return receipt;
                }
                if !matches!(
                    snapshot.status,
                    RuntimeStatus::Working
                        | RuntimeStatus::WaitingForAnswer
                        | RuntimeStatus::WaitingForApproval
                        | RuntimeStatus::Idle
                ) {
                    receipt.message =
                        "This thread is not loaded for native input; nothing was queued".into();
                    return receipt;
                }
                (
                    "thread/queue/add",
                    json!({"threadId":id,"clientUserMessageId":client_message_id,"input":[{"type":"text","text":text}]}),
                    ReceiptState::Queued,
                    "Accepted into the Codex queue; idle threads may begin immediately. This is not a local draft or evidence of completion",
                )
            }
        };
        let joining = method == "thread/resume";
        let result = self.call(method, params);
        if joining && result.is_err() {
            self.joined.remove(id);
            self.pending.retain(|_, q| q.thread_id != id);
        }
        match result {
            Ok(value) => {
                let compatible = match method {
                    "turn/steer" => value["turnId"].as_str() == receipt.turn_id.as_deref(),
                    "turn/start" => value["turn"]["id"].as_str().is_some(),
                    "thread/queue/add" => value["queuedSubmission"]["id"].as_str().is_some(),
                    "thread/resume" => value["thread"]["id"].as_str() == Some(id),
                    _ => true,
                };
                if !compatible {
                    receipt.state = ReceiptState::Unknown;
                    receipt.message = "Codex returned an incompatible acknowledgment; delivery is unknown and was not retried".into();
                    self.disconnect(receipt.message.clone());
                    return receipt;
                }
                receipt.state = accepted_state;
                receipt.message = success_message.into();
                receipt.queue_id = value["queuedSubmission"]["id"].as_str().map(str::to_owned);
                if let Some(turn) = value["turn"]["id"].as_str() {
                    receipt.turn_id = Some(turn.into());
                }
            }
            Err(e) if e.downcast_ref::<RuntimeRejection>().is_some() => {
                receipt.state = ReceiptState::Rejected;
                receipt.message = e.to_string();
            }
            Err(e) => {
                self.disconnect(e.to_string());
                receipt.state = ReceiptState::Unknown;
                receipt.message =
                    format!("Delivery unknown after connection failure; not retried: {e}");
            }
        }
        receipt
    }

    /// Cached status reads perform no IO. Event notifications update this map.
    pub fn cached_snapshots(&self) -> &BTreeMap<String, ThreadSnapshot> {
        &self.snapshots
    }

    /// Drain at most 64 wire messages, waiting at most 250 ms. Requests received
    /// while waiting for an RPC are retained here too. Never answers anything.
    pub fn poll_events(&mut self, wait: Duration) -> Vec<RuntimeEvent> {
        let deadline = Instant::now() + wait.min(Duration::from_millis(250));
        for _ in 0..64 {
            if self.disconnected {
                break;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            let timeout = remaining
                .max(Duration::from_millis(1))
                .min(Duration::from_millis(250));
            let _ = self.socket.get_mut().set_read_timeout(Some(timeout));
            match self.socket.read() {
                Ok(tungstenite::Message::Text(text)) => match serde_json::from_str(&text) {
                    Ok(value) => self.ingest(value),
                    Err(e) => {
                        self.disconnect(format!("Invalid Codex message: {e}"));
                        break;
                    }
                },
                Ok(tungstenite::Message::Close(_)) => {
                    self.disconnect("Local Codex disconnected".into());
                    break;
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(e) => {
                    self.disconnect(format!("Local Codex disconnected: {e}"));
                    break;
                }
            }
            if !wait.is_zero() && Instant::now() >= deadline {
                break;
            }
        }
        let _ = self
            .socket
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(250)));
        self.events.drain(..).collect()
    }

    fn disconnect(&mut self, reason: String) {
        if self.disconnected {
            return;
        }
        self.disconnected = true;
        self.pending.clear();
        self.joined.clear();
        for snapshot in self.snapshots.values_mut() {
            snapshot.status = RuntimeStatus::Unknown;
            snapshot.active_turn = None;
        }
        self.push_event(RuntimeEvent::Disconnected { reason });
    }

    fn push_event(&mut self, event: RuntimeEvent) {
        if self.events.len() >= 128 {
            self.pending.clear();
            self.events.clear();
            self.events.push_back(RuntimeEvent::Notice {
                message: "Runtime event backlog exceeded; native answers require rejoining".into(),
            });
        }
        self.events.push_back(event);
    }

    fn ingest(&mut self, message: Value) {
        let params = &message["params"];
        let thread = params["threadId"].as_str();
        match message["method"].as_str() {
            Some("thread/status/changed") => {
                if let Some(id) = thread {
                    let status = parse_status(&params["status"]);
                    if let Some(snapshot) = self.snapshots.get_mut(id) {
                        snapshot.status = status.clone();
                        if !matches!(
                            status,
                            RuntimeStatus::Working
                                | RuntimeStatus::WaitingForAnswer
                                | RuntimeStatus::WaitingForApproval
                        ) {
                            snapshot.active_turn = None;
                        }
                    }
                    self.push_event(RuntimeEvent::ThreadStatus {
                        thread_id: id.into(),
                        status,
                    });
                }
            }
            Some("turn/started" | "turn/completed") => {
                if let (Some(id), Some(turn)) = (thread, params["turn"]["id"].as_str()) {
                    let started = message["method"] == "turn/started";
                    if let Some(snapshot) = self.snapshots.get_mut(id) {
                        snapshot.active_turn = started.then(|| turn.to_owned());
                        snapshot.status = if started {
                            RuntimeStatus::Working
                        } else {
                            RuntimeStatus::Idle
                        };
                    }
                    if !started {
                        self.pending
                            .retain(|_, q| q.thread_id != id || q.turn_id != turn);
                    }
                    self.push_event(if started {
                        RuntimeEvent::TurnStarted {
                            thread_id: id.into(),
                            turn_id: turn.into(),
                        }
                    } else {
                        RuntimeEvent::TurnCompleted {
                            thread_id: id.into(),
                            turn_id: turn.into(),
                        }
                    });
                }
            }
            Some("item/tool/requestUserInput") => {
                if let (Some(id), Some(turn), Some(request_id)) =
                    (thread, params["turnId"].as_str(), message.get("id"))
                {
                    if !self.joined.contains(id) || !(request_id.is_string() || request_id.is_i64())
                    {
                        return;
                    }
                    if message.to_string().len() > 64 * 1024 || self.pending.len() >= 64 {
                        self.push_event(RuntimeEvent::Notice { message: "Native question exceeds local monitoring limits; answer it in Codex".into() });
                        return;
                    }
                    if let Ok(prompts) =
                        serde_json::from_value::<Vec<NativePrompt>>(params["questions"].clone())
                    {
                        let question = NativeQuestion {
                            item_id: params["itemId"].as_str().map(str::to_owned),
                            request_id: request_id.clone(),
                            thread_id: id.into(),
                            turn_id: turn.into(),
                            prompts,
                        };
                        self.pending
                            .insert(format!("{id}:{request_id}"), question.clone());
                        self.push_event(RuntimeEvent::Question(question));
                    }
                }
            }
            Some("serverRequest/resolved") => {
                if let (Some(id), Some(request_id)) = (thread, params.get("requestId")) {
                    self.pending.remove(&format!("{id}:{request_id}"));
                    self.push_event(RuntimeEvent::QuestionResolved {
                        thread_id: id.into(),
                        request_id: request_id.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn send(&mut self, value: Value) -> anyhow::Result<()> {
        self.socket
            .send(tungstenite::Message::Text(value.to_string().into()))?;
        Ok(())
    }

    fn call(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        let result = self.call_inner(method, params);
        if let Err(error) = &result
            && error.downcast_ref::<RuntimeRejection>().is_none()
        {
            self.disconnect(error.to_string());
        }
        result
    }

    fn call_inner(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        anyhow::ensure!(
            !self.disconnected,
            "Local Codex disconnected; reconnect explicitly"
        );
        self.next_id += 1;
        let id = format!("zoe-{}", self.next_id);
        self.send(json!({"id":id,"method":method,"params":params}))?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            match self.socket.read() {
                Ok(tungstenite::Message::Text(text)) => {
                    let reply: Value = serde_json::from_str(&text)?;
                    if reply["id"] == id
                        && (reply.get("result").is_some() || reply.get("error").is_some())
                    {
                        if let Some(error) = reply.get("error") {
                            return Err(RuntimeRejection(format!(
                                "Codex rejected {method}: {}",
                                error["message"]
                            ))
                            .into());
                        }
                        return Ok(reply["result"].clone());
                    }
                    self.ingest(reply);
                }
                Ok(tungstenite::Message::Close(_)) => anyhow::bail!("Local Codex disconnected"),
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(e) => return Err(anyhow::anyhow!("Local Codex disconnected: {e}")),
            }
        }
        anyhow::bail!("Local Codex did not acknowledge {method} before the deadline")
    }
}

#[cfg(unix)]
fn parse_status(value: &Value) -> RuntimeStatus {
    match value["type"].as_str() {
        Some("active") => {
            let flags = value["activeFlags"].as_array();
            if flags.is_some_and(|f| f.iter().any(|x| x == "waitingOnUserInput")) {
                RuntimeStatus::WaitingForAnswer
            } else if flags.is_some_and(|f| f.iter().any(|x| x == "waitingOnApproval")) {
                RuntimeStatus::WaitingForApproval
            } else {
                RuntimeStatus::Working
            }
        }
        Some("idle") => RuntimeStatus::Idle,
        Some("notLoaded") => RuntimeStatus::NotLoaded,
        Some("systemError") => RuntimeStatus::Error,
        _ => RuntimeStatus::Unknown,
    }
}

#[cfg(unix)]
const CONTROL_USAGE: &str = "Local Codex controls (Unix, existing shared runtime only):
  zoe control [--socket PATH] loaded
  zoe control [--socket PATH] status THREAD
  zoe control [--socket PATH] guide THREAD --turn TURN TEXT
  zoe control [--socket PATH] guide THREAD --idle TEXT
  zoe control [--socket PATH] queue THREAD --message-id STABLE_ID TEXT
  zoe control [--socket PATH] stop THREAD --turn TURN
  zoe control [--socket PATH] join THREAD
  zoe control [--socket PATH] answer THREAD --turn TURN --request JSON_ID --answers JSON
Queue submission may start idle work immediately. Join and answer rejoin only an already-loaded thread without settings overrides. Answer verifies a current question replay; no defaults are submitted. Accepted input does not mean completed work. Unknown delivery is never retried automatically.";

/// Headless executable interface. Returns `None` for another subcommand.
/// This is blocking IO; the TUI must instead call the controller on its worker.
pub fn run_cli(args: &[String]) -> Option<anyhow::Result<()>> {
    if args.first().map(String::as_str) != Some("control") {
        return None;
    }
    Some(run_control_cli(&args[1..]))
}

#[cfg(unix)]
fn run_control_cli(args: &[String]) -> anyhow::Result<()> {
    if args.is_empty()
        || args
            .iter()
            .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        println!("{CONTROL_USAGE}");
        return Ok(());
    }
    let mut args = args.iter().map(String::as_str).peekable();
    let socket = if args.peek() == Some(&"--socket") {
        args.next();
        PathBuf::from(
            args.next()
                .ok_or_else(|| anyhow::anyhow!("--socket requires a path"))?,
        )
    } else {
        default_socket()
    };
    let action = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("Usage: zoe control [--socket PATH] status THREAD"))?;
    if action == "loaded" {
        anyhow::ensure!(args.next().is_none(), "Unexpected argument");
        println!(
            "{}",
            serde_json::to_string_pretty(&Controller::connect(&socket)?.loaded_threads()?)?
        );
        return Ok(());
    }
    let thread = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("control requires a thread ID"))?;
    let mut controller = Controller::connect(&socket)?;
    if action == "status" {
        anyhow::ensure!(args.next().is_none(), "Unexpected argument");
        println!(
            "{}",
            serde_json::to_string_pretty(&controller.snapshot(thread)?)?
        );
    } else {
        let target = ControlTarget {
            agent: AgentKey {
                session: crate::console::SessionKey {
                    provider: Provider::Codex,
                    session_id: thread.into(),
                },
                agent_id: "main".into(),
            },
            replay: false,
        };
        let action = match action {
            "guide" => {
                let expected_turn = match args.next() {
                    Some("--turn") => Some(
                        args.next()
                            .ok_or_else(|| anyhow::anyhow!("--turn requires an ID"))?
                            .to_owned(),
                    ),
                    Some("--idle") => None,
                    _ => anyhow::bail!("guide requires --turn ID or --idle"),
                };
                let text = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("guide requires guidance text"))?;
                ControlAction::Guide {
                    text: text.into(),
                    expected_turn,
                }
            }
            "join" => ControlAction::Join,
            "answer" => {
                anyhow::ensure!(args.next() == Some("--turn"), "answer requires --turn ID");
                let turn_id = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--turn requires an ID"))?
                    .to_owned();
                anyhow::ensure!(
                    args.next() == Some("--request"),
                    "answer requires --request JSON_ID"
                );
                let request_id = serde_json::from_str(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--request requires a JSON ID"))?,
                )?;
                anyhow::ensure!(
                    args.next() == Some("--answers"),
                    "answer requires --answers JSON"
                );
                let answers = serde_json::from_str(
                    args.next()
                        .ok_or_else(|| anyhow::anyhow!("--answers requires JSON"))?,
                )?;
                let joined = controller.execute(&target, ControlAction::Join);
                anyhow::ensure!(joined.state == ReceiptState::Accepted, "{}", joined.message);
                let events = controller.poll_events(Duration::from_millis(100));
                controller.events.extend(events);
                ControlAction::Answer {
                    request_id,
                    turn_id,
                    answers,
                }
            }
            "stop" => {
                anyhow::ensure!(args.next() == Some("--turn"), "stop requires --turn ID");
                let turn = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--turn requires an ID"))?;
                ControlAction::Stop {
                    expected_turn: turn.into(),
                }
            }
            "queue" => {
                anyhow::ensure!(
                    args.next() == Some("--message-id"),
                    "queue requires a stable --message-id ID; it may start idle work immediately"
                );
                let id = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--message-id requires an ID"))?;
                let text = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("queue requires input text"))?;
                ControlAction::Queue {
                    text: text.into(),
                    client_message_id: id.into(),
                }
            }
            other => anyhow::bail!("Unknown control action {other}"),
        };
        anyhow::ensure!(args.next().is_none(), "Unexpected argument");
        let joining = matches!(action, ControlAction::Join);
        let receipt = controller.execute(&target, action);
        println!("{}", serde_json::to_string_pretty(&receipt)?);
        anyhow::ensure!(
            matches!(
                receipt.state,
                ReceiptState::Accepted | ReceiptState::Queued | ReceiptState::Submitted
            ),
            "{}",
            receipt.message
        );
        if joining {
            println!(
                "{}",
                serde_json::to_string_pretty(&controller.poll_events(Duration::from_millis(100)))?
            );
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn run_control_cli(_: &[String]) -> anyhow::Result<()> {
    anyhow::bail!("Local Codex control currently requires a Unix socket; replay remains available")
}
