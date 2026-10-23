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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FleetNode {
    pub id: FleetNodeId,
    pub parent: Option<FleetNodeId>,
    pub label: String,
    pub description: Option<String>,
    pub internal: bool,
    pub archived: bool,
    pub expanded: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FleetGraph {
    pub nodes: Vec<FleetNode>,
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
        }
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
        self.graph = self.build_graph();
    }

    pub fn command(&mut self, command: ConsoleCommand) {
        match command {
            ConsoleCommand::ToggleHistory => self.history = !self.history,
            ConsoleCommand::ToggleInternal => self.show_internal = !self.show_internal,
            ConsoleCommand::Search(query) => self.search = query.trim().to_lowercase(),
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

    fn build_graph(&self) -> FleetGraph {
        let mut graph = FleetGraph::default();
        let mut projects = BTreeSet::new();
        for session in &self.sessions {
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
            if !self.history && (archived || !recent) {
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
                });
            }
            let key = SessionKey {
                provider: session.provider,
                session_id: session.id.clone(),
            };
            let expanded = self.expanded.contains(&key) || matching_child;
            graph.nodes.push(FleetNode {
                id: FleetNodeId::Session(key.clone()),
                parent: Some(FleetNodeId::Project(project)),
                label: metadata.title.clone().unwrap_or_else(|| session.id.clone()),
                description: metadata.description.clone(),
                internal: false,
                archived,
                expanded,
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
                    });
                }
            }
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
        Text::from(lines.join("\n"))
    }
}
