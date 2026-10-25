//! Fleet discovery and navigation above the single-session replay application.
//!
//! Reading the fleet never opens a model connection or changes native history.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::SystemTime;

use ratatui::text::Text;

use crate::provider::{self, FileRole, Provider, Scope, Session};

/// Local storage roots. `None` disables a provider, useful for isolated homes.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryRoots {
    pub claude_projects: Option<PathBuf>,
    pub codex_home: Option<PathBuf>,
}

impl DiscoveryRoots {
    /// The installed providers' configured homes.
    pub fn local() -> Self {
        Self {
            claude_projects: crate::provider::claude::discovery::claude_projects_root(),
            codex_home: crate::provider::codex::discovery::codex_home(),
        }
    }
}

/// Provider-qualified session identity, independent of path or display label.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct SessionKey {
    pub provider: Provider,
    pub session_id: String,
}

/// An agent's canonical fact identity within one provider-qualified session.
/// The root is `main`; its native thread identity is `session.session_id`.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct AgentKey {
    pub session: SessionKey,
    pub agent_id: String,
}

/// Navigation shared by terminal and other local frontends.
#[derive(Debug, Clone)]
pub enum ConsoleCommand {
    ToggleHistory,
    ToggleExpanded(SessionKey),
    ToggleInternal,
    Select(FleetNodeId),
    Search(String),
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum FleetNodeId {
    Project(String),
    Session(SessionKey),
    Agent(AgentKey),
}

/// A visible graph node. Parent links are graph edges; the list is in display order.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FleetNode {
    pub id: FleetNodeId,
    pub parent: Option<FleetNodeId>,
    pub label: String,
    pub description: Option<String>,
    pub internal: bool,
    pub archived: bool,
    pub expanded: bool,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub activity: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FleetGraph {
    pub nodes: Vec<FleetNode>,
}

/// An authoritative live observation, separate from transcript recency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveActivity {
    Working,
    WaitingForAnswer,
    WaitingForApproval,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveThread {
    pub name: Option<String>,
    pub project: Option<String>,
    pub activity: LiveActivity,
}

/// A local fleet console. Discovery is explicit, so construction performs no IO.
pub struct Console {
    roots: DiscoveryRoots,
    sessions: Vec<Session>,
    metadata: std::collections::BTreeMap<PathBuf, provider::metadata::Indexed>,
    expanded: BTreeSet<SessionKey>,
    show_internal: bool,
    history: bool,
    now: SystemTime,
    selection: Option<FleetNodeId>,
    search: String,
    titles: provider::metadata::Titles,
    graph: FleetGraph,
    live_threads: BTreeMap<String, LiveThread>,
    #[cfg(feature = "native")]
    work_store: Option<crate::workstate::Store>,
    #[cfg(feature = "native")]
    work: BTreeMap<SessionKey, crate::workstate::WorkRecord>,
    #[cfg(feature = "native")]
    work_versions: BTreeMap<SessionKey, Option<(u64, SystemTime)>>,
    #[cfg(feature = "native")]
    work_errors: Vec<String>,
    #[cfg(feature = "native")]
    work_summaries: BTreeMap<SessionKey, crate::workstate::WorkSummary>,
    #[cfg(feature = "native")]
    work_order: std::collections::VecDeque<SessionKey>,
    #[cfg(feature = "native")]
    work_sizes: BTreeMap<SessionKey, u64>,
    #[cfg(feature = "native")]
    work_search_matches: BTreeSet<SessionKey>,
}

impl Console {
    pub fn new(roots: DiscoveryRoots) -> Self {
        Self {
            roots,
            sessions: Vec::new(),
            metadata: Default::default(),
            expanded: Default::default(),
            show_internal: false,
            history: false,
            now: SystemTime::UNIX_EPOCH,
            selection: None,
            search: String::new(),
            titles: Default::default(),
            graph: Default::default(),
            live_threads: Default::default(),
            #[cfg(feature = "native")]
            work_store: None,
            #[cfg(feature = "native")]
            work: Default::default(),
            #[cfg(feature = "native")]
            work_versions: Default::default(),
            #[cfg(feature = "native")]
            work_errors: Default::default(),
            #[cfg(feature = "native")]
            work_summaries: Default::default(),
            #[cfg(feature = "native")]
            work_order: Default::default(),
            #[cfg(feature = "native")]
            work_sizes: Default::default(),
            #[cfg(feature = "native")]
            work_search_matches: Default::default(),
        }
    }

    /// Enable durable state. Construction performs no IO; refresh/update run on
    /// the caller's inventory worker. Native transcripts remain read-only.
    #[cfg(feature = "native")]
    pub fn with_state(roots: DiscoveryRoots, state_root: PathBuf) -> Self {
        let mut console = Self::new(roots);
        console.work_store = Some(crate::workstate::Store::new(state_root));
        console
    }

    /// Cached full work, without filesystem access. Large inventories evict
    /// older full records; use `load_work` on the worker before opening detail.
    #[cfg(feature = "native")]
    pub fn work(&self, key: &SessionKey) -> Option<&crate::workstate::WorkRecord> {
        self.work.get(key)
    }

    /// Compact cached graph projection, always available after refresh.
    #[cfg(feature = "native")]
    pub fn work_summary(&self, key: &SessionKey) -> Option<&crate::workstate::WorkSummary> {
        self.work_summaries.get(key)
    }

    /// Load selected work on an inventory worker. Full records use a bounded
    /// cache: 8 MiB encoded bytes/32 entries, plus a pinned selected record.
    #[cfg(feature = "native")]
    pub fn load_work(&mut self, key: &SessionKey) -> anyhow::Result<&crate::workstate::WorkRecord> {
        if !self.work.contains_key(key) {
            let store = self
                .work_store
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("local state is not enabled"))?;
            let record = store.read(key)?;
            let stamp = store.stamp(key);
            self.cache_work(key.clone(), record, stamp);
        }
        Ok(&self.work[key])
    }

    #[cfg(feature = "native")]
    fn cache_work(
        &mut self,
        key: SessionKey,
        record: crate::workstate::WorkRecord,
        stamp: Option<(u64, SystemTime)>,
    ) {
        let size = stamp.map_or(256, |stamp| stamp.0.max(256));
        self.work_summaries.insert(key.clone(), record.summary());
        if !self.search.is_empty() {
            if record.matches(&self.search) {
                self.work_search_matches.insert(key.clone());
            } else {
                self.work_search_matches.remove(&key);
            }
        }
        self.work_versions.insert(key.clone(), stamp);
        self.work_sizes.insert(key.clone(), size);
        self.work_order.retain(|old| old != &key);
        self.work_order.push_back(key.clone());
        self.work.insert(key.clone(), record);
        let pinned = match &self.selection {
            Some(FleetNodeId::Session(key)) => Some(key.clone()),
            Some(FleetNodeId::Agent(agent)) => Some(agent.session.clone()),
            _ => None,
        };
        let mut attempts = self.work_order.len();
        while attempts > 0
            && (self.work.len() > 32 || self.work_sizes.values().sum::<u64>() > 8 * 1024 * 1024)
        {
            attempts -= 1;
            let Some(old) = self.work_order.pop_front() else {
                break;
            };
            if old == key || Some(&old) == pinned.as_ref() {
                self.work_order.push_back(old);
                continue;
            }
            self.work.remove(&old);
            self.work_sizes.remove(&old);
        }
    }

    #[cfg(feature = "native")]
    fn search_work(&mut self) {
        self.work_search_matches.clear();
        if self.search.is_empty() {
            return;
        }
        let Some(store) = self.work_store.clone() else {
            return;
        };
        // Explicit searches run on the worker. Uncached records are read one
        // at a time and immediately dropped; scope history never joins the index.
        for key in self.work_summaries.keys() {
            let cached = self.work.get(key);
            let loaded;
            let record = if let Some(record) = cached {
                record
            } else {
                match store.read(key) {
                    Ok(record) => {
                        loaded = record;
                        &loaded
                    }
                    Err(error) => {
                        self.work_errors.push(error.to_string());
                        continue;
                    }
                }
            };
            if record.matches(&self.search) {
                self.work_search_matches.insert(key.clone());
            }
        }
    }

    #[cfg(feature = "native")]
    pub fn work_errors(&self) -> &[String] {
        &self.work_errors
    }

    /// Persist one explicit edit and update the cached projection. Blocking IO.
    #[cfg(feature = "native")]
    pub fn update_work(
        &mut self,
        key: &SessionKey,
        update: crate::workstate::WorkUpdate,
    ) -> anyhow::Result<&crate::workstate::WorkRecord> {
        let store = self
            .work_store
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("local state is not enabled"))?;
        let record = store.apply(key, update)?;
        let stamp = store.stamp(key);
        self.cache_work(key.clone(), record, stamp);
        self.graph = self.build_graph();
        Ok(&self.work[key])
    }

    #[cfg(feature = "native")]
    fn refresh_work(&mut self) {
        let Some(store) = self.work_store.clone() else {
            return;
        };
        self.work_errors.clear();
        let keys: Vec<_> = self
            .sessions
            .iter()
            .map(|session| SessionKey {
                provider: session.provider,
                session_id: session.id.clone(),
            })
            .collect();
        for key in keys {
            let stamp = store.stamp(&key);
            if self.work_versions.get(&key) == Some(&stamp) {
                continue;
            }
            match store.read(&key) {
                Ok(record) => {
                    self.cache_work(key, record, stamp);
                }
                Err(error) => self.work_errors.push(error.to_string()),
            }
        }
    }

    #[cfg(feature = "native")]
    fn import_recorded_work(&mut self) {
        let Some(store) = self.work_store.clone() else {
            for indexed in self.metadata.values_mut() {
                let _ = indexed.work.take();
            }
            return;
        };
        let mut updates: BTreeMap<SessionKey, Vec<crate::workstate::WorkUpdate>> = BTreeMap::new();
        for indexed in self.metadata.values_mut() {
            let observed = indexed.work.take();
            if observed.is_empty() {
                continue;
            }
            let key = SessionKey {
                provider: indexed.file.provider,
                session_id: indexed.file.session.clone(),
            };
            updates.entry(key).or_default().extend(observed);
        }
        for (key, updates) in updates {
            match store.apply_many(&key, updates) {
                Ok(record) => {
                    let stamp = store.stamp(&key);
                    self.cache_work(key, record, stamp);
                }
                Err(error) => self.work_errors.push(error.to_string()),
            }
        }
    }

    /// Apply cached authoritative status without filesystem or runtime IO.
    pub fn set_live_threads(&mut self, threads: BTreeMap<String, LiveThread>) {
        if self.live_threads != threads {
            self.live_threads = threads;
            self.graph = self.build_graph();
        }
    }
    fn live_rank(&self, session: &Session) -> u8 {
        if session.provider != Provider::Codex {
            return 3;
        }
        let rank = |id: &str| {
            self.live_threads
                .get(id)
                .map(|live| match live.activity {
                    LiveActivity::Working => 1,
                    _ => 0,
                })
                .unwrap_or(3)
        };
        session
            .every_file()
            .filter_map(|f| self.metadata.get(&f.path))
            .fold(rank(&session.id), |best, indexed| {
                best.min(rank(&indexed.metadata.agent))
            })
    }
    fn needs_attention(&self, key: &SessionKey) -> bool {
        #[cfg(feature = "native")]
        {
            self.work_summary(key)
                .is_some_and(|s| s.pending_questions > 0 || s.unfinished_drafts)
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = key;
            false
        }
    }

    /// Cached provider-recorded names for producer/recipient labels, including
    /// known children of collapsed goals. No transcript loading is performed.
    pub fn agent_names(&self) -> BTreeMap<AgentKey, String> {
        let mut names = BTreeMap::new();
        for indexed in self.metadata.values() {
            if let Some(name) = &indexed.metadata.name {
                names.insert(
                    AgentKey {
                        session: SessionKey {
                            provider: indexed.file.provider,
                            session_id: indexed.file.session.clone(),
                        },
                        agent_id: indexed.metadata.agent.clone(),
                    },
                    name.clone(),
                );
            }
        }
        names
    }

    /// Cached transcript sources, independent of the current selection/filter.
    pub fn session_sources(&self) -> BTreeMap<SessionKey, PathBuf> {
        self.sessions
            .iter()
            .map(|session| {
                (
                    SessionKey {
                        provider: session.provider,
                        session_id: session.id.clone(),
                    },
                    session.root.path.clone(),
                )
            })
            .collect()
    }

    /// Refresh local inventory. Missing homes and malformed files are skipped.
    pub fn refresh(&mut self, now: SystemTime) {
        self.now = now;
        if let Some(home) = &self.roots.codex_home {
            self.titles.refresh(home);
        }
        let mut files = Vec::new();
        let mut seen = BTreeSet::new();
        for (provider, root) in [
            (Provider::Claude, &self.roots.claude_projects),
            (Provider::Codex, &self.roots.codex_home),
        ] {
            if let Some(root) = root {
                for path in provider.all_paths_in(root, &Scope::ALL) {
                    seen.insert(path.clone());
                    provider::metadata::refresh_index(&mut self.metadata, provider, path.clone());
                    if let Some(indexed) = self.metadata.get(&path) {
                        files.push(indexed.file.clone());
                    }
                }
            }
        }
        // Once a lazily discovered child has a name, keep that compact index
        // searchable while its root still exists, even when the goal is closed.
        let roots: BTreeSet<_> = files
            .iter()
            .filter(|file| file.role == FileRole::Root)
            .map(|file| (file.provider, file.session.clone()))
            .collect();
        let known: Vec<_> = self
            .metadata
            .values()
            .filter(|indexed| {
                indexed.file.provider == Provider::Claude
                    && indexed.file.role != FileRole::Root
                    && roots.contains(&(indexed.file.provider, indexed.file.session.clone()))
                    && !seen.contains(&indexed.file.path)
            })
            .map(|indexed| indexed.file.path.clone())
            .collect();
        for path in known {
            if !path.is_file() {
                continue;
            }
            seen.insert(path.clone());
            provider::metadata::refresh_index(&mut self.metadata, Provider::Claude, path.clone());
            if let Some(indexed) = self.metadata.get(&path) {
                files.push(indexed.file.clone());
            }
        }
        for file in files
            .clone()
            .into_iter()
            .filter(|file| file.provider == Provider::Claude && file.role == FileRole::Root)
        {
            let key = SessionKey {
                provider: file.provider,
                session_id: file.session.clone(),
            };
            if !self.expanded.contains(&key) {
                continue;
            }
            for path in file.provider.related_paths(&file) {
                if !seen.insert(path.clone()) {
                    continue;
                }
                provider::metadata::refresh_index(&mut self.metadata, file.provider, path.clone());
                if let Some(indexed) = self.metadata.get(&path) {
                    files.push(indexed.file.clone());
                }
            }
        }
        self.metadata.retain(|path, _| seen.contains(path));
        self.sessions = provider::assemble(files);
        #[cfg(feature = "native")]
        self.refresh_work();
        #[cfg(feature = "native")]
        self.import_recorded_work();
        self.graph = self.build_graph();
    }

    pub fn command(&mut self, command: ConsoleCommand) {
        match command {
            ConsoleCommand::ToggleHistory => self.history = !self.history,
            ConsoleCommand::ToggleInternal => self.show_internal = !self.show_internal,
            ConsoleCommand::Search(query) => {
                self.search = query.trim().to_lowercase();
                #[cfg(feature = "native")]
                self.search_work();
            }
            ConsoleCommand::Select(id) => {
                if self.graph().nodes.iter().any(|node| node.id == id) {
                    self.selection = Some(id);
                }
                return;
            }
            ConsoleCommand::ToggleExpanded(key) => {
                if !self.expanded.remove(&key) {
                    self.expanded.insert(key);
                    self.refresh(self.now);
                    return;
                }
            }
        }
        self.graph = self.build_graph();
    }

    /// Selection stays pinned even while a filter temporarily hides its node.
    pub fn selection(&self) -> Option<&FleetNodeId> {
        self.selection.as_ref()
    }

    /// The selected goal's source files for the existing single-session viewer.
    pub fn selected_session(&self) -> Option<&Session> {
        let key = match self.selection.as_ref()? {
            FleetNodeId::Session(key) => key,
            FleetNodeId::Agent(key) => &key.session,
            FleetNodeId::Project(_) => return None,
        };
        self.sessions
            .iter()
            .find(|session| session.provider == key.provider && session.id == key.session_id)
    }

    /// Cached visible nodes and parent edges. Borrowing the graph does no IO or
    /// reconstruction, so frontends can render frames without scanning history.
    pub fn graph(&self) -> &FleetGraph {
        &self.graph
    }

    fn work_title(&self, key: &SessionKey) -> Option<String> {
        #[cfg(feature = "native")]
        {
            self.work_summary(key)
                .and_then(|summary| summary.title.clone())
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = key;
            None
        }
    }

    fn work_matches(&self, key: &SessionKey, query: &str) -> bool {
        #[cfg(feature = "native")]
        {
            let _ = query;
            self.work_search_matches.contains(key)
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = (key, query);
            false
        }
    }

    fn build_graph(&self) -> FleetGraph {
        let mut graph = FleetGraph::default();
        let mut projects = BTreeSet::new();
        let mut ordered: Vec<_> = self
            .sessions
            .iter()
            .map(|session| (self.live_rank(session), session))
            .collect();
        ordered.sort_by_key(|(rank, _)| *rank);
        for (live_rank, session) in ordered {
            let archived = session
                .root
                .path
                .components()
                .any(|c| c.as_os_str() == "archived_sessions");
            let recent = self
                .now
                .duration_since(session.last_modified)
                .unwrap_or_default()
                .as_secs()
                < 120;
            let key = SessionKey {
                provider: session.provider,
                session_id: session.id.clone(),
            };
            if !self.history
                && (archived || !recent)
                && live_rank > 1
                && !self.needs_attention(&key)
            {
                continue;
            }
            let mut metadata = self.metadata[&session.root.path].metadata.clone();
            if session.provider == Provider::Codex
                && let Some(name) = self.titles.names.get(&session.id)
            {
                metadata.title = Some(name.clone());
            }
            let project = metadata
                .cwd
                .as_ref()
                .unwrap_or(&session.project_key)
                .clone();
            let matching_child = !self.search.is_empty()
                && session.files.iter().any(|file| {
                    let metadata = &self.metadata[&file.path].metadata;
                    (!metadata.internal || self.show_internal) && metadata.matches(&self.search)
                });
            if !self.search.is_empty()
                && !project.to_lowercase().contains(&self.search)
                && !session.id.to_lowercase().contains(&self.search)
                && !metadata.matches(&self.search)
                && !matching_child
                && !self.work_matches(&key, &self.search)
            {
                continue;
            }
            if projects.insert(project.clone()) {
                graph.nodes.push(FleetNode {
                    id: FleetNodeId::Project(project.clone()),
                    parent: None,
                    label: project.clone(),
                    description: None,
                    internal: false,
                    archived: false,
                    expanded: true,
                    goal: None,
                    activity: None,
                    command: None,
                    purpose: None,
                });
            }
            let expanded = self.expanded.contains(&key) || matching_child;
            graph.nodes.push(FleetNode {
                id: FleetNodeId::Session(key.clone()),
                parent: Some(FleetNodeId::Project(project)),
                label: self
                    .work_title(&key)
                    .or_else(|| metadata.title.clone())
                    .unwrap_or_else(|| session.id.clone()),
                description: metadata.description.clone(),
                internal: false,
                archived,
                expanded,
                goal: metadata.goal.clone(),
                activity: metadata
                    .activity
                    .text
                    .clone()
                    .or(metadata.last_action.clone()),
                command: metadata.activity.command.clone(),
                purpose: metadata.activity.purpose.clone(),
            });
            if expanded {
                let sidecars: BTreeMap<_, _> = session
                    .files
                    .iter()
                    .filter(|f| f.role == FileRole::Sidecar)
                    .map(|file| {
                        let extra = &self.metadata[&file.path].metadata;
                        (extra.agent.as_str(), extra)
                    })
                    .collect();
                for file in session.every_file().filter(|f| f.role != FileRole::Sidecar) {
                    let mut metadata = self.metadata[&file.path].metadata.clone();
                    if let Some(extra) = sidecars.get(metadata.agent.as_str()) {
                        if extra.name.is_some() {
                            metadata.name = extra.name.clone();
                        }
                        if extra.parent.is_some() {
                            metadata.parent = extra.parent.clone();
                        }
                    }
                    if metadata.internal && !self.show_internal {
                        continue;
                    }
                    let parent = metadata
                        .parent
                        .as_ref()
                        .map(|id| {
                            FleetNodeId::Agent(AgentKey {
                                session: key.clone(),
                                agent_id: id.clone(),
                            })
                        })
                        .unwrap_or_else(|| FleetNodeId::Session(key.clone()));
                    graph.nodes.push(FleetNode {
                        id: FleetNodeId::Agent(AgentKey {
                            session: key.clone(),
                            agent_id: metadata.agent.clone(),
                        }),
                        parent: Some(parent),
                        label: metadata
                            .name
                            .clone()
                            .unwrap_or_else(|| metadata.agent.clone()),
                        description: metadata.description.clone(),
                        internal: metadata.internal,
                        archived,
                        expanded: false,
                        goal: metadata.goal.clone(),
                        activity: metadata
                            .activity
                            .text
                            .clone()
                            .or(metadata.last_action.clone()),
                        command: metadata.activity.command.clone(),
                        purpose: metadata.activity.purpose.clone(),
                    });
                }
            }
        }
        let known: BTreeSet<_> =
            self.sessions
                .iter()
                .filter(|s| s.provider == Provider::Codex)
                .flat_map(|s| {
                    std::iter::once(s.id.clone()).chain(s.every_file().filter_map(|f| {
                        self.metadata.get(&f.path).map(|i| i.metadata.agent.clone())
                    }))
                })
                .collect();
        for (id, live) in &self.live_threads {
            if known.contains(id) {
                continue;
            }
            let project = live
                .project
                .clone()
                .unwrap_or_else(|| "Live Codex work".into());
            let key = SessionKey {
                provider: Provider::Codex,
                session_id: id.clone(),
            };
            let label = self
                .work_title(&key)
                .or_else(|| live.name.clone())
                .unwrap_or_else(|| "Unlinked live agent".into());
            if !self.search.is_empty()
                && !project.to_lowercase().contains(&self.search)
                && !label.to_lowercase().contains(&self.search)
                && !id.to_lowercase().contains(&self.search)
            {
                continue;
            }
            if projects.insert(project.clone()) {
                graph.nodes.push(FleetNode {
                    id: FleetNodeId::Project(project.clone()),
                    parent: None,
                    label: project.clone(),
                    description: None,
                    internal: false,
                    archived: false,
                    expanded: true,
                    goal: None,
                    activity: None,
                    command: None,
                    purpose: None,
                });
            }
            graph.nodes.push(FleetNode {
                id: FleetNodeId::Session(key),
                parent: Some(FleetNodeId::Project(project)),
                label,
                description: Some("Live work; local transcript unavailable".into()),
                internal: false,
                archived: false,
                expanded: false,
                goal: None,
                activity: None,
                command: None,
                purpose: None,
            });
        }
        // The inventory is newest-first. Traverse parent links to keep each
        // project's sessions and each agent's descendants together for display.
        fn visit(
            at: usize,
            nodes: &[FleetNode],
            children: &BTreeMap<Option<&FleetNodeId>, Vec<usize>>,
            seen: &mut BTreeSet<usize>,
            out: &mut Vec<FleetNode>,
        ) {
            if !seen.insert(at) {
                return;
            }
            let node = &nodes[at];
            out.push(node.clone());
            if let Some(descendants) = children.get(&Some(&node.id)) {
                for child in descendants {
                    visit(*child, nodes, children, seen, out);
                }
            }
        }
        let nodes = std::mem::take(&mut graph.nodes);
        let mut children: BTreeMap<Option<&FleetNodeId>, Vec<usize>> = BTreeMap::new();
        for (at, node) in nodes.iter().enumerate() {
            children.entry(node.parent.as_ref()).or_default().push(at);
        }
        let mut seen = BTreeSet::new();
        if let Some(roots) = children.get(&None) {
            for root in roots {
                visit(*root, &nodes, &children, &mut seen, &mut graph.nodes);
            }
        }
        // Keep metadata with a missing parent visible too; ancestry remains
        // explicit rather than silently claiming another agent spawned it.
        for at in 0..nodes.len() {
            visit(at, &nodes, &children, &mut seen, &mut graph.nodes);
        }
        graph
    }

    /// A plain terminal rendering of the fleet, also usable by simple frontends.
    pub fn render(&self) -> Text<'static> {
        let mut lines = vec![
            if self.history {
                "All history"
            } else {
                "Current work"
            }
            .to_string(),
        ];
        for node in &self.graph().nodes {
            let indent = match node.id {
                FleetNodeId::Project(_) => "",
                FleetNodeId::Session(_) => "  ",
                FleetNodeId::Agent(_) => "    ",
            };
            lines.push(format!(
                "{indent}{}{}",
                node.label,
                if node.archived { " [archived]" } else { "" }
            ));
        }
        #[cfg(feature = "native")]
        if let Some(id) = &self.selection {
            let target = match id {
                FleetNodeId::Session(key) => Some(AgentKey {
                    session: key.clone(),
                    agent_id: "main".into(),
                }),
                FleetNodeId::Agent(target) => Some(target.clone()),
                FleetNodeId::Project(_) => None,
            };
            if let Some(target) = target
                && let Some(work) = self.work(&target.session)
            {
                lines.push(work.progress().label);
                if let Some(draft) = work.drafts.iter().find(|draft| draft.target == target) {
                    lines.push(format!("Draft for {}: {}", target.agent_id, draft.text));
                }
            }
        }
        Text::from(lines.join("\n"))
    }
}
