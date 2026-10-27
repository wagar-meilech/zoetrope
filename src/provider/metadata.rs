//! Bounded, incremental fleet metadata. Format knowledge stays with providers.

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{FileRole, Provider, ReadMode, SessionFile, Stream};
use crate::fact::{FactKind, Statement};

const PREVIEW_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Default)]
pub(crate) struct Metadata {
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub agent: String,
    pub parent: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub goal: Option<String>,
    pub activity: super::activity::PublicActivity,
    pub last_action: Option<String>,
    pub internal: bool,
}

impl Metadata {
    pub fn matches(&self, query: &str) -> bool {
        [
            &self.title,
            &self.cwd,
            &self.name,
            &self.description,
            &self.goal,
            &self.activity.text,
        ]
        .into_iter()
        .flatten()
        .any(|text| text.to_lowercase().contains(query))
            || self.agent.to_lowercase().contains(query)
    }

    fn apply(&mut self, statement: Statement) {
        for fact in statement.facts {
            match fact.kind {
                FactKind::Title(value) => self.title = Some(value),
                FactKind::Session { label, value } if label == "cwd" => self.cwd = Some(value),
                FactKind::Agent {
                    description,
                    parent,
                    ..
                } if fact.agent.as_ref() == Some(&self.agent) => {
                    if parent.is_some() {
                        self.parent = parent;
                    }
                    if self.name.is_none() {
                        self.name = description;
                    }
                }
                FactKind::Prompt(value) => {
                    if self.goal.is_none() {
                        self.goal = Some(super::summary::truncate_summary(&value));
                    }
                    self.description = Some(super::summary::truncate_summary(&value))
                }
                FactKind::ToolStart { name, .. } => {
                    self.last_action = Some(super::activity::tool_label(&name).into())
                }
                _ => {}
            }
        }
    }
}

/// Cached classification and a metadata projection, never a full session model.
/// Unchanged files are statted only. New large histories sample their head/tail;
/// appends consume new complete records, with a bounded tail fallback for bursts.
pub(crate) struct Indexed {
    pub file: SessionFile,
    pub metadata: Metadata,
    stream: Stream,
    offset: u64,
    len: u64,
    modified: SystemTime,
    #[cfg(feature = "native")]
    pub work: super::work::WorkStream,
}

impl Indexed {
    pub fn open(provider: Provider, path: &Path) -> Option<Self> {
        let stat = std::fs::metadata(path).ok()?;
        let codex_meta = if provider == Provider::Codex {
            if !super::codex::discovery::is_rollout_file(path) {
                return None;
            }
            Some(super::codex::discovery::read_meta(path)?)
        } else {
            None
        };
        let file = if let Some(meta) = &codex_meta {
            super::codex::discovery::from_meta(
                path,
                meta,
                stat.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            )?
        } else {
            provider.session_file(path)?
        };
        let mut metadata = Metadata {
            agent: "main".into(),
            ..Metadata::default()
        };
        if provider == Provider::Claude
            && file.role != FileRole::Root
            && let Some(agent) = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix("agent-"))
                .and_then(|n| {
                    n.strip_suffix(".meta.json")
                        .or_else(|| n.strip_suffix(".jsonl"))
                })
        {
            metadata.agent = agent.to_string();
        }
        if let FileRole::Agent { parent } = &file.role {
            metadata.parent = Some(if parent == &file.session {
                "main".into()
            } else {
                parent.clone()
            });
        }
        let mut stream = provider.stream_for(&file);
        #[cfg(feature = "native")]
        let own_from = codex_meta
            .as_ref()
            .and_then(|meta| meta.subagent_history_start_ordinal);
        if let Some(meta) = codex_meta {
            metadata.cwd = meta.cwd.clone();
            metadata.internal = meta.is_internal();
            if file.role != FileRole::Root {
                metadata.agent = meta.id.clone().unwrap_or_default();
                metadata.name = if metadata.internal {
                    Some("approval review".into())
                } else {
                    meta.nickname().or_else(|| meta.path()).map(str::to_owned)
                };
            }
            // A real session_meta can itself exceed the preview budget because
            // it embeds instructions. Seed from its parsed header, not a cut line.
            if let Stream::Codex(parser) = &mut stream
                && let Some(statement) = parser.push_line(&super::codex::wire::Line {
                    timestamp: None,
                    ordinal: None,
                    payload: super::codex::wire::Payload::SessionMeta(Box::new(meta)),
                })
            {
                metadata.apply(statement);
            }
        }
        #[cfg(feature = "native")]
        let work = super::work::WorkStream::new(
            file.provider,
            file.session.clone(),
            metadata.agent.clone(),
            metadata.cwd.clone(),
            own_from,
        );
        let mut indexed = Self {
            file,
            metadata,
            stream,
            offset: 0,
            len: stat.len(),
            modified: stat.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            #[cfg(feature = "native")]
            work,
        };
        indexed.read_from(0, false);
        if indexed.len > PREVIEW_BYTES {
            indexed.read_from(indexed.len.saturating_sub(PREVIEW_BYTES), true);
        }
        Some(indexed)
    }

    pub fn refresh(&mut self) {
        let Ok(stat) = std::fs::metadata(&self.file.path) else {
            return;
        };
        let modified = stat.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if stat.len() == self.len && modified == self.modified {
            return;
        }
        if stat.len() <= self.len {
            if let Some(replacement) = Self::open(self.file.provider, &self.file.path) {
                *self = replacement;
            }
            return;
        }
        self.len = stat.len();
        self.modified = modified;
        self.file.modified = modified;
        if self.len.saturating_sub(self.offset) > PREVIEW_BYTES {
            self.read_from(self.len.saturating_sub(PREVIEW_BYTES), true);
        } else {
            self.read_from(self.offset, false);
        }
    }

    fn read_from(&mut self, start: u64, skip_partial: bool) {
        let Ok(mut input) = std::fs::File::open(&self.file.path) else {
            return;
        };
        if input.seek(SeekFrom::Start(start)).is_err() {
            return;
        }
        let mut bytes = Vec::new();
        if input.take(PREVIEW_BYTES).read_to_end(&mut bytes).is_err() {
            return;
        }
        if self.file.read == ReadMode::Whole {
            if let Ok(text) = std::str::from_utf8(&bytes)
                && let Some(statement) = self.file.provider.sidecar(&self.file, text)
            {
                self.metadata.apply(statement);
            }
            self.offset = self.len;
            return;
        }
        let begin = if skip_partial || (start == 0 && self.file.provider == Provider::Codex) {
            let Some(newline) = bytes.iter().position(|b| *b == b'\n') else {
                return;
            };
            newline + 1
        } else {
            0
        };
        let Some(end) = bytes.iter().rposition(|b| *b == b'\n') else {
            return;
        };
        self.offset = start + end as u64 + 1;
        if begin > end {
            return;
        }
        for line in bytes[begin..=end].split(|b| *b == b'\n') {
            if let Ok(line) = std::str::from_utf8(line) {
                #[cfg(feature = "native")]
                self.work.push(line);
                if let Some(statement) = self.stream.push(line) {
                    self.metadata.apply(statement);
                    self.metadata.activity.observe(self.file.provider, line);
                }
            }
        }
    }
}

pub(crate) fn refresh_index(
    index: &mut std::collections::BTreeMap<PathBuf, Indexed>,
    provider: Provider,
    path: PathBuf,
) {
    if let Some(indexed) = index.get_mut(&path) {
        indexed.refresh();
    } else if let Some(indexed) = Indexed::open(provider, &path) {
        index.insert(path, indexed);
    }
}

/// Codex's compact native name index, read once and then from its append edge.
#[derive(Default)]
pub(crate) struct Titles {
    pub names: std::collections::BTreeMap<String, String>,
    offset: u64,
    len: u64,
    modified: Option<SystemTime>,
}

impl Titles {
    pub fn refresh(&mut self, home: &Path) {
        use std::io::{BufRead, BufReader};
        let path = home.join("session_index.jsonl");
        let Ok(stat) = std::fs::metadata(&path) else {
            *self = Self::default();
            return;
        };
        let modified = stat.modified().ok();
        if self.modified == modified && self.len == stat.len() {
            return;
        }
        if stat.len() <= self.len {
            self.names.clear();
            self.offset = 0;
        }
        self.len = stat.len();
        self.modified = modified;
        let Ok(mut file) = std::fs::File::open(path) else {
            return;
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return;
        }
        let mut input = BufReader::new(file);
        let mut line = String::new();
        loop {
            line.clear();
            let Ok(read) = input.read_line(&mut line) else {
                break;
            };
            if read == 0 || !line.ends_with('\n') {
                break;
            }
            self.offset += read as u64;
            #[derive(serde::Deserialize)]
            struct Name {
                id: String,
                thread_name: String,
            }
            if let Ok(name) = serde_json::from_str::<Name>(&line) {
                self.names.insert(name.id, name.thread_name);
            }
        }
    }
}
