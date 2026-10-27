//! Selected-goal work from already cached explicit records and native plans.
use super::*;
use crate::workstate::{PlanProposal, PlanStatus, WorkItem, WorkStatus};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) enum Selection {
    Report(String),
    Plan { proposal: String, step: usize },
    Missing(String),
}

#[derive(Clone, Copy)]
enum Entry<'a> {
    Report(&'a WorkItem),
    Source(&'a crate::tracking::SourceTask),
    Plan(&'a PlanProposal, usize),
}
impl<'a> Entry<'a> {
    fn selection(self) -> Selection {
        match self {
            Self::Report(work) => Selection::Report(work.report.id.clone()),
            Self::Source(task) => Selection::Report(task.id.clone()),
            Self::Plan(plan, step) => Selection::Plan {
                proposal: plan.id.clone(),
                step,
            },
        }
    }
    fn matches(self, selected: &Selection) -> bool {
        match (self, selected) {
            (Self::Report(work), Selection::Report(id)) => &work.report.id == id,
            (Self::Source(task), Selection::Report(id)) => &task.id == id,
            (Self::Plan(plan, index), Selection::Plan { proposal, step }) => {
                &plan.id == proposal && index == *step
            }
            _ => false,
        }
    }
    fn label(self) -> &'a str {
        match self {
            Self::Report(work) => &work.report.label,
            Self::Source(task) => &task.label,
            Self::Plan(plan, at) => &plan.steps[at].step,
        }
    }
    fn owner(self) -> Option<&'a AgentKey> {
        match self {
            Self::Report(work) => Some(&work.report.owner),
            Self::Source(task) => task.owner.as_ref(),
            Self::Plan(plan, _) => Some(&plan.producer),
        }
    }
    fn status(self) -> Option<WorkStatus> {
        match self {
            Self::Report(work) => Some(work.report.status),
            Self::Source(task) => task.status,
            Self::Plan(plan, at) => match plan.steps[at].status {
                PlanStatus::Planned => Some(WorkStatus::Planned),
                PlanStatus::Active => Some(WorkStatus::Active),
                PlanStatus::Done => Some(WorkStatus::Done),
                PlanStatus::Unknown(_) => None,
            },
        }
    }
    fn status_label(self) -> &'a str {
        match self.status() {
            Some(WorkStatus::Planned) => "planned",
            Some(WorkStatus::Active) => "in progress",
            Some(WorkStatus::Blocked) => "blocked",
            Some(WorkStatus::Done) => "reported done",
            Some(WorkStatus::Cancelled) => "cancelled",
            None => match self {
                Self::Plan(plan, at) if !plan.steps[at].status.as_str().is_empty() => {
                    plan.steps[at].status.as_str()
                }
                Self::Source(task) => &task.status_label,
                _ => "status unknown",
            },
        }
    }
    fn display_status(self) -> String {
        if self.status().is_none() {
            format!("unknown: {}", self.status_label())
        } else {
            self.status_label().into()
        }
    }
    fn source(self) -> &'a str {
        match self {
            Self::Report(work) => &work.report.source,
            Self::Source(task) => &task.source,
            Self::Plan(plan, _) => &plan.source,
        }
    }
    fn recorded_at(self) -> &'a str {
        match self {
            Self::Report(work) => &work.updated_at,
            Self::Source(task) => &task.updated_at,
            Self::Plan(plan, _) => plan.recorded_at.as_deref().unwrap_or("unknown"),
        }
    }
    fn owner_label(self, ui: &FleetConsole) -> String {
        if let Self::Source(task) = self {
            return if task.owner_label.is_empty() {
                "Unassigned".into()
            } else {
                task.owner_label.clone()
            };
        }
        self.owner()
            .map(|owner| ui.work_owner_label(owner))
            .unwrap_or_else(|| match self {
                Self::Source(task) if !task.owner_label.is_empty() => task.owner_label.clone(),
                _ => "Unassigned".into(),
            })
    }
    fn reference(self) -> Option<&'a str> {
        match self {
            Self::Report(work) => Some(&work.report.id),
            Self::Source(task) => Some(&task.id),
            Self::Plan(_, _) => None,
        }
    }
    fn parent(self) -> Option<&'a str> {
        match self {
            Self::Report(work) => work.report.parent_id.as_deref(),
            Self::Source(task) => task.parent_id.as_deref(),
            Self::Plan(_, _) => None,
        }
    }
    fn dependencies(self) -> Option<&'a Vec<String>> {
        match self {
            Self::Report(work) => work.report.depends_on.as_ref(),
            Self::Source(task) => task.depends_on.as_ref(),
            Self::Plan(_, _) => None,
        }
    }
    fn links(self) -> &'a [crate::workstate::WorkOutcomeLink] {
        match self {
            Self::Report(work) => &work.report.outcome_links,
            Self::Source(task) => &task.outcome_links,
            Self::Plan(_, _) => &[],
        }
    }
    fn context(self) -> [(&'static str, Option<&'a str>); 4] {
        match self {
            Self::Report(work) => [
                ("Purpose", work.report.purpose.as_deref()),
                ("Changed", work.report.change.as_deref()),
                ("Next", work.report.next.as_deref()),
                ("Blocker", work.report.blocker.as_deref()),
            ],
            Self::Source(task) => [
                ("Purpose", task.purpose.as_deref()),
                ("Changed", task.change.as_deref()),
                ("Next", task.next.as_deref()),
                ("Blocker", task.blocker.as_deref()),
            ],
            Self::Plan(_, _) => [
                ("Purpose", None),
                ("Changed", None),
                ("Next", None),
                ("Blocker", None),
            ],
        }
    }
}

/// An explicit expired selection never falls through to another work item.
fn current_entry(entries: &[Entry<'_>], selected: Option<&Selection>) -> Option<usize> {
    if let Some(selected) = selected {
        return entries.iter().position(|entry| entry.matches(selected));
    }
    entries
        .iter()
        .position(|entry| entry.status() == Some(WorkStatus::Active))
        .or_else(|| {
            entries
                .iter()
                .position(|entry| entry.status() == Some(WorkStatus::Blocked))
        })
        .or_else(|| (!entries.is_empty()).then_some(0))
}

impl FleetConsole {
    fn work_entries(&self) -> Vec<Entry<'_>> {
        let Some(record) = &self.selected_work else {
            return Vec::new();
        };
        if record.source_snapshot {
            return record
                .source_tasks
                .iter()
                .take(512)
                .map(Entry::Source)
                .collect();
        }
        let reported: BTreeSet<_> = record
            .work_items
            .iter()
            .filter_map(|work| self.resolved_owner(&work.report.owner))
            .collect();
        let mut entries: Vec<_> = record.source_tasks.iter().map(Entry::Source).collect();
        entries.extend(
            record
                .work_items
                .iter()
                .filter(|work| {
                    !record
                        .source_tasks
                        .iter()
                        .any(|task| task.id == work.report.id)
                })
                .map(Entry::Report),
        );
        entries.truncate(512);
        for plan in record.plan_drafts.iter().take(512) {
            if self
                .resolved_owner(&plan.producer)
                .is_some_and(|owner| reported.contains(&owner))
            {
                continue;
            }
            entries.extend(
                (0..plan.steps.len())
                    .take(512usize.saturating_sub(entries.len()))
                    .map(|at| Entry::Plan(plan, at)),
            );
            if entries.len() == 512 {
                break;
            }
        }
        entries
    }
    pub(super) fn reported_progress(&self, compact: bool) -> Vec<ratatui::text::Line<'static>> {
        use ratatui::text::{Line, Span};
        let entries = self.work_entries();
        let mut counts = [0usize; 6];
        for entry in &entries {
            let at = match entry.status() {
                Some(WorkStatus::Done) => 0,
                Some(WorkStatus::Active) => 1,
                Some(WorkStatus::Blocked) => 2,
                Some(WorkStatus::Planned) => 3,
                Some(WorkStatus::Cancelled) => 4,
                None => 5,
            };
            counts[at] += 1;
        }
        let mut spans = Vec::new();
        for (at, (label, status)) in [
            ("done", "reported done"),
            ("active", "in progress"),
            ("blocked", "blocked"),
            ("planned", "planned"),
            ("cancelled", "cancelled"),
            ("unknown", "unknown"),
        ]
        .into_iter()
        .enumerate()
        {
            if !compact && at == 0 {
                continue;
            }
            let cue = visual::Cue::work(status);
            spans.push(Span::styled(
                if compact {
                    format!("{}{} ", cue.glyph, counts[at])
                } else {
                    format!("{} {} {label}  ", cue.glyph, counts[at])
                },
                Style::default().fg(cue.color),
            ));
        }
        if compact {
            spans.insert(0, Span::raw("Reported: "));
            vec![Line::from(spans)]
        } else {
            vec![
                Line::raw(format!(
                    "Reported work: {} of {} done",
                    counts[0],
                    entries.len()
                )),
                Line::from(spans),
            ]
        }
    }
    pub(super) fn work_owner_label(&self, owner: &AgentKey) -> String {
        let resolved = self.resolved_owner(owner);
        let name = self.producer_label(resolved.as_ref().unwrap_or(owner));
        if name == "Agent" {
            format!("Agent {}", crate::ui::truncate(&owner.agent_id, 20))
        } else {
            name
        }
    }
    pub(super) fn open_commands(&mut self) {
        self.pane = Pane::Commands;
        self.scroll = 0;
        self.select_recipient(self.selected_agent());
        self.status = "Recorded commands and public messages for the selected owner".into();
    }
    pub(super) fn open_agents(&mut self) {
        if !self.agents_mode {
            self.remember_navigation();
        }
        self.agents_mode = true;
        self.work_canvas = false;
        self.work_selection = None;
        self.work_mode = false;
        self.pane = Pane::Graph;
        self.scroll = 0;
        let collapsed = self
            .view
            .selection
            .as_ref()
            .and_then(session_key)
            .and_then(|key| {
                self.view
                    .graph
                    .nodes
                    .iter()
                    .find(|node| node.id == FleetNodeId::Session(key.clone()))
            })
            .is_some_and(|node| !node.expanded);
        if collapsed {
            self.toggle_expansion();
        }
        self.select_recipient(self.selected_agent());
        self.status = "Agent inspection · arrows select an agent · w returns to work".into();
    }
    pub(super) fn selected_work_owner(&self) -> Option<AgentKey> {
        let selected = self.work_selection.as_ref()?;
        let entries = self.work_entries();
        current_entry(&entries, Some(selected)).and_then(|at| entries[at].owner().cloned())
    }
    pub(super) fn selected_work_is_source(&self) -> bool {
        let Some(selected) = self.work_selection.as_ref() else {
            return false;
        };
        let entries = self.work_entries();
        current_entry(&entries, Some(selected))
            .is_some_and(|at| matches!(entries[at], Entry::Source(_)))
    }
    pub(super) fn focus_work(&mut self) {
        if self.view.selection.as_ref().and_then(session_key).is_none() {
            return;
        }
        if self.agents_mode
            && self
                .navigation
                .last()
                .is_some_and(|previous| previous.work_canvas)
        {
            self.back_to_fleet();
            return;
        }
        if !self.work_canvas {
            self.remember_navigation();
        }
        self.agents_mode = false;
        self.work_canvas = true;
        self.work_mode = true;
        let entries = self.work_entries();
        let entry = current_entry(&entries, self.work_selection.as_ref())
            .or_else(|| current_entry(&entries, None))
            .map(|at| entries[at]);
        if let Some(entry) = entry {
            let selection = entry.selection();
            self.work_selection = Some(selection);
            self.sync_work_graph();
            self.work_graph.flow.select_node(&work_graph::selection_id(
                self.work_selection.as_ref().unwrap(),
            ));
            self.work_graph.pending_center = true;
            self.pane = Pane::Graph;
            self.scroll = 0;
            self.select_recipient(self.selected_agent());
            self.status = "Work selected · [ / ] or arrows choose · f steer its owner".into();
        } else {
            self.status = "No reported work; Enable tracking to request updates".into();
        }
    }
    fn remember_navigation(&mut self) {
        if self.navigation.len() == 8 {
            self.navigation.remove(0);
        }
        self.navigation.push(Navigation {
            selection: self.view.selection.clone(),
            work_selection: self.work_selection.clone(),
            work_mode: self.work_mode,
            work_canvas: self.work_canvas,
            agents_mode: self.agents_mode,
            viewport: if self.work_canvas {
                self.work_graph.flow.viewport
            } else {
                self.flow.viewport
            },
        });
    }
    pub(super) fn back_to_fleet(&mut self) {
        if let Some(previous) = self.navigation.pop() {
            self.work_canvas = previous.work_canvas;
            self.agents_mode = previous.agents_mode;
            self.work_mode = previous.work_mode;
            self.work_selection = previous.work_selection;
            self.view.selection = previous.selection.clone();
            self.requested_selection = previous.selection.clone();
            if let Some(selection) = previous.selection {
                self.navigate(ConsoleCommand::Select(selection));
            }
            if self.work_canvas {
                self.work_graph.flow.viewport = previous.viewport;
                if let Some(selected) = &self.work_selection {
                    self.work_graph
                        .flow
                        .select_node(&work_graph::selection_id(selected));
                }
                self.work_graph.pending_center = false;
            } else {
                self.flow.viewport = previous.viewport;
                if let Some(selected) = &self.view.selection {
                    self.flow.select_node(&flow_id(selected));
                }
            }
        } else {
            self.work_canvas = false;
            self.agents_mode = false;
            self.work_selection = None;
            self.work_mode = true;
        }
        self.pending_center = false;
        self.select_recipient(self.selected_agent());
        self.status = if self.work_canvas {
            "Work graph · restored selection"
        } else {
            "Fleet · arrows select · Enter or w opens Work graph"
        }
        .into();
    }
    pub(super) fn sync_work_graph(&mut self) {
        let tasks = self
            .work_entries()
            .iter()
            .map(|entry| work_graph::Task {
                id: work_graph::selection_id(&entry.selection()),
                title: entry.label().into(),
                status: entry.display_status(),
                owner: entry.owner_label(self),
                dependencies: entry.dependencies().cloned(),
                reference: entry.reference().map(str::to_owned),
                parent: entry.parent().map(str::to_owned),
            })
            .collect();
        self.work_graph.sync(
            self.view.selection.as_ref().and_then(session_key).cloned(),
            tasks,
        );
    }
    pub(super) fn move_work(&mut self, delta: isize) {
        let entries = self.work_entries();
        let at = self
            .work_selection
            .as_ref()
            .and_then(|selected| entries.iter().position(|entry| entry.matches(selected)));
        let next = at.map_or(0, |at| {
            (at as isize + delta).clamp(0, entries.len().saturating_sub(1) as isize) as usize
        });
        if let Some(entry) = entries.get(next) {
            self.work_selection = Some(entry.selection());
            self.work_graph.flow.select_node(&work_graph::selection_id(
                self.work_selection.as_ref().unwrap(),
            ));
            self.work_graph.pending_center = true;
            self.scroll = 0;
            self.select_recipient(self.selected_agent());
        }
    }
    pub(super) fn inspection_text(&self, compact: bool, width: u16) -> String {
        let activity = if self.work_canvas {
            String::new()
        } else {
            self.activity_text()
        };
        if !self.work_mode {
            return format!(
                "Agent inspection · arrows select · w work\n{}\n{}{}",
                self.runtime_word(),
                self.steering_text(),
                activity
            );
        }
        if compact {
            let entries = self.work_entries();
            let selected = self.work_selection.is_some();
            if let Some(entry) =
                current_entry(&entries, self.work_selection.as_ref()).map(|at| entries[at])
            {
                let mut lines = vec![
                    format!(
                        "{} [{}] {}",
                        if selected { "›" } else { "→" },
                        crate::ui::truncate(&entry.display_status(), 40),
                        crate::ui::truncate(entry.label(), 120)
                    ),
                    format!(
                        "Owner: {}",
                        crate::ui::truncate(&entry.owner_label(self), 120)
                    ),
                ];
                let context = entry.context();
                for at in [0, 2, 3, 1] {
                    let (label, value) = context[at];
                    if let Some(value) = value {
                        lines.push(format!("{label}: {value}"));
                    }
                }
                lines.push(
                    [
                        ("Guidance", FeedbackMode::Guidance),
                        ("Queue", FeedbackMode::Queue),
                        ("Stop", FeedbackMode::Stop),
                    ]
                    .into_iter()
                    .map(|(name, mode)| {
                        format!(
                            "{name} {}",
                            if self.control_availability(mode).is_ok() {
                                "available"
                            } else {
                                "unavailable"
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("  "),
                );
                let compact = lines
                    .iter()
                    .map(|line| crate::ui::truncate(line, width as usize))
                    .collect::<Vec<_>>()
                    .join("\n");
                return format!(
                    "{compact}\n\nRecorded detail · PageDown scroll\n{}\n{}{}",
                    self.work_text(),
                    self.steering_text(),
                    activity
                );
            }
        }
        format!(
            "{}\n{}\n{}{}\n\nf feedback · Q saved questions · c criteria\nd documents · R read-only replay",
            self.work_text(),
            self.runtime_word(),
            self.steering_text(),
            activity
        )
    }
    pub(super) fn work_text(&self) -> String {
        let Some(record) = &self.selected_work else {
            return String::new();
        };
        if let Some(Selection::Missing(reference)) = &self.work_selection {
            return format!(
                "Unresolved prerequisite: {reference}\nNo matching task is recorded. Ownership is unavailable; no recipient is selected.\nChoose a task with arrows or w."
            );
        }
        let entries = self.work_entries();
        let mut text = String::from("Work toward goal\n");
        for diagnostic in record.source_diagnostics.iter().take(4) {
            text.push_str(&format!("Source: {diagnostic}\n"));
        }
        if entries.is_empty() {
            text.push_str("No reported work; Enable tracking to request updates.\n");
            return text;
        }
        let Some(current) = current_entry(&entries, self.work_selection.as_ref()) else {
            text.push_str("Selected work is no longer recorded; press w to choose current work.\n");
            return text;
        };
        let selected = self.work_selection.is_some().then_some(current);
        let start = current.saturating_sub(1);
        for (at, entry) in entries
            .iter()
            .enumerate()
            .skip(start)
            .take(if self.work_canvas { 0 } else { 4 })
        {
            let marker = if selected == Some(at) { "› " } else { "" };
            text.push_str(&format!(
                "{marker}[{}] {}\n",
                crate::ui::truncate(&entry.display_status(), 40),
                crate::ui::truncate(entry.label(), 120)
            ));
        }
        if entries.len() > 4 && !self.work_canvas {
            text.push_str(&format!("{} work entries · w select work\n", entries.len()));
        }
        if entries.len() == 512 || record.plan_drafts.len() > 512 {
            text.push_str("Work view limit: 512 entries; use local context for recorded detail.\n");
        }
        let current = entries[current];
        if selected.is_some() {
            text.push_str(&format!(
                "Selected work: {}\n",
                crate::ui::truncate(current.label(), 160)
            ));
        }
        if self.work_canvas {
            text.push_str(&format!("Status: {}\n", current.display_status()));
        }
        text.push_str(&format!(
            "Owner: {}\n",
            crate::ui::truncate(&current.owner_label(self), 120)
        ));
        if self.work_canvas {
            for (label, value) in current.context() {
                if let Some(value) = value {
                    text.push_str(&format!("{label}: {}\n", crate::ui::truncate(value, 240)));
                }
            }
        }
        let recorded = if self.work_canvas {
            chrono::DateTime::parse_from_rfc3339(current.recorded_at())
                .map(|time| {
                    time.with_timezone(&chrono::Utc)
                        .format("%Y-%m-%d %H:%M UTC")
                        .to_string()
                })
                .unwrap_or_else(|_| crate::ui::truncate(current.recorded_at(), 80))
        } else {
            crate::ui::truncate(current.recorded_at(), 80)
        };
        text.push_str(&format!(
            "Source: {}\nRecorded: {}\n",
            crate::ui::truncate(current.source(), 160),
            recorded
        ));
        if let Some(owner) = current.owner()
            && self.producer_label(owner) == "Agent"
        {
            text.push_str(&format!(
                "Owner ID: {}\n",
                crate::ui::truncate(&owner.agent_id, 256)
            ));
        }
        match current {
            Entry::Report(_) | Entry::Source(_) => {
                if let Some(parent) = current.parent() {
                    text.push_str(&format!(
                        "Parent: {} · grouping only\n",
                        self.work_reference(parent)
                    ));
                }
                text.push_str("Dependencies: ");
                match current.dependencies() {
                    None => text.push_str("not declared\n"),
                    Some(dependencies) if dependencies.is_empty() => {
                        text.push_str("none declared\n")
                    }
                    Some(dependencies) => {
                        text.push_str(
                            &dependencies
                                .iter()
                                .take(4)
                                .map(|id| self.work_reference(id))
                                .collect::<Vec<_>>()
                                .join(", "),
                        );
                        if dependencies.len() > 4 {
                            text.push_str(" · more recorded");
                        }
                        text.push('\n');
                    }
                }
                for link in current.links().iter().take(4) {
                    let relation = match &record.goal {
                        Some(goal) if goal.revision != link.goal_revision => format!(
                            "{} (stale scope: revision {}; current {})",
                            link.criterion_id, link.goal_revision, goal.revision
                        ),
                        Some(goal) => goal
                            .criteria
                            .iter()
                            .find(|criterion| criterion.id == link.criterion_id)
                            .map(|criterion| {
                                format!(
                                    "{} (revision {})",
                                    crate::ui::truncate(&criterion.label, 120),
                                    goal.revision
                                )
                            })
                            .unwrap_or_else(|| format!("{} (unresolved)", link.criterion_id)),
                        None => format!("{} (unresolved goal)", link.criterion_id),
                    };
                    text.push_str(&format!(
                        "Outcome: {}\n",
                        crate::ui::truncate(&relation, 200)
                    ));
                }
                if current.links().len() > 4 {
                    text.push_str(
                        "More outcome links recorded; inspect local context for detail.\n",
                    );
                }
                if !self.work_canvas {
                    for (label, value) in current.context() {
                        if let Some(value) = value {
                            text.push_str(&format!(
                                "{label}: {}\n",
                                crate::ui::truncate(value, 240)
                            ));
                        }
                    }
                }
                if let Entry::Source(task) = current {
                    text.push_str(&format!(
                        "Source ID: {} / {}\n",
                        task.source_id, task.task_id
                    ));
                    if task.stale {
                        text.push_str("Source stale · retaining last successful synchronization\n");
                    }
                }
            }
            Entry::Plan(_, _) => {
                text.push_str("Dependencies: not declared · plan status is unverified\n")
            }
        }
        text
    }
    fn work_reference(&self, id: &str) -> String {
        self.work_entries()
            .into_iter()
            .find(|entry| entry.reference() == Some(id))
            .map(|entry| {
                format!(
                    "{} ({})",
                    crate::ui::truncate(entry.label(), 120),
                    match entry {
                        Entry::Source(task) => task.task_id.as_str(),
                        _ => id,
                    }
                )
            })
            .unwrap_or_else(|| format!("{id} (unresolved)"))
    }
}
