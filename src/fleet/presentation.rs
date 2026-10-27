//! Cached terminal presentation and pointer routing. Never performs local IO.
use super::*;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::Position,
    widgets::{Clear, Gauge},
};

#[derive(Clone, Copy)]
pub(super) enum Action {
    Back,
    Steer,
    Work,
    Agents,
    Commands,
    Replay,
    Tracking,
    Coordinator,
    Guidance,
    Queue,
    Stop,
}

fn activate(ui: &mut FleetConsole, action: Action) {
    if !matches!(ui.pane, Pane::Graph | Pane::Feedback) {
        return;
    }
    match action {
        Action::Back if ui.edit.is_none() => ui.back_to_fleet(),
        Action::Back => {}
        Action::Steer if ui.edit.is_none() => ui.open_feedback(),
        Action::Steer => {}
        Action::Work if ui.edit.is_none() => ui.focus_work(),
        Action::Work => {}
        Action::Agents if ui.edit.is_none() => ui.open_agents(),
        Action::Agents => {}
        Action::Commands if ui.edit.is_none() => ui.open_commands(),
        Action::Commands => {}
        Action::Replay if ui.edit.is_none() => ui.request_replay(),
        Action::Replay => {}
        Action::Tracking if ui.edit.is_none() => ui.prepare_tracking(),
        Action::Tracking => {}
        Action::Coordinator if ui.edit.is_none() => ui.select_coordinator(),
        Action::Coordinator => {}
        Action::Guidance => ui.send_feedback(FeedbackMode::Guidance),
        Action::Queue => ui.send_feedback(FeedbackMode::Queue),
        Action::Stop => ui.send_feedback(FeedbackMode::Stop),
    }
}

fn draw_actions(ui: &mut FleetConsole, frame: &mut Frame, area: Rect, compact: bool) -> Rect {
    if !matches!(ui.pane, Pane::Graph | Pane::Feedback) || ui.selected_work.is_none() {
        return area;
    }
    let actions: &[(&str, Action, Option<FeedbackMode>)] =
        if ui.pane == Pane::Feedback && ui.edit.is_some() {
            &[
                (
                    "[Send Ctrl-S]",
                    Action::Guidance,
                    Some(FeedbackMode::Guidance),
                ),
                ("[Queue Ctrl-Q]", Action::Queue, Some(FeedbackMode::Queue)),
                ("[Stop Ctrl-X]", Action::Stop, Some(FeedbackMode::Stop)),
            ]
        } else if compact {
            &[
                ("[Steer f]", Action::Steer, None),
                ("[Agents a]", Action::Agents, None),
                ("[Commands x]", Action::Commands, None),
                ("[Replay R]", Action::Replay, None),
                ("[Coordinator C]", Action::Coordinator, None),
                ("[Enable tracking T]", Action::Tracking, None),
            ]
        } else {
            &[
                ("[Steer f]", Action::Steer, None),
                ("[Coordinator C]", Action::Coordinator, None),
                ("[Enable tracking T]", Action::Tracking, None),
                ("[Work w]", Action::Work, None),
                ("[Agents a]", Action::Agents, None),
                ("[Commands x]", Action::Commands, None),
                ("[Replay R]", Action::Replay, None),
            ]
        };
    let mut x = area.x;
    let mut y = area.y;
    for &(label, action, mode) in actions {
        let width = (label.len() as u16).min(area.width);
        if x > area.x && x.saturating_add(width) > area.right() {
            x = area.x;
            y = y.saturating_add(1);
        }
        if width == 0 || y >= area.bottom() {
            break;
        }
        let button = Rect::new(x, y, width, 1);
        let available = mode.is_none_or(|mode| ui.control_availability(mode).is_ok());
        frame.render_widget(
            Paragraph::new(label).style(Style::default().fg(if available {
                visual::ACCENT
            } else {
                Color::DarkGray
            })),
            button,
        );
        ui.action_areas.push((button, action));
        x = x.saturating_add(width + 1);
    }
    let used = if area.height == 0 {
        0
    } else {
        y.saturating_sub(area.y) + 1
    };
    Rect::new(
        area.x,
        area.y.saturating_add(used),
        area.width,
        area.height.saturating_sub(used),
    )
}

pub(super) fn draw(ui: &mut FleetConsole, frame: &mut Frame) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .areas(frame.area());
    let waiting = ui
        .snapshots
        .values()
        .filter(|s| {
            matches!(
                s.status,
                crate::control::RuntimeStatus::WaitingForAnswer
                    | crate::control::RuntimeStatus::WaitingForApproval
            )
        })
        .count();
    let description = if ui.query.is_empty() {
        if waiting > 0 {
            format!("{waiting} waiting for attention · goal progress needs evidence")
        } else {
            "Goal progress, agents and guidance".into()
        }
    } else {
        format!(
            "{}: {}",
            if ui.searching {
                "Find"
            } else if ui.view.query == ui.query {
                "Search"
            } else {
                "Searching"
            },
            ui.query
        )
    };
    let breadcrumb = if ui.work_canvas {
        let project = ui
            .view
            .selection
            .as_ref()
            .and_then(session_key)
            .and_then(|key| {
                ui.view
                    .graph
                    .nodes
                    .iter()
                    .find(|n| n.id == FleetNodeId::Session(key.clone()))
            })
            .and_then(|node| node.parent.as_ref())
            .and_then(|id| match id {
                FleetNodeId::Project(path) => PathBuf::from(path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
                _ => None,
            })
            .unwrap_or_default();
        format!(
            "[Back Esc] Fleet / {project} / {} · Work graph",
            ui.selected_label()
        )
    } else if ui.agents_mode {
        format!("[Back Esc] Agents / {}", ui.selected_label())
    } else {
        description.clone()
    };
    let breadcrumb = if (ui.work_canvas || ui.agents_mode) && (!ui.query.is_empty() || ui.searching)
    {
        format!("{breadcrumb} · {description}")
    } else {
        breadcrumb
    };
    frame.render_widget(
        Paragraph::new(format!(
            "Zoetrope · {} · {}\n{breadcrumb}",
            if ui.work_canvas {
                "Work graph"
            } else if ui.agents_mode {
                "Agents"
            } else {
                "Fleet"
            },
            if ui.history {
                "All history"
            } else {
                "Current work"
            }
        ))
        .style(
            Style::default()
                .fg(visual::ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        header,
    );
    let compact_work =
        body.width < 96 && ui.work_canvas && ui.pane == Pane::Graph && ui.edit.is_none();
    let (canvas, panel) = if body.width < 96 {
        let graph_height = if compact_work {
            body.height.saturating_sub(14).max(body.height / 3)
        } else if ui.pane == Pane::Graph && !ui.work_mode && ui.edit.is_none() {
            body.height / 2
        } else {
            body.height.min(5)
        };
        let [a, b] =
            Layout::vertical([Constraint::Length(graph_height), Constraint::Fill(1)]).areas(body);
        (a, b)
    } else {
        let [a, b] = Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(body);
        (a, b)
    };
    let canvas = if ui.work_canvas {
        let [legend, graph] = Layout::vertical([
            Constraint::Length(if canvas.width < 80 { 3 } else { 2 }),
            Constraint::Fill(1),
        ])
        .areas(canvas);
        let [summary, statuses] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(legend);
        frame.render_widget(
            Paragraph::new(crate::ui::truncate(
                &ui.work_graph.summary,
                summary.width as usize,
            )),
            summary,
        );
        frame.render_widget(
            Paragraph::new(visual::legend()).wrap(Wrap { trim: true }),
            statuses,
        );
        graph
    } else {
        canvas
    };
    if ui.work_canvas {
        let overview = ui.work_graph.overview;
        if ui.pane == Pane::Graph && ui.edit.is_none() {
            ui.work_graph.adapt_to(canvas, frame.area());
        }
        if ui.frame_area != frame.area() {
            if overview {
                ui.work_graph.flow.request_fit_view();
                ui.work_graph.pending_center = false;
            } else {
                ui.work_graph.pending_center = true;
            }
        }
    }
    ui.frame_area = frame.area();
    ui.canvas_area = canvas;
    frame.render_widget(ui.active_flow(), canvas);
    if ui.work_canvas && ui.work_graph.pending_center {
        ui.work_graph.flow.center_on_selected();
        ui.work_graph.pending_center = false;
        frame.render_widget(Clear, canvas);
        frame.render_widget(&mut ui.work_graph.flow, canvas);
    }
    if !ui.work_canvas && ui.pending_center {
        ui.flow.center_on_selected();
        ui.pending_center = false;
        frame.render_widget(Clear, canvas);
        frame.render_widget(&mut ui.flow, canvas);
    }
    if ui.view.ready && ui.view.graph.nodes.is_empty() {
        frame.render_widget(
            Paragraph::new(if ui.history {
                "No matching history. Clear search to see all work."
            } else {
                "No current work. Press H to open history."
            })
            .wrap(Wrap { trim: true }),
            canvas,
        );
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(visual::ACCENT))
        .title(match ui.pane {
            Pane::Deliveries => " Saved deliveries ",
            Pane::Questions => " Saved questions ",
            Pane::Criteria => " Completion criteria ",
            Pane::Documents => " Documents ",
            Pane::Feedback => " Feedback ",
            Pane::Commands => " Commands and public activity ",
            Pane::Graph => " Selected work ",
        });
    let inner = block.inner(panel);
    frame.render_widget(block, panel);
    ui.progress_area = None;
    ui.action_areas.clear();
    if ui.work_canvas || ui.agents_mode {
        ui.action_areas
            .push((Rect::new(header.x, header.y + 1, 10, 1), Action::Back));
    }
    let mut content = inner;
    if let Some(record) = &ui.selected_work {
        let [summary, gauge, rest] = Layout::vertical([
            Constraint::Length(if compact_work { 0 } else { 2 }),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(inner);
        content = rest;
        let goal = crate::ui::truncate(
            &ui.selected_label(),
            summary.width.saturating_sub(10) as usize,
        );
        let recipient = crate::ui::truncate(
            &ui.recipient_label(),
            summary.width.saturating_sub(11) as usize,
        );
        frame.render_widget(
            Paragraph::new(format!("Selected: {goal}\nRecipient: {recipient}")),
            summary,
        );
        let progress = record.progress();
        ui.progress_area = Some(gauge);
        if let Some(percent) = progress.percent {
            let complete = progress.total > 0 && progress.verified == progress.total;
            let max_ratio = if complete {
                1.0
            } else {
                gauge.width.saturating_sub(1) as f64 / gauge.width.max(1) as f64
            };
            let ratio = (percent / 100.0).min(max_ratio);
            frame.render_widget(
                Gauge::default()
                    .ratio(ratio)
                    .label(format!("Outcomes: {}", progress.label))
                    .gauge_style(Style::default().fg(if complete {
                        Color::Green
                    } else {
                        visual::ACCENT
                    })),
                gauge,
            );
        } else {
            frame.render_widget(
                Paragraph::new("Progress not tracked · click to define outcomes"),
                gauge,
            );
        }
    }
    if ui.selected_work.is_some() && ui.pane == Pane::Graph {
        let progress = ui.reported_progress(compact_work);
        let [reported, rest] = Layout::vertical([
            Constraint::Length(if compact_work { 1 } else { 3 }),
            Constraint::Fill(1),
        ])
        .areas(content);
        frame.render_widget(Paragraph::new(progress).wrap(Wrap { trim: true }), reported);
        content = rest;
    }
    content = draw_actions(ui, frame, content, compact_work);
    if ui.edit.is_some() {
        draw_editor(ui, frame, content);
    } else {
        let details = match ui.pane {
            Pane::Deliveries => ui.deliveries_text(),
            Pane::Documents => ui.documents_text(),
            Pane::Questions => ui.questions_text(),
            Pane::Criteria => ui.criteria_text(),
            Pane::Commands => format!(
                "Owner: {}\n{}\nEsc back · PageUp/PageDown scroll",
                ui.recipient_label(),
                ui.activity_text()
            ),
            Pane::Feedback => format!(
                "{}\n{}\n\nf edit feedback · F2 Send guidance · F3 Queue · F4 Stop\nCtrl-S Send · Ctrl-Q Queue · Ctrl-X Stop",
                ui.runtime_word(),
                ui.steering_text()
            ),
            Pane::Graph => {
                if ui.selected_work.is_some() {
                    ui.inspection_text(frame.area().width < 96, content.width)
                } else {
                    "Select a goal or agent to inspect its work".into()
                }
            }
        };
        frame.render_widget(
            Paragraph::new(if ui.pane == Pane::Graph {
                visual::details(&details)
            } else {
                ratatui::text::Text::raw(&details)
            })
            .wrap(Wrap { trim: true })
            .scroll((ui.scroll, 0)),
            content,
        );
    }
    let hints = if frame.area().width < 96 {
        "H history  / search  w work  a agents  f steer  ? help"
    } else {
        "H history  / search  w work  a agents  f steer  Q questions  c goals  d docs  ? help  q quit"
    };
    frame.render_widget(Paragraph::new(format!("{hints}\n{}", ui.status)), footer);
    if ui.show_help {
        draw_help(ui, frame);
    }
}
fn draw_editor(ui: &FleetConsole, frame: &mut Frame, area: Rect) {
    let Some(editor) = &ui.edit else {
        return;
    };
    if area.height == 0 || area.width == 0 {
        return;
    }
    let (intro, hints) = match ui.edit_kind {
        Some(EditKind::Feedback) => (
            format!(
                "{} · draft only until you send\n{}",
                ui.runtime_word(),
                ui.steering_text()
            ),
            "F2 Send guidance · Ctrl-S Send\nF3 Queue · Ctrl-Q · F4 Stop · Ctrl-X\nEsc saves · Ctrl-U clears",
        ),
        Some(EditKind::Answer { .. }) => (
            "Answer draft · saving does not submit it".into(),
            "F2 save draft · Esc saves and closes\nCtrl-U clears",
        ),
        Some(EditKind::Goal(_)) => (
            "Goal title first; one completion outcome on each following line".into(),
            "F2 save criteria · Esc cancel\nCtrl-U clears",
        ),
        _ => (
            "Explain the evidence or your explicit attestation".into(),
            "F2 verify outcome · Esc cancel\nCtrl-U clears",
        ),
    };
    let hint_height = if matches!(ui.edit_kind, Some(EditKind::Feedback)) {
        3
    } else {
        2
    };
    let [intro_area, entry, hint_area] = Layout::vertical([
        Constraint::Length(if matches!(ui.edit_kind, Some(EditKind::Feedback)) {
            5
        } else {
            2
        }),
        Constraint::Fill(1),
        Constraint::Length(hint_height),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(intro).wrap(Wrap { trim: true }), intro_area);
    let (lines, row, column) = editor.wrapped(entry.width as usize);
    let offset = row.saturating_sub(entry.height.saturating_sub(1) as usize);
    frame.render_widget(
        Paragraph::new(lines.join("\n")).scroll((offset.min(u16::MAX as usize) as u16, 0)),
        entry,
    );
    if entry.width > 0 && entry.height > 0 {
        frame.set_cursor_position((
            entry.x + column.min(entry.width.saturating_sub(1) as usize) as u16,
            entry.y
                + row
                    .saturating_sub(offset)
                    .min(entry.height.saturating_sub(1) as usize) as u16,
        ));
    }
    frame.render_widget(Paragraph::new(hints), hint_area);
}
fn draw_help(ui: &FleetConsole, frame: &mut Frame) {
    let area = frame.area();
    let popup = Rect::new(
        area.x + 1,
        area.y + 1,
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    frame.render_widget(Clear, popup);
    let text = "H history · / search · ↑↓ select · Enter expand\n←→ pan · + / - zoom · o overview · r readable\nw select work · [ / ] or arrows choose · a Agents · x Commands\nf feedback · F2 Send guidance · F3 Queue · F4 Stop\nCtrl-S Send · Ctrl-Q Queue · Ctrl-X Stop\nQ saved questions · L listen for native questions\nTab next prompt · 1–9 choose · e write an answer\nF2 submit answer · F5 copy a closed request to feedback\nx dismiss locally; native request is unchanged\nc criteria · e edit · v attest/verify · u reopen\n[ / ] choose draft plan · a adopt as unverified\np checkpoint preference · A prepare criteria request\nT Enable tracking: prepare coordinator reporting draft\nC explicitly select Coordinator; drafts keep their owner\nb saved deliveries · d documents · Enter preview\nR read-only replay · q returns from replay\nI reveal internal helpers · PageUp/PageDown detail\nEsc back / save draft · Ctrl-U clear editor\nq / Ctrl-C quit · watching makes no model calls";
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard help · Esc closes "),
            )
            .wrap(Wrap { trim: true })
            .scroll((ui.help_scroll, 0)),
        popup,
    );
}
pub(super) fn mouse(ui: &mut FleetConsole, event: MouseEvent) {
    if ui.show_help || ui.searching {
        return;
    }
    let point = Position::new(event.column, event.row);
    if matches!(event.kind, MouseEventKind::Down(MouseButton::Left))
        && let Some((_, action)) = ui
            .action_areas
            .iter()
            .find(|(area, _)| area.contains(point))
    {
        let action = *action;
        activate(ui, action);
        return;
    }
    if ui.edit.is_some() {
        return;
    }
    if matches!(event.kind, MouseEventKind::Down(MouseButton::Left))
        && ui.progress_area.is_some_and(|area| area.contains(point))
    {
        ui.pane = Pane::Criteria;
        ui.scroll = 0;
        return;
    }
    if !ui.canvas_area.contains(point) {
        match event.kind {
            MouseEventKind::ScrollDown => ui.scroll = ui.scroll.saturating_add(3),
            MouseEventKind::ScrollUp => ui.scroll = ui.scroll.saturating_sub(3),
            _ => {}
        }
        return;
    }
    if ui.work_canvas {
        let events: Vec<_> = ui
            .work_graph
            .flow
            .handle_mouse_event(event)
            .into_events()
            .collect();
        for event in events {
            match event {
                rataflow::FlowEvent::SelectionChanged { node_ids, .. } => {
                    if let Some(selected) = node_ids
                        .first()
                        .and_then(|id| work_graph::parse_selection(id))
                    {
                        ui.work_selection = Some(selected);
                        ui.scroll = 0;
                        ui.select_recipient(ui.selected_agent());
                    }
                }
                rataflow::FlowEvent::NodeDragged { node_id }
                | rataflow::FlowEvent::NodeDragEnded { node_id } => {
                    ui.work_graph.remember_drag(&node_id)
                }
                _ => {}
            }
        }
        return;
    }
    let events: Vec<_> = ui.flow.handle_mouse_event(event).into_events().collect();
    for event in events {
        match event {
            rataflow::FlowEvent::SelectionChanged { node_ids, .. } => {
                if let Some(id) = node_ids
                    .first()
                    .and_then(|id| serde_json::from_str::<FleetNodeId>(id).ok())
                {
                    ui.select(id);
                    ui.pending_center = false;
                }
            }
            rataflow::FlowEvent::NodeDragged { node_id }
            | rataflow::FlowEvent::NodeDragEnded { node_id } => {
                if let (Some(node), Ok(id)) = (
                    ui.flow.node(&node_id),
                    serde_json::from_str::<FleetNodeId>(&node_id),
                ) {
                    ui.positions.insert(id, (node.position.x, node.position.y));
                }
            }
            _ => {}
        }
    }
}
