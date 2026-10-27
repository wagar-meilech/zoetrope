//! Structured work recorded by supported transcript versions. This projection
//! reuses the inventory's bounded bytes; it never scans a history itself.

use crate::console::{AgentKey, SessionKey};
use crate::provider::Provider;
use crate::workstate::{
    DocumentChange, DocumentRecord, PlanProposal, PlanStep, QuestionOption, QuestionPrompt,
    QuestionRecord, QuestionState, WorkUpdate,
};
use serde_json::Value;
use std::path::PathBuf;

pub(crate) struct WorkStream {
    provider: Provider,
    producer: AgentKey,
    cwd: Option<PathBuf>,
    own_from: Option<u64>,
    pending: Vec<WorkUpdate>,
}
impl WorkStream {
    pub fn new(
        provider: Provider,
        session_id: String,
        agent_id: String,
        cwd: Option<String>,
        own_from: Option<u64>,
    ) -> Self {
        Self {
            provider,
            producer: AgentKey {
                session: SessionKey {
                    provider,
                    session_id,
                },
                agent_id,
            },
            cwd: cwd.map(PathBuf::from),
            own_from,
            pending: Vec::new(),
        }
    }
    pub fn take(&mut self) -> Vec<WorkUpdate> {
        std::mem::take(&mut self.pending)
    }
    pub fn push(&mut self, line: &str) {
        if self.provider != Provider::Codex
            || ![
                "update_plan",
                "request_user_input",
                "FileChange",
                "fileChange",
                "function_call_output",
            ]
            .iter()
            .any(|needle| line.contains(needle))
        {
            return;
        }
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            return;
        };
        if let (Some(ordinal), Some(own_from)) = (record["ordinal"].as_u64(), self.own_from)
            && ordinal < own_from
        {
            return;
        }
        let payload = &record["payload"];
        if record["type"] == "response_item" && payload["type"] == "function_call" {
            let Some(call) = payload["call_id"].as_str() else {
                return;
            };
            let Some(arguments) = payload["arguments"]
                .as_str()
                .and_then(|text| serde_json::from_str::<Value>(text).ok())
            else {
                return;
            };
            match payload["name"].as_str() {
                Some("update_plan" | "functions.update_plan") => {
                    let Some(steps) = arguments.get("plan").and_then(|value| {
                        serde_json::from_value::<Vec<PlanStep>>(value.clone()).ok()
                    }) else {
                        return;
                    };
                    if steps.is_empty() {
                        return;
                    }
                    self.pending.push(WorkUpdate::ImportPlan {
                        proposal: PlanProposal {
                            id: format!("{}:{call}", self.producer.agent_id),
                            producer: self.producer.clone(),
                            steps,
                            source: "Codex recorded plan; statuses are not verification evidence"
                                .into(),
                            recorded_at: record["timestamp"]
                                .as_str()
                                .filter(|value| chrono::DateTime::parse_from_rfc3339(value).is_ok())
                                .map(str::to_owned),
                        },
                    });
                }
                Some("request_user_input_async" | "functions.request_user_input_async") => {
                    let Some(questions) = arguments["questions"].as_array() else {
                        return;
                    };
                    let mut prompts = Vec::new();
                    for (at, question) in questions.iter().enumerate() {
                        let Some(title) =
                            question["title"].as_str().filter(|s| !s.trim().is_empty())
                        else {
                            return;
                        };
                        let options = match question.get("options") {
                            None | Some(Value::Null) => Vec::new(),
                            Some(Value::Array(options)) => {
                                let Some(options) = options
                                    .iter()
                                    .map(|option| {
                                        option.as_str().map(|label| QuestionOption {
                                            label: label.into(),
                                            description: String::new(),
                                        })
                                    })
                                    .collect::<Option<Vec<_>>>()
                                else {
                                    return;
                                };
                                options
                            }
                            _ => return,
                        };
                        prompts.push(QuestionPrompt {
                            id: format!("async-prompt-{}", at + 1),
                            header: format!("Question {}", at + 1),
                            question: title.into(),
                            options,
                            is_other: true,
                            is_secret: false,
                        });
                    }
                    if prompts.is_empty() {
                        return;
                    }
                    self.pending.push(WorkUpdate::CaptureQuestion {
                        question: QuestionRecord {
                            id: format!("transcript:{}:{call}", self.producer.agent_id),
                            target: self.producer.clone(),
                            request_id: None,
                            turn_id: None,
                            prompts,
                            state: QuestionState::Expired,
                            answers: Default::default(),
                        },
                    });
                }
                Some("request_user_input" | "functions.request_user_input") => {
                    let Some(prompts) = arguments
                        .get("questions")
                        .and_then(|value| serde_json::from_value(value.clone()).ok())
                    else {
                        return;
                    };
                    self.pending.push(WorkUpdate::CaptureQuestion {
                        question: QuestionRecord {
                            id: format!("transcript:{}:{call}", self.producer.agent_id),
                            target: self.producer.clone(),
                            request_id: None,
                            turn_id: None,
                            prompts,
                            state: QuestionState::Expired,
                            answers: Default::default(),
                        },
                    });
                }
                _ => {}
            }
        }
        if record["type"] == "response_item" && payload["type"] == "function_call_output" {
            if let Some(call) = payload["call_id"].as_str() {
                let output = payload["output"]
                    .as_str()
                    .and_then(|text| serde_json::from_str::<Value>(text).ok())
                    .unwrap_or_else(|| payload["output"].clone());
                if let Some(answers) = output["answers"].as_object() {
                    let answered = answers.values().any(|value| {
                        value["answers"].as_array().is_some_and(|a| {
                            a.iter()
                                .any(|v| v.as_str().is_some_and(|s| !s.trim().is_empty()))
                        })
                    });
                    self.pending.push(WorkUpdate::ObserveQuestionResolution {
                        question_id: format!("transcript:{}:{call}", self.producer.agent_id),
                        answered,
                    });
                }
            }
        }
        if record["type"] == "event_msg" && payload["type"] == "item_completed" {
            let item = &payload["item"];
            if !matches!(item["type"].as_str(), Some("FileChange" | "fileChange"))
                || item["status"] != "completed"
            {
                return;
            }
            let Some(call) = item["id"].as_str() else {
                return;
            };
            let Some(changes) = item["changes"].as_object() else {
                return;
            };
            for (path, kind) in changes {
                let path = PathBuf::from(path);
                if !matches!(
                    path.extension()
                        .and_then(|extension| extension.to_str())
                        .map(str::to_ascii_lowercase)
                        .as_deref(),
                    Some("md" | "markdown")
                ) {
                    continue;
                }
                let path = if path.is_absolute() {
                    path
                } else if let Some(cwd) = &self.cwd {
                    cwd.join(path)
                } else {
                    continue;
                };
                let change = match kind["type"].as_str().or_else(|| kind["kind"].as_str()) {
                    Some("add") => DocumentChange::Created,
                    Some("delete") => DocumentChange::Deleted,
                    _ => DocumentChange::Changed,
                };
                self.pending.push(WorkUpdate::AddDocument {
                    document: DocumentRecord {
                        id: format!("{}:{call}", self.producer.agent_id),
                        path,
                        producer: self.producer.clone(),
                        source: "Codex completed file change".into(),
                        change,
                        changed_at: record["timestamp"].as_str().map(str::to_owned),
                    },
                });
            }
        }
    }
}
