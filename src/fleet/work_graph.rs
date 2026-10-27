//! Bounded cached task canvas, separate from Fleet's camera and selection.
use super::*;

const MAX_EDGES: usize = 2048;
const MAX_MISSING: usize = 64;

pub(super) fn selection_id(selection: &work_view::Selection) -> String {
    format!(
        "work:{}",
        serde_json::to_string(selection).expect("work identity serializes")
    )
}
pub(super) fn parse_selection(id: &str) -> Option<work_view::Selection> {
    serde_json::from_str(id.strip_prefix("work:")?).ok()
}
pub(super) struct Task {
    pub id: String,
    pub reference: Option<String>,
    pub title: String,
    pub status: String,
    pub owner: String,
    pub dependencies: Option<Vec<String>>,
    pub parent: Option<String>,
}
#[derive(Default, PartialEq, Eq)]
struct Topology(BTreeMap<String, Option<BTreeSet<String>>>);
pub(super) struct WorkGraph {
    pub flow: Flow<Card, visual::SemanticEdge>,
    goal: Option<SessionKey>,
    topology: Topology,
    positions: BTreeMap<String, (f64, f64)>,
    vertical: bool,
    terminal_area: Option<Rect>,
    cyclic: BTreeSet<String>,
    pub pending_center: bool,
    pub overview: bool,
    pub summary: String,
}
impl WorkGraph {
    pub fn new() -> Self {
        Self {
            flow: Flow::new()
                .with_theme(visual::theme())
                .with_deselect_on_pane_click(false)
                .with_min_zoom(0.1),
            goal: None,
            topology: Topology::default(),
            positions: BTreeMap::new(),
            vertical: false,
            terminal_area: None,
            cyclic: BTreeSet::new(),
            pending_center: true,
            overview: false,
            summary: String::new(),
        }
    }
    /// Reorient only when the actual canvas changes shape. Status refreshes
    /// retain both dragged positions and camera; a tall canvas can spend its
    /// height on prerequisite layers instead of shrinking a long horizontal line.
    pub fn adapt_to(&mut self, area: Rect, terminal: Rect) {
        if self.terminal_area == Some(terminal) {
            return;
        }
        self.terminal_area = Some(terminal);
        let vertical = u32::from(area.height) * 4 >= u32::from(area.width);
        if vertical == self.vertical {
            return;
        }
        self.vertical = vertical;
        let ids = self
            .flow
            .nodes()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        let edges = self
            .flow
            .edges()
            .iter()
            .map(|edge| (edge.source.clone(), edge.target.clone()))
            .collect::<Vec<_>>();
        self.positions = layout(&ids, &edges, vertical);
        let nodes = self
            .flow
            .nodes()
            .cloned()
            .map(|mut node| {
                node.position = self
                    .positions
                    .get(&node.id)
                    .copied()
                    .unwrap_or_default()
                    .into();
                node.handles = handles(vertical);
                node
            })
            .collect();
        let _ = self.flow.set_nodes(nodes);
        if self.overview {
            self.flow.request_fit_view();
            self.pending_center = false;
        } else {
            self.pending_center = true;
        }
    }
    pub fn remember_drag(&mut self, id: &str) {
        if let Some(node) = self.flow.node(id) {
            self.positions
                .insert(id.into(), (node.position.x, node.position.y));
        }
    }
    pub fn sync(&mut self, goal: Option<SessionKey>, tasks: Vec<Task>) {
        if self.goal != goal {
            *self = Self::new();
            self.goal = goal;
        }
        let selected = self.flow.first_selected_node_id();
        let topology = Topology(
            tasks
                .iter()
                .map(|task| {
                    (
                        task.id.clone(),
                        task.dependencies
                            .as_ref()
                            .map(|dependencies| dependencies.iter().cloned().collect()),
                    )
                })
                .collect(),
        );
        let references: BTreeMap<_, _> = tasks
            .iter()
            .filter_map(|task| Some((task.reference.as_ref()?, task.id.clone())))
            .collect();
        let mut edges = Vec::new();
        let mut analysis_edges = Vec::new();
        let mut missing = BTreeSet::new();
        let mut omitted = 0usize;
        for task in &tasks {
            for dependency in task.dependencies.iter().flatten() {
                let source = if let Some(source) = references.get(dependency) {
                    source.clone()
                } else {
                    if !missing.contains(dependency) && missing.len() >= MAX_MISSING {
                        omitted += 1;
                        continue;
                    }
                    missing.insert(dependency.clone());
                    selection_id(&work_view::Selection::Missing(dependency.clone()))
                };
                analysis_edges.push((source.clone(), task.id.clone()));
                if edges.len() == MAX_EDGES {
                    omitted += 1;
                    continue;
                }
                edges.push((source, task.id.clone()));
            }
        }
        let all_ids: Vec<_> = tasks
            .iter()
            .map(|task| task.id.clone())
            .chain(
                missing
                    .iter()
                    .map(|id| selection_id(&work_view::Selection::Missing(id.clone()))),
            )
            .collect();
        if self.topology != topology {
            self.positions = layout(&all_ids, &edges, self.vertical);
            self.cyclic = cyclic_tasks(&all_ids, &analysis_edges);
            self.topology = topology;
        }
        let mut nodes: Vec<_> = tasks
            .into_iter()
            .map(|task| {
                let mut prerequisites = match &task.dependencies {
                    None => "deps unknown".into(),
                    Some(deps) if deps.is_empty() => "no prerequisites".into(),
                    Some(deps) => format!("{} prerequisite(s)", deps.len()),
                };
                if self.cyclic.contains(&task.id) {
                    prerequisites = format!("Cycle · {prerequisites}");
                }
                let parent = task
                    .parent
                    .map(|parent| format!("Group: {parent}"))
                    .unwrap_or_default();
                let pos = self.positions.get(&task.id).copied().unwrap_or_default();
                Node::new(
                    task.id,
                    pos,
                    (30.0, 7.0),
                    Card {
                        cue: visual::Cue::work(&task.status),
                        title: task.title,
                        detail: task.status,
                        progress: format!("Owner: {}", task.owner),
                        bar: prerequisites,
                        activity: parent,
                    },
                )
                .with_deletable(false)
                .with_connectable(false)
                .with_handles(handles(self.vertical))
            })
            .collect();
        for reference in &missing {
            let id = selection_id(&work_view::Selection::Missing(reference.clone()));
            let pos = self.positions.get(&id).copied().unwrap_or_default();
            nodes.push(
                Node::new(
                    id,
                    pos,
                    (30.0, 7.0),
                    Card {
                        cue: visual::Cue::work("unknown"),
                        title: format!("Unresolved: {reference}"),
                        detail: "missing prerequisite".into(),
                        progress: "Owner unavailable".into(),
                        bar: String::new(),
                        activity: String::new(),
                    },
                )
                .with_deletable(false)
                .with_connectable(false)
                .with_handles(handles(self.vertical)),
            );
        }
        self.summary = format!(
            "Work graph · {} tasks · {} prerequisite edge{}",
            all_ids.len() - missing.len(),
            edges.len(),
            if edges.len() == 1 { "" } else { "s" }
        );
        if !self.cyclic.is_empty() {
            self.summary
                .push_str(&format!(" · {} cyclic tasks", self.cyclic.len()));
        }
        if !missing.is_empty() {
            self.summary
                .push_str(&format!(" · {} unresolved", missing.len()));
        }
        if omitted > 0 {
            self.summary = format!("LIMIT: {omitted} references omitted · {}", self.summary);
        }
        let edges = edges
            .into_iter()
            .enumerate()
            .map(|(at, (source, target))| {
                Edge::new(format!("work-edge-{at}"), source, target)
                    .with_label("requires")
                    .with_selectable(false)
                    .with_deletable(false)
                    .with_reconnectable(Reconnectable::None)
            })
            .collect();
        let _ = self.flow.set_nodes(nodes);
        let _ = self.flow.set_edges(edges);
        if let Some(selected) = selected {
            self.flow.select_node(&selected);
        }
    }
}

/// Linear bounded layering includes isolated vertices; cyclic residuals get
/// explicit finite slots rather than asking a DAG algorithm to converge.
fn handles(vertical: bool) -> Vec<Handle> {
    vec![
        Handle::source(if vertical {
            HandlePosition::Bottom
        } else {
            HandlePosition::Right
        })
        .with_hidden(true),
        Handle::target(if vertical {
            HandlePosition::Top
        } else {
            HandlePosition::Left
        })
        .with_hidden(true),
    ]
}
fn layout(
    ids: &[String],
    edges: &[(String, String)],
    vertical: bool,
) -> BTreeMap<String, (f64, f64)> {
    let indices: BTreeMap<_, _> = ids.iter().enumerate().map(|(at, id)| (id, at)).collect();
    let mut children = vec![Vec::new(); ids.len()];
    let mut degree = vec![0usize; ids.len()];
    let mut connected = vec![false; ids.len()];
    for (source, target) in edges {
        let (Some(&source), Some(&target)) = (indices.get(source), indices.get(target)) else {
            continue;
        };
        children[source].push(target);
        degree[target] += 1;
        connected[source] = true;
        connected[target] = true;
    }
    let mut queue: std::collections::VecDeque<_> = degree
        .iter()
        .enumerate()
        .filter_map(|(at, count)| (*count == 0).then_some(at))
        .collect();
    let mut ranks = vec![0usize; ids.len()];
    while let Some(at) = queue.pop_front() {
        for &child in &children[at] {
            ranks[child] = ranks[child].max(ranks[at] + 1);
            degree[child] -= 1;
            if degree[child] == 0 {
                queue.push_back(child);
            }
        }
    }
    if vertical {
        let mut columns = BTreeMap::<usize, usize>::new();
        let mut occupied = BTreeSet::new();
        let mut positions = BTreeMap::new();
        for (at, id) in ids.iter().enumerate().filter(|(at, _)| connected[*at]) {
            let rank = ranks[at].min(512);
            let column = columns.entry(rank).or_default();
            occupied.insert((rank, *column));
            positions.insert(id.clone(), (*column as f64 * 36.0, rank as f64 * 10.0));
            *column += 1;
        }
        // Isolated work shares otherwise empty space alongside the dependency
        // layers. It remains edge-free; its placement declares no prerequisite.
        let mut slot = 0usize;
        for (_, id) in ids.iter().enumerate().filter(|(at, _)| !connected[*at]) {
            while occupied.contains(&(slot / 2, slot % 2)) {
                slot += 1;
            }
            occupied.insert((slot / 2, slot % 2));
            positions.insert(
                id.clone(),
                ((slot % 2) as f64 * 36.0, (slot / 2) as f64 * 10.0),
            );
            slot += 1;
        }
        return positions;
    }
    let mut rows = BTreeMap::<usize, usize>::new();
    ids.iter()
        .enumerate()
        .map(|(at, id)| {
            let rank = ranks[at].min(512);
            let row = rows.entry(rank).or_default();
            let pos = (rank as f64 * 40.0, *row as f64 * 8.0);
            *row += 1;
            (id.clone(), pos)
        })
        .collect()
}

/// Strongly connected components identify cycles without marking their
/// downstream dependents cyclic. Both traversals are bounded by displayed IO.
fn cyclic_tasks(ids: &[String], edges: &[(String, String)]) -> BTreeSet<String> {
    let indices: BTreeMap<_, _> = ids.iter().enumerate().map(|(at, id)| (id, at)).collect();
    let mut forward = vec![Vec::new(); ids.len()];
    let mut reverse = vec![Vec::new(); ids.len()];
    for (source, target) in edges {
        let (Some(&source), Some(&target)) = (indices.get(source), indices.get(target)) else {
            continue;
        };
        forward[source].push(target);
        reverse[target].push(source);
    }
    fn visit(at: usize, edges: &[Vec<usize>], seen: &mut [bool], order: &mut Vec<usize>) {
        if seen[at] {
            return;
        }
        seen[at] = true;
        for &next in &edges[at] {
            visit(next, edges, seen, order);
        }
        order.push(at);
    }
    let mut seen = vec![false; ids.len()];
    let mut order = Vec::new();
    for at in 0..ids.len() {
        visit(at, &forward, &mut seen, &mut order);
    }
    seen.fill(false);
    let mut cyclic = BTreeSet::new();
    for at in order.into_iter().rev() {
        if seen[at] {
            continue;
        }
        let mut component = Vec::new();
        visit(at, &reverse, &mut seen, &mut component);
        if component.len() > 1 || forward[at].contains(&at) {
            cyclic.extend(component.into_iter().map(|index| ids[index].clone()));
        }
    }
    cyclic
}
