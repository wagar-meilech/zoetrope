//! Public narration and explicitly recorded shell input. This projection never
//! consumes reasoning/thinking facts, encrypted content, or code strings.
use super::Provider;
use serde_json::Value;
#[derive(Debug, Clone, Default)]
pub(crate) struct PublicActivity {
    pub text: Option<String>,
    pub text_at: Option<String>,
    pub text_source: Option<String>,
    pub command: Option<String>,
    pub command_at: Option<String>,
    pub purpose: Option<String>,
}
fn short(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect()
}
fn detail_text(text: &str) -> String {
    let mut end = text.len().min(8 * 1024);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let mut value = text[..end].to_owned();
    if end < text.len() {
        value.push_str("\n\n… Message display limited to 8 KiB; more remains in the recording.");
    }
    value
}
fn recorded_at(record: &Value) -> Option<String> {
    record["timestamp"]
        .as_str()
        .filter(|value| chrono::DateTime::parse_from_rfc3339(value).is_ok())
        .map(str::to_owned)
}
impl PublicActivity {
    /// Returns whether this record supplies a public message.
    pub fn observe(&mut self, provider: Provider, line: &str) -> bool {
        self.observe_mode(provider, line, false)
    }
    #[cfg(feature = "native")]
    pub fn detail(provider: Provider, line: &str) -> Option<String> {
        let mut activity = Self::default();
        activity.observe_mode(provider, line, true);
        activity.text
    }
    fn observe_mode(&mut self, provider: Provider, line: &str, detailed: bool) -> bool {
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            return false;
        };
        let mut message = false;
        match provider {
            Provider::Codex => {
                let p = &record["payload"];
                if record["type"] == "response_item"
                    && p["type"] == "message"
                    && p["role"] == "assistant"
                    && match p["channel"].as_str() {
                        Some(channel) => matches!(channel, "commentary" | "final" | "final_answer"),
                        None => matches!(p["phase"].as_str(), Some("commentary" | "final_answer")),
                    }
                    && let Some(parts) = p["content"].as_array()
                {
                    let text = parts
                        .iter()
                        .filter(|part| part["type"] == "output_text")
                        .filter_map(|part| part["text"].as_str())
                        .collect::<Vec<_>>()
                        .join(if detailed { "\n\n" } else { " " });
                    if !text.trim().is_empty() {
                        self.text = Some(if detailed {
                            detail_text(&text)
                        } else {
                            short(&text)
                        });
                        message = true;
                        self.text_at = recorded_at(&record);
                        self.text_source = Some(
                            if p["channel"] == "commentary" || p["phase"] == "commentary" {
                                "Codex commentary"
                            } else {
                                "Codex final answer"
                            }
                            .into(),
                        );
                    }
                }
                if record["type"] == "response_item"
                    && p["type"] == "function_call"
                    && matches!(
                        p["name"].as_str(),
                        Some("exec_command" | "functions.exec_command" | "Bash" | "bash" | "shell")
                    )
                    && let Some(input) = p["arguments"]
                        .as_str()
                        .and_then(|s| serde_json::from_str::<Value>(s).ok())
                {
                    self.shell(&input, recorded_at(&record));
                }
                if record["type"] == "event_msg"
                    && matches!(p["type"].as_str(), Some("item_started" | "item_completed"))
                    && matches!(
                        p["item"]["type"].as_str(),
                        Some("CommandExecution" | "commandExecution")
                    )
                {
                    let item = &p["item"];
                    let command = item["command"].as_str().map(str::to_owned).or_else(|| {
                        item["command"].as_array().map(|a| {
                            a.iter()
                                .filter_map(Value::as_str)
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                    });
                    if let Some(command) = command {
                        self.command = Some(command.chars().take(2048).collect());
                        self.command_at = recorded_at(&record);
                        self.purpose = item["description"].as_str().map(short);
                    }
                }
            }
            Provider::Claude => {
                if record["type"] == "assistant"
                    && let Some(parts) = record["message"]["content"].as_array()
                {
                    let text = parts
                        .iter()
                        .filter(|part| part["type"] == "text")
                        .filter_map(|part| part["text"].as_str())
                        .collect::<Vec<_>>()
                        .join(if detailed { "\n\n" } else { " " });
                    if !text.trim().is_empty() {
                        self.text = Some(if detailed {
                            detail_text(&text)
                        } else {
                            short(&text)
                        });
                        self.text_at = recorded_at(&record);
                        self.text_source = Some("Claude assistant message".into());
                        message = true;
                    }
                    for part in parts {
                        if part["type"] == "tool_use" && part["name"] == "Bash" {
                            self.shell(&part["input"], recorded_at(&record));
                        }
                    }
                }
            }
        }
        message
    }
    fn shell(&mut self, input: &Value, recorded_at: Option<String>) {
        if let Some(command) = input["cmd"].as_str().or_else(|| input["command"].as_str()) {
            self.command = Some(command.chars().take(2048).collect());
            self.command_at = recorded_at;
            self.purpose = input["description"]
                .as_str()
                .or_else(|| input["justification"].as_str())
                .map(short);
        }
    }
}
pub(crate) fn tool_label(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
        .as_str()
    {
        "read" | "read_file" | "open_file" => "Read files",
        "write" | "write_file" | "edit" | "edit_file" | "apply_patch" => "Update files",
        "grep" | "rg" | "glob" | "find" => "Find files or text",
        "bash" | "shell" | "exec_command" => "Run a shell command",
        "update_plan" => "Update the plan",
        "request_user_input" | "request_user_input_async" => "Ask a question",
        "spawn_agent" | "agent" => "Delegate a task",
        "wait" | "wait_agent" => "Wait for related work",
        _ => "Run a tool",
    }
}
