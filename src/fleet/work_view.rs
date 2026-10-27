//! Selected-goal work from already cached explicit records and native plans.
use super::*;
use crate::workstate::{PlanProposal, PlanStatus, WorkItem, WorkStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Selection {
    Report(String),
    Plan { proposal: String, step: usize },
}

#[derive(Clone, Copy)]
enum Entry<'a> {
    Report(&'a WorkItem),
    Plan(&'a PlanProposal, usize),
}
impl<'a> Entry<'a> {
    fn selection(self) -> Selection {
        match self {
            Self::Report(work) => Selection::Report(work.report.id.clone()),
            Self::Plan(plan, step) => Selection::Plan {
                proposal: plan.id.clone(),
                step,
            },
        }
    }
    fn matches(self, selected: &Selection) -> bool {
        match (self, selected) {
            (Self::Report(work), Selection::Report(id)) => &work.report.id == id,
            (Self::Plan(plan, index), Selection::Plan { proposal, step }) => {
                &plan.id == proposal && index == *step
            }
            _ => false,
        }
    }
    fn label(self) -> &'a str {
        match self {
            Self::Report(work) => &work.report.label,
            Self::Plan(plan, at) => &plan.steps[at].step,
        }
    }
    fn owner(self) -> &'a AgentKey {
        match self {
            Self::Report(work) => &work.report.owner,
            Self::Plan(plan, _) => &plan.producer,
        }
    }
    fn status(self) -> Option<WorkStatus> {
        match self {
            Self::Report(work) => Some(work.report.status),
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
                _ => "status unknown",
            },
        }
    }
    fn source(self) -> &'a str {
        match self {
            Self::Report(work) => &work.report.source,
            Self::Plan(plan, _) => &plan.source,
        }
    }
    fn recorded_at(self) -> &'a str {
        match self {
            Self::Report(work) => &work.updated_at,
            Self::Plan(plan, _) => plan.recorded_at.as_deref().unwrap_or("unknown"),
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
        let reported: BTreeSet<_> = record
            .work_items
            .iter()
            .map(|work| &work.report.owner)
            .collect();
        let mut entries: Vec<_> = record.work_items.iter().map(Entry::Report).collect();
        for plan in record.plan_drafts.iter().take(512) {
            if reported.contains(&plan.producer) {
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
        current_entry(&entries, Some(selected)).map(|at| entries[at].owner().clone())
    }
    pub(super) fn focus_work(&mut self) {
        self.work_mode = true;
        let entries = self.work_entries();
        let entry = current_entry(&entries, self.work_selection.as_ref())
            .or_else(|| current_entry(&entries, None))
            .map(|at| entries[at]);
        if let Some(entry) = entry {
            let selection = entry.selection();
            self.work_selection = Some(selection);
            self.pane = Pane::Graph;
            self.scroll = 0;
            self.select_recipient(self.selected_agent());
            self.status = "Work selected · [ / ] or arrows choose · f steer its owner".into();
        } else {
            self.status = "No reported work; Enable tracking to request updates".into();
        }
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
            self.scroll = 0;
            self.select_recipient(self.selected_agent());
        }
    }
    pub(super) fn inspection_text(&self, compact: bool, width: u16) -> String {
        if !self.work_mode {
            return format!(
                "Agent inspection · arrows select · w work\n{}\n{}{}",
                self.runtime_word(),
                self.steering_text(),
                self.activity_text()
            );
        }
        if compact {
            let entries = self.work_entries();
            let selected = self.work_selection.is_some();
            if let Some(entry) =
                current_entry(&entries, self.work_selection.as_ref()).map(|at| entries[at])
            {
                let mut lines = vec![
                    "Work toward goal".to_owned(),
                    format!(
                        "{} [{}] {}",
                        if selected { "›" } else { "→" },
                        crate::ui::truncate(entry.status_label(), 40),
                        crate::ui::truncate(entry.label(), 120)
                    ),
                    format!(
                        "Owner: {}",
                        crate::ui::truncate(&self.work_owner_label(entry.owner()), 120)
                    ),
                ];
                if let Entry::Report(work) = entry {
                    for (label, value) in [
                        ("Purpose", &work.report.purpose),
                        ("Next", &work.report.next),
                        ("Blocker", &work.report.blocker),
                    ] {
                        if let Some(value) = value {
                            lines.push(format!("{label}: {value}"));
                        }
                    }
                }
                lines.push(self.runtime_word().into());
                lines.extend(self.steering_text().lines().map(str::to_owned));
                let compact = lines
                    .iter()
                    .map(|line| crate::ui::truncate(line, width as usize))
                    .collect::<Vec<_>>()
                    .join("\n");
                return format!(
                    "{compact}\n\nRecorded detail · PageDown scroll\n{}\n{}{}",
                    self.work_text(),
                    self.steering_text(),
                    self.activity_text()
                );
            }
        }
        format!(
            "{}\n{}\n{}{}\n\nf feedback · Q saved questions · c criteria\nd documents · R read-only replay",
            self.work_text(),
            self.runtime_word(),
            self.steering_text(),
            self.activity_text()
        )
    }
    pub(super) fn work_text(&self) -> String {
        let Some(record) = &self.selected_work else {
            return String::new();
        };
        let entries = self.work_entries();
        let mut text = String::from("Work toward goal\n");
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
        for (at, entry) in entries.iter().enumerate().skip(start).take(4) {
            let marker = if selected == Some(at) { "› " } else { "" };
            text.push_str(&format!(
                "{marker}[{}] {}\n",
                crate::ui::truncate(entry.status_label(), 40),
                crate::ui::truncate(entry.label(), 120)
            ));
        }
        if entries.len() > 4 {
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
        text.push_str(&format!(
            "Owner: {}\nSource: {}\nRecorded: {}\n",
            crate::ui::truncate(&self.work_owner_label(current.owner()), 120),
            crate::ui::truncate(current.source(), 160),
            crate::ui::truncate(current.recorded_at(), 80)
        ));
        if self.producer_label(current.owner()) == "Agent" {
            text.push_str(&format!(
                "Owner ID: {}\n",
                crate::ui::truncate(&current.owner().agent_id, 256)
            ));
        }
        match current {
            Entry::Report(work) => {
                let report = &work.report;
                if let Some(parent) = &report.parent_id {
                    text.push_str(&format!("Parent: {}\n", self.work_reference(parent)));
                }
                text.push_str("Dependencies: ");
                match &report.depends_on {
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
                for link in report.outcome_links.iter().take(4) {
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
                if report.outcome_links.len() > 4 {
                    text.push_str(
                        "More outcome links recorded; inspect local context for detail.\n",
                    );
                }
                for (label, value) in [
                    ("Purpose", &report.purpose),
                    ("Changed", &report.change),
                    ("Next", &report.next),
                    ("Blocker", &report.blocker),
                ] {
                    if let Some(value) = value {
                        text.push_str(&format!("{label}: {}\n", crate::ui::truncate(value, 240)));
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
        self.selected_work
            .as_ref()
            .and_then(|record| record.work_items.iter().find(|work| work.report.id == id))
            .map(|work| format!("{} ({})", crate::ui::truncate(&work.report.label, 120), id))
            .unwrap_or_else(|| format!("{id} (unresolved)"))
    }
}
