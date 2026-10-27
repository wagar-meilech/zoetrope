//! Native fleet frontend. Drawing and input consume cached data; local IO lives
//! on workers. The public key/render interface is shared by the terminal and tests.
mod control_worker;
mod document_worker;
mod editor;
mod presentation;
pub mod runtime;
mod state_worker;
mod work_view;
use crate::console::AgentKey;
use crate::control::ThreadSnapshot;
use crate::workstate::FeedbackMode;
use crate::workstate::{
    CriterionInput, CriterionStatus, Evidence, EvidenceKind, QuestionRecord, QuestionState,
    WorkUpdate,
};
use control_worker::{ControlCommand, ControlWorker};
use document_worker::DocumentWorker;
use editor::Editor;
use state_worker::{StateCommand, StateWorker};

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use crate::console::{
    Console, ConsoleCommand, DiscoveryRoots, FleetGraph, FleetNodeId, SessionKey,
};
use crate::documents::DocumentLauncher;
use crate::workstate::{Store, WorkRecord, WorkSummary};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use rataflow::{
    Edge, Flow, Handle, HandlePosition, Node, NodeContent, NodeRenderContext, Reconnectable,
    StepEdge,
};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Native locations and external adapters. Construction does no local IO.
#[derive(Clone)]
pub struct FleetConfig {
    pub roots: DiscoveryRoots,
    pub state_root: PathBuf,
    pub socket: PathBuf,
    pub documents: DocumentLauncher,
}
impl Default for FleetConfig {
    fn default() -> Self {
        Self {
            roots: DiscoveryRoots::local(),
            state_root: Store::default_root()
                .unwrap_or_else(|_| std::env::temp_dir().join("zoetrope-state")),
            socket: crate::control::default_socket(),
            documents: DocumentLauncher::default(),
        }
    }
}
#[derive(Default)]
struct InventoryView {
    ready: bool,
    graph: FleetGraph,
    query: String,
    summaries: BTreeMap<SessionKey, WorkSummary>,
    versions: BTreeMap<SessionKey, Option<(u64, SystemTime)>>,
    agent_names: BTreeMap<AgentKey, String>,
    activity: Option<(AgentKey, Option<crate::console::RecordedActivity>)>,
    sources: BTreeMap<SessionKey, PathBuf>,
    selection: Option<FleetNodeId>,
    errors: Vec<String>,
}
enum InventoryCommand {
    Navigate(ConsoleCommand),
    Activity(AgentKey),
    Live(BTreeMap<String, crate::console::LiveThread>),
    Reload,
    Shutdown,
}
struct Inventory {
    tx: mpsc::SyncSender<InventoryCommand>,
    latest: Arc<Mutex<Option<InventoryView>>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Inventory {
    fn start(config: &FleetConfig, notify: Arc<tokio::sync::Notify>) -> Self {
        let (tx, rx) = mpsc::sync_channel(32);
        let latest = Arc::new(Mutex::new(None));
        let output = latest.clone();
        let roots = config.roots.clone();
        let root = config.state_root.clone();
        let worker = thread::spawn(move || {
            let mut console = Console::with_state(roots, root);
            let mut refresh = Instant::now();
            let mut query = String::new();
            let mut activity_target: Option<AgentKey> = None;
            loop {
                if Instant::now() >= refresh {
                    console.refresh(SystemTime::now());
                    refresh = Instant::now() + Duration::from_secs(5);
                }
                if console.selection().is_none()
                    && let Some(node) = console
                        .graph()
                        .nodes
                        .iter()
                        .find(|n| matches!(n.id, FleetNodeId::Session(_)))
                {
                    let id = node.id.clone();
                    console.command(ConsoleCommand::Select(id));
                }
                let mut summaries = BTreeMap::new();
                for node in &console.graph().nodes {
                    if let Some(key) = session_key(&node.id)
                        && let Some(summary) = console.work_summary(key)
                    {
                        summaries.insert(key.clone(), summary.clone());
                    }
                }
                let activity = activity_target
                    .clone()
                    .or_else(|| match console.selection()? {
                        FleetNodeId::Session(session) => Some(AgentKey {
                            session: session.clone(),
                            agent_id: "main".into(),
                        }),
                        FleetNodeId::Agent(agent) => Some(agent.clone()),
                        _ => None,
                    })
                    .map(|target| {
                        let activity = console.recorded_activity(&target);
                        (target, activity)
                    });
                let view = InventoryView {
                    ready: true,
                    agent_names: console.agent_names(),
                    activity,
                    graph: console.graph().clone(),
                    query: query.clone(),
                    summaries,
                    versions: console.work_versions().clone(),
                    sources: console.session_sources(),
                    selection: console.selection().cloned(),
                    errors: console.work_errors().to_vec(),
                };
                *output.lock().unwrap_or_else(|e| e.into_inner()) = Some(view);
                notify.notify_one();
                match rx.recv_timeout(refresh.saturating_duration_since(Instant::now())) {
                    Ok(InventoryCommand::Navigate(command)) => {
                        if let ConsoleCommand::Search(text) = &command {
                            query = text.clone();
                        }
                        console.command(command);
                    }
                    Ok(InventoryCommand::Activity(target)) => activity_target = Some(target),
                    Ok(InventoryCommand::Reload) => refresh = Instant::now(),
                    Ok(InventoryCommand::Live(threads)) => console.set_live_threads(threads),
                    Ok(InventoryCommand::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            }
        });
        Self {
            tx,
            latest,
            thread: Some(worker),
        }
    }
}
fn session_key(id: &FleetNodeId) -> Option<&SessionKey> {
    match id {
        FleetNodeId::Session(key) => Some(key),
        FleetNodeId::Agent(agent) => Some(&agent.session),
        _ => None,
    }
}
fn flow_id(id: &FleetNodeId) -> String {
    serde_json::to_string(id).expect("fleet identity serializes")
}
#[derive(Debug, Clone, PartialEq)]
struct Card {
    title: String,
    detail: String,
    progress: String,
    bar: String,
    activity: String,
}
impl NodeContent for Card {
    fn render(&self, ctx: &NodeRenderContext, buf: &mut Buffer) {
        let color = if ctx.selected {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let mut lines = crate::ui::wrap(&self.title, ctx.area.width.saturating_sub(2) as usize, 2);
        if !self.detail.is_empty() {
            lines.push(self.detail.clone());
        }
        if !self.progress.is_empty() {
            lines.push(self.progress.clone());
        }
        if !self.bar.is_empty() {
            lines.push(self.bar.clone());
        }
        if !self.activity.is_empty() {
            lines.extend(crate::ui::wrap(
                &self.activity,
                ctx.area.width.saturating_sub(2) as usize,
                2,
            ));
        }
        Paragraph::new(lines.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color)),
            )
            .wrap(Wrap { trim: true })
            .render(ctx.area, buf);
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Pane {
    Graph,
    Commands,
    Feedback,
    Criteria,
    Questions,
    Documents,
    Deliveries,
}
enum EditKind {
    Feedback,
    Goal(SessionKey),
    Evidence {
        key: SessionKey,
        id: String,
        revision: u64,
    },
    Answer {
        key: SessionKey,
        question: String,
        prompt: String,
    },
}

struct PendingSave {
    id: u64,
    text: String,
}

/// Cached terminal state. All inputs are public crossterm events; `poll` applies
/// worker results, `draw` does no filesystem/socket/editor work.
pub struct FleetConsole {
    inventory: Inventory,
    state: StateWorker,
    state_root: PathBuf,
    plan_index: usize,
    control: ControlWorker,
    documents: DocumentWorker,
    document_pending: bool,
    snapshots: BTreeMap<String, ThreadSnapshot>,
    control_connected: bool,
    control_pending: bool,
    inflight_id: Option<String>,
    selected_work: Option<WorkRecord>,
    work_selection: Option<work_view::Selection>,
    work_mode: bool,
    activity_request: Option<AgentKey>,
    replay_requested: bool,
    work_reload: Option<SessionKey>,
    edit: Option<Editor>,
    feedback_target: Option<AgentKey>,
    feedback_open: bool,
    feedback_requested: bool,
    pane: Pane,
    edit_kind: Option<EditKind>,
    item_index: usize,
    prompt_index: usize,
    pending_feedback: Option<String>,
    scroll: u16,
    query: String,
    searching: bool,
    dirty_since: Option<Instant>,
    pending_save: Option<PendingSave>,
    save_sequence: u64,
    close_after_save: bool,
    quit_after_save: bool,
    notify: Arc<tokio::sync::Notify>,
    view: InventoryView,
    flow: Flow<Card, StepEdge>,
    positions: BTreeMap<FleetNodeId, (f64, f64)>,
    history: bool,
    pending_center: bool,
    show_help: bool,
    help_scroll: u16,
    canvas_area: Rect,
    progress_area: Option<Rect>,
    action_areas: Vec<(Rect, presentation::Action)>,
    requested_selection: Option<FleetNodeId>,
    expanding: Option<(SessionKey, bool)>,
    pub(crate) quit: bool,
    status: String,
}
impl FleetConsole {
    pub fn new(mut config: FleetConfig) -> Self {
        let root_error = if config.state_root.is_relative() {
            match std::env::current_dir() {
                Ok(cwd) => {
                    config.state_root = cwd.join(&config.state_root);
                    None
                }
                Err(error) => Some(format!("Cannot resolve state folder: {error}")),
            }
        } else {
            None
        };
        let notify = Arc::new(tokio::sync::Notify::new());
        let inventory = Inventory::start(&config, notify.clone());
        let state = StateWorker::start(config.state_root.clone(), notify.clone());
        let control = ControlWorker::start(config.clone(), notify.clone());
        let documents = DocumentWorker::start(config.documents.clone(), notify.clone());
        Self {
            inventory,
            state,
            state_root: config.state_root.clone(),
            plan_index: 0,
            control,
            documents,
            document_pending: false,
            snapshots: BTreeMap::new(),
            control_connected: false,
            control_pending: false,
            inflight_id: None,
            selected_work: None,
            work_selection: None,
            work_mode: true,
            activity_request: None,
            replay_requested: false,
            work_reload: None,
            edit: None,
            feedback_target: None,
            feedback_open: false,
            feedback_requested: false,
            pane: Pane::Graph,
            edit_kind: None,
            item_index: 0,
            prompt_index: 0,
            pending_feedback: None,
            scroll: 0,
            query: String::new(),
            searching: false,
            dirty_since: None,
            pending_save: None,
            save_sequence: 0,
            close_after_save: false,
            quit_after_save: false,
            notify,
            view: InventoryView::default(),
            flow: Flow::new()
                .with_deselect_on_pane_click(false)
                .with_min_zoom(0.1),
            positions: BTreeMap::new(),
            history: false,
            pending_center: true,
            show_help: false,
            help_scroll: 0,
            canvas_area: Rect::default(),
            progress_area: None,
            action_areas: Vec::new(),
            requested_selection: None,
            expanding: None,
            quit: false,
            status: root_error.unwrap_or_else(|| "Reading local work…".into()),
        }
    }
    /// Apply cached worker output. Returns whether the display changed.
    pub fn poll(&mut self) -> bool {
        if self
            .dirty_since
            .is_some_and(|at| at.elapsed() >= Duration::from_millis(500))
        {
            self.save_feedback();
        }
        let mut changed = false;
        let view = self
            .inventory
            .latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(mut view) = view {
            if let Some(requested) = self.requested_selection.as_ref() {
                if view.selection.as_ref() == Some(requested) {
                    self.requested_selection = None;
                } else {
                    view.selection = Some(requested.clone());
                }
            }
            let selection_changed = self.view.selection != view.selection;
            let work_changed = view
                .selection
                .as_ref()
                .and_then(session_key)
                .is_some_and(|key| self.view.versions.get(key) != view.versions.get(key));
            changed = self.view.activity != view.activity
                || self.view.agent_names != view.agent_names
                || self.view.ready != view.ready
                || self.view.query != view.query
                || self.view.graph != view.graph
                || self.view.summaries != view.summaries
                || selection_changed
                || self.view.errors != view.errors;
            self.view = view;
            if let Some((key, expanded)) = &self.expanding
                && self
                    .view
                    .graph
                    .nodes
                    .iter()
                    .any(|n| n.id == FleetNodeId::Session(key.clone()) && n.expanded == *expanded)
            {
                self.status = if *expanded {
                    "Agents expanded".into()
                } else {
                    "Goal collapsed".into()
                };
                self.expanding = None;
                changed = true;
            }
            if selection_changed || work_changed || self.selected_work.is_none() {
                if let Some(key) = self.view.selection.as_ref().and_then(session_key) {
                    self.work_reload = Some(key.clone());
                }
                if selection_changed {
                    self.work_selection = None;
                    self.selected_work = None;
                    self.pending_center = true;
                    self.select_recipient(self.selected_agent());
                }
            }
            if changed {
                if let Some(error) = self.view.errors.first() {
                    self.status = error.clone();
                } else if self.status == "Reading local work…" {
                    self.status = "Local monitoring · no model calls".into();
                }
                self.sync_graph();
            }
        }
        let output =
            std::mem::take(&mut *self.state.output.lock().unwrap_or_else(|e| e.into_inner()));
        let mut kept_editor = false;
        if let Some((key, record)) = output.record
            && self.view.selection.as_ref().and_then(session_key) == Some(&key)
        {
            if self.selected_work.as_ref() != Some(&record) {
                changed = true;
            }
            if let Some(id) = self.pending_feedback.as_ref()
                && let Some(item) = record.outbox.iter().find(|item| &item.id == id)
            {
                // Copy is local and durable in the outbox. Its late
                // completion must never replace a composer opened or
                // edited while the disk worker was busy.
                if self.edit.is_none() {
                    self.feedback_target = Some(item.target.clone());
                    self.select_recipient(Some(item.target.clone()));
                    self.feedback_open = true;
                    self.pane = Pane::Feedback;
                    self.edit_kind = Some(EditKind::Feedback);
                    self.edit = Some(Editor::new(item.text.clone()));
                    self.dirty_since = Some(Instant::now());
                } else {
                    kept_editor = true;
                }
                self.pending_feedback = None;
            }
            self.view.summaries.insert(key.clone(), record.summary());
            if let Some(goal) = &record.goal
                && let Some(node) = self
                    .view
                    .graph
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == FleetNodeId::Session(key.clone()))
            {
                node.label = goal.title.clone();
            }
            let previous_target = self.control_target();
            self.selected_work = Some(record);
            if self.control_target() != previous_target {
                self.select_recipient(self.control_target());
            }
            if self.feedback_requested {
                self.open_feedback();
            }
            self.sync_graph();
        }
        if let Some(status) = output.status {
            self.status = if kept_editor {
                "Answer copied locally; your current draft was kept".into()
            } else {
                status
            };
            changed = true;
            let _ = self.inventory.tx.try_send(InventoryCommand::Reload);
        }
        if let Some((id, result)) = output.saved
            && self.pending_save.as_ref().is_some_and(|save| save.id == id)
        {
            let saved = self.pending_save.take().unwrap();
            match result {
                Ok(()) => {
                    let unchanged = self.edit.as_ref().is_some_and(|e| e.text == saved.text);
                    if unchanged && self.close_after_save {
                        self.close_editor();
                        if self.quit_after_save {
                            self.quit = true;
                        }
                    } else if !unchanged {
                        if self.draft_update().is_some() {
                            self.dirty_since = Some(Instant::now());
                            if self.close_after_save {
                                self.save_feedback();
                            }
                        } else if self.close_after_save {
                            self.commit_edit();
                        }
                    }
                }
                Err(error) => {
                    self.dirty_since = None;
                    self.quit_after_save = false;
                    self.status =
                        format!("Could not save draft: {error}. Draft retained; Esc retries.");
                }
            }
            changed = true;
        }
        let mut native = self
            .control
            .output
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if self.snapshots != native.snapshots || self.control_connected != native.connected {
            self.snapshots = native.snapshots.clone();
            self.control_connected = native.connected;
            changed = true;
            let live = self
                .snapshots
                .iter()
                .filter_map(|(id, snapshot)| {
                    use crate::console::{LiveActivity, LiveThread};
                    use crate::control::RuntimeStatus;
                    let activity = match snapshot.status {
                        RuntimeStatus::Working => LiveActivity::Working,
                        RuntimeStatus::WaitingForAnswer => LiveActivity::WaitingForAnswer,
                        RuntimeStatus::WaitingForApproval => LiveActivity::WaitingForApproval,
                        _ => return None,
                    };
                    Some((
                        id.clone(),
                        LiveThread {
                            name: snapshot.name.clone(),
                            project: snapshot.cwd.clone(),
                            activity,
                        },
                    ))
                })
                .collect();
            let _ = self.inventory.tx.try_send(InventoryCommand::Live(live));
        }
        let dirty = std::mem::take(&mut native.dirty);
        let notice = native.notice.take();
        let receipt = native.receipt.take();
        if self.inflight_id != native.inflight_id {
            self.inflight_id = native.inflight_id.clone();
            changed = true;
        }
        drop(native);
        if let Some(message) = notice {
            self.status = message;
            changed = true;
        }
        if let Some(key) = self.view.selection.as_ref().and_then(session_key)
            && dirty.contains(key)
        {
            self.work_reload = Some(key.clone());
        }
        if changed {
            self.sync_graph();
        }
        if let Some(receipt) = receipt {
            self.control_pending = false;
            if let Some(text) = receipt.cleared_text.as_ref() {
                if self.feedback_target.as_ref() == Some(&receipt.target)
                    && matches!(self.edit_kind, Some(EditKind::Feedback))
                    && let Some(editor) = self.edit.as_mut().filter(|e| &e.text == text)
                {
                    *editor = Editor::new(String::new());
                    self.dirty_since = None;
                }
                if let Some(work) = self.selected_work.as_mut() {
                    for draft in &mut work.drafts {
                        if draft.target == receipt.target && &draft.text == text {
                            draft.text.clear();
                        }
                    }
                }
            }
            self.status = receipt.message;
            if self.view.selection.as_ref().and_then(session_key) == Some(&receipt.target.session) {
                self.work_reload = Some(receipt.target.session);
            }
            changed = true;
        }
        if let Some(key) = self.work_reload.as_ref()
            && self
                .state
                .tx
                .try_send(StateCommand::Load(key.clone()))
                .is_ok()
        {
            self.work_reload = None;
        }
        if let Some(message) = self
            .documents
            .output
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            self.status = message;
            self.document_pending = false;
            changed = true;
        }
        if let Some(target) = self.activity_request.as_ref()
            && self
                .inventory
                .tx
                .try_send(InventoryCommand::Activity(target.clone()))
                .is_ok()
        {
            self.activity_request = None;
        }
        changed
    }
    fn deliveries_text(&self) -> String {
        use crate::workstate::{DeliveryState as S, FeedbackMode as M};
        let mut text = String::from(
            "Saved deliveries\nDelivery records confirm receipt, not completed work.\n",
        );
        if let Some(work) = &self.selected_work {
            for item in work.outbox.iter().rev().take(32) {
                let mode = match item.mode {
                    M::Guidance => "Guidance",
                    M::Queue => "Queued instruction",
                    M::Stop => "Stop request",
                    M::Answer => "Answer",
                    M::CriteriaRequest => "Criteria request",
                };
                let state = if self.inflight_id.as_ref() == Some(&item.id) {
                    "Sending"
                } else {
                    match item.state {
                        S::Draft => "Saved locally",
                        S::Sending | S::Unknown => "Delivery unknown",
                        S::Queued => "Queued by Codex",
                        S::Accepted => "Accepted by Codex",
                        S::Submitted => "Submitted; awaiting resolution",
                        S::Rejected => "Rejected by Codex",
                        S::Unavailable => "Unavailable",
                    }
                };
                text.push_str(&format!(
                    "\n{mode} to {} · {state}\n",
                    self.producer_label(&item.target)
                ));
                if item.mode != M::Answer {
                    text.push_str(&crate::ui::truncate(
                        item.text.lines().next().unwrap_or_default(),
                        160,
                    ));
                    text.push('\n');
                }
            }
            if work.outbox.is_empty() {
                text.push_str("\nNo deliveries yet. Write feedback with f.");
            }
        }
        text
    }
    fn recipient_label(&self) -> String {
        if self.pane == Pane::Feedback
            && let Some(target) = &self.feedback_target
        {
            if target.agent_id == "main" {
                return "Coordinator".into();
            }
            if let Some(node) = self
                .view
                .graph
                .nodes
                .iter()
                .find(|n| n.id == FleetNodeId::Agent(target.clone()))
            {
                return if node.label == target.agent_id {
                    self.work_owner_label(target)
                } else {
                    node.label.clone()
                };
            }
            return self
                .snapshots
                .get(&target.agent_id)
                .and_then(|s| s.name.clone())
                .unwrap_or_else(|| self.work_owner_label(target));
        }
        if self.work_selection.is_some() {
            return self
                .selected_work_owner()
                .map(|owner| self.work_owner_label(&owner))
                .unwrap_or_else(|| "Selected work unavailable".into());
        }
        if matches!(self.view.selection, Some(FleetNodeId::Session(_))) {
            return "Coordinator".into();
        }
        self.view
            .selection
            .as_ref()
            .and_then(|id| self.flow.node(&flow_id(id)))
            .map(|n| n.content.title.clone())
            .unwrap_or_else(|| "No agent selected".into())
    }
    fn select_recipient(&mut self, target: Option<AgentKey>) {
        self.activity_request = target.clone();
        self.control.select(target);
    }
    fn control_target(&self) -> Option<AgentKey> {
        self.feedback_target
            .clone()
            .filter(|_| self.pane == Pane::Feedback)
            .or_else(|| self.selected_agent())
    }
    fn runtime_snapshot(&self) -> Option<&ThreadSnapshot> {
        let target = self.control_target()?;
        let id = if target.agent_id == "main" {
            &target.session.session_id
        } else {
            &target.agent_id
        };
        self.snapshots.get(id)
    }
    fn runtime_word(&self) -> &str {
        if self
            .selected_agent()
            .is_some_and(|agent| agent.session.provider == crate::provider::Provider::Claude)
        {
            return "Transcript viewing only; live controls unavailable for this provider";
        }
        self.runtime_snapshot()
            .map(|s| control_worker::status_word(&s.status))
            .unwrap_or(if self.control_connected {
                "Checking Codex…"
            } else {
                "Codex controls unavailable"
            })
    }
    fn control_availability(&self, mode: FeedbackMode) -> Result<&'static str, &'static str> {
        use crate::control::RuntimeStatus;
        let target = self.control_target().ok_or("select a goal or agent")?;
        if target.session.provider != crate::provider::Provider::Codex {
            return Err("transcript viewing only");
        }
        if !self.control_connected {
            return Err("Codex disconnected");
        }
        if self.control_pending {
            return Err("waiting for the previous action");
        }
        let snapshot = self.runtime_snapshot().ok_or("checking selected agent")?;
        if snapshot.status == RuntimeStatus::NotLoaded {
            return Err("not loaded in Codex");
        }
        if matches!(
            snapshot.status,
            RuntimeStatus::Unknown | RuntimeStatus::Error
        ) {
            return Err("runtime state unavailable");
        }
        if mode == FeedbackMode::Stop {
            return if snapshot.active_turn.is_some() {
                Ok("current turn")
            } else {
                Err("no active turn")
            };
        }
        if snapshot.can_accept_direct_input == Some(false) {
            return Err("agent does not accept direct input");
        }
        if mode == FeedbackMode::Queue {
            return Ok(if snapshot.status == RuntimeStatus::Idle {
                "idle work may start immediately"
            } else {
                "native queue"
            });
        }
        if snapshot.active_turn.is_some() {
            Ok("current turn")
        } else if snapshot.status == RuntimeStatus::Idle {
            Ok("new input starts idle work")
        } else {
            Err("active turn unavailable")
        }
    }
    fn steering_text(&self) -> String {
        [
            ("Guidance", FeedbackMode::Guidance),
            ("Queue", FeedbackMode::Queue),
            ("Stop", FeedbackMode::Stop),
        ]
        .into_iter()
        .map(|(name, mode)| match self.control_availability(mode) {
            Ok(reason) => format!("{name} available: {reason}"),
            Err(reason) => format!("{name} unavailable: {reason}"),
        })
        .collect::<Vec<_>>()
        .join("\n")
    }
    fn send_feedback(&mut self, mode: FeedbackMode) {
        if self.control_pending {
            self.status = "A control action is still waiting for Codex".into();
            return;
        }
        let Some(target) = self.control_target() else {
            self.status = "Select a goal or agent first".into();
            return;
        };
        if let Err(reason) = self.control_availability(mode) {
            let name = match mode {
                FeedbackMode::Stop => "Stop",
                FeedbackMode::Queue => "Queue",
                _ => "Guidance",
            };
            self.status = format!("{name} unavailable: {reason}; draft kept");
            return;
        }
        let text = self
            .edit
            .as_ref()
            .filter(|_| matches!(self.edit_kind, Some(EditKind::Feedback)))
            .map(|e| e.text.clone())
            .or_else(|| {
                self.selected_work
                    .as_ref()
                    .and_then(|r| r.drafts.iter().find(|d| d.target == target))
                    .map(|d| d.text.clone())
            })
            .unwrap_or_default();
        if mode != FeedbackMode::Stop && text.trim().is_empty() {
            self.status = "Write feedback before sending it".into();
            return;
        }
        let expected_turn = self.runtime_snapshot().and_then(|s| s.active_turn.clone());
        self.save_feedback();
        if self
            .control
            .tx
            .try_send(ControlCommand::Send {
                target,
                text,
                mode,
                expected_turn,
            })
            .is_ok()
        {
            self.control_pending = true;
            self.status = "Sending to the selected agent…".into();
        } else {
            self.status = "Control queue is busy; nothing was sent".into();
        }
    }
    fn selected_agent(&self) -> Option<AgentKey> {
        if self.work_selection.is_some() {
            return self.selected_work_owner();
        }
        match self.view.selection.as_ref()? {
            FleetNodeId::Session(key) => Some(AgentKey {
                session: key.clone(),
                agent_id: "main".into(),
            }),
            FleetNodeId::Agent(agent) => Some(agent.clone()),
            _ => None,
        }
    }
    fn selected_label(&self) -> String {
        self.selected_work
            .as_ref()
            .and_then(|r| r.goal.as_ref())
            .map(|g| g.title.clone())
            .or_else(|| {
                self.view
                    .selection
                    .as_ref()
                    .and_then(|id| self.view.graph.nodes.iter().find(|n| &n.id == id))
                    .and_then(|n| self.flow.node(&flow_id(&n.id)))
                    .map(|n| n.content.title.clone())
            })
            .unwrap_or_else(|| "Select a goal".into())
    }
    fn draft_update(&self) -> Option<(SessionKey, WorkUpdate, &'static str)> {
        let edit = self.edit.as_ref()?;
        match self.edit_kind.as_ref()? {
            EditKind::Answer {
                key,
                question,
                prompt,
            } => {
                let mut answers = self
                    .selected_work
                    .as_ref()
                    .and_then(|r| r.questions.iter().find(|q| &q.id == question))
                    .map(|q| q.answers.clone())
                    .unwrap_or_default();
                answers.insert(prompt.clone(), vec![edit.text.clone()]);
                Some((
                    key.clone(),
                    WorkUpdate::SaveAnswer {
                        question_id: question.clone(),
                        answers,
                    },
                    "Answer draft saved; not submitted",
                ))
            }
            EditKind::Feedback => {
                let target = self.feedback_target.clone()?;
                Some((
                    target.session.clone(),
                    WorkUpdate::SaveDraft {
                        target,
                        text: edit.text.clone(),
                    },
                    "Draft saved",
                ))
            }
            _ => None,
        }
    }
    fn save_feedback(&mut self) {
        if self.pending_save.is_some() {
            return;
        }
        let Some((key, update, message)) = self.draft_update() else {
            return;
        };
        self.save_editor(key, update, message);
    }
    fn save_editor(&mut self, key: SessionKey, update: WorkUpdate, message: &str) -> bool {
        if self.pending_save.is_some() {
            return false;
        }
        self.save_sequence += 1;
        let id = self.save_sequence;
        if self
            .state
            .tx
            .try_send(StateCommand::Update(
                key,
                Box::new(update),
                message.into(),
                Some(id),
            ))
            .is_ok()
        {
            self.pending_save = Some(PendingSave {
                id,
                text: self.edit.as_ref().unwrap().text.clone(),
            });
            self.dirty_since = None;
            self.status = "Saving draft… waiting for local storage".into();
            true
        } else {
            if self.draft_update().is_some() {
                self.dirty_since = Some(Instant::now());
            }
            self.status = "Save queue busy; edit remains in the composer. Try saving again.".into();
            false
        }
    }
    fn close_editor(&mut self) {
        let answer = matches!(self.edit_kind, Some(EditKind::Answer { .. }));
        let criteria = matches!(
            self.edit_kind,
            Some(EditKind::Goal(_) | EditKind::Evidence { .. })
        );
        self.edit = None;
        self.edit_kind = None;
        self.feedback_open = false;
        self.dirty_since = None;
        self.close_after_save = false;
        if answer {
            self.pane = Pane::Questions;
        } else if criteria {
            self.pane = Pane::Criteria;
        }
    }
    fn request_close_editor(&mut self) {
        if self.pending_save.is_some() {
            self.close_after_save = true;
        } else if self.draft_update().is_some() {
            self.close_after_save = true;
            self.save_feedback();
        } else {
            self.close_editor();
        }
    }
    fn open_feedback(&mut self) {
        if self.selected_work.is_none() {
            self.feedback_requested = true;
            self.status = "Loading saved work; feedback will open shortly".into();
            return;
        }
        self.feedback_requested = false;
        let target = if self.pane == Pane::Questions {
            self.current_question().map(|q| q.target.clone())
        } else {
            self.selected_agent()
        };
        if let Some(target) = target {
            let text = self
                .selected_work
                .as_ref()
                .and_then(|r| r.drafts.iter().find(|d| d.target == target))
                .map(|d| d.text.clone())
                .unwrap_or_default();
            self.feedback_target = Some(target);
            self.feedback_open = true;
            self.pane = Pane::Feedback;
            self.edit_kind = Some(EditKind::Feedback);
            self.edit = Some(Editor::new(text));
            self.status = "Editing feedback · Esc saves draft".into();
        }
    }
    fn update_state(&mut self, key: SessionKey, update: WorkUpdate, message: &str) -> bool {
        if self
            .state
            .tx
            .try_send(StateCommand::Update(
                key,
                Box::new(update),
                message.into(),
                None,
            ))
            .is_ok()
        {
            true
        } else {
            self.status = "Local save queue is busy; keep this edit and try again".into();
            false
        }
    }
    fn edit_goal(&mut self) {
        if let Some(target) = self.selected_agent() {
            let mut text = self.selected_label();
            if let Some(goal) = self.selected_work.as_ref().and_then(|r| r.goal.as_ref()) {
                for criterion in &goal.criteria {
                    text.push('\n');
                    text.push_str(&criterion.label);
                }
            }
            self.edit = Some(Editor::new(text));
            self.edit_kind = Some(EditKind::Goal(target.session));
        }
    }
    fn edit_evidence(&mut self) {
        if let Some(record) = &self.selected_work
            && let Some(goal) = &record.goal
            && let Some(criterion) = goal.criteria.get(self.item_index)
        {
            self.edit_kind = Some(EditKind::Evidence {
                key: record.session.clone(),
                id: criterion.id.clone(),
                revision: goal.revision,
            });
            self.edit = Some(Editor::new(String::new()));
        }
    }
    fn commit_edit(&mut self) {
        let Some(editor) = self.edit.as_ref() else {
            return;
        };
        let text = editor.text.clone();
        match self.edit_kind.as_ref() {
            Some(EditKind::Answer { .. }) => {
                // F2 remains Save while editing. Closing explicitly with Esc
                // keeps an ordinary held-key Press from becoming submission.
                self.save_feedback();
                return;
            }
            Some(EditKind::Goal(key)) => {
                let mut lines = text.lines().map(str::trim).filter(|s| !s.is_empty());
                let title = lines.next().unwrap_or("").to_string();
                let old = self.selected_work.as_ref().and_then(|r| r.goal.as_ref());
                let mut used = BTreeSet::new();
                let criteria: Vec<_> = lines
                    .enumerate()
                    .map(|(at, label)| {
                        let same = old.and_then(|g| {
                            g.criteria
                                .iter()
                                .find(|c| c.label == label && !used.contains(&c.id))
                        });
                        let (id, weight) =
                            same.map(|c| (c.id.clone(), c.weight)).unwrap_or_else(|| {
                                (
                                    format!(
                                        "outcome-{}-{at}",
                                        SystemTime::now()
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_nanos()
                                    ),
                                    1.0,
                                )
                            });
                        used.insert(id.clone());
                        CriterionInput {
                            id,
                            label: label.into(),
                            weight,
                        }
                    })
                    .collect();
                if title.is_empty() || criteria.is_empty() {
                    self.status = "Enter a goal title and at least one completion outcome".into();
                    return;
                }
                self.save_editor(
                    key.clone(),
                    WorkUpdate::SetGoal {
                        title,
                        criteria,
                        reason: "Operator edited completion criteria".into(),
                        provenance: "Operator in Zoetrope".into(),
                    },
                    "Goal criteria saved",
                )
            }
            Some(EditKind::Evidence { key, id, revision }) => {
                if text.trim().is_empty() {
                    self.status =
                        "Explain the evidence or your attestation before verifying".into();
                    return;
                }
                self.save_editor(
                    key.clone(),
                    WorkUpdate::Verify {
                        id: id.clone(),
                        evidence: vec![Evidence {
                            kind: EvidenceKind::UserAttestation,
                            detail: text.trim().into(),
                            provenance: "Operator in Zoetrope".into(),
                        }],
                        revision: Some(*revision),
                    },
                    "Verification saved",
                )
            }
            _ => return,
        };
        self.close_after_save = true;
    }
    fn reopen_criterion(&mut self) {
        let update = self.selected_work.as_ref().and_then(|record| {
            record.goal.as_ref().and_then(|goal| {
                goal.criteria.get(self.item_index).map(|criterion| {
                    (
                        record.session.clone(),
                        WorkUpdate::SetCriterionStatus {
                            id: criterion.id.clone(),
                            status: CriterionStatus::Pending,
                            revision: Some(goal.revision),
                            reason: "Reopened by operator".into(),
                        },
                    )
                })
            })
        });
        if let Some((key, update)) = update {
            self.update_state(key, update, "Outcome reopened; prior verification retained");
        }
    }
    fn criteria_text(&self) -> String {
        let Some(record) = &self.selected_work else {
            return "Loading selected goal…".into();
        };
        let mut text = String::new();
        if let Some(goal) = &record.goal {
            for (at, c) in goal.criteria.iter().enumerate() {
                text.push_str(&format!(
                    "{} {} {}\n",
                    if at == self.item_index { "›" } else { " " },
                    if c.status == CriterionStatus::Verified {
                        "✓"
                    } else {
                        "○"
                    },
                    c.label
                ));
                for evidence in &c.evidence {
                    text.push_str(&format!("  Evidence: {}\n", evidence.detail));
                }
            }
            text.push_str(&format!(
                "\nScope revision {} · {} earlier scopes\n",
                goal.revision,
                record.scope_history.len()
            ));
        } else {
            text.push_str("Progress not tracked. Press e to add outcomes.\n");
        }
        if !record.criterion_history.is_empty() {
            text.push_str("Prior verification retained in outcome history.\n");
        }
        text.push_str(&format!(
            "\nCheckpoints: {} (preference only)\n",
            if record.checkpoints {
                "requested"
            } else {
                "off"
            }
        ));
        if let Some(plan) = record.plan_drafts.get(self.plan_index) {
            text.push_str(&format!(
                "\nDraft plan (unverified) {} of {}\nProposed by {}\n",
                self.plan_index + 1,
                record.plan_drafts.len(),
                self.producer_label(&plan.producer)
            ));
            for step in &plan.steps {
                text.push_str(&format!("  {}\n", step.step));
            }
            text.push_str("[ / ] choose plan · a adopt as pending outcomes\n");
        }
        text.push_str("\ne edit · v verify · u reopen · p checkpoints\nA prepare criteria request · PageUp/PageDown scroll");
        text
    }

    fn listen_questions(&mut self) {
        if self.control_pending {
            return;
        }
        if let Some(target) = self.selected_agent()
            && self
                .control
                .tx
                .try_send(ControlCommand::Join(target))
                .is_ok()
        {
            self.control_pending = true;
            self.status = "Checking current native questions…".into();
        }
    }
    fn submit_question(&mut self) {
        if self.control_pending {
            return;
        }
        let Some(question) = self.current_question().cloned() else {
            return;
        };
        if question.state != QuestionState::Pending
            || question.request_id.is_none()
            || question.turn_id.is_none()
        {
            self.status="Original question is no longer confirmed pending; keep the draft or copy it to feedback".into();
            return;
        }
        if question.prompts.iter().any(|p| {
            !question.answers.get(&p.id).is_some_and(|answers| {
                !answers.is_empty() && answers.iter().all(|a| !a.trim().is_empty())
            })
        }) {
            self.status =
                "Choose or write an answer for every prompt; no defaults were submitted".into();
            return;
        }
        if self
            .control
            .tx
            .try_send(ControlCommand::Answer(question))
            .is_ok()
        {
            self.control_pending = true;
            self.status = "Submitting your explicit answers…".into();
        }
    }

    fn questions(&self) -> Vec<&QuestionRecord> {
        let mut questions: Vec<_> = self
            .selected_work
            .iter()
            .flat_map(|r| r.questions.iter().enumerate())
            .filter(|(_, q)| {
                self.history
                    || !matches!(q.state, QuestionState::Dismissed | QuestionState::Answered)
            })
            .collect();
        questions.sort_by_key(|(at, q)| {
            (
                match q.state {
                    QuestionState::Pending => 0,
                    QuestionState::Expired => 1,
                    _ => 2,
                },
                std::cmp::Reverse(*at),
            )
        });
        questions.into_iter().map(|(_, q)| q).collect()
    }
    fn current_question(&self) -> Option<&QuestionRecord> {
        self.questions().get(self.item_index).copied()
    }
    fn edit_answer(&mut self) {
        let item = self.current_question().and_then(|q| {
            q.prompts.get(self.prompt_index).map(|p| {
                (
                    q.target.session.clone(),
                    q.id.clone(),
                    p.id.clone(),
                    p.is_secret,
                    q.answers
                        .get(&p.id)
                        .map(|a| a.join("\n"))
                        .unwrap_or_default(),
                )
            })
        });
        if let Some((key, question, prompt, secret, text)) = item {
            self.edit_kind = Some(EditKind::Answer {
                key,
                question,
                prompt,
            });
            let mut editor = Editor::new(text);
            editor.secret = secret;
            self.edit = Some(editor);
        }
    }
    fn choose_answer(&mut self, number: usize) {
        let update = self.current_question().and_then(|q| {
            q.prompts.get(self.prompt_index).and_then(|p| {
                p.options.get(number.checked_sub(1)?).map(|option| {
                    let mut answers = q.answers.clone();
                    answers.insert(p.id.clone(), vec![option.label.clone()]);
                    (
                        q.target.session.clone(),
                        WorkUpdate::SaveAnswer {
                            question_id: q.id.clone(),
                            answers,
                        },
                    )
                })
            })
        });
        if let Some((key, update)) = update {
            self.update_state(key, update, "Answer draft saved; not submitted");
        }
    }
    fn dismiss_question(&mut self) {
        let update = self.current_question().map(|q| {
            (
                q.target.session.clone(),
                WorkUpdate::ResolveQuestion {
                    question_id: q.id.clone(),
                    state: QuestionState::Dismissed,
                },
            )
        });
        if let Some((key, update)) = update {
            self.update_state(
                key,
                update,
                "Dismissed locally; the native request was not answered or cancelled",
            );
        }
    }
    fn copy_answer(&mut self) {
        if self.pending_feedback.is_some() {
            self.status = "Copy is being saved locally; nothing was sent".into();
            return;
        }
        if self.current_question().is_some_and(|q| {
            q.prompts.iter().any(|p| {
                p.is_secret
                    && q.answers
                        .get(&p.id)
                        .is_some_and(|a| a.iter().any(|s| !s.trim().is_empty()))
            })
        }) {
            self.status = "Secret answers cannot be copied. Press f to write new non-secret feedback for this agent.".into();
            return;
        }
        let update = self
            .current_question()
            .map(|q| (q.target.session.clone(), q.id.clone()));
        if let Some((key, question_id)) = update {
            let id = format!(
                "answer-draft-{}",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            if self.update_state(
                key,
                WorkUpdate::AnswerAsFeedback {
                    question_id,
                    feedback_id: id.clone(),
                    mode: FeedbackMode::Guidance,
                },
                "Answer copied into feedback; not sent",
            ) {
                self.pending_feedback = Some(id);
            }
        }
    }
    fn questions_text(&self) -> String {
        let questions = self.questions();
        let Some(question) = questions.get(self.item_index) else {
            return "Saved questions\n\nNo unanswered questions for this goal.\nL listen for this agent's native questions\nH also reveals deliberately dismissed questions".into();
        };
        let mut text = format!(
            "Questions · {} of {}\n{}\n\n",
            self.item_index + 1,
            questions.len(),
            match question.state {
                QuestionState::Pending => "Pending in the current Codex connection",
                QuestionState::Expired => "Original request expired · draft retained",
                QuestionState::Resolved => "Resolved in Codex · text and draft retained",
                QuestionState::Dismissed => "Dismissed locally",
                QuestionState::Answered | QuestionState::AnsweredElsewhere =>
                    "Resolved in Codex · recorded answer",
            }
        );
        if let Some(prompt) = question.prompts.get(self.prompt_index) {
            text.push_str(&format!("{}\n{}\n", prompt.header, prompt.question));
            for (at, option) in prompt.options.iter().enumerate() {
                text.push_str(&format!(
                    "{} {} {}\n",
                    at + 1,
                    if question
                        .answers
                        .get(&prompt.id)
                        .is_some_and(|a| a.contains(&option.label))
                    {
                        "●"
                    } else {
                        "○"
                    },
                    option.label
                ));
                if !option.description.is_empty() {
                    text.push_str(&format!("  {}\n", option.description));
                }
            }
            match question.answers.get(&prompt.id) {
                Some(answer) => text.push_str(&format!(
                    "\nAnswer draft: {}\n",
                    if prompt.is_secret {
                        "Saved (hidden)".into()
                    } else {
                        answer.join("; ")
                    }
                )),
                None => {
                    text.push_str("\nNo answer chosen in Zoetrope. Defaults are never submitted.\n")
                }
            }
        }
        text.push_str("\n↑↓ question · Tab prompt · 1–9 choose · e write\nF2 submit answer · F5 copy closed request to feedback\nx dismiss locally · L listen · Esc back");
        text
    }

    fn producer_label(&self, target: &AgentKey) -> String {
        if target.agent_id == "main" {
            return "Coordinator".into();
        }
        self.view
            .agent_names
            .get(target)
            .cloned()
            .or_else(|| {
                self.view
                    .graph
                    .nodes
                    .iter()
                    .find(|n| n.id == FleetNodeId::Agent(target.clone()))
                    .map(|n| {
                        if n.label == target.agent_id {
                            "Agent".into()
                        } else {
                            n.label.clone()
                        }
                    })
            })
            .or_else(|| {
                self.snapshots
                    .get(&target.agent_id)
                    .and_then(|s| s.name.clone())
            })
            .unwrap_or_else(|| "Agent".into())
    }
    fn documents_text(&self) -> String {
        let mut text = String::from("Documents created or changed by agents\n\n");
        if let Some(record) = &self.selected_work {
            for (at, document) in record.documents.iter().enumerate() {
                text.push_str(&format!(
                    "{} {}\n  {} · {}\n",
                    if at == self.item_index { "›" } else { " " },
                    document
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                    self.producer_label(&document.producer),
                    match document.change {
                        crate::workstate::DocumentChange::Created => "Created",
                        crate::workstate::DocumentChange::Changed => "Changed",
                        crate::workstate::DocumentChange::Deleted => "Removed",
                    }
                ));
            }
            if record.documents.is_empty() {
                text.push_str("No recorded Markdown documents for this goal.\n");
            }
            if let Some(document) = record.documents.get(self.item_index) {
                text.push_str(&format!("\nSelected path: {}\n", document.path.display()));
            }
        }
        text.push_str(
            "\n↑↓ select · Enter request MD Reader preview\nEsc back · Opening is always explicit",
        );
        text
    }
    fn open_document(&mut self) {
        if self.document_pending {
            self.status = "An editor-open request is already in progress".into();
            return;
        }
        if let Some(document) = self
            .selected_work
            .as_ref()
            .and_then(|r| r.documents.get(self.item_index))
        {
            if self.documents.tx.try_send(document.path.clone()).is_ok() {
                self.document_pending = true;
                self.status = "Requesting the selected document in VS Code…".into();
            } else {
                self.status = "Editor launcher is busy; nothing else was opened".into();
            }
        }
    }

    fn activity_text(&self) -> String {
        let Some(target) = self.control_target() else {
            return String::new();
        };
        if let Some((owner, activity)) = self
            .view
            .activity
            .as_ref()
            .filter(|(owner, _)| owner == &target)
        {
            let Some(activity) = activity else {
                return format!(
                    "\nNo indexed public activity for {}",
                    self.producer_label(owner)
                );
            };
            let mut text = String::new();
            if let Some(narration) = &activity.narration {
                text.push_str(&format!(
                    "\nRecorded public message\nSource: {}\nRecorded: {}\n{}\n",
                    activity
                        .narration_source
                        .as_deref()
                        .unwrap_or("public transcript"),
                    activity.narration_at.as_deref().unwrap_or("unknown"),
                    narration
                ));
            }
            if let Some(command) = &activity.command {
                text.push_str(&format!(
                    "\nLatest shell command\nRecorded: {}\n{}\n",
                    activity.command_at.as_deref().unwrap_or("unknown"),
                    command
                ));
                if let Some(purpose) = &activity.purpose {
                    text.push_str(&format!("Purpose: {purpose}\n"));
                }
            }
            if text.is_empty() {
                text.push_str("\nNo recorded public message or shell command for this owner.");
            }
            return text;
        }
        "\nLoading indexed activity for the selected owner…".into()
    }

    fn toggle_checkpoints(&mut self) {
        if let Some(record) = &self.selected_work {
            self.update_state(
                record.session.clone(),
                WorkUpdate::SetCheckpoints {
                    enabled: !record.checkpoints,
                },
                "Checkpoint preference saved; send a criteria request to instruct the agent",
            );
        }
    }
    fn select_coordinator(&mut self) {
        let Some(session) = self
            .control_target()
            .map(|target| target.session)
            .or_else(|| self.view.selection.as_ref().and_then(session_key).cloned())
        else {
            self.status = "Select a goal or agent first".into();
            return;
        };
        self.feedback_target = None;
        self.feedback_open = false;
        self.pane = Pane::Graph;
        self.scroll = 0;
        self.select(FleetNodeId::Session(session));
        self.status =
            "Coordinator selected explicitly; saved drafts retain their recipients".into();
    }
    fn prepare_tracking(&mut self) {
        let Some(selected) = self
            .selected_agent()
            .map(|target| target.session)
            .or_else(|| self.view.selection.as_ref().and_then(session_key).cloned())
        else {
            self.status = "Select a goal or agent to enable tracking".into();
            return;
        };
        let Some(record) = &self.selected_work else {
            self.status = "Loading saved work; try Enable tracking again shortly".into();
            return;
        };
        if !self.state_root.is_absolute() {
            self.status =
                "State folder could not be resolved; restart from a valid working directory".into();
            return;
        }
        let Some(root) = self.state_root.to_str() else {
            self.status = "This state folder cannot be represented in a copyable command".into();
            return;
        };
        let target = AgentKey {
            session: selected,
            agent_id: "main".into(),
        };
        let provider = match target.session.provider {
            crate::provider::Provider::Codex => "codex",
            crate::provider::Provider::Claude => "claude",
        };
        let root = shell_quote(root);
        let session = shell_quote(&target.session.session_id);
        let mut instruction = format!(
            "Enable Zoetrope work tracking for this goal. Keep a short plan of meaningful work items with stable IDs, actual agent owners, reported status and known outcome/dependency links. Report purpose, important changes, next steps or blockers when useful.\n\nRead compact context:\nzoe state --root {root} context {provider} {session}\n\nPublish changed work with upsert_work_item through:\nzoe state --root {root} apply {provider} {session} '<update JSON>'\nUse zoe state --help for fields and revision checks. Keep outcomes unverified until supported by evidence. Publish only at meaningful transitions: plan/scope changes, work starts/completes, blockers, or verification. Batch changes; routine reads and commands need no report."
        );
        if record
            .goal
            .as_ref()
            .is_none_or(|goal| goal.criteria.is_empty())
        {
            instruction.push_str("\n\nNo completion criteria are recorded yet. Propose concise, editable outcome criteria including tests and final checks; use set_goal through the same local command to record the scope. Keep every outcome unverified until supported by evidence.");
        }
        let existing = record
            .drafts
            .iter()
            .find(|draft| draft.target == target)
            .map(|draft| draft.text.as_str())
            .unwrap_or_default();
        let text = if existing.contains(&instruction) {
            existing.to_owned()
        } else if existing.is_empty() {
            instruction
        } else {
            format!("{existing}\n\n{instruction}")
        };
        if text.len() > 65_536 {
            self.status =
                "Tracking instruction exceeds the draft limit; existing feedback was kept".into();
            return;
        }
        self.feedback_target = Some(target.clone());
        self.select_recipient(Some(target));
        self.feedback_open = true;
        self.pane = Pane::Feedback;
        self.edit_kind = Some(EditKind::Feedback);
        self.edit = Some(Editor::new(text));
        self.save_feedback();
        self.status =
            "Tracking draft for Coordinator; choose Send guidance or Queue explicitly".into();
    }
    fn request_criteria(&mut self) {
        let Some(target) = self.selected_agent() else {
            self.status = "Select the intended agent first".into();
            return;
        };
        if !self.state_root.is_absolute() {
            self.status =
                "State folder could not be resolved; restart from a valid working directory".into();
            return;
        }
        let Some(root) = self.state_root.to_str() else {
            self.status="This state folder cannot be represented in a copyable command; edit criteria locally".into();
            return;
        };
        let provider = match target.session.provider {
            crate::provider::Provider::Codex => "codex",
            crate::provider::Provider::Claude => "claude",
        };
        let root = shell_quote(root);
        let session = shell_quote(&target.session.session_id);
        let mut text = format!(
            "Please propose concise, editable completion criteria for this goal: {}. Include testing and final checks. Keep outcomes unverified until supported by evidence.\n\nRead the exact goal record with:\nzoe state --root {root} show {provider} {session}\n\nUpdate this same goal record with:\nzoe state --root {root} apply {provider} {session} '<update JSON>'\nUse zoe state --help for update fields.\n",
            self.selected_label()
        );
        if self.selected_work.as_ref().is_some_and(|r| r.checkpoints) {
            text.push_str("\nAsk me before meaningful milestones, and keep the question available until I answer.\n");
        }
        self.feedback_target = Some(target.clone());
        self.feedback_open = true;
        self.pane = Pane::Feedback;
        self.edit_kind = Some(EditKind::Feedback);
        self.edit = Some(Editor::new(text));
        self.save_feedback();
        self.status =
            "Criteria request draft prepared; choose Send guidance or Queue explicitly".into();
    }
    fn adopt_plan(&mut self) {
        let update = self.selected_work.as_ref().and_then(|record| {
            record.plan_drafts.get(self.plan_index).map(|plan| {
                (
                    record.session.clone(),
                    WorkUpdate::AdoptPlan {
                        proposal_id: plan.id.clone(),
                        title: self.selected_label(),
                        reason: "Operator explicitly adopted a recorded plan".into(),
                        provenance: "Operator in Zoetrope".into(),
                    },
                )
            })
        });
        if let Some((key, update)) = update {
            self.update_state(key, update, "Plan adopted as unverified outcomes");
        }
    }

    fn select(&mut self, id: FleetNodeId) {
        self.work_mode = matches!(id, FleetNodeId::Session(_));
        self.work_selection = None;
        self.feedback_requested = false;
        let new_key = session_key(&id).cloned();
        let same = self.view.selection.as_ref().and_then(session_key) == new_key.as_ref();
        self.requested_selection = Some(id.clone());
        self.view.selection = Some(id.clone());
        self.flow.select_node(&flow_id(&id));
        self.pending_center = true;
        if !same {
            self.selected_work = None;
            if let Some(key) = new_key {
                self.work_reload = Some(key);
            }
        }
        self.select_recipient(self.selected_agent());
        self.navigate(ConsoleCommand::Select(id));
    }

    fn toggle_expansion(&mut self) {
        if self.expanding.is_some() {
            return;
        }
        if let Some(key) = self.view.selection.as_ref().and_then(session_key).cloned()
            && let Some(node) = self
                .view
                .graph
                .nodes
                .iter()
                .find(|n| n.id == FleetNodeId::Session(key.clone()))
        {
            self.expanding = Some((key.clone(), !node.expanded));
            self.status = "Loading this goal's agents…".into();
            self.navigate(ConsoleCommand::ToggleExpanded(key));
        }
    }
    pub(super) fn request_replay(&mut self) {
        self.replay_requested = true;
        self.status = "Read-only replay requested".into();
    }
    pub(super) fn replay_source(&self) -> Option<(SessionKey, PathBuf)> {
        let key = self.view.selection.as_ref().and_then(session_key)?.clone();
        self.view.sources.get(&key).cloned().map(|path| (key, path))
    }

    fn move_selection(&mut self, delta: isize) {
        let nodes = &self.view.graph.nodes;
        let current = self
            .view
            .selection
            .as_ref()
            .and_then(|id| nodes.iter().position(|n| &n.id == id));
        let next = if let Some(at) = current {
            (at as isize + delta).clamp(0, nodes.len().saturating_sub(1) as isize) as usize
        } else {
            nodes
                .iter()
                .position(|n| matches!(n.id, FleetNodeId::Session(_)))
                .unwrap_or(0)
        };
        if let Some(node) = nodes.get(next) {
            self.select(node.id.clone());
        }
    }

    fn sync_graph(&mut self) {
        let mut nodes = Vec::with_capacity(self.view.graph.nodes.len());
        let mut edges = Vec::with_capacity(self.view.graph.nodes.len());
        let visible: BTreeSet<_> = self.view.graph.nodes.iter().map(|node| &node.id).collect();
        // Stable positions include dragged and currently hidden nodes. Index
        // their slots once instead of scanning the entire map for each card.
        fn slot((x, y): (f64, f64)) -> (u64, u64) {
            let bits = |n: f64| if n == 0.0 { 0 } else { n.to_bits() };
            (bits(x), bits(y))
        }
        let mut occupied: std::collections::HashSet<_> =
            self.positions.values().copied().map(slot).collect();
        let mut depths = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for (at, node) in self.view.graph.nodes.iter().enumerate() {
            if !seen.insert(&node.id) {
                continue;
            }
            let depth = node
                .parent
                .as_ref()
                .and_then(|p| depths.get(p))
                .copied()
                .unwrap_or(0usize)
                + usize::from(node.parent.is_some());
            depths.insert(node.id.clone(), depth.min(8));
            let pos = if let Some(pos) = self.positions.get(&node.id) {
                *pos
            } else {
                let x = depth.min(8) as f64 * 36.0;
                let mut y = at as f64 * 10.0;
                while !occupied.insert(slot((x, y))) {
                    y += 10.0;
                }
                self.positions.insert(node.id.clone(), (x, y));
                (x, y)
            };
            let progress = session_key(&node.id)
                .and_then(|k| self.view.summaries.get(k))
                .map(|s| s.progress.label.clone())
                .unwrap_or_default();
            let title = match &node.id {
                FleetNodeId::Project(path) => PathBuf::from(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone()),
                FleetNodeId::Agent(agent) if agent.agent_id == "main" => "Coordinator".into(),
                FleetNodeId::Session(key) if node.label == key.session_id => node
                    .goal
                    .clone()
                    .or_else(|| node.description.clone())
                    .unwrap_or_else(|| "Untitled goal".into()),
                FleetNodeId::Agent(agent) if node.label == agent.agent_id => "Agent".into(),
                _ => node.label.clone(),
            };
            let native_id = match &node.id {
                FleetNodeId::Session(key) => Some(key.session_id.as_str()),
                FleetNodeId::Agent(agent) => Some(if agent.agent_id == "main" {
                    agent.session.session_id.as_str()
                } else {
                    agent.agent_id.as_str()
                }),
                _ => None,
            };
            let detail = native_id
                .and_then(|id| self.snapshots.get(id))
                .map(|s| control_worker::status_word(&s.status).to_owned())
                .unwrap_or_else(|| {
                    if node.archived {
                        "Archived".into()
                    } else {
                        String::new()
                    }
                });
            let bar = if matches!(node.id, FleetNodeId::Session(_)) {
                session_key(&node.id)
                    .and_then(|key| self.view.summaries.get(key))
                    .and_then(|summary| {
                        summary.progress.percent.map(|percent| {
                            let cells = if summary.progress.verified == summary.progress.total {
                                20
                            } else {
                                ((percent / 100.0 * 20.0) as usize).min(19)
                            };
                            format!("[{}{}]", "#".repeat(cells), "-".repeat(20 - cells))
                        })
                    })
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let card = Card {
                bar,
                title,
                detail,
                activity: node
                    .activity
                    .as_ref()
                    .map(|activity| format!("Recorded: {activity}"))
                    .unwrap_or_default(),
                progress: if matches!(node.id, FleetNodeId::Session(_)) {
                    progress
                } else {
                    String::new()
                },
            };
            let id = flow_id(&node.id);
            nodes.push(
                Node::new(id.clone(), pos, (34.0, 9.0), card)
                    .with_deletable(false)
                    .with_connectable(false)
                    .with_handles(vec![
                        Handle::source(HandlePosition::Right).with_hidden(true),
                        Handle::target(HandlePosition::Left).with_hidden(true),
                    ]),
            );
            if let Some(parent) = node
                .parent
                .as_ref()
                .filter(|parent| **parent != node.id && visible.contains(parent))
            {
                edges.push(
                    Edge::new(format!("edge-{id}"), flow_id(parent), id)
                        .with_selectable(false)
                        .with_deletable(false)
                        .with_reconnectable(Reconnectable::None),
                );
            }
        }
        // Batch geometry resolution: the single-node APIs re-walk all prior
        // nodes after every insertion/position update.
        let _ = self.flow.set_nodes(nodes);
        let _ = self.flow.set_edges(edges);
        if let Some(id) = &self.view.selection {
            self.flow.select_node(&flow_id(id));
        }
    }
    fn navigate(&mut self, command: ConsoleCommand) {
        if self
            .inventory
            .tx
            .try_send(InventoryCommand::Navigate(command))
            .is_err()
        {
            self.status = "Inventory busy; try again shortly".into();
        }
    }
    /// Handle one key/mouse event. Returns true only for an explicit quit.
    pub fn handle_event(&mut self, event: &Event) -> bool {
        if let Event::Mouse(mouse) = event {
            presentation::mouse(self, *mouse);
            return false;
        }
        if let Event::Paste(text) = event {
            if let Some(edit) = self.edit.as_mut() {
                let inserted = edit.insert(text);
                if inserted.is_err() {
                    self.status = "Input exceeds the 64 KiB draft limit; nothing was added".into();
                } else if inserted == Ok(true)
                    && matches!(
                        self.edit_kind,
                        Some(EditKind::Feedback | EditKind::Answer { .. })
                    )
                {
                    self.dirty_since = Some(Instant::now());
                    self.status = "Saving draft…".into();
                }
            } else if self.searching {
                for ch in text.chars().filter(|ch| !ch.is_control()) {
                    if self.query.len() + ch.len_utf8() <= 512 {
                        self.query.push(ch);
                    }
                }
            }
            return false;
        }
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Release
                || (key.kind == KeyEventKind::Repeat && matches!(key.code, KeyCode::F(_)))
            {
                return false;
            }
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                if self.draft_update().is_some()
                    || self.pending_save.is_some()
                    || self.close_after_save
                {
                    self.quit_after_save = true;
                    if self.draft_update().is_some() || self.pending_save.is_some() {
                        self.request_close_editor();
                    } else {
                        self.commit_edit();
                    }
                    return false;
                }
                self.quit = true;
                return true;
            }
            if self.show_help {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('?') => self.show_help = false,
                    KeyCode::PageDown => self.help_scroll = self.help_scroll.saturating_add(8),
                    KeyCode::PageUp => self.help_scroll = self.help_scroll.saturating_sub(8),
                    KeyCode::Char('q') => self.quit = true,
                    _ => {}
                }
                return self.quit;
            }
            if !self.searching
                && (matches!(self.edit_kind, Some(EditKind::Feedback))
                    || (self.edit.is_none() && matches!(self.pane, Pane::Graph | Pane::Feedback)))
            {
                if key.modifiers == KeyModifiers::CONTROL {
                    let mode = match key.code {
                        KeyCode::Char('s') => Some(FeedbackMode::Guidance),
                        KeyCode::Char('q') => Some(FeedbackMode::Queue),
                        KeyCode::Char('x') => Some(FeedbackMode::Stop),
                        _ => None,
                    };
                    if let Some(mode) = mode {
                        if key.kind != KeyEventKind::Repeat {
                            self.send_feedback(mode);
                        }
                        return false;
                    }
                }
                match key.code {
                    KeyCode::F(2) => {
                        self.send_feedback(FeedbackMode::Guidance);
                        return false;
                    }
                    KeyCode::F(3) => {
                        self.send_feedback(FeedbackMode::Queue);
                        return false;
                    }
                    KeyCode::F(4) => {
                        self.send_feedback(FeedbackMode::Stop);
                        return false;
                    }
                    _ => {}
                }
            }
            if self.searching {
                match key.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.searching = false;
                        self.navigate(ConsoleCommand::Search(self.query.clone()));
                    }
                    KeyCode::Backspace => {
                        self.query.pop();
                    }
                    KeyCode::Char(ch) if self.query.len() < 512 => {
                        self.query.push(ch);
                    }
                    _ => {}
                }
                return false;
            }
            if let Some(edit) = self.edit.as_mut() {
                if key.code == KeyCode::F(2) && !matches!(self.edit_kind, Some(EditKind::Feedback))
                {
                    self.commit_edit();
                } else if key.code == KeyCode::Esc {
                    self.request_close_editor();
                } else {
                    match edit.key(key) {
                        Err(()) => {
                            self.status =
                                "Input exceeds the 64 KiB draft limit; nothing was added".into()
                        }
                        Ok(true)
                            if matches!(
                                self.edit_kind,
                                Some(EditKind::Feedback | EditKind::Answer { .. })
                            ) =>
                        {
                            self.dirty_since = Some(Instant::now());
                            self.status = "Saving draft…".into();
                        }
                        _ => {}
                    }
                }
                return false;
            }
            match key.code {
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Char('?') => {
                    self.show_help = true;
                    self.help_scroll = 0;
                }
                KeyCode::Char('+') | KeyCode::Char('=') => self.flow.zoom_in(),
                KeyCode::Char('-') => self.flow.zoom_out(),
                KeyCode::Char('o') => self.flow.request_fit_view(),
                KeyCode::Char('r') => {
                    self.flow.zoom_to(1.0);
                    self.pending_center = true;
                }
                KeyCode::Left => {
                    self.flow.pan(8.0, 0.0);
                    self.pending_center = false;
                }
                KeyCode::Right => {
                    self.flow.pan(-8.0, 0.0);
                    self.pending_center = false;
                }
                KeyCode::Char('I') => self.navigate(ConsoleCommand::ToggleInternal),
                KeyCode::Char('H' | 'h') => {
                    self.history = !self.history;
                    self.navigate(ConsoleCommand::ToggleHistory);
                }
                KeyCode::Char('/') => self.searching = true,
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.pane == Pane::Graph && self.work_selection.is_some() {
                        self.move_work(1);
                    } else if self.pane == Pane::Documents {
                        self.item_index = self.item_index.saturating_add(1).min(
                            self.selected_work
                                .as_ref()
                                .map_or(0, |r| r.documents.len().saturating_sub(1)),
                        );
                    } else if self.pane == Pane::Questions {
                        self.item_index = self
                            .item_index
                            .saturating_add(1)
                            .min(self.questions().len().saturating_sub(1));
                        self.prompt_index = 0;
                    } else if self.pane == Pane::Criteria {
                        self.item_index = self.item_index.saturating_add(1).min(
                            self.selected_work
                                .as_ref()
                                .and_then(|r| r.goal.as_ref())
                                .map_or(0, |g| g.criteria.len().saturating_sub(1)),
                        );
                    } else {
                        self.move_selection(1);
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.pane == Pane::Graph && self.work_selection.is_some() {
                        self.move_work(-1);
                    } else if matches!(
                        self.pane,
                        Pane::Criteria | Pane::Questions | Pane::Documents
                    ) {
                        self.item_index = self.item_index.saturating_sub(1);
                    } else {
                        self.move_selection(-1);
                    }
                }
                KeyCode::Enter if self.pane == Pane::Documents => self.open_document(),
                KeyCode::Enter | KeyCode::Char(' ') if self.pane == Pane::Graph => {
                    self.toggle_expansion()
                }
                KeyCode::Char('w') if matches!(self.pane, Pane::Graph | Pane::Feedback) => {
                    self.focus_work()
                }
                KeyCode::Char('a') if matches!(self.pane, Pane::Graph | Pane::Feedback) => {
                    self.open_agents()
                }
                KeyCode::Char(']') if self.pane == Pane::Graph => self.move_work(1),
                KeyCode::Char('[') if self.pane == Pane::Graph => self.move_work(-1),
                KeyCode::Char('x') if matches!(self.pane, Pane::Graph | Pane::Feedback) => {
                    self.open_commands()
                }
                KeyCode::Char('R') => self.request_replay(),
                KeyCode::Char('f') => self.open_feedback(),
                KeyCode::Char('T') => self.prepare_tracking(),
                KeyCode::Char('C') => self.select_coordinator(),
                KeyCode::Char('b') => {
                    self.pane = Pane::Deliveries;
                    self.scroll = 0;
                }
                KeyCode::Char('d') => {
                    self.pane = Pane::Documents;
                    self.item_index = 0;
                    self.scroll = 0;
                }
                KeyCode::Char('Q') => {
                    self.pane = Pane::Questions;
                    self.item_index = 0;
                    self.prompt_index = 0;
                    self.scroll = 0;
                }
                KeyCode::Char('e') if self.pane == Pane::Questions => self.edit_answer(),
                KeyCode::Char('L') if self.pane == Pane::Questions => self.listen_questions(),
                KeyCode::F(2) if self.pane == Pane::Questions => self.submit_question(),
                KeyCode::Char('x') if self.pane == Pane::Questions => self.dismiss_question(),
                KeyCode::F(5) if self.pane == Pane::Questions => self.copy_answer(),
                KeyCode::Char(ch) if self.pane == Pane::Questions && ch.is_ascii_digit() => {
                    self.choose_answer(ch.to_digit(10).unwrap_or(0) as usize)
                }
                KeyCode::Tab if self.pane == Pane::Questions => {
                    self.prompt_index = self.prompt_index.saturating_add(1);
                    let count = self
                        .current_question()
                        .map_or(1, |q| q.prompts.len().max(1));
                    self.prompt_index %= count;
                }
                KeyCode::Char('c') => {
                    self.pane = Pane::Criteria;
                    self.item_index = 0;
                    self.scroll = 0;
                }
                KeyCode::Char('e') if self.pane == Pane::Criteria => self.edit_goal(),
                KeyCode::Char('v') if self.pane == Pane::Criteria => self.edit_evidence(),
                KeyCode::Char('u') if self.pane == Pane::Criteria => self.reopen_criterion(),
                KeyCode::Char('p') if self.pane == Pane::Criteria => self.toggle_checkpoints(),
                KeyCode::Char('A') => self.request_criteria(),
                KeyCode::Char('a') if self.pane == Pane::Criteria => self.adopt_plan(),
                KeyCode::Char(']') if self.pane == Pane::Criteria => {
                    self.plan_index = self.plan_index.saturating_add(1).min(
                        self.selected_work
                            .as_ref()
                            .map_or(0, |r| r.plan_drafts.len().saturating_sub(1)),
                    );
                }
                KeyCode::Char('[') if self.pane == Pane::Criteria => {
                    self.plan_index = self.plan_index.saturating_sub(1)
                }
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(8),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(8),
                KeyCode::Esc => {
                    self.pane = Pane::Graph;
                    self.scroll = 0;
                }
                _ => {}
            }
        }
        self.quit
    }

    /// Render cached state into a terminal frame.
    pub fn draw(&mut self, frame: &mut Frame) {
        presentation::draw(self, frame);
    }
    /// Flush and stop workers after an orderly exit. No native work is stopped.
    pub fn shutdown(&mut self) {
        if let Err(error) = self.try_shutdown() {
            eprintln!("{error}");
        }
    }
    /// Flush editable data before stopping. On failure it remains in the composer
    /// and workers remain usable, so the caller can repair storage and retry.
    pub fn try_shutdown(&mut self) -> anyhow::Result<()> {
        if self.edit.is_some() && self.draft_update().is_none() && self.close_after_save {
            anyhow::bail!(
                "Criteria save is not confirmed; edit retained. F2 retries; Esc cancels after the pending write finishes."
            );
        }
        if let Some((key, update, _)) = self.draft_update() {
            let (reply, result) = mpsc::channel();
            self.state
                .tx
                .send(StateCommand::Flush(key, Box::new(update), reply))
                .map_err(|_| {
                    anyhow::anyhow!("Local save worker stopped; draft remains in the composer")
                })?;
            if let Err(error) = result.recv().map_err(|_| {
                anyhow::anyhow!("Local save worker stopped before confirming the draft")
            })? {
                self.status =
                    format!("Could not save draft: {error}. Draft retained; Esc retries.");
                self.quit = false;
                self.quit_after_save = false;
                self.dirty_since = None;
                anyhow::bail!("{}", self.status);
            }
            self.close_editor();
            self.pending_save = None;
        }
        self.control.shutdown();
        self.state.shutdown();
        let _ = self.inventory.tx.send(InventoryCommand::Shutdown);
        if let Some(worker) = self.inventory.thread.take() {
            let _ = worker.join();
        }
        Ok(())
    }
}
impl Drop for FleetConsole {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn shell_quote(text: &str) -> String {
    let apostrophe = char::from(39);
    let escaped = String::from_iter([apostrophe, char::from(92), apostrophe, apostrophe]);
    format!("'{}'", text.replace(apostrophe, &escaped))
}
