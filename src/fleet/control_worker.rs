use super::*;
use crate::control::{
    ControlAction, ControlTarget, DeliveryReceipt, ReceiptState, RuntimeStatus, ThreadSnapshot,
};
use crate::workstate::{DeliveryState, FeedbackMode, QuestionOption, QuestionPrompt};
pub(super) enum ControlCommand {
    Select(Option<AgentKey>),
    Join(AgentKey),
    Answer(QuestionRecord),
    Send {
        target: AgentKey,
        text: String,
        mode: FeedbackMode,
        expected_turn: Option<String>,
    },
    Shutdown,
}
pub(super) struct Receipt {
    pub target: AgentKey,
    pub message: String,
    pub cleared_text: Option<String>,
}
#[derive(Default)]
pub(super) struct ControlOutput {
    pub snapshots: BTreeMap<String, ThreadSnapshot>,
    pub receipt: Option<Receipt>,
    pub connected: bool,
    pub dirty: BTreeSet<SessionKey>,
    pub notice: Option<String>,
    pub inflight_id: Option<String>,
}
pub(super) struct ControlWorker {
    pub tx: mpsc::SyncSender<ControlCommand>,
    pub output: Arc<Mutex<ControlOutput>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl ControlWorker {
    pub fn start(config: FleetConfig, notify: Arc<tokio::sync::Notify>) -> Self {
        let (tx, rx) = mpsc::sync_channel(32);
        let output = Arc::new(Mutex::new(ControlOutput::default()));
        let shared = output.clone();
        let worker = thread::spawn(move || run(config, rx, shared, notify));
        Self {
            tx,
            output,
            thread: Some(worker),
        }
    }
    pub fn shutdown(&mut self) {
        if let Some(worker) = self.thread.take() {
            let _ = self.tx.send(ControlCommand::Shutdown);
            let _ = worker.join();
        }
    }
}
fn thread_id(target: &AgentKey) -> &str {
    if target.agent_id == "main" {
        &target.session.session_id
    } else {
        &target.agent_id
    }
}
fn receipt_message(receipt: &DeliveryReceipt, mode: FeedbackMode) -> String {
    match receipt.state {
        ReceiptState::Accepted if mode == FeedbackMode::Stop => {
            "Codex accepted the interruption request".into()
        }
        ReceiptState::Accepted => "Accepted by Codex; requested work is not yet verified".into(),
        ReceiptState::Queued => "Queued by Codex; idle work may start immediately".into(),
        ReceiptState::Submitted => "Answer submitted; waiting for native resolution".into(),
        ReceiptState::Unknown => "Delivery unknown; the action was not retried".into(),
        ReceiptState::Rejected => "Codex rejected this action; your draft was kept".into(),
        ReceiptState::Unavailable if receipt.message.contains("does not accept direct input") => {
            "This agent does not accept direct input. Select Coordinator to guide it.".into()
        }
        ReceiptState::Unavailable if receipt.message.contains("turn changed") => {
            "The active turn changed. Review the target before sending again.".into()
        }
        ReceiptState::Unavailable => {
            "This action is unavailable for the selected work; your draft was kept".into()
        }
    }
}
fn persist_send(
    store: &Store,
    target: &AgentKey,
    text: &str,
    mode: FeedbackMode,
    expected_turn: Option<String>,
    answer: Option<&QuestionRecord>,
    started: impl FnOnce(&str),
    execute: impl FnOnce(ControlAction) -> DeliveryReceipt,
) -> (String, Option<String>) {
    let id = format!(
        "zoe-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let input = if mode == FeedbackMode::Stop {
        "Stop current turn"
    } else {
        text
    };
    let saved = store.apply_many(
        &target.session,
        [
            WorkUpdate::CreateFeedback {
                id: id.clone(),
                target: target.clone(),
                text: input.into(),
                mode,
                expected_turn: expected_turn.clone(),
                source_question: answer.map(|q| q.id.clone()),
            },
            WorkUpdate::RecordDelivery {
                id: id.clone(),
                state: DeliveryState::Sending,
                message: None,
                queue_id: None,
            },
        ],
    );
    if saved.is_err() {
        return (
            "Could not save the delivery record; nothing was sent".into(),
            None,
        );
    }
    started(&id);
    let action = if let Some(question) = answer {
        ControlAction::Answer {
            request_id: question
                .request_id
                .clone()
                .unwrap_or(serde_json::Value::Null),
            turn_id: question.turn_id.clone().unwrap_or_default(),
            answers: question.answers.clone(),
        }
    } else {
        match mode {
            FeedbackMode::Stop => ControlAction::Stop {
                expected_turn: expected_turn.unwrap_or_default(),
            },
            FeedbackMode::Queue => ControlAction::Queue {
                text: text.into(),
                client_message_id: id.clone(),
            },
            _ => ControlAction::Guide {
                text: text.into(),
                expected_turn,
            },
        }
    };
    let receipt = execute(action);
    let message = receipt_message(&receipt, mode);
    let state = match receipt.state {
        ReceiptState::Accepted => DeliveryState::Accepted,
        ReceiptState::Queued => DeliveryState::Queued,
        ReceiptState::Submitted => DeliveryState::Submitted,
        ReceiptState::Rejected => DeliveryState::Rejected,
        ReceiptState::Unavailable => DeliveryState::Unavailable,
        ReceiptState::Unknown => DeliveryState::Unknown,
    };
    if store
        .apply(
            &target.session,
            WorkUpdate::RecordDelivery {
                id,
                state,
                message: Some(message.clone()),
                queue_id: receipt.queue_id,
            },
        )
        .is_err()
    {
        return (
            format!(
                "{message}. Could not save the final delivery state; do not resend automatically."
            ),
            None,
        );
    }
    let cleared_text = if matches!(receipt.state, ReceiptState::Accepted | ReceiptState::Queued)
        && matches!(
            mode,
            FeedbackMode::Guidance | FeedbackMode::Queue | FeedbackMode::CriteriaRequest
        ) {
        let _ = store.apply(
            &target.session,
            WorkUpdate::ClearDraftIfMatches {
                target: target.clone(),
                expected_text: text.into(),
            },
        );
        Some(text.into())
    } else {
        None
    };
    (message, cleared_text)
}
#[cfg(unix)]
fn run(
    config: FleetConfig,
    rx: mpsc::Receiver<ControlCommand>,
    output: Arc<Mutex<ControlOutput>>,
    notify: Arc<tokio::sync::Notify>,
) {
    use crate::control::{Controller, RuntimeEvent};
    let store = Store::new(config.state_root);
    let mut controller: Option<Controller> = None;
    let mut reconnect = Instant::now();
    let mut selected: Option<AgentKey> = None;
    let mut refresh = Instant::now();
    let mut targets = BTreeMap::<String, AgentKey>::new();
    let mut discover = Instant::now();
    let mut observe = std::collections::VecDeque::<String>::new();
    loop {
        if controller.is_none() && Instant::now() >= reconnect {
            controller = Controller::connect(&config.socket).ok();
            reconnect = Instant::now() + Duration::from_secs(10);
            output.lock().unwrap_or_else(|e| e.into_inner()).connected = controller.is_some();
            notify.notify_one();
        }
        if Instant::now() >= discover {
            if let Some(runtime) = controller.as_mut() {
                if let Ok(ids) = runtime.loaded_threads() {
                    let loaded: BTreeSet<_> = ids.iter().cloned().collect();
                    output
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .snapshots
                        .retain(|id, _| loaded.contains(id));
                    observe = ids.into();
                }
            }
            discover = Instant::now() + Duration::from_secs(10);
        }
        if let (Some(runtime), Some(id)) = (controller.as_mut(), observe.pop_front()) {
            if let Ok(snapshot) = runtime.snapshot(&id) {
                output
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .snapshots
                    .insert(id, snapshot);
                notify.notify_one();
            }
        }
        if Instant::now() >= refresh {
            if let (Some(runtime), Some(target)) = (controller.as_mut(), selected.as_ref()) {
                if target.session.provider == crate::provider::Provider::Codex {
                    if let Ok(snapshot) = runtime.snapshot(thread_id(target)) {
                        let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
                        let changed = out.snapshots.get(&snapshot.thread_id) != Some(&snapshot);
                        if out.snapshots.len() >= 4096 {
                            if let Some(key) = out.snapshots.keys().next().cloned() {
                                out.snapshots.remove(&key);
                            }
                        }
                        out.snapshots.insert(snapshot.thread_id.clone(), snapshot);
                        drop(out);
                        if changed {
                            notify.notify_one();
                        }
                    }
                }
            }
            refresh = Instant::now() + Duration::from_secs(2);
        }
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(ControlCommand::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(ControlCommand::Select(target)) => {
                if let Some(target) = &target {
                    targets.insert(thread_id(target).into(), target.clone());
                }
                selected = target;
                refresh = Instant::now();
            }
            Ok(ControlCommand::Join(target)) => {
                targets.insert(thread_id(&target).into(), target.clone());
                let message = match controller.as_mut() {
                    Some(runtime) => {
                        let receipt = runtime.execute(
                            &ControlTarget {
                                agent: target.clone(),
                                replay: false,
                            },
                            ControlAction::Join,
                        );
                        if receipt.state == ReceiptState::Accepted {
                            "Listening for this agent's native questions".into()
                        } else {
                            "Native questions are unavailable for this agent; saved drafts remain accessible".into()
                        }
                    }
                    None => "Codex controls unavailable; saved questions remain accessible".into(),
                };
                output.lock().unwrap_or_else(|e| e.into_inner()).receipt = Some(Receipt {
                    target,
                    message,
                    cleared_text: None,
                });
                notify.notify_one();
            }
            Ok(ControlCommand::Answer(question)) => {
                let target = question.target.clone();
                let text = serde_json::to_string(&question.answers).unwrap_or_default();
                let native = ControlTarget {
                    agent: target.clone(),
                    replay: false,
                };
                let (message, cleared_text) = persist_send(
                    &store,
                    &target,
                    &text,
                    FeedbackMode::Answer,
                    question.turn_id.clone(),
                    Some(&question),
                    |id| {
                        let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
                        out.inflight_id = Some(id.into());
                        out.dirty.insert(target.session.clone());
                        drop(out);
                        notify.notify_one();
                    },
                    |action| match controller.as_mut() {
                        Some(runtime) => runtime.execute(&native, action),
                        None => DeliveryReceipt {
                            state: ReceiptState::Unavailable,
                            message: "Codex connection unavailable".into(),
                            thread_id: None,
                            turn_id: None,
                            queue_id: None,
                        },
                    },
                );
                let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
                out.dirty.insert(target.session.clone());
                out.inflight_id = None;
                out.receipt = Some(Receipt {
                    target,
                    message,
                    cleared_text,
                });
                drop(out);
                notify.notify_one();
            }
            Ok(ControlCommand::Send {
                target,
                text,
                mode,
                expected_turn,
            }) => {
                let native = ControlTarget {
                    agent: target.clone(),
                    replay: false,
                };
                let (message, cleared_text) = persist_send(
                    &store,
                    &target,
                    &text,
                    mode,
                    expected_turn,
                    None,
                    |id| {
                        let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
                        out.inflight_id = Some(id.into());
                        out.dirty.insert(target.session.clone());
                        drop(out);
                        notify.notify_one();
                    },
                    |action| match controller.as_mut() {
                        Some(runtime) => runtime.execute(&native, action),
                        None => DeliveryReceipt {
                            state: ReceiptState::Unavailable,
                            message: "Local Codex connection unavailable".into(),
                            thread_id: None,
                            turn_id: None,
                            queue_id: None,
                        },
                    },
                );
                let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
                out.inflight_id = None;
                out.receipt = Some(Receipt {
                    target,
                    message,
                    cleared_text,
                });
                drop(out);
                notify.notify_one();
                refresh = Instant::now();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let events = controller
            .as_mut()
            .map(|runtime| runtime.poll_events(Duration::ZERO))
            .unwrap_or_default();
        let mut disconnected = false;
        for event in events {
            match event {
                RuntimeEvent::Question(question) => {
                    if let Some(target) = targets.get(&question.thread_id) {
                        let id = question
                            .item_id
                            .as_ref()
                            .map(|item| format!("transcript:{}:{item}", target.agent_id))
                            .unwrap_or_else(|| {
                                format!(
                                    "native/{}/{}/{}",
                                    question.thread_id, question.turn_id, question.request_id
                                )
                            });
                        let question = QuestionRecord {
                            id,
                            target: target.clone(),
                            request_id: Some(question.request_id),
                            turn_id: Some(question.turn_id),
                            prompts: question
                                .prompts
                                .into_iter()
                                .map(|p| QuestionPrompt {
                                    id: p.id,
                                    header: p.header,
                                    question: p.question,
                                    options: p
                                        .options
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|o| QuestionOption {
                                            label: o.label,
                                            description: o.description,
                                        })
                                        .collect(),
                                    is_other: p.is_other,
                                    is_secret: p.is_secret,
                                })
                                .collect(),
                            state: QuestionState::Pending,
                            answers: BTreeMap::new(),
                        };
                        if store
                            .apply(&target.session, WorkUpdate::CaptureQuestion { question })
                            .is_ok()
                        {
                            output
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .dirty
                                .insert(target.session.clone());
                            notify.notify_one();
                        }
                    }
                }
                RuntimeEvent::QuestionResolved {
                    thread_id,
                    request_id,
                } => {
                    if let Some(target) = targets.get(&thread_id) {
                        if let Ok(record) = store.read(&target.session) {
                            for question in record.questions.iter().filter(|q| {
                                q.target == *target
                                    && q.request_id.as_ref() == Some(&request_id)
                                    && q.state == QuestionState::Pending
                            }) {
                                let _ = store.apply(
                                    &target.session,
                                    WorkUpdate::ResolveQuestion {
                                        question_id: question.id.clone(),
                                        state: QuestionState::Resolved,
                                    },
                                );
                            }
                            output
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .dirty
                                .insert(target.session.clone());
                            notify.notify_one();
                        }
                    }
                }
                RuntimeEvent::ThreadStatus { thread_id, status } => {
                    if let Some(snapshot) = output
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .snapshots
                        .get_mut(&thread_id)
                    {
                        snapshot.status = status;
                    }
                    notify.notify_one();
                }
                RuntimeEvent::TurnStarted { thread_id, turn_id } => {
                    if let Some(snapshot) = output
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .snapshots
                        .get_mut(&thread_id)
                    {
                        snapshot.status = RuntimeStatus::Working;
                        snapshot.active_turn = Some(turn_id);
                    }
                    notify.notify_one();
                }
                RuntimeEvent::TurnCompleted { thread_id, .. } => {
                    if let Some(snapshot) = output
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .snapshots
                        .get_mut(&thread_id)
                    {
                        snapshot.status = RuntimeStatus::Idle;
                        snapshot.active_turn = None;
                    }
                    if let Some(target) = targets.get(&thread_id) {
                        let _ = store.apply(
                            &target.session,
                            WorkUpdate::ExpirePendingQuestions {
                                target: Some(target.clone()),
                            },
                        );
                        output
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .dirty
                            .insert(target.session.clone());
                    }
                    notify.notify_one();
                }
                RuntimeEvent::Disconnected { .. } => disconnected = true,
                RuntimeEvent::Notice { .. } => {
                    output.lock().unwrap_or_else(|e| e.into_inner()).notice = Some(
                        "Codex event stream needs attention; ordinary Codex remains available"
                            .into(),
                    );
                    notify.notify_one();
                }
            }
        }
        if disconnected {
            controller = None;
            for target in targets.values() {
                let _ = store.apply(
                    &target.session,
                    WorkUpdate::ExpirePendingQuestions {
                        target: Some(target.clone()),
                    },
                );
            }
            let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
            out.connected = false;
            for snapshot in out.snapshots.values_mut() {
                snapshot.status = RuntimeStatus::Unknown;
                snapshot.active_turn = None;
            }
            out.dirty
                .extend(targets.values().map(|t| t.session.clone()));
            out.notice = Some("Codex disconnected; question and feedback drafts were kept".into());
            drop(out);
            notify.notify_one();
        }
    }
}
#[cfg(not(unix))]
fn run(
    config: FleetConfig,
    rx: mpsc::Receiver<ControlCommand>,
    output: Arc<Mutex<ControlOutput>>,
    notify: Arc<tokio::sync::Notify>,
) {
    let store = Store::new(config.state_root);
    while let Ok(command) = rx.recv() {
        match command {
            ControlCommand::Shutdown => break,
            ControlCommand::Select(_) => {}
            ControlCommand::Join(target) => {
                output.lock().unwrap_or_else(|e| e.into_inner()).receipt = Some(Receipt {
                    target,
                    message: "Live Codex questions are unavailable on this platform".into(),
                    cleared_text: None,
                });
                notify.notify_one();
            }
            ControlCommand::Answer(question) => {
                output.lock().unwrap_or_else(|e| e.into_inner()).receipt = Some(Receipt {
                    target: question.target,
                    message: "Live answers are unavailable on this platform; draft kept".into(),
                    cleared_text: None,
                });
                notify.notify_one();
            }
            ControlCommand::Send {
                target,
                text,
                mode,
                expected_turn,
            } => {
                let (message, cleared_text) = persist_send(
                    &store,
                    &target,
                    &text,
                    mode,
                    expected_turn,
                    None,
                    |_| {},
                    |_| DeliveryReceipt {
                        state: ReceiptState::Unavailable,
                        message: "Live Codex control is unavailable on this platform".into(),
                        thread_id: None,
                        turn_id: None,
                        queue_id: None,
                    },
                );
                let mut out = output.lock().unwrap_or_else(|e| e.into_inner());
                out.inflight_id = None;
                out.receipt = Some(Receipt {
                    target,
                    message,
                    cleared_text,
                });
                drop(out);
                notify.notify_one();
            }
        }
    }
}
pub(super) fn status_word(status: &RuntimeStatus) -> &'static str {
    match status {
        RuntimeStatus::Working => "Working",
        RuntimeStatus::WaitingForAnswer => "Waiting for your answer",
        RuntimeStatus::WaitingForApproval => "Waiting for approval in Codex",
        RuntimeStatus::Idle => "Idle in Codex",
        RuntimeStatus::NotLoaded => "Not loaded in Codex",
        RuntimeStatus::Error => "Codex reported a problem",
        RuntimeStatus::Unknown => "Live status unknown",
    }
}
