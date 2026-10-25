//! Zoetrope-owned goal records. Native transcripts are never written here.
//!
//! All filesystem operations are blocking. Frontends keep a cached projection
//! and run refresh/update calls on an inventory worker, never the render loop.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::console::{AgentKey, SessionKey};
use crate::provider::Provider;

const FORMAT_VERSION: u32 = 1;
const MAX_RECORD_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CriterionInput {
    pub id: String,
    pub label: String,
    #[serde(default = "equal_weight")]
    pub weight: f64,
}
fn equal_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Reference,
    UserAttestation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub detail: String,
    pub provenance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionStatus {
    Pending,
    InProgress,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Criterion {
    pub id: String,
    pub label: String,
    pub weight: f64,
    pub status: CriterionStatus,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoalContract {
    pub title: String,
    pub revision: u64,
    pub criteria: Vec<Criterion>,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopeRevision {
    pub goal: GoalContract,
    pub reason: String,
    pub changed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackDraft {
    pub target: AgentKey,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionOption {
    pub label: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionPrompt {
    pub id: String,
    #[serde(default)]
    pub header: String,
    pub question: String,
    #[serde(default)]
    pub options: Vec<QuestionOption>,
    #[serde(default)]
    pub is_other: bool,
    #[serde(default)]
    pub is_secret: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionState {
    Pending,
    Expired,
    Answered,
    AnsweredElsewhere,
    Resolved,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuestionRecord {
    pub id: String,
    pub target: AgentKey,
    #[serde(default)]
    pub request_id: Option<serde_json::Value>,
    #[serde(default)]
    pub turn_id: Option<String>,
    pub prompts: Vec<QuestionPrompt>,
    pub state: QuestionState,
    #[serde(default)]
    pub answers: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackMode {
    Guidance,
    Queue,
    Stop,
    Answer,
    CriteriaRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryState {
    Draft,
    Sending,
    Queued,
    Accepted,
    Submitted,
    Rejected,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackRecord {
    pub id: String,
    pub target: AgentKey,
    pub text: String,
    pub mode: FeedbackMode,
    #[serde(default)]
    pub expected_turn: Option<String>,
    #[serde(default)]
    pub source_question: Option<String>,
    pub state: DeliveryState,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub queue_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentChange {
    Created,
    Changed,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentRecord {
    pub id: String,
    pub path: PathBuf,
    pub producer: AgentKey,
    pub source: String,
    pub change: DocumentChange,
    #[serde(default)]
    pub changed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanStep {
    pub step: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanProposal {
    pub id: String,
    pub producer: AgentKey,
    pub steps: Vec<PlanStep>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CriterionRevision {
    pub criterion: Criterion,
    pub goal_revision: u64,
    pub reason: String,
    pub changed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkRecord {
    pub version: u32,
    pub session: SessionKey,
    pub goal: Option<GoalContract>,
    #[serde(default)]
    pub scope_history: Vec<ScopeRevision>,
    #[serde(default)]
    pub drafts: Vec<FeedbackDraft>,
    #[serde(default)]
    pub questions: Vec<QuestionRecord>,
    #[serde(default)]
    pub observed_question_resolutions: BTreeMap<String, bool>,
    #[serde(default)]
    pub outbox: Vec<FeedbackRecord>,
    #[serde(default)]
    pub documents: Vec<DocumentRecord>,
    #[serde(default)]
    pub plan_drafts: Vec<PlanProposal>,
    #[serde(default)]
    pub criterion_history: Vec<CriterionRevision>,
    #[serde(default)]
    pub checkpoints: bool,
}
impl WorkRecord {
    fn empty(session: SessionKey) -> Self {
        Self {
            version: FORMAT_VERSION,
            session,
            goal: None,
            scope_history: Vec::new(),
            drafts: Vec::new(),
            questions: Vec::new(),
            observed_question_resolutions: BTreeMap::new(),
            outbox: Vec::new(),
            documents: Vec::new(),
            plan_drafts: Vec::new(),
            criterion_history: Vec::new(),
            checkpoints: false,
        }
    }
    pub fn summary(&self) -> WorkSummary {
        WorkSummary {
            title: self.goal.as_ref().map(|goal| goal.title.clone()),
            progress: self.progress(),
            pending_questions: self
                .questions
                .iter()
                .filter(|question| {
                    matches!(
                        question.state,
                        QuestionState::Pending | QuestionState::Expired | QuestionState::Resolved
                    )
                })
                .count(),
            unfinished_drafts: self.drafts.iter().any(|draft| {
                !draft.text.is_empty()
                    && !self.outbox.iter().any(|item| {
                        item.target == draft.target
                            && item.text == draft.text
                            && matches!(item.state, DeliveryState::Accepted | DeliveryState::Queued)
                    })
            }),
        }
    }
    pub fn matches(&self, query: &str) -> bool {
        self.goal.as_ref().is_some_and(|goal| {
            goal.title.to_lowercase().contains(query)
                || goal
                    .criteria
                    .iter()
                    .any(|criterion| criterion.label.to_lowercase().contains(query))
        }) || self.documents.iter().any(|document| {
            document
                .path
                .to_string_lossy()
                .to_lowercase()
                .contains(query)
        })
    }

    pub fn progress(&self) -> Progress {
        let total = self.goal.as_ref().map_or(0, |goal| goal.criteria.len());
        if total == 0 {
            return Progress {
                verified: 0,
                total: 0,
                percent: None,
                label: "Progress not tracked".into(),
            };
        }
        let criteria = &self.goal.as_ref().expect("nonempty goal").criteria;
        let max_weight = criteria.iter().map(|c| c.weight).fold(0.0, f64::max);
        let verified = criteria
            .iter()
            .filter(|c| c.status == CriterionStatus::Verified && !c.evidence.is_empty())
            .count();
        let total_weight: f64 = criteria.iter().map(|c| c.weight / max_weight).sum();
        let verified_weight: f64 = criteria
            .iter()
            .filter(|c| c.status == CriterionStatus::Verified && !c.evidence.is_empty())
            .map(|c| c.weight / max_weight)
            .sum();
        let percent = if verified == 0 {
            0.0
        } else {
            (verified_weight / total_weight * 100.0).clamp(0.0, 100.0)
        };
        // Floating point and whole-percent rounding cannot turn an unfinished
        // contract into a completion claim. The count remains authoritative.
        let percent = if verified < total {
            percent.min(100.0 - 100.0 * f64::EPSILON)
        } else {
            100.0
        };
        let display = if verified < total && percent >= 99.5 {
            "<100%".into()
        } else if percent > 0.0 && percent < 0.5 {
            "<1%".into()
        } else {
            format!("{percent:.0}%")
        };
        Progress {
            verified,
            total,
            percent: Some(percent),
            label: format!("{verified} of {total} verified · {display}"),
        }
    }
}

/// Compact graph projection. Full histories, drafts and outbox stay in the
/// bounded lazy cache, rather than being retained for every historical goal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkSummary {
    pub title: Option<String>,
    pub progress: Progress,
    pub pending_questions: usize,
    pub unfinished_drafts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Progress {
    pub verified: usize,
    pub total: usize,
    pub percent: Option<f64>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum WorkUpdate {
    ExpirePendingQuestions {
        #[serde(default)]
        target: Option<AgentKey>,
    },
    SetCheckpoints {
        enabled: bool,
    },
    RequestCriteria {
        target: AgentKey,
    },
    AdoptPlan {
        proposal_id: String,
        title: String,
        reason: String,
        provenance: String,
    },
    SetCriterionStatus {
        id: String,
        status: CriterionStatus,
        #[serde(default)]
        revision: Option<u64>,
        reason: String,
    },
    ImportPlan {
        proposal: PlanProposal,
    },
    AddDocument {
        document: DocumentRecord,
    },
    CreateFeedback {
        id: String,
        target: AgentKey,
        text: String,
        mode: FeedbackMode,
        #[serde(default)]
        expected_turn: Option<String>,
        #[serde(default)]
        source_question: Option<String>,
    },
    RecordDelivery {
        id: String,
        state: DeliveryState,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        queue_id: Option<String>,
    },
    CaptureQuestion {
        question: QuestionRecord,
    },
    SaveAnswer {
        question_id: String,
        answers: BTreeMap<String, Vec<String>>,
    },
    ObserveQuestionResolution {
        question_id: String,
        answered: bool,
    },
    ResolveQuestion {
        question_id: String,
        state: QuestionState,
    },
    AnswerAsFeedback {
        question_id: String,
        feedback_id: String,
        mode: FeedbackMode,
    },
    SaveDraft {
        target: AgentKey,
        text: String,
    },
    /// Clear an acknowledged draft only if no newer edit replaced it.
    ClearDraftIfMatches {
        target: AgentKey,
        expected_text: String,
    },
    Verify {
        id: String,
        evidence: Vec<Evidence>,
        #[serde(default)]
        revision: Option<u64>,
    },
    SetGoal {
        title: String,
        criteria: Vec<CriterionInput>,
        reason: String,
        provenance: String,
    },
}

/// Atomic local record access. Constructing a store does no IO.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}
impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn default_root() -> Result<PathBuf> {
        if let Some(root) = std::env::var_os("ZOETROPE_STATE_DIR") {
            return Ok(root.into());
        }
        if let Some(root) = std::env::var_os("XDG_STATE_HOME") {
            return Ok(PathBuf::from(root).join("zoetrope"));
        }
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .context("cannot locate local state home; set ZOETROPE_STATE_DIR")?;
        Ok(PathBuf::from(home).join(".local/state/zoetrope"))
    }
    fn path(&self, session: &SessionKey) -> PathBuf {
        // Encoding identity bytes avoids path traversal and provider collisions.
        let id: String = session
            .session_id
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        self.root.join(format!(
            "{}-{id}.json",
            match session.provider {
                Provider::Codex => "codex",
                Provider::Claude => "claude",
            }
        ))
    }
    pub(crate) fn stamp(&self, session: &SessionKey) -> Option<(u64, std::time::SystemTime)> {
        let metadata = fs::metadata(self.path(session)).ok()?;
        Some((metadata.len(), metadata.modified().ok()?))
    }
    pub fn read(&self, session: &SessionKey) -> Result<WorkRecord> {
        if session.session_id.is_empty() || session.session_id.len() > 100 {
            bail!("session identity must contain 1 to 100 UTF-8 bytes");
        }
        let path = self.path(session);
        let file = match fs::File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(WorkRecord::empty(session.clone()));
            }
            Err(error) => return Err(error).with_context(|| format!("reading {}", path.display())),
        };
        let mut bytes = Vec::new();
        file.take(MAX_RECORD_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_RECORD_BYTES {
            bail!("local goal record exceeds 8 MiB; it was left unchanged");
        }
        let mut record: WorkRecord = serde_json::from_slice(&bytes).with_context(|| {
            format!(
                "invalid local goal record {}; it was left unchanged",
                path.display()
            )
        })?;
        if record.version != FORMAT_VERSION {
            bail!(
                "unsupported local goal record version {}; it was left unchanged",
                record.version
            );
        }
        if let Some(goal) = &record.goal
            && (goal.title.len() > 512
                || goal.criteria.len() > 512
                || goal
                    .criteria
                    .iter()
                    .any(|criterion| !criterion.weight.is_finite() || criterion.weight <= 0.0))
        {
            bail!("invalid local goal contract; record was left unchanged");
        }
        if record.session != *session {
            bail!("local record identity does not match the selected session");
        }
        // An interrupted process cannot prove whether a write reached Codex.
        // Keep its target/text/expected turn, but never auto-resend it.
        for feedback in &mut record.outbox {
            if feedback.state == DeliveryState::Sending {
                feedback.state = DeliveryState::Unknown;
                feedback.message=Some("Delivery uncertain after interruption; inspect Codex before explicitly trying again".into());
            }
        }
        Ok(record)
    }
    pub fn apply(&self, session: &SessionKey, update: WorkUpdate) -> Result<WorkRecord> {
        self.apply_many(session, std::iter::once(update))
    }

    /// Commit a bounded set of observed updates in one lock/atomic write.
    pub fn apply_many(
        &self,
        session: &SessionKey,
        updates: impl IntoIterator<Item = WorkUpdate>,
    ) -> Result<WorkRecord> {
        if session.session_id.is_empty() || session.session_id.len() > 100 {
            bail!("session identity must contain 1 to 100 UTF-8 bytes");
        }
        fs::create_dir_all(&self.root)?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        // The lock file is stable while record files are atomically replaced.
        // OS locks are released if a writer crashes, unlike sentinel files.
        let lock = options.open(self.path(session).with_extension("lock"))?;
        fs2::FileExt::lock_exclusive(&lock).context("locking local goal record")?;
        let mut record = self.read(session)?;
        for update in updates {
            let update = match update {
                WorkUpdate::AdoptPlan {
                    proposal_id,
                    title,
                    reason,
                    provenance,
                } => {
                    let proposal = record
                        .plan_drafts
                        .iter()
                        .find(|proposal| proposal.id == proposal_id)
                        .context("unknown draft plan")?;
                    WorkUpdate::SetGoal {
                        title,
                        criteria: proposal
                            .steps
                            .iter()
                            .enumerate()
                            .map(|(index, step)| CriterionInput {
                                id: format!("plan-{}-{}", proposal.producer.agent_id, index + 1),
                                label: step.step.clone(),
                                weight: 1.0,
                            })
                            .collect(),
                        reason,
                        provenance,
                    }
                }
                WorkUpdate::RequestCriteria { target } => {
                    let mut text = String::from(
                        "Please propose concise, editable completion criteria for this goal, including final checks. Use Zoetrope's local goal-tracking command (zoe state --help) to record agreed outcomes and attach evidence when verifying them. Keep missing evidence unverified.",
                    );
                    if record.checkpoints {
                        text.push_str(" Ask me before proceeding beyond meaningful milestones; keep the question available until I answer.");
                    }
                    WorkUpdate::SaveDraft { target, text }
                }
                other => other,
            };
            match update {
                WorkUpdate::ExpirePendingQuestions { target } => {
                    if target
                        .as_ref()
                        .is_some_and(|target| target.session != *session)
                    {
                        bail!("question recipient does not belong to this session");
                    }
                    for question in &mut record.questions {
                        if question.state == QuestionState::Pending
                            && question.request_id.is_some()
                            && target
                                .as_ref()
                                .is_none_or(|target| target == &question.target)
                        {
                            question.state = QuestionState::Expired;
                        }
                    }
                }
                WorkUpdate::SetCheckpoints { enabled } => record.checkpoints = enabled,
                WorkUpdate::AdoptPlan { .. } | WorkUpdate::RequestCriteria { .. } => {
                    unreachable!("converted into ordinary record edits")
                }
                WorkUpdate::SetCriterionStatus {
                    id,
                    status,
                    revision,
                    reason,
                } => {
                    if status == CriterionStatus::Verified {
                        bail!("use verify with new evidence to mark an outcome verified");
                    }
                    if reason.trim().is_empty() {
                        bail!("changing outcome status requires a reason");
                    }
                    let goal = record.goal.as_mut().context("progress is not tracked")?;
                    if revision.is_some_and(|revision| revision != goal.revision) {
                        bail!("scope changed; review current criteria before updating");
                    }
                    let criterion = goal
                        .criteria
                        .iter_mut()
                        .find(|criterion| criterion.id == id)
                        .context("unknown criterion")?;
                    record.criterion_history.push(CriterionRevision {
                        criterion: criterion.clone(),
                        goal_revision: goal.revision,
                        reason,
                        changed_at: chrono::Utc::now().to_rfc3339(),
                    });
                    criterion.status = status;
                }
                WorkUpdate::ImportPlan { proposal } => {
                    if proposal.producer.session != *session || proposal.id.trim().is_empty() {
                        bail!("plan source does not belong to this session");
                    }
                    if let Some(existing) = record
                        .plan_drafts
                        .iter_mut()
                        .find(|entry| entry.producer == proposal.producer)
                    {
                        *existing = proposal;
                    } else {
                        record.plan_drafts.push(proposal);
                    }
                }
                WorkUpdate::AddDocument { document } => {
                    if document.producer.session != *session {
                        bail!("document producer does not belong to this session");
                    }
                    if document.id.trim().is_empty() || !document.path.is_absolute() {
                        bail!("document requires a stable identifier and absolute path");
                    }
                    if let Some(existing) = record.documents.iter_mut().find(|entry| {
                        entry.path == document.path && entry.producer == document.producer
                    }) {
                        *existing = document;
                    } else {
                        record.documents.push(document);
                    }
                }
                WorkUpdate::CreateFeedback {
                    id,
                    target,
                    text,
                    mode,
                    expected_turn,
                    source_question,
                } => {
                    if target.session != *session {
                        bail!("feedback recipient does not belong to this session");
                    }
                    if id.trim().is_empty() || record.outbox.iter().any(|entry| entry.id == id) {
                        bail!("feedback requires a new stable identifier");
                    }
                    if text.trim().is_empty() && mode != FeedbackMode::Stop {
                        bail!("feedback cannot be empty");
                    }
                    record.outbox.push(FeedbackRecord {
                        id,
                        target,
                        text,
                        mode,
                        expected_turn,
                        source_question,
                        state: DeliveryState::Draft,
                        message: Some("Saved locally; not sent to Codex".into()),
                        queue_id: None,
                    });
                }
                WorkUpdate::RecordDelivery {
                    id,
                    state,
                    message,
                    queue_id,
                } => {
                    let feedback = record
                        .outbox
                        .iter_mut()
                        .find(|entry| entry.id == id)
                        .context("unknown feedback identifier")?;
                    feedback.state = state;
                    feedback.message = message;
                    feedback.queue_id = queue_id.or_else(|| feedback.queue_id.clone());
                }
                WorkUpdate::CaptureQuestion { mut question } => {
                    if let Some(answered) = record.observed_question_resolutions.get(&question.id) {
                        question.state = if *answered {
                            QuestionState::Answered
                        } else {
                            QuestionState::Resolved
                        };
                    }
                    if question.target.session != *session {
                        bail!("question recipient does not belong to this session");
                    }
                    if question.id.trim().is_empty() || question.prompts.is_empty() {
                        bail!("question requires a stable identifier and prompts");
                    }
                    let mut ids = std::collections::BTreeSet::new();
                    if question.prompts.iter().any(|prompt| {
                        prompt.id.trim().is_empty()
                            || prompt.question.trim().is_empty()
                            || !ids.insert(&prompt.id)
                    }) {
                        bail!("question prompts require unique identifiers and text");
                    }
                    if let Some(existing) = record
                        .questions
                        .iter_mut()
                        .find(|existing| existing.id == question.id)
                    {
                        if existing.target != question.target {
                            bail!("question source identity cannot move to another recipient");
                        }
                        if let Some(answered) =
                            record.observed_question_resolutions.get(&question.id)
                        {
                            existing.state = if *answered {
                                QuestionState::Answered
                            } else {
                                QuestionState::Resolved
                            };
                        }
                        if existing.state == QuestionState::Expired
                            && question.state == QuestionState::Pending
                        {
                            existing.state = QuestionState::Pending;
                        }
                        if question.request_id.is_some() || existing.request_id.is_none() {
                            existing.request_id = question.request_id;
                        }
                        if question.turn_id.is_some() || existing.turn_id.is_none() {
                            existing.turn_id = question.turn_id;
                        }
                        existing.prompts = question.prompts;
                    } else {
                        record.questions.push(question);
                    }
                }
                WorkUpdate::SaveAnswer {
                    question_id,
                    answers,
                } => {
                    let question = record
                        .questions
                        .iter_mut()
                        .find(|q| q.id == question_id)
                        .context("unknown question")?;
                    if answers
                        .keys()
                        .any(|id| !question.prompts.iter().any(|prompt| prompt.id == *id))
                    {
                        bail!("answer identifier does not match this question");
                    }
                    question.answers.extend(answers);
                }
                WorkUpdate::ObserveQuestionResolution {
                    question_id,
                    answered,
                } => {
                    if question_id.trim().is_empty() {
                        bail!("recorded resolution needs a source identity");
                    }
                    let known = record
                        .observed_question_resolutions
                        .entry(question_id.clone())
                        .or_default();
                    *known |= answered;
                    if let Some(question) =
                        record.questions.iter_mut().find(|q| q.id == question_id)
                    {
                        if question.state != QuestionState::Dismissed {
                            question.state = if *known {
                                QuestionState::Answered
                            } else {
                                QuestionState::Resolved
                            };
                        }
                    }
                }
                WorkUpdate::ResolveQuestion { question_id, state } => {
                    if state == QuestionState::Pending {
                        bail!("only a current runtime request can make a question pending");
                    }
                    let question = record
                        .questions
                        .iter_mut()
                        .find(|q| q.id == question_id)
                        .context("unknown question")?;
                    question.state = state;
                }
                WorkUpdate::AnswerAsFeedback {
                    question_id,
                    feedback_id,
                    mode,
                } => {
                    let question = record
                        .questions
                        .iter()
                        .find(|q| q.id == question_id)
                        .context("unknown question")?;
                    if !matches!(
                        question.state,
                        QuestionState::Expired | QuestionState::Resolved
                    ) {
                        bail!(
                            "copy-as-feedback is available when the original request is no longer pending"
                        );
                    }
                    if !matches!(mode, FeedbackMode::Guidance | FeedbackMode::Queue) {
                        bail!("choose guidance or queue for answer feedback");
                    }
                    if feedback_id.trim().is_empty()
                        || record.outbox.iter().any(|entry| entry.id == feedback_id)
                    {
                        bail!("feedback requires a new stable identifier");
                    }
                    let mut text = String::from(
                        "Answer to an earlier question (the original request is no longer pending):\n",
                    );
                    for prompt in &question.prompts {
                        if let Some(answers) = question.answers.get(&prompt.id)
                            && answers.iter().any(|answer| !answer.trim().is_empty())
                        {
                            text.push_str(&format!(
                                "{}\n{}\n",
                                prompt.question,
                                answers.join("\n")
                            ));
                        }
                    }
                    if question
                        .answers
                        .values()
                        .flatten()
                        .all(|answer| answer.trim().is_empty())
                    {
                        bail!("save an answer before sending as feedback");
                    }
                    record.outbox.push(FeedbackRecord {
                        id: feedback_id,
                        target: question.target.clone(),
                        text,
                        mode,
                        expected_turn: None,
                        source_question: Some(question_id),
                        state: DeliveryState::Draft,
                        message: Some("Saved locally; not sent to Codex".into()),
                        queue_id: None,
                    });
                }
                WorkUpdate::SaveDraft { target, text } => {
                    if target.session != *session {
                        bail!("draft recipient must belong to the selected session");
                    }
                    if let Some(draft) = record
                        .drafts
                        .iter_mut()
                        .find(|draft| draft.target == target)
                    {
                        draft.text = text;
                    } else {
                        record.drafts.push(FeedbackDraft { target, text });
                    }
                }
                WorkUpdate::ClearDraftIfMatches {
                    target,
                    expected_text,
                } => {
                    if target.session != *session {
                        bail!("draft recipient must belong to the selected session");
                    }
                    if let Some(draft) = record
                        .drafts
                        .iter_mut()
                        .find(|d| d.target == target && d.text == expected_text)
                    {
                        draft.text.clear();
                    }
                }
                WorkUpdate::Verify {
                    id,
                    evidence,
                    revision,
                } => {
                    let goal = record
                        .goal
                        .as_mut()
                        .context("progress is not tracked; add criteria first")?;
                    if revision.is_some_and(|revision| revision != goal.revision) {
                        bail!("scope changed; review current criteria before verifying");
                    }
                    if evidence.is_empty()
                        || evidence
                            .iter()
                            .any(|e| e.detail.trim().is_empty() || e.provenance.trim().is_empty())
                    {
                        bail!(
                            "verification requires recorded evidence or explicit user attestation with provenance"
                        );
                    }
                    let criterion = goal
                        .criteria
                        .iter_mut()
                        .find(|c| c.id == id)
                        .context("unknown criterion identifier")?;
                    criterion.status = CriterionStatus::Verified;
                    criterion.evidence.extend(evidence);
                }
                WorkUpdate::SetGoal {
                    title,
                    criteria,
                    provenance,
                    reason,
                } => {
                    if title.trim().is_empty() || title.len() > 512 {
                        bail!("goal title must contain 1 to 512 UTF-8 bytes");
                    }
                    if criteria.len() > 512 {
                        bail!("a goal can contain at most 512 criteria");
                    }
                    let mut ids = std::collections::BTreeSet::new();
                    for criterion in &criteria {
                        if criterion.id.trim().is_empty()
                            || criterion.label.trim().is_empty()
                            || !ids.insert(&criterion.id)
                        {
                            bail!("criteria require unique nonempty identifiers and labels");
                        }
                        if !criterion.weight.is_finite() || criterion.weight <= 0.0 {
                            bail!("criterion weight must be finite and positive");
                        }
                    }
                    if reason.trim().is_empty() || provenance.trim().is_empty() {
                        bail!("scope changes require a reason and provenance");
                    }
                    let previous = record.goal.take();
                    let revision = previous.as_ref().map_or(1, |goal| goal.revision + 1);
                    let criteria = criteria
                        .into_iter()
                        .map(|c| {
                            let old = previous.as_ref().and_then(|goal| {
                                goal.criteria
                                    .iter()
                                    .find(|old| old.id == c.id && old.label == c.label)
                            });
                            Criterion {
                                id: c.id,
                                label: c.label,
                                weight: c.weight,
                                status: old.map_or(CriterionStatus::Pending, |old| old.status),
                                evidence: old.map_or_else(Vec::new, |old| old.evidence.clone()),
                            }
                        })
                        .collect();
                    if let Some(goal) = previous {
                        record.scope_history.push(ScopeRevision {
                            goal,
                            reason,
                            changed_at: chrono::Utc::now().to_rfc3339(),
                        });
                    }
                    record.goal = Some(GoalContract {
                        title,
                        revision,
                        criteria,
                        provenance,
                    });
                }
            }
        }
        self.write(&record)?;
        Ok(record)
    }
    fn write(&self, record: &WorkRecord) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        let path = self.path(&record.session);
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let temp = path.with_extension(format!(
            "{}.{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        let result = (|| -> Result<()> {
            serde_json::to_writer(&mut file, record)?;
            file.write_all(b"\n")?;
            if file.metadata()?.len() > MAX_RECORD_BYTES {
                bail!("local goal record would exceed 8 MiB; previous state was left unchanged");
            }
            file.sync_all()?;
            fs::rename(&temp, &path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temp);
        }
        result
    }
}

const USAGE: &str = r#"zoe state [--root DIR] show PROVIDER SESSION
zoe state [--root DIR] apply PROVIDER SESSION JSON

Local Zoetrope records only; PROVIDER is codex or claude. No runtime input is
sent by these commands. apply returns a compact acknowledgment; show explicitly
reads the full record. State uses ZOETROPE_STATE_DIR, XDG_STATE_HOME/zoetrope,
or ~/.local/state/zoetrope.

JSON updates (op is required):
  set_goal: title, criteria [{id,label,weight?}], reason, provenance
  verify: id, evidence [{kind:reference|user_attestation,detail,provenance}], revision?
  set_criterion_status: id, status:pending|in_progress, reason, revision?
  import_plan: proposal {id,producer,steps:[{step,status}],source}
  adopt_plan: proposal_id, title, reason, provenance
  request_criteria: target                         save a proposal-request draft
  set_checkpoints: enabled                         per-goal preference
  save_draft: target, text
  create_feedback: id, target, text, mode:guidance|queue|stop|answer|criteria_request,
                   expected_turn?, source_question?
  record_delivery: id, state:draft|sending|queued|accepted|submitted|rejected|
                   unavailable|unknown, message?, queue_id?
  capture_question: question {id,target,request_id?,turn_id?,prompts,state,answers?}
  save_answer: question_id, answers {prompt_id:[answer,...]}
  resolve_question: question_id, state:expired|answered|answered_elsewhere|resolved|dismissed
  expire_pending_questions: target?                invalidate stored native request IDs
  answer_as_feedback: question_id, feedback_id, mode:guidance|queue
  add_document: document {id,path,producer,source,change:created|changed|deleted,changed_at?}

A target/producer is {"session":{"provider":"codex","session_id":"FULL_ID"},"agent_id":"main"}.
Use the actual child ID for a child. A prompt is {id,question,header?,options?:[{label,description?}]}.
Marking verified requires new evidence. Native plan completion alone earns no
verified credit. Queue/accepted delivery does not prove requested work is done.
Checkpoints do not change Codex policy: send the explicit proposal-request draft
to ask the selected agent to pause at milestones. Questions/drafts never submit
by themselves; expired answers can only become feedback by an explicit action.
"#;

/// Dispatch the public local command. Arguments begin after `state`.
pub fn run_cli(args: impl Iterator<Item = String>) -> Result<()> {
    let mut args = args.peekable();
    if args.peek().is_none()
        || matches!(
            args.peek().map(String::as_str),
            Some("--help" | "-h" | "help")
        )
    {
        println!("{USAGE}");
        return Ok(());
    }
    let root = if args.peek().map(String::as_str) == Some("--root") {
        args.next();
        PathBuf::from(args.next().context("--root requires a path")?)
    } else {
        Store::default_root()?
    };
    let operation = args.next().context(USAGE)?;
    let provider = args
        .next()
        .and_then(|value| Provider::parse(&value))
        .context("expected provider codex or claude")?;
    let session = SessionKey {
        provider,
        session_id: args.next().context("expected full session identity")?,
    };
    let update = match operation.as_str() {
        "show" => None,
        "apply" => Some(serde_json::from_str::<WorkUpdate>(
            &args.next().context("apply requires a JSON update")?,
        )?),
        _ => bail!("unknown local state command {operation:?}\n{USAGE}"),
    };
    if args.next().is_some() {
        bail!("unexpected state arguments\n{USAGE}");
    }
    let store = Store::new(root);
    let record = if let Some(update) = update {
        store.apply(&session, update)?
    } else {
        store.read(&session)?
    };
    if operation == "show" {
        println!(
            "{}",
            serde_json::json!({"progress":record.progress(),"record":record})
        );
    } else {
        // Agent-maintained criteria must not echo megabytes of saved drafts or
        // old evidence into the model context after every small update.
        println!(
            "{}",
            serde_json::json!({"saved":true,"session":session,"goal_revision":record.goal.as_ref().map(|goal|goal.revision),"summary":record.summary()})
        );
    }
    Ok(())
}
